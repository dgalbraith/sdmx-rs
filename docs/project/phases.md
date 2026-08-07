# Phase Completion Criteria & Promotion Schedule

This document defines the explicit criteria for completing each development phase and the schedule for when code quality standards (linting, documentation, API stability) are promoted to stricter levels.

**Audience**: All contributors and maintainers. Use this to understand when a phase is complete and what standards apply in each phase.

Phase 0 has no section below, since it has no version trigger and no go/no-go gate; its tasks are listed in [ROADMAP.md](../../ROADMAP.md) alone.

---

## Phase 1: Information Model Foundations (`sdmx-types`)

**Target**: Implement the base layer of the SDMX information model and the message envelope in pure Rust with `#![no_std]` compatibility.

### Completion Criteria (Go/No-Go for Phase 2)

Phase 1 is **complete** when ALL of the following conditions are met:

- [ ] All Phase 1 tasks in [ROADMAP.md](../../ROADMAP.md) are checked off
- [ ] The `sdmx-types` base layer is settled: the remaining message families extend the model rather than rework it
- [ ] Code coverage ≥ **85%** for `sdmx-types` (per `codecov.yaml`)
- [ ] All public items (`pub fn`, `pub struct`, `pub enum`, `pub mod`) have rustdoc with `///` comments
- [ ] Rustdoc examples compile (`cargo test --doc`)
- [ ] MSRV validation passes against declared `rust-version` in `Cargo.toml`
- [ ] WASM gate passes: `just verify-wasm` (compilation check plus headless Node/V8 test execution)
- [x] Property-based tests written for domain invariants (e.g., `ConstraintModel` version handling)

### When Phase 2 Starts

- Versioning: every crate reaches **0.1.0** in lockstep
- Breaking changes to the `sdmx-types` public API remain permitted before `1.0.0` and ride the phase minor bump
- Focus shifts to the remaining message families and their parsers and writers

---

## Phase 2: Message Families and Serialisation

**Target**: Complete the remaining SDMX message families and implement streaming XML, JSON, and CSV parsing and writing with minimal memory overhead.

### Completion Criteria (Go/No-Go for Phase 3)

Phase 2 is **complete** when ALL of the following conditions are met:

- [ ] All Phase 2 tasks in [ROADMAP.md](../../ROADMAP.md) are checked off
- [ ] Every message family is complete in `sdmx-types` against the pinned SDMX 3.0 and 3.1 schemas
- [ ] Every family has a parser cell covering the formats that represent it
- [ ] `sdmx-parsers` public API is **stable**
- [ ] Round-trip property-based tests pass per family cell: `parse(serialize(x)) == x` with zero field loss
- [ ] Round-trip is asserted per format over what that format can represent (SDMX-CSV represents observations, not a whole message)
- [ ] Code coverage ≥ **85%** for `sdmx-types`, **75%** for `sdmx-parsers`, and **80%** for `sdmx-writers`
- [ ] All public items have rustdoc with examples
- [ ] Benchmark baseline established (`criterion` benchmarks for parse/write paths)
- [ ] WASM compilation passes for `sdmx-parsers` and `sdmx-writers`

### When Phase 3 Starts

- Versioning: every crate reaches **0.2.0** in lockstep
- Parser API solidifies; breaking changes ride the phase minor bump until `1.0.0`
- Focus shifts to `sdmx-client` HTTP orchestration

---

## Phase 3: HTTP Client (`sdmx-client`)

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

- Versioning: every crate reaches **0.3.0** in lockstep
- Client API solidifies; breaking changes ride the phase minor bump until `1.0.0`

---

## Phase 4: Extended Queries (Schema & Metadata)

**Target**: Implement schema/metadata query endpoints, extending data discovery and validation coverage.

### Completion Criteria (Go/No-Go for Phase 5)

Phase 4 is **complete** when ALL of the following conditions are met:

- [ ] All Phase 4 tasks in [ROADMAP.md](../../ROADMAP.md) are checked off
- [ ] Schema queries (`/schema/`) return schema media types unparsed and structure media types through the parser
- [ ] Metadata query endpoints (`/metadata/`) fully functional
- [ ] Code coverage meets every per-crate target in `codecov.yaml`
- [ ] All public items have rustdoc with examples

### When Phase 5 (Stabilisation) Starts

- Versioning: every crate reaches **0.4.0** in lockstep
- API freeze: All public APIs are final; breaking changes require MAJOR version bumps

---

## Phase 5: Stabilisation

**Target**: Finalise the public APIs and complete the documentation for the 1.0.0 milestone.

### Completion Criteria (Release Ready)

Phase 5 is **complete** when ALL of the following conditions are met:

- [ ] All Phase 5 tasks in [ROADMAP.md](../../ROADMAP.md) are checked off
- [ ] API review complete; no remaining design TODOs
- [ ] All ADRs and design docs finalised (no "draft" status)
- [ ] Linting strictness promoted (see Promotion Schedule below)
- [ ] All public items have complete rustdoc (summary + examples + error cases + panics)
- [ ] Parser fuzzing campaign completes over the XML, JSON, and CSV targets without crashes
- [ ] Code coverage remains ≥ every per-crate target in `codecov.yaml`
- [ ] Documentation is comprehensive (API docs, user guide, architecture guide)

---

## Linting & Policy Promotion Schedule

Code quality and documentation standards become stricter as the project approaches 1.0.0. The table below shows when each policy changes.

| Rule / Policy                 | Phases 1–4                              | Phase 5                                   | Rationale                                                   |
|-------------------------------|-----------------------------------------|-------------------------------------------|-------------------------------------------------------------|
| **`missing_docs`**            | `warn`                                  | `deny`                                    | Complete documentation required before 1.0.0                |
| **`missing_errors_doc`**      | `allow`                                 | `warn`                                    | Error documentation becomes required                        |
| **`missing_panics_doc`**      | `allow`                                 | `warn`                                    | Panic conditions must be documented                         |
| **Semver in CONTRIBUTING.md** | Conservative bumps (patch for features) | Standard semver (minor for features)      | Pre-1.0 allows loose semver; post-1.0 follows strict semver |
| **Breaking changes SLA**      | May happen per ADR within phase         | Not permitted (MAJOR version only)        | 1.0.0+ must honour stability contract                       |
| **API Review**                | Implicit (design-by-implementation)     | Explicit checklist (see Phase 5 criteria) | Phase 5 requires formal API audit                           |
| **Unsafe code**               | `forbid` (unchanged)                    | `forbid` (unchanged)                      | Always forbidden across all phases (ADR-0002)               |

### Promotion Details

**When Phase 5 Begins**:

1. Promote the clippy documentation lints in the workspace manifest:
   ```toml
   [workspace.lints.clippy]
   missing_errors_doc = "warn"  # Changed from "allow"
   missing_panics_doc = "warn"  # Changed from "allow"
   ```

2. Promote `missing_docs` to `deny` in each crate reaching 1.0.0, at its `lib.rs`, leaving the workspace level at `warn`:
   ```rust
   #![deny(missing_docs)]
   ```

3. Update [CONTRIBUTING.md](../../CONTRIBUTING.md) § Commit Requirements:
   ```markdown
   | Prefix              | Changelog section | Semantic intent    | Note                 |
   |---------------------|-------------------|--------------------|----------------------|
   | `feat(scope): ...`  | Features          | MINOR version bump | (changed from PATCH) |
   | `fix(scope): ...`   | Bug Fixes         | PATCH version bump | (unchanged)          |
   | `feat!(scope): ...` | Breaking Changes  | MAJOR version bump | (unchanged)          |
   ```

4. Add Phase 5 API Review Checklist to a new ADR or design doc if not already present.

**When Phase 5 (1.0.0) is Released**:

- No further policy changes; standards remain at Phase 5 level
- Future 2.0.0 may introduce new policies (e.g., stricter performance benchmarks, API surface constraints)

---

## Relationship to Other Documents

- **[ROADMAP.md](../../ROADMAP.md)**: Lists tasks for each phase; use with this document to understand both "what to do" (tasks) and "when we're done" (completion criteria)
- **[ARCHITECTURE.md](../../ARCHITECTURE.md)**: Design decisions and invariants; consulted during completion criteria review
- **[CONTRIBUTING.md](../../CONTRIBUTING.md)**: Workflow and standards; semver guidance changes per promotion schedule above
- **[releasing.md](releasing.md)**: Release workflow; uses completion criteria to determine release readiness
- **[msrv.md](msrv.md)**: MSRV policy; applies across all phases
- **[ADR-0001](../adr/0001-record-architecture-decisions.md)**: ADR process; consulted during phase reviews
