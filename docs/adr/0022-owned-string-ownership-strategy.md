# 22. Owned String Ownership Strategy

Date: 2026-06-11

## Status

Accepted

---

## Context

Every textual field in the `sdmx-types` domain model — identifiers, names, URLs, raw lexical forms — could be represented as borrowed `&'a str` slices into parse buffers, as `Cow<'a, str>`, or as owned `String`s. This architectural choice cascades: a single borrowed field gives its struct a lifetime parameter, which propagates to every containing type, every collection, every cache, every client API, and every consumer signature in the workspace. SDMX metadata is long-lived by nature (codelists and DSDs are fetched once and consulted for the lifetime of a session or cache), which makes the transient nature of borrowed data unsuitable.

Register decision D-0007 settled this during M0. This ADR promotes it, because it is a workspace-wide API commitment — `sdmx-parsers`, `sdmx-client`, and `sdmx-writers` all build on the domain types being lifetimeless — and because it resolves a genuine trade-off (allocation cost vs ergonomics) that warrants formal documentation.

## Decision Drivers

* Domain values must be storable, cacheable, and shareable without lifetime plumbing (`'static` types).
* `no_std` + `alloc` compatibility (no `std`-only ownership machinery).
* Parse-time allocation cost must be bounded and attributable.
* Consumer-facing API simplicity across the whole workspace, including WASM targets.

---

## Options Considered

### Option A — Borrowed or `Cow` zero-copy domain types

Domain structs borrow from the parse buffer (`&'a str` / `Cow<'a, str>`), avoiding allocation where the input is already in memory.

**Pros**:

* Minimal allocation during parsing; achieves zero-copy parsing.

**Cons**:

* Lifetime parameters propagate through every type, collection, cache, and client API in the workspace; user code carries `'a` annotations forever.
* Long-lived metadata pinned to its parse buffer; caching requires re-owning anyway, paying the allocation cost while keeping the complexity.
* `Cow` doubles every field's states without removing the lifetime.

**Verdict**: Rejected.

### Option B — Owned `String` throughout

Every text field owns its data; lifetimes are confined to parser tokenize loops.

**Pros**:

* Lifetimeless (`'static`) domain values: trivially cacheable, storable, movable across tasks and the WASM boundary; simple signatures everywhere.
* Allocation is confined to parse time, where it is bounded by document size and measurable.

**Cons**:

* Higher allocation count during parsing than a zero-copy design.

**Verdict**: Accepted.

---

## Decision

**Adopt Option B: owned `String` for all text fields.** The domain model owns its data; lifetime complexity is confined strictly to the parsers' internal tokenize loops. This is a deliberate trade-off prioritizing workspace-wide API simplicity and cacheability over parse-time allocation overhead.

---

## Consequences

* **Positive**: `'static` domain types end-to-end; no lifetime parameters in any public API; caching and client storage are trivial; raw-lexeme preservation (ADR-0023's lossless `raw` fields) composes naturally with ownership.
* **Negative**: Parsing allocates per text field; bulk-parsing throughput is reduced. If profiling ever demands, interning or arena strategies are additive *parser-internal* optimisations — the domain API does not change.
* **Neutral**: Register entry D-0007 is promoted to this ADR (body retained as audit trail).

---

## References

* [ADR-0005: Adopt No-Std with Alloc for Sdmx Types and Sdmx Parsers](0005-adopt-no-std-with-alloc-for-sdmx-types-and-sdmx-parsers.md)
* [ADR-0023: Two Layer Infoset Store and Derived Views Architecture](0023-two-layer-infoset-store-and-derived-views-architecture.md)
* [Decision register](../decisions.md): D-0007 (promoted here)
* [Design 0010 — SDMX Core Domain Types](../design/0010-sdmx-core-domain-types-design.md)
