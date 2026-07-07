//! Core SDMX domain types, data structures, and validation invariants.
//!
//! This crate provides the foundational, minimal-dependency core domain
//! representations for the [SDMX](https://sdmx.org) (Statistical Data and Metadata Exchange) standard.
//! It defines the structural keys, metadata frameworks, and validation
//! invariants consumed by all other crates in the `sdmx-rs` workspace.
//!
//! # Design Constraints
//!
//! - Minimal external dependencies: `serde` (serialisation), `thiserror` (error modelling),
//!   and `chrono` (the validity-window timestamps), all `no_std` + `alloc` compatible.
//! - No unsafe code.
//! - No binary output: this crate is a pure domain model library.
//!
//! # Stated and effective values
//!
//! Many types expose a value in two forms. The **stated** form is the value exactly as the wire
//! carried it, with statedness preserved: an absent field is `None`, not a default. The
//! **effective** form is the resolved value with the schema default applied, exposed through the
//! `effective_*` accessors. So `usage()` returns `Option<Usage>` (was it stated?) while
//! `effective_usage()` returns `Usage` (what applies). Accessor docs labelled `Stated:` or
//! `Effective:` name which form they return.
//!
//! # Status
//!
//! Implementation follows the milestones of design document 0010. The foundation layer
//! (identifier validators, lexical newtypes, localised strings, the annotation and metadata
//! leaves, and the artefact trait hierarchy), the item-scheme layer (the generic item scheme,
//! codes and codelists, concepts, agencies, value lists, and the component representation system),
//! the data structure layer (the dimension, attribute, and measure components, the descriptor
//! lists and groups, and the `DataStructureDefinition` and `Dataflow` maintainables), and the
//! constraint layer (the cube-region and data-key-set trees, the constraint-attachment references
//! and enums, and the `DataConstraint`, `AvailabilityConstraint`, and unified `ConstraintModel`)
//! are in place, completing the structural model of design document 0010.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The two-layer value model the public docs call "stated" and "effective" is the design's Layer-1 and
Layer-2: Layer-1 (the infoset) is rendered as **stated** (the value as the wire carried it), and
Layer-2 (the interpreted view) as **effective** (the schema default applied). The design documents
and per-type Design Notes use the Layer-1/Layer-2 numbering; the public API uses the stated/effective
terms.
"#
)]
#![no_std]

extern crate alloc;

mod annotation;
mod artefact;
mod attribute;
mod codelist;
mod component;
mod concept;
mod constraint;
mod data_structure;
mod dataflow;
mod descriptor;
mod dimension;
mod error;
mod fixed;
mod lexical;
mod localised;
mod measure;
mod metadata;
mod organisation;
mod reference;
mod representation;
mod scheme;
mod validate;
mod valuelist;

#[cfg(test)]
mod test_support;

// Property-test strategies are wasm-excluded with the rest of the property suite: the
// properties verify platform-independent invariants, and proptest itself is a host-only
// dev-dependency (see Cargo.toml and docs/dev/testing.md).
#[cfg(all(test, not(target_arch = "wasm32")))]
mod test_strategy;

pub use crate::{
    annotation::{Annotation, AnnotationUrl, Link},
    artefact::{IdentifiableArtefact, MaintainableArtefact, NameableArtefact, VersionableArtefact},
    attribute::{
        Attribute, AttributeListMember, AttributeRelationship, DimensionIds, DimensionRef, GroupId,
        MeasureRelationship, MetadataAttributeUsage,
    },
    codelist::{
        Cascade, Code, CodeSelection, Codelist, CodelistExtension, MemberValue, MemberValues,
    },
    component::{ComponentMetadata, Usage},
    concept::{Concept, ConceptScheme, IsoConceptReference},
    constraint::{
        AvailabilityConstraint, AvailabilityConstraintAttachment, ComponentSelection,
        ComponentValueSet, ConstraintModel, ConstraintRole, CubeKeyValue, CubeKeyValues,
        CubeRegion, CubeRegionKey, CubeRegions, DataComponentSelection, DataComponentValue,
        DataComponentValueSet, DataComponentValues, DataConstraint, DataConstraintAttachment,
        DataKey, DataKeySet, DataKeyValue, DataKeys, DataStructureRefs, DataflowRefs,
        KeyValueSelection, ProvisionAgreementRefs, QueryableDataSource, ReleaseCalendar,
        SimpleComponentValue, SimpleComponentValues, SimpleDataSources, SimpleKeyValues,
        TimePeriodRange, TimeRange, TimeRangeKind,
    },
    data_structure::DataStructureDefinition,
    dataflow::{Dataflow, DimensionConstraint},
    descriptor::{AttributeList, DimensionList, Group, GroupDimensions, MeasureList},
    dimension::{Dimension, TimeDimension},
    error::Error,
    fixed::FixedInclude,
    lexical::{
        Granularity, ObservationalTimePeriod, SdmxDecimal, SdmxInteger, SdmxTimePeriod,
        SdmxTimePeriodKind, SdmxTimeRange, SdmxVersion, VersionDisplay, VersionRef,
        WildcardPosition,
    },
    localised::{LocalisedString, LocalisedText},
    measure::Measure,
    metadata::{IdentifiableMetadata, MaintainableMetadata, NameableMetadata, VersionableMetadata},
    organisation::{Agency, AgencyScheme, Contact, ContactDetail},
    reference::{
        CodelistReference, ConceptReference, DataProviderReference, DataflowReference,
        DsdReference, ProvisionAgreementReference, ValueListReference,
    },
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
    use alloc::{string::String, vec, vec::Vec};

    use super::*;

    #[test]
    fn public_surface_constructs_in_no_std() {
        // Smoke test over the re-exported foundation API: a validated identifier, a localised
        // name, and the trait hierarchy are all reachable from the crate root.
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some(String::from("en")),
            text: String::from("Currency"),
        }])
        .unwrap();
        let identifiable = IdentifiableMetadata::new(
            String::from("CL_CURRENCY"),
            None,
            None,
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let nameable = NameableMetadata::new(identifiable, names, None);
        assert_eq!(nameable.id(), "CL_CURRENCY");
        assert_eq!(nameable.names().first(), "Currency");
    }
}
