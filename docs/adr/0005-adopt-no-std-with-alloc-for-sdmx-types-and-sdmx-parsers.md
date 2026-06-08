# 5. Adopt `#![no_std]` with `alloc` for `sdmx-types` and `sdmx-parsers`

Date: 2026-05-17

## Status

Accepted

---

## Context

The workspace is partitioned into four crates in a strict unidirectional dependency chain:
`sdmx-types` → `sdmx-parsers` → `sdmx-client` → `sdmx-rs` (facade). The two
innermost crates — `sdmx-types` and `sdmx-parsers` — carry no transport or
async dependencies and have no inherent requirement for a hosted operating
system environment.

A foundational decision must be made about what execution environment these
two crates may assume. Three meaningful targets exist for Rust crates:

1. **`std`-full**: Requires a hosted OS (threads, filesystem, network sockets,
   process environment).
2. **`no_std` + `alloc`**: Requires only a heap allocator. No OS services.
   Compiles for WASM, embedded RTOS targets, and custom allocator environments.
3. **`no_std` without `alloc`**: Requires only a stack. No heap allocation at
   all. Applies exclusively to bare-metal firmware without a global allocator.

The choice for the two inner crates is architecturally significant:

- It determines which downstream consumers can adopt the library without
  modification.
- It gates the allowed dependency set for `sdmx-types` and `sdmx-parsers` —
  any dependency that transitively pulls in `std` will break `#![no_std]`
  compilation silently unless the WASM check gate is enforced.
- It directly affects whether the library is usable in WASM environments (a
  primary portability target given SDMX's role in data infrastructure tooling).

This decision also implicitly defines the lower boundary: bare-metal targets
without a global allocator are out of scope, because `sdmx-types` fundamentally
requires heap-allocated collections (`Vec`, `String`, `BTreeMap`) to represent
SDMX domain structures such as codelists, annotations, and concept schemes.

---

## Decision Drivers

- **WASM Portability**: SDMX data processing is a natural fit for WebAssembly
  runtimes (browser-side data exploration, serverless data pipelines). The
  inner crates must compile to `wasm32-unknown-unknown` without modification.
- **Dependency Discipline**: Adopting `#![no_std]` in `sdmx-types` and
  `sdmx-parsers` provides a compile-time enforcement mechanism that prevents
  accidental introduction of `std`-only dependencies into the domain core.
  If a new dependency pulls in `std`, the WASM check gate will fail.
- **Dependency Honesty**: Both `serde` and `thiserror` (the only external
  dependencies permitted in `sdmx-types`) explicitly support `no_std` + `alloc`
  compilation. `quick-xml` and `serde_json` (used in `sdmx-parsers`) also have
  `alloc`-only feature modes. The constraint is achievable at zero API cost.
- **Exemplar Value**: `#![no_std]` with `alloc` is an idiomatic, deliberate
  portability statement. It documents intent, not just implementation, and
  teaches correct dependency scoping in a public library.
- **Crate Boundary Isolation**: An error type or domain model in `sdmx-types`
  must never transitively force `reqwest` or `tokio` into a consumer that only
  wants to work with parsed data structures. `#![no_std]` makes this boundary
  structurally unbreakable.

---

## Options Considered

### Option A — `std`-full (No `#![no_std]` declaration)

Allow `sdmx-types` and `sdmx-parsers` to assume a full hosted environment by
default. No explicit `#![no_std]` attribute. Dependencies may transitively
pull in `std` without warning.

**Pros**:

- Zero additional constraint on the dependency set.
- No WASM-specific configuration required.
- Simplest initial implementation — no `alloc` imports needed, `std` collections
  used directly.

**Cons**:

- Eliminates WASM portability for the domain core. Any WASM consumer of
  `sdmx-rs` without the `client` feature enabled would still be blocked
  from using `sdmx-types` if it assumes `std`.
- Removes the compile-time guard against accidental `std` leakage into the
  domain layer. A future contributor adding a `std`-only dependency silently
  breaks the portability guarantee with no immediate feedback.
- Inconsistent with the stated exemplar goal. A Rust library that unconditionally
  claims "minimal dependencies" while silently pulling in `std` services is
  misleading.
- Cannot be reversed without a breaking API change once consumer code has
  been written against it.

**Verdict**: Rejected. Convenience at the cost of portability and architectural
honesty is the wrong trade for a pre-1.0 library where the constraint costs
nothing to adopt now.

---

### Option B — `#![no_std]` without `alloc` (Pure Stack-Only)

Declare `#![no_std]` in both `sdmx-types` and `sdmx-parsers` and prohibit all
heap allocation.

**Pros**:

- Maximum portability. Compiles on bare-metal firmware targets without a
  global allocator.

**Cons**:

- Functionally impossible for `sdmx-types`. SDMX domain types inherently
  require owned, variable-length collections. A `Codelist` is a list of code
  items. An `Annotation` carries a text string. Neither can be modelled without
  heap allocation.
- Would require replacing all `Vec`, `String`, and `BTreeMap` uses with
  static arrays and fixed-size buffers, destroying API ergonomics and
  correctability for a library that is not targeting embedded firmware.
- All realistic downstream consumers of an SDMX library operate in
  environments with a heap allocator. Optimising for the vanishingly rare
  exception at the cost of the common case is an inversion of priorities.

**Verdict**: Rejected. Technically incompatible with the domain model
requirements of `sdmx-types`.

---

### Option C — `#![no_std]` + `extern crate alloc`

Declare `#![no_std]` and explicitly import the `alloc` crate in both
`sdmx-types` and `sdmx-parsers`. Use `alloc::vec::Vec`, `alloc::string::String`,
`alloc::borrow::Cow`, `alloc::collections::BTreeMap`, `alloc::collections::BTreeSet`,
and related types throughout. All external dependencies are constrained to
`no_std` + `alloc`-compatible versions.

**Pros**:

- Full WASM portability for the domain core and serialization engine.
- Provides a structural compile-time guard: the WASM check gate
  (`cargo check -p sdmx-types --target wasm32-unknown-unknown` and
  `cargo check -p sdmx-parsers --target wasm32-unknown-unknown`) enforces
  the constraint on every `just verify` invocation.
- All currently selected and planned dependencies (`serde`, `thiserror`,
  `quick-xml`, `serde_json`) are fully `alloc`-compatible without feature
  gymnastics.
- Achieves the portability goal with zero API-surface cost to downstream
  consumers. Library users import `sdmx_types::Codelist` — they never
  interact with the `alloc` internals.
- `sdmx-client` remains unrestricted (`std`-full), as it requires Tokio and
  reqwest. The `#![no_std]` boundary is enforced only where it is actually
  meaningful.

**Cons**:

- Requires `extern crate alloc;` declarations and `alloc::` prefixed imports
  rather than `std::` throughout `sdmx-types` and `sdmx-parsers`. Minor
  cognitive overhead, especially for contributors unfamiliar with `no_std`.
- Bare-metal targets without a global allocator remain out of scope. This is
  an explicit, accepted limitation (see Consequences).

**Verdict**: Accepted.

---

## Decision

**`sdmx-types` and `sdmx-parsers` will declare `#![no_std]` and use `extern crate alloc` for all heap-allocated types. `sdmx-client` and the `sdmx-rs` facade retain `std` availability as required by the async transport layer.**

The constraint is enforced structurally via two `cargo check` invocations in the
`check-wasm` Justfile recipe targeting `wasm32-unknown-unknown`, which is part
of the `just verify` quality gate.

Bare-metal targets without a global allocator are explicitly out of scope. The
`alloc` requirement is documented in `ARCHITECTURE.md` as a known minimum
platform constraint.

### Collection Type Strategy: BTreeMap and BTreeSet

Within the `no_std` + `alloc` design space, `BTreeMap` and `BTreeSet` are the
preferred collection types for SDMX metadata. This is not simply a consequence
of `HashMap`'s unavailability in `alloc`, but a deliberate architectural choice
aligned with SDMX's operational profile:

**Deterministic Ordering**: SDMX structures (codelists, concept schemes, dimension
definitions) benefit from canonical, sorted representation. `BTreeMap`'s inherent
ordering guarantees consistent serialization, which is essential for data
validation and interchange.

**Predictable Performance**: For typical SDMX workloads, `BTreeMap` lookups on
dimension sets (2–10 entries) and moderate codelists (100–5,000 entries) exhibit
superior cache locality compared to `HashMap`. The O(log n) algorithmic complexity
is mitigated by contiguous node storage and reduced hashing overhead. Worst-case
lookup latency is more predictable, which is valued in data pipeline and WASM
execution contexts where jitter is costly.

**Memory Efficiency**: SDMX metadata is often immutable post-load. `BTreeMap` requires
no load-factor tuning or bucket pre-allocation, resulting in tighter memory footprints
than equivalent `HashMap` usage.

**Phase 2 Validation**: Benchmark work (planned) will validate this assumption
across cold cache, warm cache, and real-world SDMX reference codelist sizes to
confirm the performance profile meets expectations at scale.

---

## Consequences

- **Positive**: `sdmx-types` and `sdmx-parsers` compile to
  `wasm32-unknown-unknown` without modification. WASM consumers can use the
  domain model and parser layer without enabling the `client` feature.
- **Positive**: Any future dependency addition to `sdmx-types` or
  `sdmx-parsers` that transitively requires `std` will cause the WASM quality
  gate to fail immediately, surfacing the violation before it reaches CI.
- **Positive**: The boundary is one-way and permanent. No API-breaking change
  is required to adopt `no_std` now; relaxing it later would be breaking.
  The cost of adopting it pre-1.0 is zero.
- **Positive**: `BTreeMap` and `BTreeSet` provide deterministic ordering and
  predictable cache performance for typical SDMX metadata scales. This
  architectural choice will be validated through Phase 2 benchmarking (cold cache,
  warm cache, real-world codelist sizes). If future use cases require O(1) lookups
  on very large codelists (>100k entries), a feature-gated `phf`-based alternative
  can be added without breaking the core `no_std` design.
- **Neutral**: `alloc::` prefixed imports are required instead of
  `std::` throughout the two inner crates. Standard IDE tooling and
  `rust-analyzer` handle this transparently.
- **Negative**: Bare-metal targets without a heap allocator cannot use
  `sdmx-types` or `sdmx-parsers`. This is an accepted limitation — no
  realistic SDMX use case operates on a system without dynamic memory.

---

## References

- `ARCHITECTURE.md` — Crate Dependency Graph, `sdmx-types` Design Constraints
- [ADR-0006](0006-standardise-error-handling-with-thiserror-per-crate.md): Error handling with `thiserror`; `no_std` compatibility of error types is a dependency of this decision
- [ADR-0009](0009-use-quick-xml-and-serde-json-for-streaming-deserialization.md): `quick-xml` and `serde_json` `alloc`-mode usage
- `crates/sdmx-types/src/lib.rs`
- `crates/sdmx-parsers/src/lib.rs`
- `Justfile` — `check-wasm` recipe
