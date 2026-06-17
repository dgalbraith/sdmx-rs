# 17. SDMX-CSV Parser Library Selection

Date: 2026-05-21

## Status

Accepted

---

## Context

The ROADMAP includes an SDMX-CSV data message parser as a Phase 2 deliverable. ADR-0018 records that `sdmx-client` will prefer SDMX-CSV over SDMX-JSON for data queries, making this parser a primary performance-critical path.

SDMX-CSV data messages require streaming deserialisation with O(1) memory consumption and `#![no_std]` + `alloc` compatibility (per ADR-0005). We must select a parsing library that satisfies these constraints without introducing `std` dependencies into `sdmx-parsers`.

## Decision Drivers

* **Memory Efficiency**: O(1) memory streaming regardless of dataset size (ADR-0009 principle)
* **`no_std + alloc` Compatibility**: `sdmx-parsers` must not introduce hard `std` dependencies (ADR-0005)
* **Proven & Maintainable**: The chosen library must handle RFC 4180 edge cases (quoting, embedded newlines) correctly

---

## Options Considered

### Option A — `csv-core` Crate (Preferred)

Use the `csv-core` crate for low-level state-machine CSV parsing.

**Pros**:
- Proven, well-maintained; handles RFC 4180 edge cases (quoting, embedded newlines)
- Fully `#![no_std]` compatible and does not allocate on the heap.

**Cons**:
- Only provides raw decoding; requires custom buffer management and slice splitting.

**Verdict**: Preferred — to be prototyped and validated in Phase 2.

### Option B — Manual Byte Scanning with `memchr` (Alternative)

Implement CSV scanning directly using `memchr` for delimiter/newline detection without an external CSV crate.

**Pros**:
- Zero new transitive dependencies (`memchr` is already used by `quick-xml`)
- Guaranteed `#![no_std]` + WASM compatibility
- SIMD-accelerated throughput

**Cons**:
- Requires implementing RFC 4180 quoting and embedded-CRLF handling in-house

**Verdict**: Alternative — to be compared with Option A during Phase 2 benchmarking.

### Option C — `serde` + `csv` with Fixed Struct (Rejected)

Define a fixed Rust struct mapping to SDMX-CSV columns and use `serde` deserialisation.

**Cons**:
- SDMX-CSV column sets are dynamic (vary by dataset and DSD)
- Static struct cannot represent arbitrary dimension columns

**Verdict**: Rejected — fundamentally mismatches the domain model

---

## Decision

**Use `csv-core` (Option A) or manual byte scanning (Option B) for the CSV streaming parser, as the standard `csv` crate is incompatible with `#![no_std]`.**

---

## Consequences

* **Positive**: Maintains strict `#![no_std]` compliance, enabling compilation to `wasm32-unknown-unknown`.
* **Positive**: Decouples parsing from standard library I/O traits, allowing highly optimised slice and buffer operations.
* **Negative**: Operating at a lower level (via `csv-core` or manual byte scanning) requires custom buffer feeding and record management.
* **Neutral**: Final selection between `csv-core` and manual scanning will be decided based on performance benchmarks and implementation complexity in Phase 2.

---



## References

* [ADR-0005](0005-adopt-no-std-with-alloc-for-sdmx-types-and-sdmx-parsers.md) — `no_std + alloc` constraint
* [ADR-0009](0009-use-quick-xml-and-serde-json-for-streaming-deserialisation.md) — Streaming parser pattern (parallel precedent)
* [ADR-0018](0018-content-type-negotiation-and-parser-routing.md) — CSV as preferred data format
* [docs/design/0002-sdmx-csv-stream-parsing-design.md](../design/0002-sdmx-csv-stream-parsing-design.md) — Detailed design and implementation strategy
