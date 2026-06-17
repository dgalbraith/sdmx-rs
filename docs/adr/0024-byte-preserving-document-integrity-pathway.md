# 24. Byte Preserving Document Integrity Pathway

Date: 2026-06-11

## Status

Accepted

---

## Context

The Design 0010 exactness discussion (2026-06-11) surfaced an integrity requirement the canonical model alone cannot satisfy: **if a user loads a document, edits a single attribute, and writes it back, the output must differ from the input in exactly that attribute** — byte in == byte out everywhere outside the edit. Without this, a round-trip through the library generates unnecessary diffs for documents it did not meaningfully change, breaking version-control hygiene and the expectation that untouched data remains unmodified.

A typed domain model cannot deliver this, no matter how verbatim its store: XML comments and processing instructions are not schema content and have no home in domain types; sub-Infoset lexical accidents (attribute order within a tag, quote style, character-reference choice, namespace prefix spelling, encoding) are invisible to conformant XML parsing; and ignorable whitespace is formatting, not data. A store that preserved all of that would converge on a concrete syntax tree with types painted on. ADR-0023 therefore scopes the domain store to *Infoset-completeness within schema content* — the strongest claim a typed model can guarantee — and delegates the byte-level invariant here.

A direct ecosystem precedent is `toml` (semantic model) versus `toml_edit` (lossless document preserving untouched bytes). Both exist because the two capabilities serve genuinely different use cases.

## Decision Drivers

* The integrity invariant: byte in == byte out outside the edited region — held as a strict invariant, not an aspiration.
* The canonical domain model must stay free of formatting metadata (ADR-0023's scope).
* Phase-2 parser/writer architecture must be shaped *now*, before implementations exist to refactor.
* The two pathways must share one source of truth for schema semantics (the domain types).

---

## Options Considered

### Option A — Make the domain store byte-exact

Extend `sdmx-types` until re-serialisation reproduces input bytes exactly.

**Pros**:

* One representation; no second pathway.

**Cons**:

* Structurally impossible for comments/PIs and sub-Infoset lexical detail without non-conformant lexical parsing; in practice converges on a CST while polluting every domain type with formatting fields.

**Verdict**: Rejected.

### Option B — Canonical pathway only; accept the diff overhead

Declare canonical re-serialisation the only output form; document the diff overhead.

**Pros**:

* Nothing extra to build.

**Cons**:

* Fails the integrity invariant; a one-attribute edit rewrites the entire document. Unacceptable for registry-editing and review workflows.

**Verdict**: Rejected.

### Option C — A dedicated lossless document layer

A document-integrity pathway alongside the canonical one: a lossless representation of the source (retained original bytes with surgical patching, or a lossless syntax tree), with the typed domain model acting as a *view/projection* over it. Edits are expressed against the typed view and applied as localised splices to the underlying document; untouched regions pass through byte-identical.

**Pros**:

* Satisfies the invariant exactly, including comments, PIs, and formatting.
* Leaves the canonical pathway free to canonicalise (deterministic output, ADR-0023's contract).
* Proven architecture (`toml_edit`, lossless-CST editors).

**Cons**:

* A second document representation to build and maintain, with an edit-through-view API surface; substantial Phase-2+ engineering.

**Verdict**: Accepted.

---

## Decision

**Adopt Option C as an architectural commitment.** The workspace will provide two read/write pathways over one set of domain types: the **canonical pathway** (parse → Infoset Store → canonical serialisation; ADR-0023's contract) and the **document-integrity pathway** (lossless document layer with the domain model as a typed view; byte in == byte out outside edits). The domain store's obligation to this pathway is ADR-0023's Layer 1: representing every schema-content distinction verbatim so that the model never *forces* a document change. Implementation is deferred to the parser/writer phases, but their designs are constrained by this commitment from now on.

---

## Consequences

* **Positive**: The integrity invariant is satisfiable exactly, in its natural home; the canonical model stays clean; single-edit workflows produce single-line diffs.
* **Negative**: A second representation and an edit-projection API must be designed, built, and tested in Phase 2+; the lossless layer needs its own exactness test corpus.
* **Neutral**: Whether the document layer lives in `sdmx-parsers`, a dedicated crate, or a module is deliberately undecided here; the URN/reference-contract work (Design 0010 W-22) and this pathway will meet in the Phase-2 parser design.

---

## References

* [ADR-0008: Model SDMX 3.0 and 3.1 Divergence with a Unified ConstraintModel](0008-model-sdmx-3-0-and-3-1-divergence-with-a-unified-constraintmodel.md)
* [ADR-0023: Two Layer Infoset Store and Derived Views Architecture](0023-two-layer-infoset-store-and-derived-views-architecture.md)
* Precedent: the `toml` / `toml_edit` crate pair (semantic model vs lossless editable document)
* [Design 0010 — SDMX Core Domain Types](../design/0010-sdmx-core-domain-types-design.md) (W-17 verbatim representation discussion)
