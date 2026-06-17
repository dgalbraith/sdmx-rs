# 19. XML Namespace-Aware Parsing with NsReader

Date: 2026-05-22

## Status

Accepted

---

## Context

SDMX 3.0 and 3.1 specifications define structurally identical root elements
(`<Structure>`) under distinct XML namespaces:

* **SDMX 3.0**: `http://www.sdmx.org/resources/sdmxml/schemas/v3_0/structure`
* **SDMX 3.1**: `http://www.sdmx.org/resources/sdmxml/schemas/v3_1/structure`

A naive XML parser that routes solely on local tag names (e.g., matching
`<Structure>`) cannot distinguish which specification version produced the
payload. This creates two risks:

1. **Silent misrouting**: Metadata from a 3.1 endpoint is parsed with 3.0
   structural assumptions, potentially losing or misinterpreting version-specific
   fields.
2. **Correctness failure**: The parser may crash or produce malformed domain
   objects when encountering unexpected structural divergences.

ADR-0008 commits to a unified, version-agnostic `ConstraintModel` enum. The
parsing layer (ADR-0009's quick-xml selection) must resolve the SDMX version
at parse time. This ADR specifies how that version detection occurs for SDMX-ML
payloads.

---

## Decision Drivers

* **Correctness**: Every SDMX-ML parsing branch must route based on the
  authoritative version signal: the XML namespace URI on the root element.
* **no_std Portability**: The parser crate is `#![no_std]`, prohibiting
  `std::io::Read` in the public API. The input format and constructor must
  respect this constraint.
* **Performance**: Zero-copy parsing must remain possible; the version
  discriminant must be extracted during the first element-read pass with no
  additional buffering or lookahead.
* **Error Clarity**: Unsupported or malformed namespace declarations must be
  surfaced as explicit parse errors, not silently ignored.

---

## Options Considered

### Option A — Plain Reader with Post-Hoc Namespace Extraction

Use `quick_xml::Reader` (the basic event reader) and extract namespace URIs
from attributes manually within the event-processing loop.

**Pros**:

* Minimal API surface; `Reader` is the simplest quick-xml type.

**Cons**:

* Namespace prefix-to-URI resolution is not automatic. The parser must track
  namespace declarations (via `Event::Start` attributes and scoping rules) and
  resolve them manually — error-prone and duplicative of quick-xml's built-in
  namespace handling.
* High complexity for correctness. Incorrect scope tracking or prefix resolution
  can silently produce wrong routing decisions.

**Verdict**: Rejected.

### Option B — NsReader with Automatic Namespace Resolution (Accepted)

Use `quick_xml::NsReader` which wraps `Reader` and provides automatic namespace
URI resolution.

**Pros**:

* `NsReader::resolve_element(name)` returns `(ResolveResult<'_>, LocalName<'_>)`
  directly from the `Event::Start` arm. The namespace URI is extracted
  automatically, eliminating manual scope tracking.
* Error handling is built-in: `ResolveResult::Unknown` or `ResolveResult::Unbound`
  can be converted to a hard parse error immediately.
* Zero additional overhead; the namespace resolution occurs during the same
  event-processing pass.

**Cons**:

* Requires familiarity with `ResolveResult` enum and the `Namespace` newtype;
  slightly less intuitive than plain `Reader`.

**Verdict**: Accepted.

---

## Decision

**SDMX-ML parsing in `sdmx-parsers` will use `quick_xml::NsReader` for all XML
event streams. The namespace URI resolved from every `<Structure>` root element
is the authoritative SDMX specification version discriminant.**

### Namespace URI Constants

The following namespace URIs are recognised and mapped to SDMX specification
versions:

```
http://www.sdmx.org/resources/sdmxml/schemas/v3_0/structure  →  SDMX 3.0
http://www.sdmx.org/resources/sdmxml/schemas/v3_1/structure  →  SDMX 3.1
```

These constants are defined in `crates/sdmx-parsers/src/xml/ns.rs` as byte
string literals to avoid UTF-8 allocation overhead during matching.

### SdmxVersion Discriminant

An internal enum `SdmxVersion { V3_0, V3_1 }` is derived from the resolved
namespace and passed to the version-specific parse handler. This discriminant
is internal to `sdmx-parsers` and never exposed in the public API; downstream
callers receive only the unified `ConstraintModel` from ADR-0008.

### Input Interface & conditional std support

To achieve $O(1)$ memory consumption in native production environments, `sdmx-parsers` leverages a hybrid `std` / `no_std` architecture. By default, the crate is compiled with the `std` feature active:

* **When `std` is active**: The parser exposes entry points accepting generic readers implementing `std::io::BufRead` (e.g. `NsReader::from_reader(reader)`). This enables streaming directly from network sockets or files, bypassing the need to load the entire payload into memory.
* **When `std` is inactive**: The crate compiles in `#![no_std]` + `alloc` mode, and the entry points are constrained to accept only byte slices (e.g. `NsReader::from_slice(bytes)`). This preserves compatibility for headless WebAssembly targets.

### Error Handling

An unrecognised or unbound namespace on the `<Structure>` root element is a
hard error. The parser returns `ParseError::UnsupportedSchemaNamespace(String)`
carrying the raw or decoded namespace URI. No fallback parsing is attempted —
misrouting is not tolerated.

---

## Consequences

* **Positive**: SDMX version routing is deterministic and tamper-proof,
  controlled by the payload's own namespace declaration rather than external
  configuration or heuristics.
* **Positive**: The namespace resolution is automatic and zero-cost, occurring
  during the first `Event::Start` read with no additional passes or buffering.
* **Positive**: Error cases (unknown namespace, unbound prefix) are caught
  explicitly and surfaced to the caller, preventing silent misrouting.
* **Positive**: Native configurations achieve true streaming with $O(1)$ memory consumption.
* **Positive**: WebAssembly builds maintain standard-library-free portability by using slice-based inputs.
* **Neutral**: Callers must select the appropriate API (reader-based or slice-based) depending on whether standard library support is active.
* **Neutral**: The API surface includes `ResolveResult` and `Namespace` types
  from quick-xml; consumers need minimal familiarity with these types but
  typically do not interact with them directly.
* **[Phase 2 Validation]**: The overhead of automatic namespace stack maintenance
  (push/pop on every Start/End event) should be measured empirically in Phase 2
  benchmarking (ADR-0010) using realistic SDMX payloads (1MB–1GB scale). Current
  analysis suggests this cost is negligible (<0.1% of total parse time) relative
  to character entity decoding and domain object construction, but this should be
  confirmed. If profiling reveals namespace resolution as a bottleneck, a
  fast-path for single-namespace documents can be added without API change.

---

## References

* ADR-0008 — `ConstraintModel` unified enum and version routing requirement
* ADR-0009 — `quick-xml` and `serde_json` streaming parser selection
* ADR-0018 — Content-Type negotiation; XML routing and version conveyance
* ADR-0005 — `#![no_std]` + `alloc` constraint on `sdmx-parsers`
* [quick-xml Documentation](https://docs.rs/quick-xml) — `NsReader` API
* [SDMX REST API Schema Documentation](https://github.com/sdmx-twg/sdmx-rest/blob/master/doc/schema.md) — Canonical SDMX namespace URIs
