# Phase Completion Criteria & Promotion Schedule

This document defines the explicit criteria for completing each development phase and the schedule for when code quality standards (linting, documentation, API stability) are promoted to stricter levels.

**Audience**: All contributors and maintainers. Use this to understand when a phase is complete and what standards apply in each phase.

---

## Phase 1: Core Domain Types

**Target**: Implement foundational SDMX structural metadata in pure Rust with `#![no_std]` compatibility.

### Completion Criteria (Go/No-Go for Phase 2)

Phase 1 is **complete** when ALL of the following conditions are met:

- [ ] All Phase 1 tasks in [ROADMAP.md](../../ROADMAP.md) are checked off
- [ ] `sdmx-types` public API is **stable** (no breaking changes planned)
- [ ] Code coverage ≥ **85%** for `sdmx-types` (per `codecov.yaml`)
- [ ] All public items (`pub fn`, `pub struct`, `pub enum`, `pub mod`) have rustdoc with `///` comments
- [ ] Rustdoc examples compile (`cargo test --doc`)
- [ ] `sdmx-rs` facade doc-comment version claims (e.g. `sdmx-rs = "0.1"`) match the actually published version — they describe a future release until 0.1.0 ships
- [ ] MSRV validation passes against declared `rust-version` in `Cargo.toml`
- [ ] WASM compilation passes: `cargo check -p sdmx-types --target wasm32-unknown-unknown`
- [ ] Property-based tests written for domain invariants (e.g., `ConstraintModel` version handling)

### When Phase 2 Starts

- Versioning: `sdmx-types` reaches **0.1.0** or higher
- No new breaking changes to `sdmx-types` public API are permitted in Phase 2 without a MINOR version bump
- Focus shifts to `sdmx-parsers` and `sdmx-writers` implementation

---

## Phase 2: Serialization Engine

**Target**: Implement streaming XML, JSON, and CSV parsing/writing with minimal memory overhead.

### Completion Criteria (Go/No-Go for Phase 3)

Phase 2 is **complete** when ALL of the following conditions are met:

- [ ] All Phase 2 tasks in [ROADMAP.md](../../ROADMAP.md) are checked off
- [ ] `sdmx-parsers` public API is **stable**
- [ ] `sdmx-writers` public API is **stable**
- [ ] Code coverage ≥ **75%** for `sdmx-parsers` and **80%** for `sdmx-writers`
- [ ] All public items have rustdoc with examples
- [ ] Round-trip property-based tests pass: `parse(serialize(x)) == x` for all formats
- [ ] Benchmark baseline established (`criterion` benchmarks for parse/write paths)
- [ ] WASM compilation passes for both crates

### When Phase 3 Starts

- Versioning: `sdmx-parsers` and `sdmx-writers` reach **0.1.0** or higher
- Parser API solidifies; breaking changes require MINOR version bumps
- Focus shifts to `sdmx-client` HTTP orchestration

---

## Phase 3: HTTP Client & Async Runtime

**Target**: Implement async REST client with blocking strategy support.

### Completion Criteria (Go/No-Go for Phase 4)

Phase 3 is **complete** when ALL of the following conditions are met:

- [ ] All Phase 3 tasks in [ROADMAP.md](../../ROADMAP.md) are checked off
- [ ] `sdmx-client` public API is **stable**
- [ ] Code coverage ≥ **80%** for `sdmx-client`
- [ ] All public items have rustdoc with examples
- [ ] Convert the `sdmx-client` `rust,ignore` doc examples (builder / blocking API) to compiling doctests now that the API exists, and confirm `cargo test --doc` covers them
- [ ] Query builders (typestate pattern) enforce compile-time validation
- [ ] Blocking API implementation verified against Design 0005 (Handle::try_current, BlockingStrategy variants)
- [ ] Integration tests pass with HTTP mocking (`wiremock`)
- [ ] Content-type negotiation routing works for CSV/JSON/XML responses

### When Phase 4 Starts

- Versioning: `sdmx-client` reaches **0.1.0** or higher
- Client API solidifies; breaking changes require MINOR version bumps

---

## Phase 4: Extended Queries

**Target**: Implement schema/metadata query endpoints, extending data discovery and validation coverage.

### Completion Criteria (Go/No-Go for Phase 5)

Phase 4 is **complete** when ALL of the following conditions are met:

- [ ] All Phase 4 tasks in [ROADMAP.md](../../ROADMAP.md) are checked off
- [ ] Schema query endpoints (`/schema/`) fully functional
- [ ] Metadata query endpoints (`/metadata/`) fully functional
- [ ] Code coverage ≥ **85%** for `sdmx-types`, **80%** for other crates
- [ ] All public items have rustdoc with examples

### When Phase 5 (Stabilisation) Starts

- Versioning: All crates target **1.0.0** release
- API freeze: All public APIs are final; breaking changes require MAJOR version bumps

---

## Phase 5: Stabilisation & 1.0.0 Release

**Target**: Finalize APIs, complete documentation, publish 1.0.0 across all crates.

### Completion Criteria (Release Ready)

Phase 5 is **complete** and 1.0.0 is released when ALL of the following conditions are met:

- [ ] All Phase 5 tasks in [ROADMAP.md](../../ROADMAP.md) are checked off
- [ ] API review complete; no remaining design TODOs
- [ ] All ADRs and design docs finalized (no "draft" status)
- [ ] Linting strictness promoted (see Promotion Schedule below)
- [ ] All public items have complete rustdoc (summary + examples + error cases + panics)
- [ ] Parser fuzzing suite established and passing
- [ ] Code coverage remains ≥ thresholds (85%/80%/75%/70%)
- [ ] Documentation is comprehensive (API docs, user guide, architecture guide)
- [ ] `sdmx-types` **1.0.0** published to crates.io
- [ ] `sdmx-parsers` **1.0.0** published to crates.io
- [ ] `sdmx-writers` **1.0.0** published to crates.io
- [ ] `sdmx-client` **1.0.0** published to crates.io
- [ ] `sdmx-rs` (facade) **1.0.0** published to crates.io
- [ ] GitHub Release created with comprehensive changelog

---

## Linting & Policy Promotion Schedule

Code quality and documentation standards become stricter as the project approaches 1.0.0. The table below shows when each policy changes.

| Rule / Policy                 | Phases 1–4                              | Phase 5                                   | Rationale                                                   |
|-------------------------------|-----------------------------------------|-------------------------------------------|-------------------------------------------------------------|
| **`missing_docs`**            | `warn`                                  | `deny`                                    | Complete documentation required before 1.0.0                |
| **`missing_errors_doc`**      | `allow`                                 | `warn`                                    | Error documentation becomes required                        |
| **`missing_panics_doc`**      | `allow`                                 | `warn`                                    | Panic conditions must be documented                         |
| **Semver in CONTRIBUTING.md** | Conservative bumps (patch for features) | Standard semver (minor for features)      | Pre-1.0 allows loose semver; post-1.0 follows strict semver |
| **Breaking changes SLA**      | May happen per ADR within phase         | Not permitted (MAJOR version only)        | 1.0.0+ must honor stability contract                        |
| **API Review**                | Implicit (design-by-implementation)     | Explicit checklist (see Phase 5 criteria) | Phase 5 requires formal API audit                           |
| **Unsafe code**               | `forbid` (unchanged)                    | `forbid` (unchanged)                      | Always forbidden across all phases (ADR-0002)               |

### Promotion Details

**When Phase 5 Begins**:

1. Update `Cargo.toml` lints for all crates targeting 1.0.0:
   ```toml
   [lints.rust]
   missing_docs = "deny"        # Changed from "warn"
   missing_errors_doc = "warn"  # Changed from "allow"
   missing_panics_doc = "warn"  # Changed from "allow"
   ```

2. Update [CONTRIBUTING.md](../../CONTRIBUTING.md) § Commit Requirements:
   ```markdown
   | Prefix              | Changelog section | Semantic intent    | Note                 |
   |---------------------|-------------------|--------------------|----------------------|
   | `feat(scope): ...`  | Features          | MINOR version bump | (changed from PATCH) |
   | `fix(scope): ...`   | Bug Fixes         | PATCH version bump | (unchanged)          |
   | `feat!(scope): ...` | Breaking Changes  | MAJOR version bump | (unchanged)          |
   ```

3. Add Phase 5 API Review Checklist to a new ADR or design doc if not already present.

**When Phase 5 (1.0.0) is Released**:

- No further policy changes; standards remain at Phase 5 level
- Future 2.0.0 may introduce new policies (e.g., stricter performance benchmarks, API surface constraints)

---

## Relationship to Other Documents

- **[ROADMAP.md](../../ROADMAP.md)** — Lists tasks for each phase; use with this document to understand both "what to do" (tasks) and "when we're done" (completion criteria)
- **[ARCHITECTURE.md](../../ARCHITECTURE.md)** — Design decisions and invariants; consulted during completion criteria review
- **[CONTRIBUTING.md](../../CONTRIBUTING.md)** — Workflow and standards; semver guidance changes per promotion schedule above
- **[releasing.md](releasing.md)** — Release workflow; uses completion criteria to determine release readiness
- **[msrv.md](msrv.md)** — MSRV policy; applies across all phases

---

## See Also

- [ROADMAP.md](../../ROADMAP.md) — Phase task lists and timelines
- [ARCHITECTURE.md](../../ARCHITECTURE.md) — Design rationale and constraints
- [CONTRIBUTING.md](../../CONTRIBUTING.md) — Development workflow and standards
- [ADR-0001](../../docs/adr/0001-record-architecture-decisions.md) — ADR process (consulted during phase reviews)
