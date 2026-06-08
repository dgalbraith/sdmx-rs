# Documentation Index

---

## For Library Users

**Getting started with and using sdmx-rs**:

- [User Guide](user/README.md): Quick start, API overview, common questions
- [Guides](guides/README.md): In-depth tutorials by use case (Phase 3+)
- [Runnable Examples](../examples/README.md): Workspace-wide index of runnable crate examples
- [API Reference](https://docs.rs/sdmx-rs): Full API documentation (rustdoc, once published locally via `cargo doc --open`)

Start here: [User Guide](user/README.md)

---

## For Contributors & Developers

**Contributing code and developing in this repository**:

- [CONTRIBUTING.md](../CONTRIBUTING.md): Development setup, workflow, code review standards
- [Developer Guides](dev/README.md): Code style, testing strategy, MSRV policy, common development tasks
- [Design Documentation](design/README.md): Detailed design exploration before implementation begins
- [ADRs](adr/README.md): Architectural decisions and rationale (organized by category)

Start here: [CONTRIBUTING.md](../CONTRIBUTING.md)

---

## For Architects & Design Review

**Understanding system design and architectural decisions**:

- [ARCHITECTURE.md](../ARCHITECTURE.md): Crate boundaries, API patterns, feature strategy, specification alignment
- [Design Documentation](design/README.md): Detailed design exploration (proposals, alternatives, trade-offs, implementation planning)
- [ADRs](adr/README.md): Architectural decisions organized by category (Safety, Workspace, Platform, Client API, etc.)

When implementing Phase 3+ features, consult the corresponding design doc to understand context and constraints.

Start here: [ARCHITECTURE.md](../ARCHITECTURE.md)

---

## For Project Maintainers

**Running, maintaining, and governing this project**:

- [Project Governance](project/README.md): ROADMAP, maintenance obligations, release workflow, performance targets
- [SECURITY.md](../SECURITY.md): Vulnerability reporting, disclosure timeline, version support
- [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md): Community standards and enforcement

Start here: [Project Governance](project/README.md)

---

## Future: Documentation Site

A unified **mdBook** documentation site is candidate future work (see [Future Work in ROADMAP.md](../ROADMAP.md#future-work-beyond-phase-5)), not committed to a numbered phase. If pursued, it would compile the existing documentation into:
- Markdown guides rendered as chapters
- Rustdoc integrated alongside prose
- Cross-linked architecture decisions
- Unified search and navigation

For now, documentation lives in version-controlled markdown and rustdoc in the source tree.

---

## Quick Navigation

| I want to...                       | Go to...                                                                 |
| ---------------------------------- | -------------------------------------------------------------------------|
| **Use this library**               | [User Guide](user/README.md)                                             |
| **Contribute code**                | [CONTRIBUTING.md](../CONTRIBUTING.md)                                    |
| **Understand the design**          | [ARCHITECTURE.md](../ARCHITECTURE.md)                                    |
| **Find an architectural decision** | [ADRs](adr/README.md)                                                    |
| **Set up my environment**          | [CONTRIBUTING.md § Onboarding](../CONTRIBUTING.md#onboarding-quickstart) |
| **Cut a release**                  | [Release Workflow](project/releasing.md)                                 |
| **Report a security issue**        | [SECURITY.md](../SECURITY.md)                                            |
| **See what's planned**             | [ROADMAP.md](../ROADMAP.md)                                              |
