# 8. Model SDMX 3.0 and 3.1 Divergence with a Unified ConstraintModel

Date: 2026-05-17

## Status

Accepted

---

## Context

The SDMX 3.x specification series contains structural changes between minor versions. A prominent divergence between **SDMX 3.0** and **SDMX 3.1** is the modeling of data constraints:
* **SDMX 3.0** uses a unified `DataConstraint` structure. It carries a required `role` attribute (`ConstraintRoleType`) to differentiate between `Allowed` (reporting restrictions: what codes are allowed to be uploaded) and `Actual` (availability constraints: what data actually exists in the database).
* **SDMX 3.1** refactors this area, restricting `DataConstraint` strictly to allowed-content semantics (eliminating the `role` attribute) and introducing a brand new **`AvailabilityConstraint`** type to represent actual data holdings.

We must decide how to model this structural divergence in `sdmx-types` to prevent versioning complexity from leaking into the downstream public API of `sdmx-client` and user application code.

## Decision Drivers

* **Consumer Ergonomics**: Protect downstream users from having to branch their code based on whether a server serves SDMX 3.0 or 3.1 metadata.
* **Separation of Concerns**: Keep wire-format version detection and payload transformation isolated within the serialization layer (`sdmx-parsers`).
* **Domain Model Consistency**: Represent the semantic difference between reporting rules and database availability cleanly.

---

## Options Considered

### Option A — Version-Specific Domain Structs

Exposing distinct namespaces or types for each minor specification release (e.g., `sdmx_types::v3_0::DataConstraint`, `sdmx_types::v3_1::DataConstraint`, and `sdmx_types::v3_1::AvailabilityConstraint`).

* **Pros**:
  * Maps 1-to-1 with the respective XML schemas, simplifying parser code.
* **Cons**:
  * High API friction. Downstream application developers must write dual code paths to support both 3.0 and 3.1 servers.
  * Leaks protocol versioning details across all layers of the library.
**Verdict**: Rejected.

### Option B — Unified, Version-Agnostic Enum

Defining a unified `ConstraintModel` enum in the core `sdmx-types` library that abstracts both specification versions under a single type.

```rust
pub enum ConstraintModel {
    Data(DataConstraint),
    Availability(AvailabilityConstraint),
}
```

* **Pros**:
  * Unifies the API. Callers interact with a single representation regardless of whether the source wire format is SDMX 3.0 or 3.1.
  * Isolates the mapping logic. The parsing engine (`sdmx-parsers`) detects the incoming schema version (e.g. via XML namespace or JSON version attributes) and normalizes the payload to hydrate this enum.
  * Enables sharing common behaviors (like query matching or filter intersection) through trait implementations on `ConstraintModel`.
* **Cons**:
  * Requires custom parsing routing logic in `sdmx-parsers` to map the differing tags.
**Verdict**: Accepted.

---

## Decision

**We will model the SDMX 3.0 and 3.1 constraint divergence using a unified, version-agnostic `ConstraintModel` enum in `sdmx-types`. The `sdmx-parsers` crate will perform runtime schema detection and normalize version-specific payloads onto this model.**

### Symmetry-First Design Commitment

To ensure future-proof interoperability and a complete, bidirectional library, we additionally commit to the following three architectural guardrails:

#### 1. Canonical Superset Model

`ConstraintModel` is the canonical, version-agnostic source of truth. It is designed as a superset of all SDMX 3.x structural requirements: every field reachable from any supported version must be representable in the model. The model itself performs no version conversion — it is pure data; the burden it carries is purely structural completeness, so that when the adapter crates (`sdmx-parsers`, `sdmx-writers`) convert to and from it, no version-specific information is forced to be discarded by the *target type's* shape. The `Data(DataConstraint)` / `Availability(AvailabilityConstraint)` split, together with `DataConstraint.role: Option<ConstraintRole>` (D-0037), satisfies this: the 3.1 type split is the enum discriminant, and the 3.0 `role` attribute (`Allowed`/`Actual`) — which the discriminant alone cannot encode, since both 3.0 roles live on the *same* wire type — is carried as a verbatim superset field (`None` ⟺ the 3.1 wire, which has no such attribute). Variant names reflect what each type *is* in the domain (`Data` = data constraint on a dataflow; `Availability` = actual holdings response) rather than the 3.0 wire-format `role` attribute values (`Allowed`/`Actual`), keeping the enum vocabulary version-agnostic.

#### 2. Adapter Pattern for Parsers and Serializers

`sdmx-parsers` and the future `sdmx-writers` crate are pure adapters to and from the canonical model. All wire-format, version-specific logic (e.g. "if target is 3.0, write the required `role` attribute; if 3.1, write an `AvailabilityConstraint` block") belongs exclusively in the adapter crates. `ConstraintModel` remains clean of all version-specific syntax.

#### 3. Version-Aware serialization via `TargetVersion`

Every serialization entry point must accept an explicit `TargetVersion` parameter. The caller selects the output version; the adapter handles all mapping logic internally. The conceptual API is:

```rust
let xml = model.serialize(Format::Xml, TargetVersion::V3_1)?;
```

This serialization logic is implemented in the `sdmx-writers` crate.

---

## Consequences

* **Positive**: Downstream users query constraints using a single, unified domain structure regardless of the server's SDMX version.
* **Positive**: Isolated specification updates. If a future SDMX 3.2 changes constraints again, only `sdmx-parsers` (and eventually `sdmx-writers`) needs updating; the domain model remains stable.
* **Positive**: The canonical superset guarantee makes cross-version round-trips (parse 3.0, emit 3.1) lossless by design.
* **Negative**: Parser code in `sdmx-parsers` must explicitly handle version detection and dynamic routing of structural payloads.
* **Negative**: serialization requires a `TargetVersion` decision at every call site; callers cannot omit this choice. A sensible default (e.g. latest supported version) may be provided as a convenience but must be explicit in the API contract.
* **Neutral**: The `sdmx-writers` crate introduces a new workspace member and an additional release pipeline step (see ADR-0003, amended).

---

## References

* [`ARCHITECTURE.md` — SDMX 3.0 vs 3.1 Spec Divergence (ConstraintModel)](../../ARCHITECTURE.md#sdmx-30-vs-31-spec-divergence-constraintmodel)
* `crates/sdmx-types/src/lib.rs`
* `crates/sdmx-parsers/src/lib.rs`
* ADR-0003 — workspace membership and release order including `sdmx-writers`
* [Design Document 0008](../design/0008-target-version-policy-for-serialization.md) — implementation-level record: `sdmx-writers` crate, `TargetVersion` API, and adapter design
