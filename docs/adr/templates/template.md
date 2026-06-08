# NUMBER. TITLE

Date: DATE

## Status

STATUS

<!-- Valid values: Proposed, Accepted, Superseded, Deprecated. The creator script
     fills this with the default; change it by hand if the decision is not yet
     ratified or has since been replaced (note the superseding ADR if so). -->

---

## Context

<!--
  Describe the forces at play: the technical constraints, domain requirements, and
  dependencies that make this decision necessary. State the scope — what this ADR
  does and does not cover. Write enough that a reader who wasn't in the room
  understands WHY a decision was needed, without yet stating the decision.
-->

## Decision Drivers

<!--
  List the criteria the options are judged against (one bullet each) — the things
  that actually decide the outcome (e.g. MSRV impact, no_std compatibility,
  maintenance burden, API ergonomics). One driver is fine; omit the section only
  if the decision is genuinely uncontested.
-->

* <!-- e.g. Must compile on the workspace MSRV without new transitive deps -->
* <!-- e.g. Must not pull `std` into the no_std crates -->

---

## Options Considered

<!--
  One block per option actually evaluated. Keep at least the two strongest. Each
  needs an honest pros/cons and a verdict, so the rejected paths are on record.
-->

### Option A — <!-- short name -->

<!-- One or two sentences on what this option is. -->

**Pros**:

* <!-- ... -->

**Cons**:

* <!-- ... -->

**Verdict**: <!-- Accepted | Rejected -->

### Option B — <!-- short name -->

<!-- One or two sentences on what this option is. -->

**Pros**:

* <!-- ... -->

**Cons**:

* <!-- ... -->

**Verdict**: <!-- Accepted | Rejected -->

---

## Decision

<!--
  State the chosen option plainly and give the core rationale in a few sentences.
  Do not re-litigate the options above — say what was decided and the single most
  important reason. This is the line future readers will quote.
-->

---

## Consequences

<!--
  The downstream effects of the decision, good and bad. Be honest about the
  negatives — an ADR that lists only upsides is not trusted. State what changes in
  the system and what other code/decisions are now constrained by this.
-->

* **Positive**: <!-- what this enables or simplifies -->
* **Negative**: <!-- what this costs or constrains -->
* **Neutral**: <!-- side effects that are neither, but worth recording -->

---

## References

<!--
  Link related ADRs, any design docs this spawns, downstream decisions, and the
  spec/issue references that informed it. If none, write "None".
-->

* <!-- e.g. [ADR-0003: facade pinning](../0003-...md), SDMX REST spec §6 -->
