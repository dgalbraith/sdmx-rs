# Documentation Standards

This guide documents how to write effective documentation for `sdmx-rs`. The goal is clarity: explain *why* decisions are made, what the design is, and how to use it—without creating busywork.

---

## Documentation Types and Their Purpose

Documentation flows naturally from design exploration through user guidance. Each type answers a specific question for a specific audience.

### Design Documents

**Question answered**: What could this design be? What are the alternatives and trade-offs?

**Audience**: Implementers, architects, decision-makers evaluating approaches

**Where**: `docs/design/`

**Content**:
- Problem statement
- Proposed approaches and exploration
- Alternatives considered
- Trade-offs and constraints
- Implementation strategy
- Unresolved questions

**Status**: Often untracked working material. May be committed as reference when implementation begins.

**When to write**: Before deciding on an approach; when design is complex enough to benefit from structured exploration.

---

### Architecture Decision Records (ADRs)

**Question answered**: Why did we choose this approach over alternatives?

**Audience**: Contributors, maintainers, anyone asking "why X over Y?"

**Where**: `docs/adr/`

**Content**:
- Problem statement
- Alternatives considered
- Chosen approach and rationale
- Trade-offs accepted
- Consequences
- Related decisions

**When to write**: A design choice has clear alternatives with distinct trade-offs; the decision affects multiple parts of the codebase or future work.

---

### Code Comments and Rustdoc

**Question answered**: How do I use this? What's non-obvious about its behavior?

**Audience**: Developers reading or maintaining the code

**Where**: Struct/function comments (`///`), inline comments (`//`)

**Content**:
- What the function/type does (rustdoc)
- Why behavior is non-obvious (comments)
- Links back to design decisions and design exploration
- Usage patterns, gotchas, clone semantics, thread-safety

**Guideline**: Explain *why*, not *what*. Don't over-comment simple code. `struct UserId(u64)` needs just a one-line doc comment.

**When to add**: Any type or function with non-obvious behavior. Always for public items (CI enforces rustdoc).

**How**: For the authoring conventions (the public `///` versus `design_docs` split, the `## Specification` citation discipline, and the heading, example, and `## Guarantees` idioms), see [Rustdoc Conventions](rustdoc.md).

---

### Guides

**Question answered**: How do I use this feature? What should I do if I hit this error?

**Audience**: Library users, developers encountering the feature, people seeking troubleshooting

**Where**: `docs/guides/`

**Content**:
- Step-by-step walkthroughs
- Troubleshooting (common mistakes, fixes)
- Worked examples and patterns
- Quick reference tables

**Status**: Usually untracked during development. Committed when feature ships and users need guidance.

**When to write**: Feature is shipped (or about to ship) and users need guidance or error explanation.

---

### ARCHITECTURE.md

**Question answered**: What are the system-wide design principles and long-term constraints?

**Audience**: Anyone understanding the overall workspace design and strategy

**Where**: [ARCHITECTURE.md](../../ARCHITECTURE.md)

**Content**:
- Specification scope
- Crate boundaries and dependency graph
- High-level design decisions (memory, concurrency, error handling)
- API design principles
- Invariants and guarantees

**When to update**: A decision becomes a system-wide principle affecting multiple crates; an approach shapes future phases; a constraint applies across the workspace.

**When NOT to update**: One-off feature decisions (those belong in ADRs); phase-specific exploration (that's in design docs); user how-to (that's in guides).

---

## The Sequence

```
Design Doc → ADR → Implementation → Code Comments → Guide → ARCHITECTURE.md
```

Each builds on the previous:
- **Design Doc**: Explore what's possible and trade-offs
- **ADR**: Decide why you're choosing this approach
- **Implementation**: Write the code
- **Code Comments**: Explain non-obvious behavior; link back to the decision
- **Guide**: Help users understand and use the feature
- **ARCHITECTURE.md**: Reflect system-wide principles if applicable

---

## Decision Tree

```
Exploring a complex design problem?
  → Write a design doc

Decided on an approach after exploration?
  → Write an ADR

Implementing code with non-obvious behavior?
  → Add rustdoc + inline comments

Feature shipped, users need guidance?
  → Write a guide

Is this a system-wide principle?
  → Update ARCHITECTURE.md

Simple, straightforward code?
  → Minimal rustdoc is fine
```

---

## Guidelines

**Don't over-document**:
- Simple types need only rustdoc
- Simple functions need only rustdoc
- If you're explaining line-by-line what the code does, you're over-commenting

**Document complexity where it exists**:
- Non-obvious behavior? Explain why
- Shared state? Explain concurrency assumptions
- New feature? Provide user guidance
- Major design choice? Explore and record the decision

**Links prevent staleness**:
- Comments that reference ADRs and design docs stay relevant
- Orphaned comments become stale and confusing

**Audience matters**:
- Each document type targets a specific reader
- Don't mix audiences
