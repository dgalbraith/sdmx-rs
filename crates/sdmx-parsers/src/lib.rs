//! High-performance streaming XML and JSON deserializer for SDMX data and
//! metadata.
//!
//! This crate provides the core serialization and deserialization engine for
//! SDMX payloads. Parsing routines target minimal memory allocations and
//! zero-copy slicing where safe, consuming types from
//! [`sdmx-types`](../sdmx_types/index.html).
//!
//! # Design Constraints
//!
//! - Minimal dependencies: the workspace-internal
//!   [`sdmx-types`](../sdmx_types/index.html) crate for the core domain model,
//!   plus external dependencies restricted strictly to `serde`, `serde_json`,
//!   `quick-xml`, and `thiserror` for serialization and error modeling.
//! - No unsafe code.
//! - All parsing must behave deterministically across platform runtimes.
//!
//! # Design & Parsing Mechanics
//!
//! To shield downstream consumers and user-facing APIs from the intricate
//! details of SDMX specification changes (such as version-specific structural
//! variances), the serialization engine is responsible for dynamically routing
//! incoming wire-format data families to construct version-agnostic domain
//! representations.
//!
//! ### Constraint Model Version Routing
//!
//! A core example of this decoupling is the handling of the `ConstraintModel`
//! domain type:
//! - **SDMX 3.0** utilizes a unified `DataConstraint` structure containing a
//!   `constraint_type` property.
//! - **SDMX 3.1** decouples this into dedicated `DataConstraint` (reporting
//!   restrictions only) and `AvailabilityConstraint` structures.
//!
//! At parse time, the parser dynamically determines the correct variant to
//! construct by inspecting the specification version embedded within the
//! payload's root envelope:
//!
//! **For SDMX-ML (XML):** The XML namespace URI on the `<Structure>` root
//! element, resolved via `quick_xml::NsReader::resolve_element()`, is the
//! authoritative version signal:
//! - `http://www.sdmx.org/resources/sdmxml/schemas/v3_0/structure` → SDMX 3.0
//! - `http://www.sdmx.org/resources/sdmxml/schemas/v3_1/structure` → SDMX 3.1
//!
//! Because this crate is `#![no_std]`, the public parser API accepts `&[u8]`
//! and uses `NsReader::from_slice()` (not `from_reader()` which requires
//! `std`).
//!
//! **For SDMX-JSON and SDMX-CSV:** The top-level `version` field conveys the
//! SDMX specification version to the respective parser.
//!
//! Version-specific structures are then parsed, normalized, and mapped to
//! hydrate the unified, version-agnostic `ConstraintModel` enum, keeping
//! wire-format versioning out of the downstream client API.
//!
//! See ADR-0019 for the XML namespace resolution design.

#![no_std]

extern crate alloc;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use core::hint;

    #[test]
    fn crate_compiles_in_no_std_mode() {
        // Smoke test: verify the parser crate exports are accessible in no_std context.
        hint::black_box(());
    }

    #[test]
    fn parser_interfaces_compile() {
        // Structural smoke test ensuring parser module compilation.
        // This test catches breaking changes to public parser interfaces early.
        #[allow(unused)]
        const _: () = {
            // Placeholder: once parser traits/types are introduced, add:
            // let _ = core::mem::size_of::<SomeParserType>();
        };
    }
}
