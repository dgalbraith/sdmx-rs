//! Core SDMX domain types, data structures, and validation invariants.
//!
//! This crate provides the foundational, minimal-dependency core domain
//! representations for the [SDMX](https://sdmx.org) (Statistical Data and Metadata Exchange) standard.
//! It defines the structural keys, metadata frameworks, and validation
//! invariants consumed by all other crates in the `sdmx-rs` workspace.
//!
//! # Design Constraints
//!
//! - Minimal external dependencies: `serde` (serialization), `thiserror` (error modeling),
//!   and `chrono` (the validity-window timestamps), all `no_std` + `alloc` compatible.
//! - No unsafe code.
//! - No binary output: this crate is a pure domain model library.
//!
//! # Status
//!
//! Implementation follows the milestones of design document 0010. The foundation layer
//! (identifier validators, lexical newtypes, localised strings, the annotation and metadata
//! leaves, and the artefact trait hierarchy) is in place; the item schemes, components,
//! descriptors, and constraints arrive in later milestones.

#![no_std]

extern crate alloc;

mod annotation;
mod artefact;
mod codelist;
mod concept;
mod error;
mod fixed;
mod lexical;
mod localised;
mod metadata;
mod reference;
mod representation;
mod scheme;
mod validate;
mod valuelist;

pub use crate::{
    annotation::{Annotation, AnnotationUrl, Link},
    artefact::{IdentifiableArtefact, MaintainableArtefact, NameableArtefact, VersionableArtefact},
    codelist::{
        Cascade, Code, CodeSelection, Codelist, CodelistExtension, MemberValue, MemberValues,
    },
    concept::{Concept, ConceptScheme},
    error::Error,
    fixed::FixedInclude,
    lexical::{
        Granularity, SdmxDecimal, SdmxInteger, SdmxTimePeriod, SdmxTimePeriodKind, SdmxVersion,
        VersionDisplay,
    },
    localised::LocalisedString,
    metadata::{IdentifiableMetadata, MaintainableMetadata, NameableMetadata, VersionableMetadata},
    reference::{CodelistReference, ValueListReference},
    representation::{
        DataType, EnumerationFormat, EnumerationReference, MaxOccurs, Representation,
        RepresentationChoice, TextFormat,
    },
    scheme::{ItemScheme, SchemeItem},
    valuelist::{ValueItem, ValueList},
};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn public_surface_constructs_in_no_std() {
        // Smoke test over the re-exported foundation API: a validated identifier, a localised
        // name, and the trait hierarchy are all reachable from the crate root.
        let names = LocalisedString::new(vec![(Some("en".into()), "Currency".into())]).unwrap();
        let identifiable =
            IdentifiableMetadata::new("CL_CURRENCY".into(), None, None, vec![], vec![]).unwrap();
        let nameable = NameableMetadata::new(identifiable, names, None);
        assert_eq!(nameable.id(), "CL_CURRENCY");
        assert_eq!(nameable.names().first(), "Currency");
    }
}
