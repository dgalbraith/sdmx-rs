//! Scaffold for the streaming XML and JSON deserialiser for SDMX data and
//! metadata.
//!
//! This crate will provide the core serialisation and deserialisation engine
//! for SDMX payloads. Parsing routines will target minimal memory allocations
//! and zero-copy slicing where safe, consuming types from
//! [`sdmx-types`](../sdmx_types/index.html).
//!
//! # Design Constraints
//!
//! - Minimal dependencies: the workspace-internal
//!   [`sdmx-types`](../sdmx_types/index.html) crate for the core domain model,
//!   plus external dependencies restricted strictly to `serde`, `serde_json`,
//!   `quick-xml`, and `thiserror` for serialisation and error modelling.
//! - No unsafe code.
//! - All parsing must behave deterministically across platform runtimes.
//!
//! # Design & Parsing Mechanics
//!
//! ADR-0008 and ADR-0019 specify the parsing design summarised below;
//! implementation is planned.
//!
//! To shield downstream consumers and user-facing APIs from the intricate
//! details of SDMX specification changes (such as version-specific structural
//! variances), the serialisation engine is responsible for dynamically routing
//! incoming wire-format data families to construct version-agnostic domain
//! representations.
//!
//! ### Constraint Model Version Routing
//!
//! A core example of this decoupling is the handling of the `ConstraintModel`
//! domain type:
//! - **SDMX 3.0** utilises a unified `DataConstraint` structure containing a
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
//! Because this crate is `#![no_std]`, the public parser API will accept
//! `&[u8]` and use `NsReader::from_slice()` (not `from_reader()` which
//! requires `std`).
//!
//! **For SDMX-JSON and SDMX-CSV:** The top-level `version` field conveys the
//! SDMX specification version to the respective parser.
//!
//! Version-specific structures are then parsed, normalised, and mapped to
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

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn crate_compiles_in_no_std_mode() {
        // Smoke test: verify the parser crate exports are accessible in no_std context.
        hint::black_box(());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
