# Performance Guide

This guide documents how we measure, optimize, and reason about performance in sdmx-rs. As a high-performance exemplar library, performance is not an afterthought—it's a design goal baked into architecture decisions and enforced through benchmarking.

## Performance Philosophy

**What we optimize for**:
- **Throughput over latency**: Parse gigabytes of SDMX data efficiently; total time matters more than response time of a single operation.
- **Memory efficiency**: Streaming, not DOM accumulation. Large payloads should not require proportional memory.
- **Predictable allocation**: Allocations should be visible and intentional, not hidden in library internals.
- **Zero-cost abstractions**: Type safety and ergonomics should not require runtime overhead.

**What we don't optimize for**:
- Microsecond-level latency (we're I/O-bound, not CPU-bound)
- Every possible edge case (80/20 rule: optimize the common path)
- Over-engineering for hypothetical future use cases

---

## Benchmarking

### Current Phase (Phase 1–2)

Benchmarking infrastructure is planned for **Phase 2**. Until then:
- Performance is validated through code review (allocation patterns, memory architecture)
- Real-world SDMX datasets in integration tests serve as de-facto benchmarks
- No regressions are intentionally introduced

### Phase 2+: Criterion Integration

When [Criterion](https://bheisler.github.io/criterion.rs/book/) is integrated:

```bash
# Run all benchmarks
cargo bench --all

# Run a specific benchmark
cargo bench parse_constraint

# Compare against baseline
cargo bench -- --save-baseline baseline_v0.1.0
```

Benchmark targets:
- **sdmx-parsers**: Parsing throughput (MB/s) for XML and JSON payloads
- **sdmx-writers**: Serialization throughput for large collections
- **sdmx-client**: End-to-end query latency (network I/O is dominant)

**Benchmark sources**:
- Real SDMX registry responses (ECB, BIS, IMF)
- Synthetic payloads at scale boundaries (1KB, 100KB, 10MB)
- Pathological inputs (deeply nested, many rules)

**CI behavior**:
- Benchmarks run on stable infrastructure (GitHub Actions)
- Regressions >5% throughput loss trigger a warning (not a failure)
- Baseline stored in git for reproducibility

---

## Memory Profiling

### Allocation Awareness (Always)

Code review emphasizes visible, intentional allocations:

```rust
// Good: Allocation is explicit and necessary
let mut rules = Vec::with_capacity(estimated_count);
for rule in parse_rules(input) {
    rules.push(rule);
}

// Avoid: Hidden allocation in library internals
let rules = input.rules().collect::<Vec<_>>();  // Where is capacity estimated?
```

**Thinking about allocations**:
- Stack allocation preferred (fixed-size types, temporary buffers)
- Heap allocation reserved for unbounded collections (Vec, HashMap, String)
- Pre-allocation preferred over growing (use `Vec::with_capacity()` with a reasonable estimate)
- Cow<'_, str> used for optional-copy string fields (borrow when possible, allocate when necessary)

### Memory Profilers

For detailed analysis, use:

**Valgrind (Linux)**:
```bash
# Run program under Valgrind's memcheck
valgrind --leak-check=full --show-leak-kinds=all \
  cargo test --lib parse_constraint_model
```

**Heaptrack (Linux)**:
```bash
# Track allocations
heaptrack target/debug/sdmx-rs-test parse_constraint_model
heaptrack_gui heaptrack.sdmx-rs-test.*.gz
```

**Apple Instruments (macOS)**:
- Xcode → Product → Profile → Allocations
- Track heap growth over time while parsing large SDMX documents

**When to profile**:
- If a change involves new allocations or collections
- When scaling to large payloads (10MB+)
- If code review flags allocation patterns

---

## Streaming Memory Architecture

The `sdmx-parsers` crate is designed for **streaming, not DOM accumulation**. This is a core performance feature.

### Design Pattern

Instead of:
```rust
// ❌ Materializes entire document in memory
pub fn parse(input: &str) -> Result<Document> {
    let dom = build_full_dom(input)?;
    Ok(dom)
}
```

We do:
```rust
// ✅ Streams tokens, yields items on-the-fly
pub fn parse(input: &str) -> Result<impl Iterator<Item = Item>> {
    Ok(TokenStream::new(input).map(|token| Item::from(token)))
}
```

### Key Techniques

1. **Token-Driven Parsing** — `quick-xml` yields tokens; we process them immediately
2. **Zero-Copy Slicing** — Extract text boundaries from input buffers without allocation (when no entities present)
3. **Cow<'_, str> for Decoded Text** — Borrow slices when safe; allocate only when XML entities require decoding
4. **No Global Accumulation** — Data is hydrated on-the-fly; no ever-growing intermediate structures

### Verification

Run memory-intensive tests to verify streaming behavior:

```bash
# This should not OOM even with 100MB+ SDMX payloads
cargo test --lib parse_large_payloads -- --nocapture
```

---

## Concurrency & Async Performance

### Design Goals

- **Share, don't serialize**: `SdmxClient` is `Send` + `Sync`; clone and share directly without `Arc` or `Mutex`
- **Non-blocking I/O**: All HTTP operations are async; no `block_on()` in hot paths
- **Task spawning safety**: Builders are unconditionally `'static`; can be spawned on background tasks

### Async Patterns to Avoid

❌ **Holding locks across `.await`**:
```rust
async fn bad_pattern() {
    let guard = data.lock();  // Lock held...
    let result = async_operation().await;  // ...across await point
    drop(guard);
}
```

✅ **Release locks before `.await`**:
```rust
async fn good_pattern() {
    let result = {
        let guard = data.lock();
        guard.do_something()
    };  // Lock released
    let async_result = async_operation().await;
}
```

### Tokio-Specific Patterns

**When the runtime is active**:
```rust
// Safe: spawned tasks are awaited
let handle = tokio::spawn(async { /* work */ });
handle.await?;
```

**When no runtime is active**:
```rust
// Safe: SdmxClient creates a private runtime if needed
let client = SdmxClient::new(url)?;
let result = client.fetch().await?;
```

**Avoid blocking the executor**:
```rust
// ❌ Bad: blocks the runtime
tokio::spawn(async {
    std::thread::sleep(Duration::from_secs(1));
});

// ✅ Good: async-aware
tokio::spawn(async {
    tokio::time::sleep(Duration::from_secs(1)).await;
});
```

---

## Performance Trade-offs & Decisions

### Typed Builders over String Parameters

**Trade-off**: Compile-time validation (typestate pattern) vs simpler API

**Decision**: Use typestate builders for public APIs. Type safety prevents invalid queries from reaching the network layer—a performance win (no wasted HTTP round-trips) that outweighs API complexity.

### Streaming Parsers over DOM Builders

**Trade-off**: Streaming is harder to implement vs DOM is easier to reason about

**Decision**: Invest in streaming architecture. Enables gigabyte-scale payloads without proportional memory overhead—critical for enterprise SDMX systems.

### Feature-Gated Dependencies

**Trade-off**: Compiling features increases binary size vs keeping dependencies optional

**Decision**: Gate `client` and `parsers` as optional features. Users can pull `sdmx-types` (no_std, WASM) without HTTP/async overhead.

---

## Benchmarking Roadmap

| Phase | Task                                        | Target                     |
|-------|---------------------------------------------|----------------------------|
| 1     | Code review emphasis on allocation patterns | Per-PR validation          |
| 2     | Integrate Criterion, establish baselines    | 80th percentile throughput |
| 3     | Real SDMX registry data as benchmark corpus | <100ms query latency       |
| 4     | Continuous benchmarking in CI               | Regression detection       |

---

## References

- [ARCHITECTURE.md → Streaming Parser Memory Architecture](../../ARCHITECTURE.md#streaming-parser-memory-architecture)
- [practices.md → Async/Await Patterns](../dev/practices.md#asyncawait-patterns)
- [CONTRIBUTING.md → Code Review Philosophy](../../CONTRIBUTING.md#code-review-philosophy--standards)
- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Valgrind Manual](https://valgrind.org/docs/manual/)
