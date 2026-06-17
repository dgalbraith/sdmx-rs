# 8. Target Version Policy for Serialisation

Date: 2026-05-21

## Status

Accepted

<!-- Valid statuses: Proposed, Accepted, Implemented, Superseded -->

---

## Summary

Isolate outbound SDMX wire-format serialisation logic from core domain models using a dedicated adapter crate (`sdmx-writers`), requiring callers to specify an explicit target version parameter to prevent cross-version compatibility bugs during output generation.

---

## Problem / Motivation

ADR-0008 establishes `ConstraintModel` as the canonical, version-agnostic domain type and commits to a bidirectional design. This requires a concrete implementation plan for the serialisation direction: `ConstraintModel` → SDMX wire format.

The inbound path (wire format → `ConstraintModel`) is implemented by `sdmx-parsers`. The outbound path has no crate home and no API design. Without an explicit record, the serialisation strategy risks being designed ad-hoc at implementation time, violating the adapter-pattern guardrail from ADR-0008.

Key questions that must be settled before Phase 2 implementation (the phase that implements `sdmx-parsers` and `sdmx-writers`):
* Where does serialisation code live — a method on the model, the parsers crate, or a dedicated crate?
* How is the target SDMX version expressed at the call site?
* What format enumeration does the API expose?
* How does the new crate integrate with the workspace facade in `sdmx-rs`?

### Decision Drivers

* **Separation of Concerns**: serialisation logic must be isolated from the domain model and from parsing, per the adapter-pattern guardrail in ADR-0008.
* **Caller Ergonomics**: The output version must be an explicit, required parameter. Defaults that silently choose a version are a source of subtle cross-version bugs.
* **Workspace Consistency**: The new crate must follow the same structural conventions as `sdmx-parsers`: `no_std + alloc` (see ADR-0005), re-exported through `sdmx-rs` under a `writers` feature flag (ADR-0003), version-pinned in the facade.
* **Early Scaffold Introduction**: The crate scaffold is introduced early (Phase 0) to establish workspace structure and type contracts. Concrete serialisation logic is then implemented in Phase 2 (the serialisation-engine phase, alongside `sdmx-parsers`); the early scaffold lets that work proceed against locked-down contracts rather than waiting on a later phase.

---

## Proposed Design

**We will implement SDMX serialisation in a dedicated `sdmx-writers` workspace crate, with the scaffold introduced early (Phase 0) to lock down type contracts and enable parallel development. Concrete serialisation implementation will proceed in Phase 2 (the serialisation-engine phase), alongside `sdmx-parsers`. The crate acts as a pure adapter from `ConstraintModel` — and eventually other domain types — to SDMX wire formats. Every serialisation entry point accepts an explicit `TargetVersion` parameter.**

The intended top-level API shape is (exact signatures subject to refinement at implementation time):

```rust
pub enum TargetVersion {
    V3_0,
    V3_1,
}

pub enum Format {
    Xml,
    Json,
    Csv,
}

// Primary entry point
pub fn serialize<T>(value: &T, format: Format, version: TargetVersion) -> Result<Vec<u8>, WriterError>
where
    T: SdmxSerialize;
```

`SdmxSerialize` is a sealed marker trait defined in `sdmx-types`, carrying no methods initially. This bounds the API to known domain types without importing concrete serialisation machinery into `sdmx-types`. If the trait acquires methods, the design must be reviewed against the `no_std + alloc` constraint in ADR-0005.

The `TargetVersion` enum is initially defined in `sdmx-writers`. If it later becomes useful across other crates (e.g. a query builder constructing version-specific `Accept` headers), it should be promoted to `sdmx-types`.

CSV serialisation (data payloads only) is supported alongside XML and JSON. The `Csv` format is available in the `Format` enum and produces SDMX-CSV v2 output conforming to the declared `TargetVersion` (3.0 or 3.1).

The `sdmx-writers` crate is introduced as a Phase 0 scaffold in `sdmx-rs/Cargo.toml` under the `writers` feature flag (see ADR-0003), with stub trait definitions and placeholder implementations. Full serialisation logic implementation occurs in Phase 2.

---

## Alternatives Considered

### Option A — serialisation as methods directly on `ConstraintModel`

Add `fn to_xml(&self, version: TargetVersion) -> ...` and equivalent methods directly to `ConstraintModel` in `sdmx-types`.

**Pros**:
* Single crate; no new workspace member.

**Cons**:
* Violates the adapter-pattern guardrail from ADR-0008: version-specific wire-format logic bleeds into the domain model.
* Forces `sdmx-types` to depend on serialisation libraries (e.g. `quick-xml`, `serde_json`), destroying its lightweight, `no_std + alloc` footprint.

**Verdict**: Rejected.

### Option B — serialisation housed in `sdmx-parsers` (inbound crate extended to bidirectional)

Extend `sdmx-parsers` to cover both parsing and serialisation.

**Pros**:
* No new workspace member.

**Cons**:
* `sdmx-parsers` acquires split responsibilities, complicating naming, documentation, and future division of labour.
* The crate name misleads contributors scanning for serialisation entry points.

**Verdict**: Rejected.

### Option C — Dedicated `sdmx-writers` crate

Introduce a new `sdmx-writers` workspace crate as the serialisation adapter, mirroring `sdmx-parsers` on the outbound path.

**Pros**:
* Clean separation: `sdmx-parsers` reads, `sdmx-writers` writes. Naming is unambiguous.
* `sdmx-types` zero-dependency invariant is preserved.
* Re-exported through `sdmx-rs` under a `writers` feature flag, consistent with the facade pattern in ADR-0003.
* Independent versioning and release lifecycle.

**Cons**:
* Additional workspace crate increases maintenance surface slightly.
* Release pipeline gains an extra topological step (see ADR-0003, amended).

**Verdict**: Accepted.

---

## Drawbacks / Trade-offs

* **Positive**: The adapter-pattern guardrail from ADR-0008 is enforced structurally: serialisation logic is physically isolated in a separate crate, making violations immediately visible at compile time.
* **Positive**: `sdmx-types` retains its zero-dependency, `no_std + alloc` profile.
* **Positive**: Consumers who only need parsing do not pay the compilation cost of `sdmx-writers`.
* **Positive**: Cross-version round-trips (parse 3.0, emit 3.1) are supported by design and testable end-to-end within `sdmx-writers`.
* **Negative**: Release pipeline requires publishing `sdmx-writers` before `sdmx-rs` on each release cycle (topological order: `types` → `parsers` → `writers` → `client` → `rs`).
* **Negative**: The `SdmxSerialize` trait boundary requires a design decision at implementation time about how much of the serialisation contract is expressed in `sdmx-types` vs. `sdmx-writers`.
* **Neutral**: `TargetVersion` starts in `sdmx-writers`; promotion to `sdmx-types` should be driven by a concrete cross-crate need, not anticipated speculatively.

---

## Questions & Resolutions

None.

---

## References

* [ADR-0003 — workspace membership and release order](../adr/0003-workspace-crate-facade-and-version-pinning-strategy.md)
* [ADR-0005 — no_std + alloc constraint that bounds sdmx-types and influences the SdmxSerialize trait design](../adr/0005-adopt-no-std-with-alloc-for-sdmx-types-and-sdmx-parsers.md)
* [ADR-0008 — canonical superset model, adapter-pattern guardrail, and TargetVersion policy](../adr/0008-model-sdmx-3-0-and-3-1-divergence-with-a-unified-constraintmodel.md)
* [ADR-0009 — quick-xml and serde_json usage in sdmx-parsers; the outbound path in sdmx-writers will reuse the same libraries](../adr/0009-use-quick-xml-and-serde-json-for-streaming-deserialisation.md)
* [ADR-0017 — SDMX-CSV stream parsing strategy; serialisation is the symmetric outbound path](../adr/0017-sdmx-csv-stream-parsing-strategy.md)
* [ADR-0018 — CSV format preference for data queries](../adr/0018-content-type-negotiation-and-parser-routing.md)
* [crates/sdmx-types/src/lib.rs](../../crates/sdmx-types/src/lib.rs)
