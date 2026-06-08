# 2. SDMX CSV Stream Parsing Design

Date: 2026-05-24

## Status

Proposed

<!-- Valid statuses: Proposed, Accepted, Implemented, Superseded -->

---

## Summary

Design of a streaming SDMX-CSV data message parser that operates in O(1) memory regardless of dataset size. The parser must handle SDMX-specific column semantics (domain-governed, not positional), maintain `no_std + alloc` compatibility, and expose a consistent streaming API with XML and JSON parsers. This design explores library choices (csv crate vs. manual byte scanning), streaming models, and representation strategies with validation gates to be resolved during Phase 2 implementation.

---

## Problem / Motivation

The ROADMAP includes an SDMX-CSV data message parser as a Phase 2 deliverable — a "streaming observation reader for the first-class SDMX-CSV data format." ADR-0018 records that `sdmx-client` will prefer SDMX-CSV over SDMX-JSON for data queries, making the CSV parser a primary hot path for the library.

**SDMX-CSV differs structurally from standard CSV deserialization**:
- **Column semantics are domain-governed**, not positional. The header row encodes keyed dimension identifiers (e.g., `FREQ`, `REF_AREA`, `INDICATOR`) whose meaning is defined by a Data Structure Definition (DSD), not by column order. Column order may vary between responses.
- **No self-describing version envelope.** Unlike SDMX-ML (XML namespace) and SDMX-JSON (top-level `version` field), SDMX-CSV has no wire-level marker. Version is conveyed out-of-band via the `Accept` header or inferred from `Content-Type` parameters.
- **Observations are row-per-record**, flat and tabular with all dimension values inline — fundamentally different from the hierarchical structure of SDMX-ML and SDMX-JSON.

**Constraints & Goals**:
- **Memory Efficiency**: O(1) memory streaming regardless of dataset size (consistent with ADR-0009)
- **`no_std + alloc` Compatibility**: Must not introduce hard `std` dependency (ADR-0005)
- **API Consistency**: Expose same streaming iterator contract as XML and JSON parsers
- **DSD-Free Parsing**: Produce row-level domain type without requiring DSD at parse time

---

## Proposed Design

### Architecture / Key Decisions

#### 1. CSV Library Choice: `csv-core` Crate with `no_std` support (or Manual Scanning)

Use the `csv-core` crate (Option A) or implement manual byte scanning with `memchr` (Option B) for row-by-row streaming. The standard `csv` crate does not support `#![no_std]` as it relies on `std::io::Read` and `std::io::Write`.

**Rationale**:
- `csv-core` is `#![no_std]` compatible and does not perform heap allocations, though it operates at a lower level and requires manual buffer feeding and field tracking.
- Alternatively, manual byte scanning with `memchr` (which is already a dependency in the workspace) allows a simple, lightweight implementation of CSV row decoding directly.

#### 2. Streaming Model: Row-by-Row Iterator with Single Header Parse

Parse the header row once on parser construction; stream observation rows lazily. Each row yields a `RawObservationRow` type (map of column name → string value).

#### 3. DSD-Independent Representation: `RawObservationRow`

Raw parsing produces `RawObservationRow` — a map of column name to string value without type resolution. DSD-dependent enrichment (mapping header names to typed dimension values) is a separate, decoupled step.

**Representation choice (provisional, pending Phase 2 benchmarking)**:
- **Preferred**: `BTreeMap<Box<str>, Box<str>>` — owned fields, no hasher dependency, `no_std`-safe
- **Alternative**: `BTreeMap<&'a str, &'a str>` — zero-copy if `csv` crate supports borrowing field slices from internal buffer
- **Fallback**: `hashbrown::HashMap<String, String>` if `BTreeMap` comparison overhead is material at scale

#### 4. Version Routing: Out-of-Band Parameter

SDMX version for CSV responses is conveyed as a parameter to the parser constructor from `sdmx-client` (which knows the version from the `Accept` header sent and/or `Content-Type` parameters). This contrasts with XML/JSON where version is read from the payload envelope.

---

## Alternatives Considered

### Option A — `csv-core` Crate (Preferred)

**Pros**: Proven low-level parsing state-machine; handles RFC 4180 quoting/escaping edge cases; explicitly `#![no_std]` compatible.

**Cons**: Does not provide a high-level `Reader` interface. Requires implementing manual buffer management and row slice-splitting in `sdmx-parsers`.

### Option B — Manual Byte Scanning with `memchr` (Alternative)

Implement a minimal CSV row scanner directly using `memchr` for delimiter/newline scanning.

**Pros**: Zero new transitive dependencies (beyond `memchr`, already used by `quick-xml`); guaranteed `no_std` + WASM; SIMD-accelerated throughput.

**Cons**: Requires implementing RFC 4180 quoting and embedded-CRLF handling in-house.

**Verdict**: Prototype both in Phase 2; select the one offering the best balance of safety and row throughput.

### Option C — Delegate to `serde` + `csv` with Fixed Struct (Rejected)

Define a fixed Rust struct mapping to SDMX-CSV columns and use `serde` derive for deserialization.

**Verdict**: Rejected. SDMX-CSV column sets are not fixed — they vary by dataset and DSD. A static struct cannot represent arbitrary dimension columns.

---

## Drawbacks / Trade-offs

**Performance**:
- Provisional `BTreeMap<Box<str>, Box<str>>` allocates one `Box<str>` per field per row. At high observation counts, may produce measurable allocation pressure. Phase 2 profiling should determine whether migration to borrowed `BTreeMap<&str, &str>` or `hashbrown::HashMap` is warranted.

**API Complexity**:
- Consumers requiring fully typed dimension values must perform a second DSD-enrichment pass; the parser alone is insufficient for that use case.

**Maintenance**:
- If WASM validation fails and Option B is adopted, implementing RFC 4180 quoting and embedded-CRLF handling in-house adds a bounded but real testing surface.

**Asymmetry**:
- Version routing for CSV is extrinsic (caller-supplied), unlike XML and JSON where the version is read from the payload envelope. This asymmetry must be documented in `sdmx-parsers`' public API.

---

## Questions & Resolutions

- **[Open]** - **`no_std` compatibility**: Does the `csv` crate compile under `#![no_std]` + `alloc` for `wasm32-unknown-unknown`? If this fails, adopt Option B (manual byte scanning) and update the design.

- **[Open]** - **Row throughput**: What is the absolute row throughput (rows/sec and MB/sec) against a representative SDMX-CSV fixture? Target: ≥ 500 k rows/sec on a modern desktop core. If throughput is below target, investigate Option B or `BTreeMap` → `hashbrown::HashMap` migration.

- **[Open]** - **Zero-copy borrowing**: Is the chosen `BTreeMap<Box<str>, Box<str>>` representation workable under the streaming API? Or does the `csv` crate support zero-copy borrowing (`BTreeMap<&str, &str>`) from the internal row buffer?

---

## References

* [ADR-0005: no_std + alloc Architecture](../adr/0005-adopt-no-std-with-alloc-for-sdmx-types-and-sdmx-parsers.md)
* [ADR-0008: Unified Constraint Model](../adr/0008-model-sdmx-3-0-and-3-1-divergence-with-a-unified-constraintmodel.md)
* [ADR-0009: quick-xml & serde_json Streaming](../adr/0009-use-quick-xml-and-serde-json-for-streaming-deserialization.md)
* [ADR-0018: Content-Type Negotiation and Parser Routing](../adr/0018-content-type-negotiation-and-parser-routing.md)
* ROADMAP.md — Phase 2 deliverable: SDMX-CSV data message parser
* RFC 4180 — Common Format and MIME Type for Comma-Separated Values (CSV) Files

---

## Notes for Implementation

**Preconditions (Phase 2 Validation Gates)**:

- [ ] **Gate 1 (blocking)**: Confirm whether `csv-core` or manual `memchr` scanning is used, and ensure it compiles successfully under `#![no_std]` + `alloc` for `wasm32-unknown-unknown`.
- [ ] **Gate 2 (informational)**: Benchmark row throughput ≥ 500 k rows/sec against a representative SDMX-CSV fixture. Document result in-place.
- [ ] **Gate 3 (informational)**: Define `RawObservationRow` in `sdmx-types` and confirm the `BTreeMap` representation is workable under the streaming API.

**Integration Points**:
- Parser is part of `sdmx-parsers` crate (Phase 2 delivery)
- `sdmx-client` uses version routing to dispatch to CSV parser (ADR-0018)
- `RawObservationRow` type defined in `sdmx-types` (ADR-0009 parallel pattern)
- DSD-enrichment logic is decoupled, potentially Phase 3+ feature

**Success Criteria**:
- CSV parser operates in O(1) memory
- Supports streaming observation rows lazily
- Compiles to `wasm32-unknown-unknown` without modification
- Exposed API is consistent with XML and JSON parsers
