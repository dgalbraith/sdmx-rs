# Developer Guides

Guidance for contributors writing and maintaining code in this repository.

## For Anyone Contributing Code

- [Developer Workflow](./workflow.md): Step-by-step contribution walkthrough (Issue → Commit → PR creation)
- [Practices & Code Style](./practices.md): Naming conventions, formatting, rustdoc style, module organization
- [Documentation Standards](./documentation.md): Which documentation type to write (design docs, ADRs, rustdoc, guides), and why
- [Rustdoc Conventions](./rustdoc.md): How to author doc comments (the public `///` versus `design_docs` split, `## Specification` citations, heading and example idioms)
- [Developer Tooling & Recipes](./tooling.md): Nix/direnv development environment and Justfile target reference
- [Testing Strategy](./testing.md): Test pyramid, fixtures, mocking, coverage expectations, fuzzing plan
- [Dependency Management](../project/maintenance.md#dependency-audit): How to audit and manage dependencies

---

## For Maintainers

- [MSRV Policy & Upgrades](../project/msrv.md): 6-month policy floor, upgrade procedures, automation
- [Maintenance Obligations](../project/maintenance.md): Maintenance review cycles, when to bump toolchain, dependency audits
- [Code Review Standards](../project/reviews.md): Review priorities, red flags, approval criteria
- [Release Workflow](../../CONTRIBUTING.md#release-workflow): How to cut a release, versioning strategy, crates.io publishing
- [Merge Protocol](../project/merging.md): How to merge PRs with GPG signature preservation

---

## Common Development Tasks

| Task                           | Documentation                                                                                   |
|--------------------------------|-------------------------------------------------------------------------------------------------|
| Set up development environment | [CONTRIBUTING.md § Onboarding](../../CONTRIBUTING.md#onboarding-quickstart-clone--setup--build) |
| Run quality gates locally      | [CONTRIBUTING.md § Local Quality Gates](../../CONTRIBUTING.md#local-quality-gates)              |
| Run developer recipes / tasks  | [tooling.md](./tooling.md)                                                                      |
| Write tests                    | [testing.md](./testing.md)                                                                      |
| Update MSRV                    | [docs/project/msrv.md](../project/msrv.md)                                                      |
| Make a code change             | [CONTRIBUTING.md § Workflow](../../CONTRIBUTING.md#workflow)                                    |
| Submit a PR                    | [CONTRIBUTING.md § Pull Requests](../../CONTRIBUTING.md#6-pull-requests)                        |

---

## Architecture & Design Context

Understanding *why* the code is structured this way:

- [ARCHITECTURE.md](../../ARCHITECTURE.md): Crate boundaries, API design, feature strategy
- [Design Documentation](../design/README.md): Detailed design exploration before implementation
- [ADRs](../adr/README.md): Architectural decisions and rationale

When implementing Phase 3+ features, start with the corresponding design doc to understand context and trade-offs made during planning.
