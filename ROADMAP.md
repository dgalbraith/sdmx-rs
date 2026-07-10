# Roadmap: sdmx-rs

This document records the planned development phases for `sdmx-rs`. It reflects current intent and will be updated as the project evolves.

**For explicit phase completion criteria and policy promotion schedule, see [docs/project/phases.md](docs/project/phases.md).**

<!--
Maintenance: roadmap bullets stay at planning altitude: deliverable, phase, and a
one-line scope; status is the checkbox alone. No decision-register (D-NNNN)
references and no per-item delivery narrative; that detail lives in design 0010,
the decision register, and code Design Notes.
-->

---

## Versioning Strategy

Crate versions track phases. All crates move in lockstep at the same version until `1.0.0`. Pre-1.0 minor bumps signal phase completion and may contain breaking API changes. Decoupled per-crate versioning takes effect from `1.0.0` onward per [ADR-0004](docs/adr/0004-decoupled-crate-versioning-strategy.md).

| Version | Trigger                                              |
|:-------:|------------------------------------------------------|
|   N/A   | Phase 0 complete: Infrastructure                     |
| `0.1.0` | Phase 1 complete: Core domain types usable           |
| `0.2.0` | Phase 2 complete: Serialisation engine functional    |
| `0.3.0` | Phase 3 complete: Async HTTP client functional       |
| `0.4.0` | Phase 4 complete: Extended queries (schema/metadata) |
| `1.0.0` | Phase 5 complete: API stabilisation                  |

---

## Phase 0: Repository Infrastructure

Establishing the workspace foundation before any domain code is written.

- [x] **Multi-crate workspace layout**: Isolated compilation boundaries across `sdmx-types`, `sdmx-parsers`, `sdmx-writers`, `sdmx-client`, and `sdmx-rs` (facade meta-crate)
- [x] **Deterministic toolchain**: Nix Flake + `direnv` pinning Rust 1.92.0 and all system tools cryptographically via `flake.lock`
- [x] **Quality gates**: Strict formatting (via Nix nightly `rustfmt`), `cargo clippy --workspace` (zero warnings), `cargo deny check`, and `cargo test --workspace` enforced locally and in CI
- [x] **Dependency auditing**: `cargo-deny` configured for license allowlisting and RustSec advisory database checks
- [x] **Coverage tracking**: `cargo-llvm-cov` wired into CI with Codecov reporting
- [x] **Nix-driven CI pipeline**: GitHub Actions directly evaluates the Flake to guarantee 1:1 environmental parity between local dev and CI
- [x] **Pre-populated dependency locks**: Core crates (Tokio, Serde, Reqwest, Quick-XML) locked at the workspace level
- [x] **Allow-list `.gitignore`**: Deny-by-default pattern blocking all untracked artifacts; explicit per-file registration for all tracked content
- [x] **GitHub branch governance**: `main` protected via two composed rulesets — required signed commits (`required_signatures`, no bypass), blocked force pushes and deletions, and direct push restricted to maintainers. CI is a review gate the maintainer evaluates before a local signed merge, not a platform-enforced merge gate (see [merging.md](docs/project/merging.md#ci-gates--branch-protection)).
- [x] **PR required status checks**: Configure required status checks (`test-matrix`, `clippy`, `check-formatting`, `semver-check`) on pull requests targeting `main`. Deferred until after this infrastructure baseline lands — the checks can only be marked required once `ci.yml` defines them on the default branch.
- [x] **Dual remote mirroring**: Simultaneous push to GitHub and Codeberg via `all` remote
- [x] **Release automation — local tooling**: `git-cliff` and `cargo-release` configured for per-crate `CHANGELOG.md` generation, cryptographically signed commits and tags, and all local release recipes (`release-dry-run`, `release-merge`, `release-push`, `prepublish-check`) wired into Justfile driven by Conventional Commits
- [x] **Release automation — CI publish pipeline**: Bootstrap crate name reservation on crates.io (manual, token-based, one-time per name); write `publish.yml` (Trusted Publishing via `rust-lang/crates-io-auth-action`, `actions/attest-build-provenance` for SLSA L2 provenance, `verify-signature` as a gate); register Trusted Publishers in crates.io UI; enable enforcement to disable long-lived API-token publishing. Required before the 0.1.0 publication at Phase 1 completion; built in Phase 0 to keep it out of functional work.
- [x] **Establish `sdmx-rs` Facade Crate**: Created a workspace-level facade crate at the root to re-export sub-crates under optional default features coordinated by the workspace meta-version
- [x] **Local LLVM Code Coverage**: Integrated source-based `cargo-llvm-cov` profiling, local HTML reports generation, and headless CI compatibility
- [x] **WASM Target Safety Check**: Configured target compilation gates preventing standard-library leakage in core workspace modules
- [x] **Git Pre-Commit Hook Integration**: Established cryptographically SHA-pinned local Git hooks enforcing the entire quality gate before commit permissions
- [x] **Structural documentation**: `README.md`, `CONTRIBUTING.md`, `ARCHITECTURE.md`, `SECURITY.md`, and this `ROADMAP.md`

---

## Phase 1: Core Domain Types (`sdmx-types`)

Modelling the SDMX structural metadata in pure Rust with minimal external dependencies (`serde`, `thiserror`) and strict `#![no_std]` compatibility. Resources are implemented in spec dependency order.

- [x] **Common base types**: `LocalisedString`, `Annotation`, `Name`, `Description`
- [x] **Trait hierarchy**: `IdentifiableArtefact` → `NameableArtefact` → `VersionableArtefact` → `MaintainableArtefact` base trait structure underpinning all structural metadata types
- [x] **ItemScheme / Item foundations**: Generic base traits underpinning all scheme-based structures
- [x] **Codelist**: Enumerated value domains
- [x] **ConceptScheme**: Semantic concept definitions
- [x] **AgencyScheme**: Maintenance agency registry
- [x] **ValueList**: Closed value domains for dimensions, measures, and attributes
- [x] **DataStructureDefinition (DSD)**: Structural key families, dimensions, attributes, measures
- [x] **Dataflow**: The primary REST query target, referencing a DSD
- [x] **Constraints**: Version-split data constraints for SDMX 3.0 and 3.1 via a unified `ConstraintModel`.
- [x] **Lexical grammar completion**: Version and time-period types completed to the full spec grammars, including wildcard version references.
- [x] **Reference URN contract**: Reference types own their URN parse/render contract and adopt typed version references.
- [ ] **Property-based testing**: `proptest` for construction invariants, lossless serde round-trips, and format/parse round-trips over the canonical lexical grammars.
- [ ] **WASM Test Execution**: `wasm-pack test --node` wired into `just verify` and a CI job.
- [ ] **Framework publication to crates.io**: Publish `0.1.0` full project scaffolding and data types to crates.io once the spec-exact model above is complete

---

## Phase 2: Serialisation Engine (`sdmx-parsers` & `sdmx-writers`)

Streaming CSV, JSON, and XML parsing (deserialisation) and writing (serialisation) consuming the types defined in Phase 1. The reference, version, and time-period grammars arrive settled from Phase 1; parsers consume that contract.

### Parsers (Deserialisation)

- [ ] **SDMX-JSON wire-shape policy**: The parser/writer owns the SDMX-JSON wire mapping; the domain types' `serde` stays an internal projection
- [ ] **SDMX-ML (XML) structure message parser**: Streaming deserializer using `quick-xml` with `serde` integration
- [ ] **SDMX-JSON structure message parser**: Deserializer using `serde_json`

Data parsers below are ordered by priority per [ADR-0018](docs/adr/0018-content-type-negotiation-and-parser-routing.md) (CSV preferred for data queries, then JSON, then XML fallback):

- [ ] **SDMX-CSV data message parser**: Streaming observation reader for the SDMX-CSV data format
- [ ] **SDMX-JSON data message parser**: Deserializer using `serde_json`
- [ ] **SDMX-ML data message parser**: Streaming generic observation reader
- [ ] **3.0 / 3.1 version routing**: Parser selects constraint model based on declared message version; both wire formats handled within the same parsing pipeline
- [ ] **`quick-xml` `serialize` feature evaluation**: Determine whether the `serialize` feature (enabling `quick-xml`'s `serde` integration) is actually invoked in the streaming parser implementation; if unused, remove it from `Cargo.toml` to reduce compile time and attack surface

### Writers (Serialisation)

- [ ] **SDMX-ML (XML) structure message writer**: Streaming serializer using `quick-xml`
- [ ] **SDMX-JSON structure message writer**: Serializer using `serde_json`
- [ ] **SDMX-CSV data message writer**: Streaming writer for the SDMX-CSV data format
- [ ] **SDMX-JSON data message writer**: Serializer using `serde_json`
- [ ] **SDMX-ML data message writer**: Streaming XML writer for data observations

### Infrastructure & Benchmarking

- [ ] **Benchmark baseline**: `criterion` benchmarks established for all primary parse and write paths
- [ ] **Round-trip property-based testing**: Implement parser round-trip validation using `proptest` once parsers and writers are functional; assert that `parse(serialize(x)) == x` with zero field loss for all supported formats

---

## Phase 3: HTTP Client (`sdmx-client`)

Async REST client consuming the parser and type layers.

> [!NOTE]
> While core crates (`sdmx-types`, `sdmx-parsers`) compile to `wasm32-unknown-unknown`, `sdmx-client` is native-only by default due to its direct reliance on standard async networking primitives. Browser-level WASM compilation is out of scope for Phase 3.

### SDMX REST Endpoint Coverage

| Endpoint Class   |  Phase   | Notes                                                               |
|----------------- |:--------:|---------------------------------------------------------------------|
| **Structure**    | Phase 3  | DSD, Dataflow, Codelist, ConceptScheme via `/structure/` paths      |
| **Data**         | Phase 3  | Core retrieval via `/data/` path with dimension/time filtering      |
| **Availability** | Phase 3  | Data discoverability without retrieval via `/availability/` path    |
| **Schema**       | Phase 4  | Data validity/validation queries; requires dedicated design pattern |
| **Metadata**     | Phase 4  | Reference metadata queries (structure, metadataflow, metadataset)   |
| **Registration** | Deferred | Registry discovery; deferred pending post-1.0 prioritisation        |

### Tasks

- [ ] **Tokio-based async HTTP client**: Built on `reqwest`; configure sensible default connection and request timeouts at construction time (e.g., 10 s connect, 60 s request); expose `ClientConfig` overrides so callers can adjust or disable these for their environment without forking the builder
- [ ] **In-memory cache for structural metadata**: DSD, Codelist, and ConceptScheme objects are large, expensive to parse, and change infrequently. Implement session-level in-memory caching with configurable TTL and explicit refresh API to avoid redundant HTTP requests and re-parsing within a client lifetime (HTTP conditional request support via ETags deferred to Phase 5).
- [ ] **`tracing` instrumentation**: Integrate the `tracing` crate in `sdmx-client` for structured, subscriber-agnostic observability; span URL construction and HTTP dispatch at `debug` level; record HTTP status, latency, and response size at `debug`/`trace` level; keep spans off parser inner-loops (hot-path) — instrument only higher-level parser entry points (e.g., `parse_structure_message`, `parse_data_message`) to avoid measurable throughput regression
- [ ] **Metrics instrumentation**: Integrate the `metrics` facade crate in `sdmx-client` and `sdmx-parsers` for subscriber-agnostic observability; emit counters for request count, HTTP errors, and parse failures; histograms for HTTP latency, parse duration, and cache hit ratio; gauges for in-flight requests and cache size; keep instrumentation off hot-paths in parsers and permit backends (Prometheus, StatsD, OpenTelemetry) to subscribe without library changes
- [ ] **SDMX REST endpoint coverage**: Structure queries (DSD, Dataflow, Codelist, ConceptScheme), data queries, and availability queries (data discoverability without retrieval)
- [ ] **Response routing**: Content-type negotiation directing XML or JSON payloads to the correct parser
- [ ] **Async Stream support**: Implement `Stream`/`AsyncIterator` for data observation messages to support low-memory, concurrent streaming of large statistical datasets; implement a non-blocking bridge (e.g., using `spawn_blocking` and bounded channel adapters) to pipe the async HTTP response stream into the synchronous parser
- [ ] **Error propagation**: `sdmx_client::Error` wrapping `sdmx_parsers::Error` and HTTP errors via `thiserror` `#[from]`
- [ ] **Blocking API implementation gates**: Verify `Handle::try_current()`, `BlockingStrategy` variant behaviour, and error cases match [Design 0005](docs/design/0005-synchronous-and-blocking-api-execution-bridge.md) before declaring blocking feature complete
- [ ] **Blocking API**: `blocking` feature wrapping the async client for non-async consumers
- [ ] **Resilience & Middleware**: Add `reqwest-middleware` to implement automatic retries (e.g., `reqwest-retry`) and rate-limit backoffs transparently within the client; `tracing` spans from the instrumentation step above propagate automatically through middleware layers
- [ ] **Rate-limiting strategy**: Design and document how `Retry-After` headers, per-endpoint quotas, and rate-limit state are handled; decide whether rate-limit state is per-client or per-endpoint and whether exposure via public API is needed
- [ ] **Service Discovery & Registry Support**: Support endpoint registry discovery (e.g., SDMX Global Registry) to query structures and data across multiple registries dynamically
- [ ] **Type-safe dimension filter builder**: Fluent query builder or DSL for constructing type-safe URL query parameters (e.g., `c[FREQ]=A+M`) without manual string formatting
  > [!NOTE]
  > The query builder module (`mod query`) is tracked at crate level in [crates/sdmx-client/src/lib.rs](crates/sdmx-client/src/lib.rs).

---

## Phase 4: Extended Queries (Schema & Metadata)

Enhanced query capabilities for data validation and discovery.

- [ ] **Schema queries**: Implement `/schema/{context}/{agencyID}/{resourceID}/{version}` endpoint for data validity constraints; design pattern for XSD/JSON Schema generation to communicate data expectations to providers
- [ ] **Reference metadata queries**: Implement `/metadata/structure/`, `/metadata/metadataflow/`, and `/metadata/metadataset/` endpoints for discovering and retrieving metadata attached to structures and data
- [ ] **Metadata caching**: Extend in-memory cache from Phase 3 to cover metadata objects with separate TTL controls

---

## Phase 5: Stabilisation

- [ ] **`sdmx-types` API review**: Confirm structural stability to prepare for the 1.0 milestone
- [ ] **HTTP Conditional Cache Support (ETags)**: Integrate `If-None-Match` and `If-Modified-Since` header management into the metadata cache manager to allow backend-validated caching and minimise payload transfer overhead
- [ ] **Feature flags**: `xml`, `json`, and `csv` features in `sdmx-parsers` made optional; all enabled by default
- [ ] **Strict Clippy Lints Enforcement**: Promote `clippy::missing_errors_doc` and `clippy::missing_panics_doc` to `warn`/`deny` and ensure all error returns and panics are properly documented
- [ ] **Update `CONTRIBUTING.md` commit table**: Remove pre-1.0 conservative bump guidance; update to standard post-1.0 semver conventions
- [ ] **Parser Fuzzing Suite (`cargo-fuzz`)**: Establish randomised, coverage-guided fuzz testing for XML, JSON, and CSV payload streams in `sdmx-parsers` to guarantee resilience against malicious inputs

- [ ] **Public API Documentation Lock**: Transition `missing_docs` from `warn` to `deny` at the individual crate level (`lib.rs`) for all crates reaching `1.0.0` stability; maintain `warn` at the workspace level for in-progress crates to preserve local development ergonomics
- [ ] **Workspace Member Pinning Strategy transition**: Change exact version pinning (`=`) to compatible caret requirements (`^`) in internal workspace dependencies between member crates (e.g. [sdmx-parsers](crates/sdmx-parsers/Cargo.toml) depending on [sdmx-types](crates/sdmx-types/Cargo.toml)) to enable decoupled versioning, while maintaining exact pinning (`=`) inside the facade [sdmx-rs](crates/sdmx-rs/Cargo.toml) per [ADR-0003](docs/adr/0003-workspace-crate-facade-and-version-pinning-strategy.md) and [ADR-0004](docs/adr/0004-decoupled-crate-versioning-strategy.md).
- [ ] **`sdmx-types` 1.0 release**
- [ ] **`sdmx-parsers` 1.0 release**
- [ ] **`sdmx-client` 1.0 release**
- [ ] **Publish to crates.io**

---

## Future Work (Beyond Phase 5)

Candidate enhancements not committed to any phase. Listed to capture intent; scope, sequencing, and acceptance criteria are deliberately undecided until promoted into a numbered phase.

- [ ] **Interactive Documentation Book (`mdBook`)**: Establish a comprehensive documentation site using `mdBook`
