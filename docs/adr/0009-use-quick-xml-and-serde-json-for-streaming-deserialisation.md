# 9. Use quick-xml and serde_json for Streaming Deserialisation

Date: 2026-05-17

## Status

Accepted, extended by ADR-0019

---

## Context

SDMX metadata and dataset payloads routinely scale from megabytes to gigabytes of tabular observations. Attempting to parse these payloads by loading a full Document Object Model (DOM) tree into memory presents a significant threat of Out-of-Memory (OOM) crashes in resource-constrained environments.

We must select parsing engines for `sdmx-parsers` that support streaming, sequential deserialisation. Additionally, the engines must support high-performance parsing primitives (like zero-copy string slicing) while maintaining compatibility with the `#![no_std]` + `alloc` requirement of our parser crate.

## Decision Drivers

* **Memory Efficiency**: Maintain flat, O(1) memory consumption regardless of input document size.
* **Execution Throughput**: Maximise parser parsing speed via zero-copy byte-slice operations.
* **Target Portability**: Compatibility with WebAssembly and `#![no_std]` bare-metal targets.
* **Entity Resolution Safety**: Gracefully handle XML character entity references (e.g., `&amp;`, `&lt;`) without forcing full allocation.

---

## Options Considered

### Option A — DOM-Based Parsers (e.g., `roxmltree`, `xml-rs`, standard `serde_json` Value map)

Parsing the entire XML or JSON document into an in-memory tree representation prior to traversing or mapping it to domain structures.

* **Pros**:
  * Highly intuitive API; allows arbitrary navigation up and down the document tree.
* **Cons**:
  * Massive memory consumption. Tree nodes typically occupy 5x to 10x the size of the raw document.
  * Fails to support streaming observation processing, making it impossible to parse gigabyte-scale datasets.
**Verdict**: Rejected.

### Option B — Token-Driven Streaming with `quick-xml` and `serde_json` Stream Readers

Reading elements sequentially using low-level event tokenisers (`quick-xml::Reader` and `serde_json::Deserializer::into_iter`).

* **Pros**:
  * Event-driven processing processes data iteratively, yielding flat memory usage.
  * `quick-xml` supports zero-copy slicing of XML tag attributes and text nodes directly from the input buffer.
  * Both libraries compile under `#![no_std]` with the `alloc` crate.
  * Extensively tested and maintained in the Rust parsing ecosystem.
* **Cons**:
  * High implementation complexity. Writing token-based streaming loops requires manual state management for nested elements.
  * Character entity decoding (e.g., converting `&amp;` to `&`) changes the string contents, meaning zero-copy borrowing is impossible for those text segments.
**Verdict**: Accepted.

---

## Decision

**We will use `quick-xml` (with the `serialize` feature) and `serde_json` as our serialisation engines, implementing manual token-driven streaming loops to parse massive datasets in O(1) memory.**

To achieve optimal performance with character entities, our parser models will leverage `Cow<'a, str>`. This allows the parser to slice borrowed strings directly from the buffer when no entity references are present, only allocating owned `String` instances when character decoding must modify the string.

---

## Consequences

* **Positive**: Safe execution against large-scale datasets with fixed, low memory overhead.
* **Positive**: Highly performant parsing paths for clean data (which bypasses allocation entirely).
* **Positive**: Full WebAssembly and `#![no_std]` target safety.
* **Negative**: Increased code complexity in `sdmx-parsers` due to stateful token loops and manual stream traversal.

---

## References

* `ARCHITECTURE.md` — Section 1.4 (XML Engine) & Section 1.6 (Streaming Parser Memory Architecture)
* [quick-xml Crate Documentation](https://docs.rs/quick-xml)
* [serde_json Stream Deserializer](https://docs.rs/serde_json/latest/serde_json/struct.StreamDeserializer.html)
