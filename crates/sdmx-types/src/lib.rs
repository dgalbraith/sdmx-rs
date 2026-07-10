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
mod sealed;
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
        Attribute, AttributeListMember, AttributeRelationship, DimensionRef, DimensionRefs,
        GroupId, MeasureRelationship, MetadataAttributeUsage,
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
        Granularity, ObservationalTimePeriod, SdmxDateTime, SdmxDecimal, SdmxDuration, SdmxInteger,
        SdmxTimePeriod, SdmxTimePeriodKind, SdmxTimeRange, SdmxVersion, VersionDisplay, VersionRef,
        WildcardPosition,
    },
    localised::{LocalisedString, LocalisedText},
    measure::Measure,
    metadata::{IdentifiableMetadata, MaintainableMetadata, NameableMetadata, VersionableMetadata},
    organisation::{Agency, AgencyScheme, Contact, ContactDetail},
    reference::{
        CodelistReference, ConceptReference, DataProviderReference, DataStructureReference,
        DataflowReference, ProvisionAgreementReference, ValueListReference,
    },
    representation::{
        DataType, EnumerationFormat, EnumerationReference, MaxOccurs, Representation,
        RepresentationChoice, TextFormat,
    },
    scheme::{ItemScheme, SchemeItem},
    valuelist::{ValueItem, ValueList},
};

// Compile-time assertions that every type re-exported at the crate root is Send + Sync.
//
// A concurrent consumer can hold or move any public value across threads. These
// assertions monomorphise `assert_send_sync` for each type, so a type that gains a
// non-`Send`/non-`Sync` field (an `Rc`, a `Cell`, a raw pointer) fails the build here,
// naming the offender, rather than degrading a downstream crate's bounds silently.
//
// Maintenance rule: the assertions below mirror the `pub use crate::{...}` block above,
// module by module and in the same order, so completeness is checkable by eye. When an
// export is added to that block, add the matching line to the same group here. Trait
// exports carry no value to instantiate and are skipped. Generic exports are asserted at
// a representative concrete instantiation, and lifetime-carrying exports at `'static`.
//
// The helper sits at module scope (rather than inside the block) so the coverage test in
// `tests` can also execute it at runtime; the const block remains the real check.
const fn assert_send_sync<T: Send + Sync>() {}

const _: () = {
    // annotation
    assert_send_sync::<Annotation>();
    assert_send_sync::<AnnotationUrl>();
    assert_send_sync::<Link>();

    // artefact: all exports (IdentifiableArtefact, MaintainableArtefact, NameableArtefact,
    // VersionableArtefact) are traits, so there is nothing to instantiate.

    // attribute
    assert_send_sync::<Attribute>();
    assert_send_sync::<AttributeListMember>();
    assert_send_sync::<AttributeRelationship>();
    assert_send_sync::<DimensionRef>();
    assert_send_sync::<DimensionRefs>();
    assert_send_sync::<GroupId>();
    assert_send_sync::<MeasureRelationship>();
    assert_send_sync::<MetadataAttributeUsage>();

    // codelist
    assert_send_sync::<Cascade>();
    assert_send_sync::<Code>();
    assert_send_sync::<CodeSelection>();
    assert_send_sync::<Codelist>();
    assert_send_sync::<CodelistExtension>();
    assert_send_sync::<MemberValue>();
    assert_send_sync::<MemberValues>();

    // component
    assert_send_sync::<ComponentMetadata>();
    assert_send_sync::<Usage>();

    // concept
    assert_send_sync::<Concept>();
    assert_send_sync::<ConceptScheme>();
    assert_send_sync::<IsoConceptReference>();

    // constraint
    assert_send_sync::<AvailabilityConstraint>();
    assert_send_sync::<AvailabilityConstraintAttachment>();
    assert_send_sync::<ComponentSelection>();
    assert_send_sync::<ComponentValueSet>();
    assert_send_sync::<ConstraintModel>();
    assert_send_sync::<ConstraintRole>();
    assert_send_sync::<CubeKeyValue>();
    assert_send_sync::<CubeKeyValues>();
    assert_send_sync::<CubeRegion>();
    assert_send_sync::<CubeRegionKey>();
    assert_send_sync::<CubeRegions>();
    assert_send_sync::<DataComponentSelection>();
    assert_send_sync::<DataComponentValue>();
    assert_send_sync::<DataComponentValueSet>();
    assert_send_sync::<DataComponentValues>();
    assert_send_sync::<DataConstraint>();
    assert_send_sync::<DataConstraintAttachment>();
    assert_send_sync::<DataKey>();
    assert_send_sync::<DataKeySet>();
    assert_send_sync::<DataKeyValue>();
    assert_send_sync::<DataKeys>();
    assert_send_sync::<DataStructureRefs>();
    assert_send_sync::<DataflowRefs>();
    assert_send_sync::<KeyValueSelection>();
    assert_send_sync::<ProvisionAgreementRefs>();
    assert_send_sync::<QueryableDataSource>();
    assert_send_sync::<ReleaseCalendar>();
    assert_send_sync::<SimpleComponentValue>();
    assert_send_sync::<SimpleComponentValues>();
    assert_send_sync::<SimpleDataSources>();
    assert_send_sync::<SimpleKeyValues>();
    assert_send_sync::<TimePeriodRange>();
    assert_send_sync::<TimeRange>();
    assert_send_sync::<TimeRangeKind>();

    // data_structure
    assert_send_sync::<DataStructureDefinition>();

    // dataflow
    assert_send_sync::<Dataflow>();
    assert_send_sync::<DimensionConstraint>();

    // descriptor
    assert_send_sync::<AttributeList>();
    assert_send_sync::<DimensionList>();
    assert_send_sync::<Group>();
    assert_send_sync::<GroupDimensions>();
    assert_send_sync::<MeasureList>();

    // dimension
    assert_send_sync::<Dimension>();
    assert_send_sync::<TimeDimension>();

    // error
    assert_send_sync::<Error>();

    // fixed
    assert_send_sync::<FixedInclude>();

    // lexical
    assert_send_sync::<Granularity>();
    assert_send_sync::<ObservationalTimePeriod>();
    assert_send_sync::<SdmxDateTime>();
    assert_send_sync::<SdmxDecimal>();
    assert_send_sync::<SdmxDuration>();
    assert_send_sync::<SdmxInteger>();
    assert_send_sync::<SdmxTimePeriod>();
    assert_send_sync::<SdmxTimePeriodKind>();
    assert_send_sync::<SdmxTimeRange>();
    assert_send_sync::<SdmxVersion>();
    // VersionDisplay borrows an `SdmxVersion`; assert the `'static` instantiation.
    assert_send_sync::<VersionDisplay<'static>>();
    assert_send_sync::<VersionRef>();
    assert_send_sync::<WildcardPosition>();

    // localised
    assert_send_sync::<LocalisedString>();
    assert_send_sync::<LocalisedText>();

    // measure
    assert_send_sync::<Measure>();

    // metadata
    assert_send_sync::<IdentifiableMetadata>();
    assert_send_sync::<MaintainableMetadata>();
    assert_send_sync::<NameableMetadata>();
    assert_send_sync::<VersionableMetadata>();

    // organisation
    assert_send_sync::<Agency>();
    assert_send_sync::<AgencyScheme>();
    assert_send_sync::<Contact>();
    assert_send_sync::<ContactDetail>();

    // reference
    assert_send_sync::<CodelistReference>();
    assert_send_sync::<ConceptReference>();
    assert_send_sync::<DataProviderReference>();
    assert_send_sync::<DataStructureReference>();
    assert_send_sync::<DataflowReference>();
    assert_send_sync::<ProvisionAgreementReference>();
    assert_send_sync::<ValueListReference>();

    // representation
    assert_send_sync::<DataType>();
    assert_send_sync::<EnumerationFormat>();
    assert_send_sync::<EnumerationReference>();
    assert_send_sync::<MaxOccurs>();
    assert_send_sync::<Representation>();
    assert_send_sync::<RepresentationChoice>();
    assert_send_sync::<TextFormat>();

    // scheme: ItemScheme is generic over its item type; assert a representative
    // instantiation. SchemeItem is a trait, so there is nothing to instantiate.
    assert_send_sync::<ItemScheme<Code>>();

    // valuelist
    assert_send_sync::<ValueItem>();
    assert_send_sync::<ValueList>();
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

    #[test]
    fn send_sync_helper_executes_at_runtime() {
        // The const block above handles the actual compile-time enforcement.
        // This test exists purely to execute the helper function at runtime
        // so code coverage tools count it.
        assert_send_sync::<Annotation>();
    }
}
