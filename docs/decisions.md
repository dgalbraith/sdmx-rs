# Decision Register

## What Is the Decision Register?

The decision register captures **scoped observations and their direct consequences** — findings that emerged from reading the spec, working through Rust language constraints, or resolving a specific domain modelling choice. Each entry is small enough that a full ADR would be disproportionate, and specific enough that it belongs to one area rather than the whole architecture.

Entries span SDMX specification observations, Rust language behaviour, and domain modelling choices.

## When to Add an Entry

Add a register entry when:
- A spec reading produces a concrete field-level or type-level consequence
- A Rust language constraint forces a specific implementation choice
- A domain modelling question is resolved and the resolution should be citable

Adding an entry is a three-step operation:
1. Add a row to the [Entry Index](#entry-index) — ID, Area, and one-line title
2. Add the full entry under [## Entries](#entries) using the template below
3. Update `<!-- Next ID: -->` in the index footer to the following ID (e.g. D-0005 → D-0006)

## Relationship to ADRs and Design Documents

- **ADRs** record cross-cutting architectural commitments that constrain the whole system and are expensive to reverse. If a decision introduces a new cross-cutting constraint, write an ADR in `docs/adr/` instead.
- **Design documents** explore the design space before a decision is made. A register entry may cite a design doc as the source of the discussion that produced it.
- **Entries that cite an ADR as their source** signal that they are consequences of that architectural decision, not independent commitments.

See [ADRs](adr/README.md) and [Design Documentation](design/README.md).

## Entry Template

**Step 1** — add this row to the [Entry Index](#entry-index) (replace NNNN, Area, and Title), then update the `<!-- Next ID: -->` footer:

~~~markdown
| [D-NNNN](#d-nnnn) | Area | Title |

<!-- Next ID: D-NNNN+1 -->
~~~

**Step 2** — add the full entry under [`## Entries`](#entries). Copy the block below and remove inapplicable rows and sections. `Spec ref`, `Source`, `Related`, `Rationale`, and `Consequences` are omitted entirely when not applicable — do not leave them blank.

~~~markdown
### D-NNNN — Title <!-- Short imperative title: what was decided, not what was observed -->

| **Area**     | | <!-- Domain area: Annotation, Identifiers, Collections, Serde, etc. -->
| **Phase**    | | <!-- Milestone when decided: M0, Phase-1, Phase-2, etc. -->
| **Status**   | | <!-- Active · Superseded(D-NNNN) · Deprecated -->
| **Keywords** | | <!-- Comma-separated conceptual tags, no backticks: validation, serde, no_std -->
| **Spec ref** | | <!-- Link to the XSD or spec section that is the primary evidence -->
| **Source**   | | <!-- Link to the design doc or ADR where this was discussed, if any -->
| **Related**  | | <!-- [D-NNNN](#d-nnnn) links to closely related decisions, if any -->

**Observation**: What the spec, language, or runtime does that forces a choice.

**Decision**: The ruling — what the code does as a result. One to three sentences.

**Rationale**: Why this option over the alternatives. Omit if the decision is self-evident from the observation.

**Consequences**: Downstream implications, open questions, or deferred follow-ups. Omit if none.
~~~

> **Superseded entries** retain their full body. Add a blockquote at the top:
> `> **Superseded by [D-NNNN](#d-nnnn)**`
>
> **In the Entry Index**, mark the title to signal supersession status:
> - Fully superseded: `~~title~~` — displayed as ~~title~~
> - Partially superseded: `title †[D-NNNN](#d-nnnn)` — still live but amended; link to the superseding entry

## Entry Index

| ID                | Area                        | Title                                                                                                                                         |
|-------------------|-----------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------|

<!-- Next ID: D-0001 -->

## Entries
