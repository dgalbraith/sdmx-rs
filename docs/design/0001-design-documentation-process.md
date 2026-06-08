# 1. Design Documentation Process

Date: 2026-05-23

## Status

Accepted

---

## Summary

Establish a design documentation system for exploring and ratifying detailed designs of systems, components, and features before architectural decisions are made or implementation begins.

---

## Problem / Motivation

ADRs are effective for recording *architectural decisions*—but design work often requires exploration and detailed planning *before* a decision is finalized. We need a way to:

- Explore complex design spaces and evaluate alternatives during planning
- Capture detailed design rationale, constraints, and trade-offs for a specific component or system
- Document designs that may evolve as implementation proceeds
- Capture unresolved questions and implementation unknowns
- Build educational material showing exemplar design patterns

Design documents serve this purpose: they capture **design exploration and planning**, while ADRs record **decisions that have been made**. Design docs are also more detailed and mutable than ADRs.

---

## Proposed Design

Design Documents follow an RFC-inspired structure, capturing design exploration and planning:

**Structure**:
- Numbered like ADRs (`0001-title.md`) for consistency and clarity
- Status tracking (Proposed → Accepted → Implemented → Superseded)
- Clear sections: Problem, Proposed Design, Alternatives, Trade-offs, Questions & Resolutions

**Philosophy**:
- **Planning-focused**: Designs explore "how might we build this?" during design and planning phases
- **Mutable**: Designs evolve as you learn more; refinement during planning is expected
- **Detailed**: Designs capture rationale, constraints, and trade-offs in depth
- **Before decisions**: Designs inform and precede ADRs (which record decisions that have been made)

**Lifecycle**:
1. Author creates design doc using `templates/template.md`
2. Design explores the problem space, alternatives, and trade-offs
3. Design is refined and accepted as the working approach
4. Implementation proceeds based on the design
5. Once key decisions are made and locked in, those decisions are documented as ADRs (which reference the design doc)
6. Design docs can be superseded by newer designs or marked as implemented when implementation is complete

---

## Alternatives Considered

### Alternative A — Only ADRs, no Design Docs
Rely exclusively on ADRs for all architectural documentation (use ADRs for both planning and decision recording).

**Pros**:
- Simpler system (only one document type)
- All decisions are locked and immutable

**Cons**:
- ADRs are retrospective (explain decisions already made), not prospective (planning before building)
- Loses the design exploration and rationale that led to decisions
- Harder to capture unresolved questions and evolving thinking during design
- Discourages detailed trade-off analysis before implementation

**Verdict**: Rejected. ADRs are for decisions after they're made; we need a way to document design exploration before decisions are locked.

### Alternative B — Informal Wiki / Design Docs
Use informal markdown files in `/docs/design/` without structure or templating.

**Pros**:
- Maximum flexibility
- Low friction for experimentation

**Cons**:
- Inconsistent structure makes designs hard to navigate
- No clear lifecycle (when is a design "done"?)
- Difficult to track status (proposed vs. accepted vs. superseded)
- Harder to link designs to resulting ADRs

**Verdict**: Rejected. Need enough structure for navigation and auditability.

### Alternative C — Structured Design Docs with Templates (Chosen)
Separate design documents (prospective, provisional) from ADRs (retrospective, definitive). Designs explore before decisions; ADRs lock in decisions afterward.

**Pros**:
- Clear separation of concerns: Design explores, ADR decides
- Designs can evolve without invalidating history
- Captures detailed rationale before decisions are made
- Version-controlled and auditable like ADRs
- Numbered format mirrors ADRs, keeping both systems familiar
- Can graduate to formal RFCs for community engagement
- Design → Implementation → ADR progression is clear

**Cons**:
- Requires learning another template and system (mitigated by clear guidance)
- More documentation overhead (worth it for complex designs)

**Verdict**: Accepted.

---

## Trade-offs

**Planning vs. Decision Recording**: Design docs capture planning and exploration; ADRs record decisions that have been made. This requires documenting in two phases, but the benefit is that both design rationale and decision history are preserved.

**Structure vs. Flexibility**: We enforce a template for consistency and searchability, but the template is flexible enough for lightweight designs (2 pages) and detailed specs (20+ pages).

**Two Systems vs. One**: Maintaining both ADRs and designs adds complexity, but each serves a distinct purpose with different lifecycles. The relationship between them is clear: designs inform planning, implementation proceeds, ADRs record final decisions.

**Overhead vs. Quality**: Writing a design doc takes effort, but for complex features it's worthwhile. The design becomes the blueprint for implementation, and the resulting ADRs are shorter and more focused because they reference the design doc.

---

## Questions & Resolutions

- **[Resolved]** - **Numbering scheme**: Should design doc numbering restart, or continue from ADRs (0020+)?

  *Answer:* Numbering will restart at 0001. Design docs and ADRs will maintain separate sequence streams. The directory separation (`docs/design/` vs `docs/adr/`) and file prefixes (`design-0001` vs `adr-0001`) provide sufficient disambiguation.

- **[Resolved]** - **Lifecycle closure**: When should a design be "closed" or superseded?

  *Answer:* A design is marked "Implemented" (closed) when a Pull Request fulfilling its core architectural intent is merged to `main`. If a fundamental assumption changes after implementation, a *new* design doc is created, and the old one is marked "Superseded".

- **[Resolved]** - **Implementation linkage**: How should we link designs to implementations (PRs, branches)?

  *Answer:* This will be recorded at the PR level. We will add a section to the Pull Request template asking the author to link the relevant documents if they exist.

- **[Open]** - **Community RFCs**: Will design docs be used as the basis for community RFCs?

  *Notes/Thoughts*: Currently leaning towards yes, but need to define the threshold for when an internal doc graduates to a public RFC.

---

## References

* [ADR-0001: Record Architecture Decisions](../adr/0001-record-architecture-decisions.md)
* [Rust RFC Process](https://github.com/rust-lang/rfcs) — design docs are inspired by RFC structure
* [Design Documentation Template](templates/template.md)

---

## Notes for Implementation

- Design docs should be managed with the same care as ADRs (version-controlled, immutable)
- Mirror the ADR scripts (add-design.sh, verify-design.sh) for consistency
- Consider creating a design doc for the type system as a Phase 1 exemplar
- Future community RFCs can reference or build from design docs
