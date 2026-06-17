# 21. Domain Invariant Validation and Encapsulation Strategy

Date: 2026-06-11

## Status

Accepted

---

## Context

The `sdmx-types` domain model carries mechanical invariants taken directly from the XSDs: identifier grammars (per-artefact lexical tiers), non-empty lists (`Dimension+`, chosen choice arms), bounded cardinalities (`CubeRegion maxOccurs="2"`), and fixed-value attributes. Three intertwined questions determine whether those invariants actually hold for every value in every program: *where* validation runs, *how* serde deserialisation interacts with constructors, and *which* fields are public.

Mechanically, serde's derived `Deserialize` constructs structs directly, field by field, bypassing any user-defined constructor. Consequently, a type whose invariant is enforced only in `new()` bypasses those checks during deserialisation if it uses the derive macro. Conversely, invariant-free types gain nothing from custom boilerplate.

Register decisions D-0004 (validate at construction), D-0005 (private fields + custom `Deserialize` for invariant-bearing types), and D-0017 (the field-visibility rule) settled this during M0 as three separate records. This ADR consolidates and promotes them as one cross-cutting strategy, since together they constitute the crate's construction contract — relied on by every parser, every writer, and every hand-construction site.

## Decision Drivers

* Invariants must hold for **all** callers — hand construction, serde-driven deserialisation, and future streaming accumulators — identically.
* Serde's derive-bypasses-constructors behaviour must be explicitly addressed, not obscured.
* Invariant-free carrier types should remain transparent (no boilerplate without functional benefit).
* `no_std` + `alloc`; no new dependencies; validators hand-rolled.

---

## Options Considered

### Option A — Defer validation to parsers or query time

Domain types accept raw values; parsers (or later operations) validate.

**Pros**:

* Thinnest possible types; no custom serde code.

**Cons**:

* Invalid domain values become constructible in user code; every downstream consumer must re-validate or trust unverified input.
* Splits the contract: the parser path and the hand-construction path enforce different rules.

**Verdict**: Rejected.

### Option B — Blanket encapsulation (private fields and builders everywhere)

Every type gets private fields, a validating constructor, and a custom `Deserialize`.

**Pros**:

* Uniform; nothing is ever missed.

**Cons**:

* Unnecessary boilerplate for transparent carriers where all field combinations are valid; visitor boilerplate scales with the whole type count rather than the invariant count.

**Verdict**: Rejected.

### Option C — Single write path, scoped by invariant ownership

Invariant-bearing types use **private fields and `Result`-returning `new()`** as the single write path; their `Deserialize` is **custom**, accumulating fields via a visitor and calling the same `new()`. Invariant-free types are **transparent carriers**: public fields, derived `Deserialize`. The boundary is defined by a strict rule: derived `Deserialize` delegates to each field's own impl and is therefore *correct* whenever every field enforces its own invariants — a custom impl is required only where a type owns an invariant **stricter than its fields enforce** (tightened identifier tiers, position-dependent representation rules) or **between fields** (key-derived-from-id, mechanical non-empty collections).

**Pros**:

* Enforcement is uniform across all construction paths; effort scales with invariants, not types.
* The boundary is mechanical (the strict rule), making type promotion or demotion an objective check rather than a stylistic choice.

**Cons**:

* Custom visitor impls are real boilerplate for the invariant-bearing set, and the strict rule must be re-applied whenever a type gains an invariant (a constraint that requires explicit documentation).

**Verdict**: Accepted.

---

## Decision

**Adopt Option C.** Validation runs in `new()` on the single write path; serde reaches `new()` through custom `Deserialize` impls exactly where an invariant exists that field-level impls cannot enforce; field visibility follows invariant ownership — private where mutation could violate an invariant the type owns, public where the type is a transparent carrier. Streaming parsers added in later phases call the same constructors, requiring no refactor of the domain types.

---

## Consequences

* **Positive**: An invalid value is unconstructible; wire input and programmatic input are subject to identical enforcement; transparent carriers stay ergonomic; the `Serialize` direction needs no special handling anywhere.
* **Negative**: Custom visitor boilerplate for invariant-bearing types; adding an invariant to a derived type requires converting it (or its container) to a custom impl — a constraint that requires explicit documentation.
* **Neutral**: Register entries D-0004, D-0005, and D-0017 are promoted to this ADR (bodies retained as audit trail). The per-artefact identifier tiers (D-0023) and the two-layer store rule (ADR-0023) define *what* is validated; this ADR defines *where and how*.

---

## References

* [ADR-0023: Two Layer Infoset Store and Derived Views Architecture](0023-two-layer-infoset-store-and-derived-views-architecture.md)
* [Decision register](../decisions.md): D-0004, D-0005, D-0017 (promoted here); D-0019, D-0023 (applications of the strategy)
* [Design 0010 — SDMX Core Domain Types](../design/0010-sdmx-core-domain-types-design.md) §7 (the construction contract)
* [serde derive internals](https://serde.rs/derive.html)
