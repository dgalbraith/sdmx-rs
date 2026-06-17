# Project Governance & Maintenance

Guidance for maintainers and decision-makers on running and governing this project.

---

## Project Direction

- [ROADMAP.md](../../ROADMAP.md): Phase-based development schedule, milestone versions, deliverables
- [phases.md](./phases.md): Phase completion criteria (go/no-go), policy promotion schedule, linting strictness timeline
- [ARCHITECTURE.md](../../ARCHITECTURE.md): System design, crate boundaries, long-term API strategy
- [ADRs](../adr/README.md): Architectural decisions, trade-offs, rationale

---

## Maintenance & Operations

- [MSRV Policy & Upgrades](./msrv.md): 6-month policy floor, upgrade procedures, automation, policy enforcement
- [Maintenance Obligations](./maintenance.md): Maintenance review schedules, when to bump toolchain, dependency audits
- [Dependency Audit](./maintenance.md#dependency-audit): RustSec advisories, license compliance, banned crates
- [Performance Targets](./performance.md): Throughput benchmarks, benchmarking strategy, allocation discipline expectations
- [Release Workflow](releasing.md): How to cut releases, versioning strategy, crates.io publishing, coordinated suite releases

---

## Governance & Security

- [SECURITY.md](../../SECURITY.md): Vulnerability reporting process, SLA, disclosure timeline, support versions
- [CODE_OF_CONDUCT.md](../../CODE_OF_CONDUCT.md): Community standards, enforcement
- [Code Review Standards](reviews.md): Review priorities, red flags, approval criteria
- [Merge Protocol](merging.md): PR merge process, GPG attribution, standard merge protocol
- [Forge Setup](forge-setup.md): GitHub repository configuration, branch rulesets, signing key registration, Codeberg mirror setup
- [Registry Setup](registry-setup.md): crates.io Trusted Publishing registration, enforcement, and the read-only `doctor-registry` verification

---

## Phase Milestones

| Phase   | Version | Status      | Key Deliverables                         |
|---------|:-------:|:-----------:|------------------------------------------|
| Phase 0 | 0.0.0   | ✅ Complete | Infrastructure, ADRs, scaffolding        |
| Phase 1 | 0.1.0   | 📅 Planned  | Core domain types, validation            |
| Phase 2 | 0.2.0   | 📅 Planned  | Serialisation engines, streaming parsers |
| Phase 3 | 0.3.0   | 📅 Planned  | HTTP client, REST endpoints              |
| Phase 4 | 0.4.0   | 📅 Planned  | Extended queries, optimisation           |
| Phase 5 | 1.0.0   | 📅 Planned  | Stabilisation, 1.0 release               |

See [ROADMAP.md](../../ROADMAP.md) for detailed phase schedule and version strategy.

---

## Decision-Making

- **Architecture Changes**: Propose via [ADR process](../adr/0001-record-architecture-decisions.md)
- **Design Exploration**: Document via [Design Docs](../design/0001-design-documentation-process.md)
- **Code Review**: Follow [Code Review Standards](reviews.md)
- **Issue Triage**: Open issue before starting significant work (see [CONTRIBUTING.md § Issue First](../../CONTRIBUTING.md#1-issue-first))

---

## For Contributors

If you're contributing, start with:
1. [CONTRIBUTING.md](../../CONTRIBUTING.md) — Development setup and workflow
2. [Dev Guides](../dev/README.md) — Code style, testing practices
3. [ARCHITECTURE.md](../../ARCHITECTURE.md) — Understanding the system design
4. [MSRV Policy](./msrv.md) — If working on toolchain or version upgrades
