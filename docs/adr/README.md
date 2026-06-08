# Architecture Decision Records — Index

## What Are ADRs?

Architecture Decision Records (ADRs) are **records of architectural decisions**. They document why you chose X over Y. Each ADR captures:

- **Context**: The problem and constraints
- **Drivers**: What factors influenced the choice
- **Options**: Alternatives considered with pros/cons
- **Decision**: The chosen path and rationale
- **Consequences**: Both positive and negative impacts of the decision

ADRs are **definitive and immutable**—once a decision is recorded, it becomes part of the project's decision history.

See [ADR-0001: Record Architecture Decisions](0001-record-architecture-decisions.md) for the ADR process itself.

## When to Write an ADR

Write an ADR when:
- A decision has been made that affects system architecture or multiple crates
- You need to document why you chose one approach over others
- The decision should be auditable and locked into the project's history

For design exploration *before* decisions are made, detailed component planning, or provisional technical proposals, see [Design Documentation](../design/README.md).

---

## All ADRs by Category

## Process & Documentation
- [ADR-0001: Record Architecture Decisions](0001-record-architecture-decisions.md): Establish ADR process for design documentation and auditability

## Safety & Correctness
- [ADR-0002: No unsafe code](0002-workspace-wide-safety-policy-banning-unsafe-code.md): Workspace-wide ban on unsafe code blocks
- [ADR-0006: Standardised error handling](0006-standardise-error-handling-with-thiserror-per-crate.md): Use `thiserror` crate for consistent error handling

## Workspace & Versioning
- [ADR-0003: Crate facade and version pinning](0003-workspace-crate-facade-and-version-pinning-strategy.md): Single facade crate with pinned internal versions
- [ADR-0004: Decoupled crate versioning](0004-decoupled-crate-versioning-strategy.md): Independent semantic versioning per crate

## Platform Support
- [ADR-0005: no_std + alloc architecture](0005-adopt-no-std-with-alloc-for-sdmx-types-and-sdmx-parsers.md): Support embedded and resource-constrained environments
- [ADR-0007: Headless WebAssembly verification](0007-headless-webassembly-execution-verification.md): WASM support and testing strategy

## Domain Modeling
- [ADR-0008: Unified constraint model](0008-model-sdmx-3-0-and-3-1-divergence-with-a-unified-constraintmodel.md): Single constraint model for SDMX 3.0 and 3.1

## Dependencies & Infrastructure
- [ADR-0009: quick-xml & serde_json](0009-use-quick-xml-and-serde-json-for-streaming-deserialization.md): Streaming deserialization with quick-xml and serde_json
- [ADR-0010: Fuzzing suite & panic profiling](0010-parser-fuzzing-suite-and-panic-profile-configuration.md): Cargo fuzz for parser robustness
- [ADR-0011: Tokio async runtime](0011-use-tokio-as-the-primary-async-runtime.md): Primary async runtime for concurrency
- [ADR-0012: reqwest HTTP client](0012-use-reqwest-over-hyper-and-ureq-for-the-http-client.md): High-level HTTP client with connection pooling
- [ADR-0013: rustls TLS backend](0013-use-rustls-over-native-tls-for-transport-layer-security.md): Pure-Rust TLS implementation for consistency

## Client API Design
- [ADR-0014: Fallible client construction](0014-fallible-client-construction-and-custom-error-mapping.md): Fail-fast validation with encapsulated error types
- [ADR-0015: Send and IntoFuture](0015-send-and-intofuture.md): Thread-safety requirements and future trait design
- [ADR-0016: Type parameter count](0016-type-parameter-count.md): Balancing generics for ergonomics vs. flexibility

## Serialization, Parsing & Format Handling
- [ADR-0017: SDMX-CSV parser library selection](0017-sdmx-csv-stream-parsing-strategy.md): Choose `csv` crate for SDMX-CSV parsing (fallback to manual byte scanning)
- [ADR-0018: Content-Type negotiation and parser routing](0018-content-type-negotiation-and-parser-routing.md): Accept header negotiation and format-aware parser dispatch
- [ADR-0019: XML namespace-aware parsing](0019-xml-namespace-aware-parsing.md): NsReader for SDMX 3.0/3.1 namespace distinction

## Maintenance & Observability

## Testing & Development
- [ADR-0020: Shell script test environment isolation](0020-shell-script-test-environment-isolation.md): Test reproducibility via CI variable isolation
