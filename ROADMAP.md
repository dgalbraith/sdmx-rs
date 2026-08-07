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

Each version below is published to crates.io when its phase completes. Prerelease versions mark no phase and publish outside these rows; every crate is published at `0.1.0-alpha.1` today.

| Version | Trigger                                                |
|:-------:|--------------------------------------------------------|
|   N/A   | Phase 0 complete: Infrastructure                       |
| `0.1.0` | Phase 1 complete: Information model foundations usable |
| `0.2.0` | Phase 2 complete: Message families parseable           |
| `0.3.0` | Phase 3 complete: Async HTTP client functional         |
| `0.4.0` | Phase 4 complete: Extended queries (schema/metadata)   |
| `1.0.0` | Phase 5 complete: API stabilisation                    |

---

## Phase 0: Repository Infrastructure

Establishing the workspace foundation before any domain code is written.

- [x] **Multi-crate workspace layout**: Isolated compilation boundaries across `sdmx-types`, `sdmx-parsers`, `sdmx-writers`, `sdmx-client`, and `sdmx-rs` (facade meta-crate)
- [x] **Deterministic toolchain**: Nix Flake + `direnv` pinning Rust 1.92.0 and all system tools cryptographically via `flake.lock`
- [x] **Quality gates**: Strict formatting (via Nix nightly `rustfmt`), `cargo clippy --workspace` (zero warnings), `cargo deny check`, and `cargo test --workspace` enforced locally and in CI
- [x] **Dependency auditing**: `cargo-deny` configured for license allowlisting and RustSec advisory database checks
- [x] **Coverage tracking**: `cargo-llvm-cov` wired into CI with Codecov reporting
- [x] **Nix-driven CI pipeline**: GitHub Actions directly evaluates the Flake to guarantee 1:1 environmental parity between local dev and CI
- [x] **Workspace dependency management**: Shared versions declared in `[workspace.dependencies]` with `Cargo.lock` committed
- [x] **Allow-list `.gitignore`**: Deny-by-default pattern blocking all untracked artifacts; explicit per-file registration for all tracked content
- [x] **Branch governance**: `main` protected by composed rulesets for signed commits, append-only history, and maintainer-restricted push (see [merging.md](docs/project/merging.md#ci-gates--branch-protection))
- [x] **PR required status checks**: The single `CI Quality Gate` aggregate context required on pull requests targeting `main` (see [ci-gating.md](docs/project/ci-gating.md#the-ci-quality-gate-aggregator))
- [x] **Dual remote mirroring**: Simultaneous push to GitHub and Codeberg via `all` remote
- [x] **Release automation (local tooling)**: `git-cliff` and `cargo-release` wired into Justfile recipes for per-crate changelog generation and signed release commits and tags
- [x] **Release automation (CI publish pipeline)**: `publish.yml` releasing through Trusted Publishing with SLSA L2 build provenance attestations and signature verification
- [x] **Facade crate**: Workspace-level meta-crate re-exporting the sub-crates under optional default features
- [x] **Local code coverage**: Source-based `cargo-llvm-cov` profiling with local HTML reports and headless CI compatibility
- [x] **WASM target safety check**: Target compilation gates preventing standard-library leakage in the `no_std` crates
- [x] **Pre-commit hook integration**: SHA-pinned local Git hooks enforcing the quality gate before a commit is accepted
- [x] **Structural documentation**: `README.md`, `CONTRIBUTING.md`, `ARCHITECTURE.md`, `SECURITY.md`, and this `ROADMAP.md`

---

## Phase 1: Information Model Foundations (`sdmx-types`)

Modelling the base layer of the SDMX information model in pure Rust, together with the message envelope shared by every family root. The header family is modelled whole, `BasicHeaderType` included, because the restrictions share one base type even where the messages using them sit in Future Work. Minimal external dependencies (`serde`, `chrono`, `thiserror`) and strict `#![no_std]` compatibility; resources are implemented in spec dependency order.

- [x] **Common base types**: `LocalisedString`, `Annotation`, `Name`, `Description`
- [x] **Trait hierarchy**: `IdentifiableArtefact` → `NameableArtefact` → `VersionableArtefact` → `MaintainableArtefact` base trait structure underpinning all structural metadata types
- [x] **ItemScheme / Item foundations**: Generic base traits underpinning all scheme-based structures
- [x] **Codelist**: Enumerated value domains
- [x] **ConceptScheme**: Semantic concept definitions
- [x] **AgencyScheme**: Maintenance agency registry
- [x] **ValueList**: Closed value domains for dimensions, measures, and attributes
- [x] **DataStructureDefinition (DSD)**: Structural key families, dimensions, attributes, measures
- [x] **Dataflow**: The primary REST query target, referencing a DSD
- [x] **Constraints**: Version-split data constraints for SDMX 3.0 and 3.1 via a unified `ConstraintModel`
- [x] **Lexical grammar completion**: Version and time-period types completed to the full spec grammars, including wildcard version references
- [x] **Reference URN contract**: Reference types own their URN parse/render contract and adopt typed version references
- [ ] **Message header family**: Abstract `BaseHeaderType` with the `StructureHeaderType`, `StructureSpecificDataHeaderType`, `GenericMetadataHeaderType`, and `BasicHeaderType` restrictions
- [ ] **Message footer**: The `Footer` payload from `SDMXMessageFooter.xsd`
- [ ] **Header parties**: `PartyType` and `SenderType`, with the message-namespace `ContactType`
- [ ] **Header time and action vocabulary**: `HeaderTimeType`, `TimezoneType`, and `ActionType` with its 3.1-only `Merge` member
- [ ] **Payload structure descriptors**: `PayloadStructureType` with its data and generic metadata derivation chains
- [ ] **Header reference classes**: `StructureUsageReference`, `StructureReference`, `MetadataflowReference`, and `MetadataProviderReference`
- [x] **Property-based testing**: `proptest` for construction invariants, lossless serde round-trips, and format/parse round-trips over the canonical lexical grammars
- [x] **WASM test execution**: `wasm-pack test --node` wired into `just verify` and a CI job

---

## Phase 2: Message Families and Serialisation

Completing the remaining SDMX message families in `sdmx-types` and delivering their parsers and writers, family by family. The reference, version, and time-period grammars arrive settled from Phase 1; parsers consume that contract. XML is read and written by manual token-driven streaming loops over `quick-xml`, and JSON through `serde_json`, per [ADR-0009](docs/adr/0009-use-quick-xml-and-serde-json-for-streaming-deserialisation.md).

### Foundations

- [ ] **Parser design doc**: Input model, event and typed lanes, lint container, and format-specification pins, de-risked first by a throwaway Data-by-XML spike
- [ ] **Parsing architecture ADRs**: The decision records arising from the design doc
- [ ] **Edition selection (3.0 / 3.1)**: Parsers and writers select the edition-specific model from the declared message version

### Family Cells

Each cell completes its family's model in `sdmx-types` and delivers the parser and writer that round-trip it.

- [ ] **Error cell**: Model, XML parser, and XML writer
- [ ] **Structure cell**: The remaining `StructuresType` sections, XML parser, and XML writer
  - [ ] **Organisation schemes**: `DataConsumerSchemes`, `DataProviderSchemes`, `MetadataProviderSchemes`, `OrganisationUnitSchemes`
  - [ ] **Categorisation artefacts**: `Categorisations`, `CategorySchemes`
  - [ ] **Codelist extensions**: `GeographicCodelists`, `GeoGridCodelists`, `Hierarchies`, `HierarchyAssociations`
  - [ ] **Metadata structures**: `MetadataStructures`, `Metadataflows`, `MetadataProvisionAgreements`, `MetadataConstraints`
  - [ ] **Provision and process artefacts**: `ProvisionAgreements`, `Processes`, `ReportingTaxonomies`
  - [ ] **Mappings**: `StructureMaps`, `CategorySchemeMaps`, `ConceptSchemeMaps`, `OrganisationSchemeMaps`, `ReportingTaxonomyMaps`, `RepresentationMaps`
  - [ ] **VTL schemes**: `TransformationSchemes`, `RulesetSchemes`, `UserDefinedOperatorSchemes`, `VtlMappingSchemes`, `CustomTypeSchemes`, `NamePersonalisationSchemes`
- [ ] **Data cell**: Model including the embedded `MetadataType`, the compiled layout turning a structure reference into a parse plan, XML parser and writer, and CSV parser and writer
- [ ] **GenericMetadata cell**: `MetadataSetType` and its dependent reference classes, XML parser, and XML writer

### Completion & Verification

- [ ] **SDMX-JSON retrofit**: Parsers and writers for every family the format covers, including the `errors` member declared in every SDMX-JSON schema
- [ ] **Conformance suite**: Parsed and written messages validated against the vendored schemas across the families
- [ ] **Benchmark baseline**: `criterion` benchmarks established for all primary parse and write paths
- [ ] **Round-trip property-based testing**: `parse(serialize(x)) == x` with zero field loss, asserted per format over what that format can represent

---

## Phase 3: HTTP Client (`sdmx-client`)

Async REST client consuming the parser and type layers. Content negotiation follows [ADR-0018](docs/adr/0018-content-type-negotiation-and-parser-routing.md), where `structure+json` is the preferred media type for structure queries; until the Phase 2 SDMX-JSON retrofit lands, those queries fall back to XML.

> [!NOTE]
> `sdmx-types`, `sdmx-parsers`, and `sdmx-writers` compile to `wasm32-unknown-unknown`, so the model and both serialisation directions run in browser and edge runtimes. `sdmx-client` is native-only: per [ADR-0005](docs/adr/0005-adopt-no-std-with-alloc-for-inner-crates.md), the `#![no_std]` boundary is drawn short of the transport layer, which is built on Tokio and `reqwest`. Phase 3 therefore builds no Rust-native client for the browser, where a consumer fetches over the host platform and parses in WASM.

### SDMX REST Endpoint Coverage

| Endpoint Class   |  Phase   | Notes                                                             |
|------------------|:--------:|-------------------------------------------------------------------|
| **Structure**    | Phase 3  | Every structure artefact Phase 2 models, via `/structure/` paths  |
| **Data**         | Phase 3  | Core retrieval via `/data/` path with dimension/time filtering    |
| **Availability** | Phase 3  | Data discoverability without retrieval via `/availability/` path  |
| **Schema**       | Phase 4  | Data validity queries via `/schema/` paths                        |
| **Metadata**     | Phase 4  | Reference metadata queries (structure, metadataflow, metadataset) |
| **Registration** | Phase 4  | Registration query; submission and subscription in Future Work    |

### Tasks

- [ ] **Tokio-based async HTTP client**: `reqwest` transport with default connect and request timeouts and `ClientConfig` overrides
- [ ] **In-memory cache for structural metadata**: Session-level caching of structural artefacts with configurable TTL and an explicit refresh API
- [ ] **`tracing` instrumentation**: Subscriber-agnostic spans over URL construction, HTTP dispatch, and parser entry points, never parser inner loops
- [ ] **Metrics instrumentation**: `metrics` facade counters, histograms, and gauges for request, parse, and cache behaviour
- [ ] **SDMX REST endpoint coverage**: Structure, data, and availability queries
- [ ] **Response routing**: Content-type negotiation directing XML, JSON, or CSV payloads to the correct parser
- [ ] **Async `Stream` support**: Observation streaming over a non-blocking bridge from the async response into the synchronous parser
- [ ] **Error propagation**: `sdmx_client::Error` wrapping `sdmx_parsers::Error` and HTTP errors via `thiserror` `#[from]`
- [ ] **Blocking API verification gates**: `Handle::try_current()`, `BlockingStrategy` variants, and error cases checked against [Design 0005](docs/design/0005-synchronous-and-blocking-api-execution-bridge.md)
- [ ] **Blocking API**: `blocking` feature wrapping the async client for non-async consumers
- [ ] **Resilience middleware**: `reqwest-middleware` retries and rate-limit backoff applied transparently within the client
- [ ] **Rate-limiting strategy**: `Retry-After` handling, per-endpoint quotas, and whether rate-limit state is exposed publicly
- [ ] **Service discovery**: Endpoint discovery across multiple registries such as the SDMX Global Registry
- [ ] **Type-safe dimension filter builder**: Fluent builder for URL query parameters such as `c[FREQ]=A+M`

---

## Phase 4: Extended Queries (Schema & Metadata)

Enhanced query capabilities for data validation and discovery.

- [ ] **Schema queries**: `/schema/{context}/{agencyID}/{resourceID}/{version}` endpoint, returning schema media types unparsed and structure media types through the normal parser
- [ ] **Reference metadata queries**: `/metadata/structure/`, `/metadata/metadataflow/`, and `/metadata/metadataset/` endpoints over the metadata families Phase 2 models
- [ ] **Registration query**: `QueryRegistrationRequest` and `QueryRegistrationResponse` over the registry interface
- [ ] **Metadata caching**: Extend the Phase 3 in-memory cache to metadata objects with separate TTL controls

---

## Phase 5: Stabilisation

- [ ] **`sdmx-types` API review**: Confirm structural stability to prepare for the 1.0 milestone
- [ ] **HTTP conditional cache support**: `If-None-Match` and `If-Modified-Since` handling in the metadata cache manager
- [ ] **Feature flags**: `xml`, `json`, and `csv` features in `sdmx-parsers` made optional; all enabled by default
- [ ] **Strict clippy lint enforcement**: Promote `clippy::missing_errors_doc` and `clippy::missing_panics_doc` to `warn`, documenting every error return and panic
- [ ] **Update `CONTRIBUTING.md` commit table**: Remove pre-1.0 conservative bump guidance in favour of standard post-1.0 semver conventions
- [ ] **Parser fuzzing campaign**: A budgeted `cargo-fuzz` campaign over the XML, JSON, and CSV targets completing without crashes
- [ ] **Public API documentation lock**: `missing_docs` promoted from `warn` to `deny` in each crate reaching `1.0.0`, with the workspace level staying `warn`
- [ ] **Workspace member pinning transition**: Exact (`=`) to caret (`^`) requirements between member crates, exact pinning retained inside the facade per [ADR-0003](docs/adr/0003-workspace-crate-facade-and-version-pinning-strategy.md) and [ADR-0004](docs/adr/0004-decoupled-crate-versioning-strategy.md)

---

## Future Work

Candidate enhancements committed to no phase. Each is listed with the demand that would promote it; scope and acceptance criteria are settled at promotion.

- [ ] **Interactive documentation book (`mdBook`)**: A rendered site over the existing documentation; promoted when the documentation index stops being enough to navigate it
- [ ] **JavaScript bindings crate (`sdmx-js`)**: `wasm-bindgen` bindings and browser-mode testing over the model and parser layers; promoted when a consumer calls the library from JavaScript rather than from Rust
- [ ] **Structure submission**: `SubmitStructureRequest` and its response; promoted when a user maintains structures in a registry rather than reading them
- [ ] **Registration submission**: `SubmitRegistrationsRequest` and its response; promoted alongside structure submission
- [ ] **Subscription operations**: `SubmitSubscriptionsRequest`, `QuerySubscriptionRequest`, and `NotifyRegistryEvent`; promoted when a user subscribes to a registry, which is what makes any of them return anything
- [ ] **Schema generation**: Producing XSD and JSON Schema documents from a structure; promoted when a user publishes data expectations rather than consuming them
