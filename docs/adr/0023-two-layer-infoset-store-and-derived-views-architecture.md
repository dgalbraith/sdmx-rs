# 23. Two Layer Infoset Store and Derived Views Architecture

Date: 2026-06-11

## Status

Accepted

---

## Context

`sdmx-types` exists to be the canonical, version-agnostic superset model of SDMX 3.0 and 3.1 (ADR-0008): every field reachable from any supported version must be representable, so that the adapter crates can round-trip without the target type's shape forcing information loss.

During design review a recurring defect class surfaced: decisions that bought *consumer convenience* by **collapsing a wire distinction in storage**. Instances included canonicalising `Dimension.position` into a mandatory integer (erasing stated-vs-derived), rejecting schema-valid blank localised values, rejecting the schema-valid `isExternalReference=false`+URL combination as "incoherent", identity-keyed `BTreeMap` storage (erasing element order and silently dropping duplicate-id entries that official samples actually exhibit), and storing schema-defaulted attributes as bare values (erasing stated-vs-absent). Each collapse had a locally defensible story; collectively they made lossless round-trip serialisation a per-field judgment call, and that leaves room for real losses to slip through.

Register decision D-0031 established the two-layer rule during the review; further discussions matured it to its final form, including the principle that **XSD defaulting is a view over the data, not the data itself**. This ADR promotes the matured rule to a foundational architectural commitment. Scope: the domain store in `sdmx-types` and the canonical parse → model → serialize pathway. Byte-level document editing is the companion commitment, ADR-0024.

## Decision Drivers

* The lossless canonical-superset guarantee (ADR-0008 guardrail #1) must be structural, not re-litigated per field.
* The rule must be **mechanical** — leaving no judgment room — because judgment room is the demonstrated loophole class.
* The central contract must be testable as a *total* property (no carve-outs): if the spec can express it, the store can round-trip it.
* Consumer ergonomics must survive, without being paid for out of stored information.
* `no_std` + `alloc` discipline and zero new dependencies.

---

## Options Considered

### Option A — Semantic round-trip (the model arbitrates equality)

Round-trip serialisation defined as "equal as domain values"; the model may canonicalise derivable or defaulted wire fields into a single in-memory form.

**Pros**:

* Tidier in-memory shapes; fewer `Option`s; ergonomic direct field access.

**Cons**:

* Circular: if the model collapses a distinction, the distinction is by definition "not semantic", so the guarantee holds vacuously — this is the latitude under which order, duplicates, and statedness were silently erased.
* Untestable as a total property; every collapse needs a carve-out in the round-trip test.

**Verdict**: Rejected.

### Option B — Byte-level store

The domain store reproduces input bytes exactly.

**Pros**:

* Absolute document integrity through the model itself.

**Cons**:

* Impossible for a typed domain model: comments and processing instructions are not schema content; sub-Infoset lexical accidents (attribute order, quote style, character references, prefix spelling) are invisible to conformant XML parsing. In practice, this forces the store to converge on a concrete syntax tree with types painted on.
* Pollutes every type with formatting metadata.

**Verdict**: Rejected for the domain store. The genuine requirement it serves (single edit ⟹ single diff) is met by the dedicated document-integrity pathway, ADR-0024.

### Option C — Two layers: Infoset store + derived views

**Layer 1 — the store** is a verbatim representation of the wire, *Infoset-complete within schema content*: element order, repeated elements (including schema-valid duplicates), the statedness of every optional attribute (defaulted and fixed alike — stated-vs-absent is stored as `Option`), and raw value lexemes are all preserved. `new()`/`Deserialize` reject **only** mechanically schema-invalid input — exactly what an XSD validator would reject (pattern facets, cardinalities, required members, fixed-value mismatches); mechanical invalidity is the *ceiling* of rejection, not a mandate (see the Decision's amended reject-line). Constraints the spec states only in prose become catalogued, non-destructive **lints**. **Layer 2 — views** provide every convenience the collapses used to buy: `effective_*()` accessors (applied defaults, derived positions), lookup views over ordered collections, coherence lints.

**Pros**:

* The rule is mechanical end to end; the loophole class is closed by construction.
* The round-trip contract is total and precisely stated (see Decision).
* No convenience is lost - it is relocated to views, which are non-destructive and freely revisable.

**Cons**:

* More `Option`s and `Vec`s than an ergonomics-first model; consumers reach effective values through views.
* Lookup over ordered collections is O(n) — acceptable at SDMX metadata cardinalities (tens to thousands), with additive index views available later if profiling demands.

**Verdict**: Accepted.

---

## Decision

**Adopt the two-layer architecture (Option C).** The governing rule, stated once: *friction is resolved by adding a view, never by collapsing the store.* The store preserves every distinction a schema-valid document can express; rejection at construction is licensed only by mechanical schema invalidity; prose rules are lints; **XSD defaulting and fixed-value fill-in are views over the data, not the data itself**. In the XSD literature's own vocabulary: **the store holds the pre-validation infoset (within schema content); schema assessment — the PSVI, where defaults and fixed values are filled in — is a Layer-2 view**, never the stored form.

**Reject-line amended 2026-06-11 (D-0059, the parsable-within-spec principle).** Mechanical schema invalidity is a *necessary* condition for rejection — the model never refuses anything an XSD validator would pass — but not by itself *sufficient*. Where the mechanically invalid datum is **structural** (cardinality, missing required members) or sits in an **identity- or grammar-bearing slot** (the identifier tiers, the validated lexical newtypes, fixed-value mismatches), rejection stands: the type's representation contract depends on it. Where it is a **value-level lexeme in a content slot the store can hold verbatim** — nothing structural depends on its grammar — a decision may rule it stored-plus-linted instead, because refusing representable data makes a judgment that belongs to the consuming application. First instance: the `LocalisedString` language key (D-0059) — blank/off-pattern stated tags are preserved verbatim, with well-formedness a catalogued lint. Existing rejection sites are unchanged unless individually re-ruled.

The canonical pathway's round-trip contract follows: `serialize(store(parse(doc)))` is equivalent to `doc` up to the wire format's *own* non-information layer (Canonical-XML-class lexical accidents and ignorable whitespace) — element order, duplicates, attribute statedness, and value lexemes round-trip **exactly**, and the output is byte-identical for inputs already in the writer's canonical form. Equality above that line is never the model's to define.

---

## Consequences

* **Positive**: Round-trip serialisation is structural; a single total store integrity property test ("schema-valid instance → store → serialize → canonically equal") realises the whole contract. Default-handling, ordering, and duplicate regressions become unrepresentable rather than reviewable.
* **Negative**: The store carries more `Option`s, statedness, and ordered collections than a convenience-first model would; every consumer-facing convenience must be deliberately provided as a view. This cost is paid once, structurally.
* **Neutral**: Register entry D-0031 is promoted to this ADR (body retained as the audit trail of the rule's discovery); D-0006 (BTreeMap throughout) and D-0022 (semantic round-trip) are superseded in the register; the store-wide application sweep is recorded in D-0051 (ordered collections) and D-0052 (attribute statedness).

---

## References

* [ADR-0008: Model SDMX 3.0 and 3.1 Divergence with a Unified ConstraintModel](0008-model-sdmx-3-0-and-3-1-divergence-with-a-unified-constraintmodel.md)
* [ADR-0024: Byte Preserving Document Integrity Pathway](0024-byte-preserving-document-integrity-pathway.md)
* [Decision register](../decisions.md): D-0031 (promoted here), D-0051, D-0052; superseded D-0006, D-0022
* [Design 0010 — SDMX Core Domain Types](../design/0010-sdmx-core-domain-types-design.md)
