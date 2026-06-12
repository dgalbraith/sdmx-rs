# Design Documentation — Index

## What Are Design Documents?

Design Documents capture **design exploration and planning for systems, components, and features**. Each design captures:

- **Problem**: The gap or opportunity being addressed
- **Proposed Design**: Detailed design rationale and approach
- **Alternatives**: Other approaches considered and why they were rejected
- **Trade-offs**: Benefits, drawbacks, and performance implications
- **Unresolved Questions**: Open questions and areas needing refinement during implementation

Design documents are **mutable and evolving**—they serve as working documents during design and planning phases.

See [0001: Design Documentation Process](0001-design-documentation-process.md) for the design process itself.

## When to Write a Design Document

Write a design document when:
- Planning the design of a new system, component, or significant feature
- You need to explore multiple design approaches and evaluate trade-offs
- Capturing detailed design rationale and constraints for a specific problem
- Building exemplar patterns or educational material for the project

Design documents can be lightweight (a few paragraphs exploring an idea) or comprehensive (detailed specifications for complex components like the type system).

---

## All Designs by Category

## Process & Infrastructure
- [0001: Design Documentation Process](0001-design-documentation-process.md): Establish the design documentation system and process
- [0004: Release Publish Pipeline and Supply Chain Provenance](0004-release-publish-pipeline-and-supply-chain-provenance.md): CI publish pipeline via crates.io Trusted Publishing with SLSA build provenance and dual-format SBOM attestations, gated on maintainer-signature verification and aligned to federal supply-chain standards
- [0009: Maintenance Obligation Tracking System Design](0009-maintenance-obligation-tracking-system-design.md): System design for tracking and managing maintenance obligations

## Parsing & Serialization
- [0002: SDMX CSV Stream Parsing Design](0002-sdmx-csv-stream-parsing-design.md): Design exploration for streaming CSV parser with library choices, representation strategy, and Phase 2 validation gates
- [0008: Target Version Policy for Serialization](0008-target-version-policy-for-serialization.md): Outbound serialization strategy and version targeting

## Client & Query Builder API
- [0003: Typestate Query Builders Design](0003-typestate-query-builders-design.md): Query builder pattern using typestate to enforce compile-time validation, with state machines and implementation blueprints
- [0005: Synchronous and Blocking API Execution Bridge](0005-synchronous-and-blocking-api-execution-bridge.md): Support for blocking contexts via dedicated runtimes
- [0006: Builder Field Storage Strategy for Typestate Query Builders](0006-builder-field-storage.md): Builder pattern field storage strategy
- [0007: Compile-Time Query Validation via the Typestate Pattern](0007-typestate-query-validation.md): Compile-time validation via typestate pattern

## Domain Modeling
- [0010: SDMX Core Domain Types Design](0010-sdmx-core-domain-types-design.md): Phase 1 canonical superset model for SDMX 3.0/3.1 — two-layer infoset store, ordered wire collections, per-artefact identifier tiers, and lossless round-trip serialisation

---

## Relationship to ADRs

- **Designs inform decisions**: A design document explores the design space during planning. Once a decision is made about how to proceed, an ADR records that decision.
- **Design Doc standalone**: Some designs are exploratory or educational and may never become ADRs—that's fine.
- **Cross-references**: A design doc can reference ADRs that informed earlier choices; an ADR can reference a design doc that preceded it.
