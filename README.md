# sdmx-rs

<!-- [![sdmx-rs on crates.io](https://img.shields.io/crates/v/sdmx-rs.svg)](https://crates.io/crates/sdmx-rs) -->
<!-- [![docs.rs](https://img.shields.io/docsrs/sdmx-rs)](https://docs.rs/sdmx-rs) -->
[![CI](https://github.com/dgalbraith/sdmx-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/dgalbraith/sdmx-rs/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/dgalbraith/sdmx-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/dgalbraith/sdmx-rs)
[![MSRV: 1.91.0](https://img.shields.io/badge/MSRV-1.91.0-blue)](docs/project/msrv.md)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

A Rust implementation of the [SDMX](https://sdmx.org) (Statistical Data and Metadata Exchange) standard — a global framework for the exchange of statistical data and metadata used by central banks, statistical agencies, and international organisations.

All data structures, parsing engines, and REST clients in this workspace are implemented in compliance with the authoritative [SDMX Technical Specifications](https://sdmx.org/resources/sdmx-technical-standards/).

## Status & Roadmap

**Phase 1: Core Domain Types** — modelling the SDMX structural metadata in pure Rust with `#![no_std]` compatibility.
Core domain types and validation invariants are under active development.

See [ROADMAP.md](ROADMAP.md) for detailed phase schedules, milestone versions, and expected capability maturity.

## Workspace

`sdmx-rs` is a multi-crate workspace targeting the [SDMX 3.0 and 3.1 specifications](https://sdmx.org/resources/sdmx-technical-standards/). The crates are layered with strict unidirectional dependencies:

| Crate          | Description                                                   |
|----------------|---------------------------------------------------------------|
| `sdmx-rs`      | Facade meta-crate re-exporting all layers under feature flags |
| `sdmx-types`   | Core domain types, data structures, and validation invariants |
| `sdmx-parsers` | Streaming XML and JSON serialization/deserialization engine   |
| `sdmx-writers` | Structured serialization to SDMX formats (CSV, JSON, XML)     |
| `sdmx-client`  | Async HTTP client for SDMX REST endpoints                     |

> **Specification Scope**: This workspace targets **SDMX 3.0 and 3.1 specifications only**. SDMX 2.1 is explicitly out of scope. The library handles structural metadata divergence between 3.0 and 3.1 transparently via unified abstractions (see [ADR-0008](docs/adr/0008-model-sdmx-3-0-and-3-1-divergence-with-a-unified-constraintmodel.md) for details).
>
> **Note**: The `sdmx-types` and `sdmx-parsers` crates are `#![no_std]` (requiring `alloc`). Use `sdmx-rs` with `default-features = false` for an embedded/WASM types-only build, or add `features = ["parsers"]` to include serialization support on `no_std` targets.

See [ARCHITECTURE.md](ARCHITECTURE.md) for design decisions, dependency boundaries, and invariants.
See [ROADMAP.md](ROADMAP.md) for the planned development phases.

## Documentation

**Quick navigation by role**:

| I want to...              | Start here                                                                            |
|---------------------------|---------------------------------------------------------------------------------------|
| **Use this library**      | [User Guide](docs/user/README.md) — Getting started, API overview, examples           |
| **Contribute code**       | [CONTRIBUTING.md](CONTRIBUTING.md) — Development setup, workflow, code review         |
| **Understand the design** | [ARCHITECTURE.md](ARCHITECTURE.md) — Crate boundaries, API patterns, design decisions |
| **Maintain this project** | [ROADMAP.md](ROADMAP.md) — Phases, milestones, releases                               |

For comprehensive documentation organized by audience, see the [**Documentation Index**](docs/README.md).

## What's Coming

The framework is in active early development. **Phase 1** (core domain types) is in progress.

See [ROADMAP.md](ROADMAP.md) for development phases, timelines, and planned APIs.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our Nix development environment, Conventional Commit requirements, and local quality gates.

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

### Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
