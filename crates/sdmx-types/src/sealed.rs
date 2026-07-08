//! The crate-private seal for the scheme and artefact traits.
//!
//! [`Sealed`] is a supertrait of `SchemeItem`, `IdentifiableArtefact`, `NameableArtefact`,
//! `VersionableArtefact`, and `MaintainableArtefact` (D-0078). The module is crate-private, so
//! only `sdmx-types` can name `Sealed` and add its implementations, and therefore only `sdmx-types`
//! can implement those five traits. Downstream code uses the traits fully in bounds and calls; it
//! cannot add implementations. This is the mechanism the design reserves for the `SdmxSerialize`
//! serialisation boundary (0010 §3), applied here first.

use crate::{
    Agency, AgencyScheme, Attribute, Code, Codelist, Concept, ConceptScheme, DataConstraint,
    DataStructureDefinition, Dataflow, Dimension, Group, IdentifiableMetadata, ItemScheme,
    MaintainableMetadata, Measure, NameableMetadata, SchemeItem, TimeDimension, ValueList,
    VersionableMetadata,
};

/// The seal: a crate-private supertrait of the five scheme and artefact traits. A type outside
/// `sdmx-types` cannot name it, so it cannot implement those traits (D-0078).
pub trait Sealed {}

// `ItemScheme<I>` implements the artefact hierarchy generically for any scheme item, so its seal
// is generic too (its item bound already carries `Sealed`).
impl<I: SchemeItem> Sealed for ItemScheme<I> {}

impl Sealed for Agency {}
impl Sealed for AgencyScheme {}
impl Sealed for Attribute {}
impl Sealed for Code {}
impl Sealed for Codelist {}
impl Sealed for Concept {}
impl Sealed for ConceptScheme {}
impl Sealed for DataConstraint {}
impl Sealed for DataStructureDefinition {}
impl Sealed for Dataflow {}
impl Sealed for Dimension {}
impl Sealed for Group {}
impl Sealed for IdentifiableMetadata {}
impl Sealed for MaintainableMetadata {}
impl Sealed for Measure {}
impl Sealed for NameableMetadata {}
impl Sealed for TimeDimension {}
impl Sealed for ValueList {}
impl Sealed for VersionableMetadata {}
