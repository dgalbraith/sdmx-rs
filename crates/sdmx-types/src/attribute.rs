//! Attributes and their relationships.
//!
//! An [`Attribute`] is a non-key component carrying values that qualify the data. It declares an
//! [`AttributeRelationship`] (the level its value attaches to: the whole dataflow, each observation,
//! a named group, or a set of dimensions) and, optionally, the measures it applies to
//! (a [`MeasureRelationship`]). A [`MetadataAttributeUsage`] is the other member an attribute list
//! may hold; [`AttributeListMember`] is the interleaved choice of the two.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The data-carrying relationship variants wrap private-field newtypes so their invariants are
enforced by construction, not convention: an `AttributeRelationship::Dimensions` cannot be built
without a [`DimensionRefs`], whose validating constructor rejects an empty list, and `Group` needs an
`IDType`-valid [`GroupId`]. The enum itself merely composes unit variants and already-valid
newtypes, so it carries a derived `Deserialize` that delegates to those newtypes' custom impls (§7).

Every local reference in this module validates its lexical tier at construction (D-0077):
[`GroupId`] `IDType`, [`DimensionRef`] ids / [`MeasureRelationship`] items /
`metadata_attribute_ref` `NCNameIDType`. Referential integrity (do the referenced ids name real
components?) is NOT checked here: that stays a cross-object concern above the type level (D-0020).

[`Attribute`] is a component (D-0028/D-0057): a validated-item type over a [`ComponentMetadata`] leaf
with a [`ConceptReference`] identity, its representation held to the Basic position rules. `usage` is
stored as `Option<Usage>` (statedness, D-0052); the schema default `optional` is the
`effective_usage()` view. [`MetadataAttributeUsage`] (D-0050) has no id and excludes
concept-identity and representation; the wire admits at most one `Link` (so `Option`, not `Vec`).
[`DimensionRef`] and [`MetadataAttributeUsage`] are invariant-bearing types (D-0077): private
fields, fallible `new()`, Raw-shape `Deserialize`.

Decisions: D-0012, D-0020, D-0025, D-0028, D-0048, D-0050, D-0051, D-0052, D-0057, D-0058, D-0077.
"#
)]

use alloc::{string::String, vec::Vec};

use crate::{
    annotation::{Annotation, Link},
    artefact::IdentifiableArtefact,
    component::{ComponentMetadata, Usage},
    error::{Error, to_de_error},
    reference::ConceptReference,
    representation::{Representation, validate_basic_representation},
    validate::{validate_id, validate_ncname},
};

// ---------------------------------------------------------------------------
// GroupId
// ---------------------------------------------------------------------------

/// A reference to a [`Group`](crate::Group) by its id.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// The id a [`AttributeRelationship::Group`] points at. A local reference validated against
/// `IDType` (the `Group` element's type inside `AttributeRelationshipType`, both editions);
/// whether it names a group the DSD declares stays a higher-layer concern (D-0020).
///
/// ## Guarantees
///
/// Always holds an `IDType`-valid id.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct GroupId(String);

impl GroupId {
    /// Builds a group reference, validating the id against SDMX `IDType`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidIdentifier`] if `id` is not a valid `IDType` lexeme (which also
    /// rejects the empty string).
    pub fn new(id: String) -> Result<Self, Error> {
        validate_id(&id)?;
        Ok(Self(id))
    }

    /// The referenced group's id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the newtype, returning the inner string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<GroupId> for String {
    fn from(value: GroupId) -> Self {
        value.into_inner()
    }
}

impl<'de> serde::Deserialize<'de> for GroupId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// DimensionRef, DimensionRefs
// ---------------------------------------------------------------------------

/// A reference to a dimension in an attribute's [`AttributeRelationship::Dimensions`], with its
/// per-reference `optional` flag.
///
/// ## Specification
/// - **Type**: `OptionalLocalDimensionReferenceType`
/// - **Element**: N/A (Reference Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/OptionalLocalDimensionReferenceType.md"))]
///
/// The id is a local reference validated against `NCNameIDType` (the spec type's base, both
/// editions); whether it names a dimension the key descriptor declares stays a higher-layer
/// concern (D-0020). `optional` carries statedness: `None` means the wire omitted it, and the
/// schema default (`false`) is the [`effective_is_optional`](Self::effective_is_optional) view.
///
/// ## Guarantees
///
/// Always holds an `NCNameIDType`-valid id.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct DimensionRef {
    id: String,
    optional: Option<bool>,
}

impl DimensionRef {
    /// Builds a dimension reference, validating the id against SDMX `NCNameIDType`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNcNameIdentifier`] if `id` is not a valid `NCNameIDType` lexeme
    /// (which also rejects the empty string).
    pub fn new(id: String, optional: Option<bool>) -> Result<Self, Error> {
        validate_ncname(&id)?;
        Ok(Self { id, optional })
    }

    /// The referenced dimension's id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Stated: the `optional` flag as the wire carried it (whether the attribute's value may vary
    /// when this dimension is wildcarded). `None` ⟺ absent.
    #[must_use]
    pub const fn optional(&self) -> Option<bool> {
        self.optional
    }

    /// Effective: the `optional` flag, applying the schema default of `false`.
    #[must_use]
    pub fn effective_is_optional(&self) -> bool {
        self.optional.unwrap_or(false)
    }
}

impl<'de> serde::Deserialize<'de> for DimensionRef {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            id: String,
            optional: Option<bool>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.id, raw.optional).map_err(to_de_error)
    }
}

/// A non-empty list of [`DimensionRef`]s.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps the `Dimension+` of an [`AttributeRelationship::Dimensions`]. The schema requires at least
/// one dimension reference, so the constructor rejects an empty list.
///
/// ## Guarantees
///
/// Always holds at least one [`DimensionRef`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct DimensionRefs(Vec<DimensionRef>);

impl DimensionRefs {
    /// Builds a dimension-reference list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyAttributeDimensions`] if `refs` is empty.
    pub fn new(refs: Vec<DimensionRef>) -> Result<Self, Error> {
        if refs.is_empty() {
            return Err(Error::EmptyAttributeDimensions);
        }
        Ok(Self(refs))
    }

    /// The dimension references, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[DimensionRef] {
        &self.0
    }

    /// Iterates the dimension references in order (always at least one).
    pub fn iter(&self) -> impl Iterator<Item = &DimensionRef> {
        self.0.iter()
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<DimensionRef> {
        self.0
    }
}

impl<'a> IntoIterator for &'a DimensionRefs {
    type Item = &'a DimensionRef;
    type IntoIter = core::slice::Iter<'a, DimensionRef>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for DimensionRefs {
    type Item = DimensionRef;
    type IntoIter = alloc::vec::IntoIter<DimensionRef>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<DimensionRefs> for Vec<DimensionRef> {
    fn from(value: DimensionRefs) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<DimensionRef>> for DimensionRefs {
    type Error = Error;

    fn try_from(refs: Vec<DimensionRef>) -> Result<Self, Error> {
        Self::new(refs)
    }
}

impl<'de> serde::Deserialize<'de> for DimensionRefs {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(Vec::<DimensionRef>::deserialize(deserializer)?).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// AttributeRelationship
// ---------------------------------------------------------------------------

/// The level an attribute's value attaches to.
///
/// ## Specification
/// - **Type**: `AttributeRelationshipType`
/// - **Element**: `<AttributeRelationship>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/AttributeRelationshipType.md"))]
///
/// Exhaustive: the four SDMX attachment relationships, fixed by the spec. The data-carrying
/// variants wrap validated newtypes, so use [`group`](Self::group) and
/// [`dimensions`](Self::dimensions) to build them from raw values.
///
/// # Examples
///
/// ```
/// use sdmx_types::{AttributeRelationship, DimensionRef};
///
/// // Dataflow and Observation are unit variants; the data-carrying variants validate their input.
/// let _whole_dataflow = AttributeRelationship::Dataflow;
/// let group = AttributeRelationship::group(String::from("SIBLING"))?;
/// let dimensions =
///     AttributeRelationship::dimensions(vec![DimensionRef::new(String::from("FREQ"), None)?])?;
/// assert!(matches!(group, AttributeRelationship::Group(_)));
/// assert!(matches!(dimensions, AttributeRelationship::Dimensions(_)));
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AttributeRelationship {
    /// The value attaches to the whole dataflow.
    Dataflow,
    /// The value attaches to each observation.
    Observation,
    /// The value attaches to a named [`Group`](crate::Group).
    Group(GroupId),
    /// The value attaches to a set of dimensions.
    Dimensions(DimensionRefs),
}

impl AttributeRelationship {
    /// Builds a [`Group`](Self::Group) relationship from a group id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidIdentifier`] if `id` is not a valid `IDType` lexeme.
    pub fn group(id: String) -> Result<Self, Error> {
        Ok(Self::Group(GroupId::new(id)?))
    }

    /// Builds a [`Dimensions`](Self::Dimensions) relationship from dimension references.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyAttributeDimensions`] if `refs` is empty.
    pub fn dimensions(refs: Vec<DimensionRef>) -> Result<Self, Error> {
        Ok(Self::Dimensions(DimensionRefs::new(refs)?))
    }
}

// ---------------------------------------------------------------------------
// MeasureRelationship
// ---------------------------------------------------------------------------

/// The measures an attribute applies to.
///
/// ## Specification
/// - **Type**: `MeasureRelationshipType`
/// - **Element**: `<MeasureRelationship>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/MeasureRelationshipType.md"))]
///
/// A non-empty list of local measure-id references, each validated against `NCNameIDType` (the
/// `Measure` element's type, both editions); whether each names a declared measure stays a
/// higher-layer concern (D-0020).
///
/// ## Guarantees
///
/// Always holds at least one measure id, every id `NCNameIDType`-valid.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct MeasureRelationship(Vec<String>);

impl MeasureRelationship {
    /// Builds a measure relationship, validating each id against SDMX `NCNameIDType`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyMeasureRelationship`] if `measure_ids` is empty, or
    /// [`Error::InvalidNcNameIdentifier`] if any id is not a valid `NCNameIDType` lexeme.
    pub fn new(measure_ids: Vec<String>) -> Result<Self, Error> {
        if measure_ids.is_empty() {
            return Err(Error::EmptyMeasureRelationship);
        }
        for id in &measure_ids {
            validate_ncname(id)?;
        }
        Ok(Self(measure_ids))
    }

    /// The measure ids, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    /// Iterates the measure ids in order (always at least one).
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter()
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<String> {
        self.0
    }
}

impl<'a> IntoIterator for &'a MeasureRelationship {
    type Item = &'a String;
    type IntoIter = core::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for MeasureRelationship {
    type Item = String;
    type IntoIter = alloc::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<MeasureRelationship> for Vec<String> {
    fn from(value: MeasureRelationship) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<String>> for MeasureRelationship {
    type Error = Error;

    fn try_from(measure_ids: Vec<String>) -> Result<Self, Error> {
        Self::new(measure_ids)
    }
}

impl<'de> serde::Deserialize<'de> for MeasureRelationship {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(Vec::<String>::deserialize(deserializer)?).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// Attribute
// ---------------------------------------------------------------------------

/// A non-key component carrying values that qualify the data.
///
/// ## Specification
/// - **Type**: `AttributeType`
/// - **Element**: `<Attribute>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/AttributeType.md"))]
///
/// A validated component: [`new`](Self::new) holds the representation to the Basic position rules,
/// and the fields are private. The id is optional, inherited from the concept when absent.
/// `concept_roles` holds the zero-or-more concept references defining roles the attribute serves,
/// in wire order. `usage` stores statedness; its schema default (`optional`) is the
/// [`effective_usage`](Self::effective_usage) view.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     Attribute, AttributeRelationship, ComponentMetadata, ConceptReference, IdentifiableArtefact,
/// };
///
/// let metadata = ComponentMetadata::new(
///     Some(String::from("OBS_STATUS")),
///     None,
///     None,
///     Vec::new(),
///     Vec::new(),
/// )?;
/// let concept = ConceptReference {
///     agency: String::from("SDMX"),
///     scheme_id: String::from("CS"),
///     version: "1.0.0".parse().unwrap(),
///     id: String::from("OBS_STATUS"),
/// };
/// let attribute = Attribute::new(
///     metadata,
///     concept,
///     Vec::new(),
///     None,
///     AttributeRelationship::Observation,
///     None,
///     None,
/// )?;
/// assert_eq!(attribute.id(), "OBS_STATUS");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Attribute {
    metadata: ComponentMetadata,
    concept: ConceptReference,
    concept_roles: Vec<ConceptReference>,
    representation: Option<Representation>,
    relationship: AttributeRelationship,
    measure_relationship: Option<MeasureRelationship>,
    usage: Option<Usage>,
}

impl Attribute {
    /// Builds an attribute, validating its representation against the Basic position rules. The
    /// stated id, if any, was already validated by the [`ComponentMetadata`] leaf, and
    /// the relationship newtypes are already valid.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTextTypeForComponent`] if the representation states a `textType`
    /// outside the Basic subset.
    pub fn new(
        metadata: ComponentMetadata,
        concept: ConceptReference,
        concept_roles: Vec<ConceptReference>,
        representation: Option<Representation>,
        relationship: AttributeRelationship,
        measure_relationship: Option<MeasureRelationship>,
        usage: Option<Usage>,
    ) -> Result<Self, Error> {
        validate_basic_representation("Attribute", representation.as_ref())?;
        Ok(Self {
            metadata,
            concept,
            concept_roles,
            representation,
            relationship,
            measure_relationship,
            usage,
        })
    }

    /// Stated: the id exactly as the wire carried it. `None` means the id was absent and the
    /// attribute inherits its concept's identity.
    #[must_use]
    pub fn stated_id(&self) -> Option<&str> {
        self.metadata.stated_id()
    }

    /// The concept this attribute takes its identity from.
    #[must_use]
    pub const fn concept(&self) -> &ConceptReference {
        &self.concept
    }

    /// The concepts defining roles this attribute serves, in wire order (possibly empty).
    #[must_use]
    pub fn concept_roles(&self) -> &[ConceptReference] {
        &self.concept_roles
    }

    /// The attribute's local representation, if any.
    #[must_use]
    pub const fn representation(&self) -> Option<&Representation> {
        self.representation.as_ref()
    }

    /// The level the attribute's value attaches to.
    #[must_use]
    pub const fn relationship(&self) -> &AttributeRelationship {
        &self.relationship
    }

    /// The measures the attribute applies to, if any.
    #[must_use]
    pub const fn measure_relationship(&self) -> Option<&MeasureRelationship> {
        self.measure_relationship.as_ref()
    }

    /// Stated: the `usage` flag as the wire carried it. `None` ⟺ absent.
    #[must_use]
    pub const fn usage(&self) -> Option<Usage> {
        self.usage
    }

    /// Effective: the `usage`, applying the schema default of [`Usage::Optional`].
    #[must_use]
    pub fn effective_usage(&self) -> Usage {
        self.usage.unwrap_or(Usage::Optional)
    }
}

impl IdentifiableArtefact for Attribute {
    fn id(&self) -> &str {
        // The effective identity (Layer 2): the stated id, else the concept's id (D-0057).
        self.metadata.stated_id().unwrap_or(self.concept.id.as_str())
    }
    fn urn(&self) -> Option<&str> {
        self.metadata.urn()
    }
    fn uri(&self) -> Option<&str> {
        self.metadata.uri()
    }
    fn annotations(&self) -> &[Annotation] {
        self.metadata.annotations()
    }
    fn links(&self) -> &[Link] {
        self.metadata.links()
    }
}

impl<'de> serde::Deserialize<'de> for Attribute {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            metadata: ComponentMetadata,
            concept: ConceptReference,
            concept_roles: Vec<ConceptReference>,
            representation: Option<Representation>,
            relationship: AttributeRelationship,
            measure_relationship: Option<MeasureRelationship>,
            usage: Option<Usage>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(
            raw.metadata,
            raw.concept,
            raw.concept_roles,
            raw.representation,
            raw.relationship,
            raw.measure_relationship,
            raw.usage,
        )
        .map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// MetadataAttributeUsage, AttributeListMember
// ---------------------------------------------------------------------------

/// A usage of a metadata attribute defined in the referenced metadata structure.
///
/// ## Specification
/// - **Type**: `MetadataAttributeUsageType`
/// - **Element**: `<MetadataAttributeUsage>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/MetadataAttributeUsageType.md"))]
///
/// Has no id (the schema prohibits it), and excludes concept identity and local representation:
/// what remains is the optional `urn`/`uri` identification, the local reference into the metadata
/// structure, plus a full [`AttributeRelationship`]. The wire admits at most one `Link`, so it is
/// an `Option`, not a `Vec`. The local reference is validated against `NCNameIDType` (the
/// `MetadataAttributeReference` element's type, both editions); whether it names a metadata
/// attribute in the referenced MSD stays a higher-layer concern (D-0020).
///
/// ## Guarantees
///
/// Always holds an `NCNameIDType`-valid `metadata_attribute_ref`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct MetadataAttributeUsage {
    metadata_attribute_ref: String,
    annotations: Vec<Annotation>,
    link: Option<Link>,
    urn: Option<String>,
    uri: Option<String>,
    relationship: AttributeRelationship,
}

impl MetadataAttributeUsage {
    /// Builds a metadata-attribute usage, validating the local reference against SDMX
    /// `NCNameIDType`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNcNameIdentifier`] if `metadata_attribute_ref` is not a valid
    /// `NCNameIDType` lexeme (which also rejects the empty string).
    pub fn new(
        metadata_attribute_ref: String,
        annotations: Vec<Annotation>,
        link: Option<Link>,
        urn: Option<String>,
        uri: Option<String>,
        relationship: AttributeRelationship,
    ) -> Result<Self, Error> {
        validate_ncname(&metadata_attribute_ref)?;
        Ok(Self { metadata_attribute_ref, annotations, link, urn, uri, relationship })
    }

    /// The local reference to the metadata attribute.
    #[must_use]
    pub fn metadata_attribute_ref(&self) -> &str {
        &self.metadata_attribute_ref
    }

    /// Annotations carried on the usage.
    #[must_use]
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// The single optional link (the wire admits at most one here).
    #[must_use]
    pub const fn link(&self) -> Option<&Link> {
        self.link.as_ref()
    }

    /// The usage's URN, if any.
    #[must_use]
    pub fn urn(&self) -> Option<&str> {
        self.urn.as_deref()
    }

    /// The usage's URI, if any.
    #[must_use]
    pub fn uri(&self) -> Option<&str> {
        self.uri.as_deref()
    }

    /// The level the usage attaches to.
    #[must_use]
    pub const fn relationship(&self) -> &AttributeRelationship {
        &self.relationship
    }
}

impl<'de> serde::Deserialize<'de> for MetadataAttributeUsage {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            metadata_attribute_ref: String,
            annotations: Vec<Annotation>,
            link: Option<Link>,
            urn: Option<String>,
            uri: Option<String>,
            relationship: AttributeRelationship,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(
            raw.metadata_attribute_ref,
            raw.annotations,
            raw.link,
            raw.urn,
            raw.uri,
            raw.relationship,
        )
        .map_err(to_de_error)
    }
}

/// A member of an attribute list: an [`Attribute`] or a [`MetadataAttributeUsage`].
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Projects the repeated `Attribute | MetadataAttributeUsage` choice of an attribute list into a
/// Rust enum, so the two kinds can be stored interleaved in wire order. Exhaustive: exactly
/// these two arms.
// The two members differ in size (an `Attribute` carries far more than a `MetadataAttributeUsage`),
// but both are owned values of a small attribute list stored in wire order; boxing the larger arm
// would add indirection the design does not model for no practical gain at this scale.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AttributeListMember {
    /// An attribute.
    Attribute(Attribute),
    /// A metadata attribute usage.
    MetadataAttributeUsage(MetadataAttributeUsage),
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::*;
    use crate::representation::{DataType, RepresentationChoice, TextFormat};

    fn concept(id: &str) -> ConceptReference {
        ConceptReference {
            agency: String::from("SDMX"),
            scheme_id: String::from("CS"),
            version: "1.0.0".parse().unwrap(),
            id: id.into(),
        }
    }

    fn metadata(id: Option<&str>) -> ComponentMetadata {
        ComponentMetadata::new(id.map(Into::into), None, None, Vec::new(), Vec::new()).unwrap()
    }

    fn basic_attribute(usage: Option<Usage>) -> Attribute {
        Attribute::new(
            metadata(Some("OBS_STATUS")),
            concept("OBS_STATUS"),
            Vec::new(),
            None,
            AttributeRelationship::Observation,
            None,
            usage,
        )
        .unwrap()
    }

    #[test]
    fn group_id_validates_the_idtype_grammar() {
        // IDType is the loosest tier: leading digits, @, $ are all valid group references.
        for ok in ["G1", "1", "@INTERNAL", "EUR$"] {
            assert_eq!(GroupId::new(String::from(ok)).unwrap().as_str(), ok);
        }
        // Empty is just one off-grammar lexeme; there is no bespoke empty variant.
        for bad in ["", "a b", "a.b"] {
            assert_eq!(
                GroupId::new(String::from(bad)).unwrap_err(),
                Error::InvalidIdentifier(String::from(bad))
            );
        }
    }

    #[test]
    fn dimension_ref_validates_the_ncname_grammar() {
        let dim = DimensionRef::new(String::from("FREQ"), Some(true)).unwrap();
        assert_eq!(dim.id(), "FREQ");
        assert_eq!(dim.optional(), Some(true));
        // NCNameIDType: a leading digit, @, $, ., and empty are all rejected.
        for bad in ["", "1BAD", "@X", "EUR$", "A.B"] {
            assert_eq!(
                DimensionRef::new(String::from(bad), None).unwrap_err(),
                Error::InvalidNcNameIdentifier(String::from(bad))
            );
        }
    }

    #[test]
    fn dimension_ref_effective_is_optional_defaults_false() {
        assert!(!DimensionRef::new(String::from("FREQ"), None).unwrap().effective_is_optional());
        assert!(
            DimensionRef::new(String::from("FREQ"), Some(true)).unwrap().effective_is_optional()
        );
    }

    #[test]
    fn dimension_ref_deserialize_enforces_the_ncname_grammar() {
        // DimensionRef's Deserialize declares `Raw { id: String, optional: Option<bool> }` and
        // routes through new(). postcard is positional, so a tuple of those field types carrying an
        // off-grammar id proves the wire path re-runs the check, while a valid value round-trips.
        let raw = (String::from("1BAD"), None::<bool>);
        assert!(
            postcard::from_bytes::<DimensionRef>(&postcard::to_allocvec(&raw).unwrap()).is_err()
        );
        crate::test_support::round_trip(
            &DimensionRef::new(String::from("FREQ"), Some(true)).unwrap(),
        );
    }

    #[test]
    fn dimension_ids_reject_empty() {
        let refs = vec![DimensionRef::new(String::from("FREQ"), None).unwrap()];
        assert_eq!(DimensionRefs::new(refs).unwrap().as_slice().len(), 1);
        assert_eq!(DimensionRefs::new(Vec::new()).unwrap_err(), Error::EmptyAttributeDimensions);

        // The invariant holds on the wire: DimensionRefs' Deserialize routes the raw Vec<DimensionRef>
        // through new(), so an empty list is rejected on deserialisation. Any composite that wraps it
        // (AttributeRelationship::Dimensions, and Attribute above that) is protected because serde
        // bubbles this nested failure up.
        let empty = postcard::to_allocvec(&Vec::<DimensionRef>::new()).unwrap();
        assert!(postcard::from_bytes::<DimensionRefs>(&empty).is_err());
    }

    #[test]
    fn relationship_ergonomic_constructors_validate_their_input() {
        assert!(matches!(
            AttributeRelationship::group(String::from("G1")).unwrap(),
            AttributeRelationship::Group(_)
        ));
        assert_eq!(
            AttributeRelationship::group(String::new()).unwrap_err(),
            Error::InvalidIdentifier(String::new())
        );
        assert!(matches!(
            AttributeRelationship::dimensions(vec![
                DimensionRef::new(String::from("FREQ"), None).unwrap()
            ])
            .unwrap(),
            AttributeRelationship::Dimensions(_)
        ));
        assert_eq!(
            AttributeRelationship::dimensions(Vec::new()).unwrap_err(),
            Error::EmptyAttributeDimensions
        );
    }

    #[test]
    fn measure_relationship_rejects_empty_list_and_off_grammar_items() {
        assert_eq!(
            MeasureRelationship::new(vec![String::from("OBS_VALUE")]).unwrap().as_slice().len(),
            1
        );
        assert_eq!(
            MeasureRelationship::new(Vec::new()).unwrap_err(),
            Error::EmptyMeasureRelationship
        );
        // The list-level invariant and the per-item grammar are distinct checks: a non-empty list
        // still rejects an off-NCName item (empty item included).
        assert_eq!(
            MeasureRelationship::new(vec![String::from("OBS_VALUE"), String::new()]).unwrap_err(),
            Error::InvalidNcNameIdentifier(String::new())
        );
        assert_eq!(
            MeasureRelationship::new(vec![String::from("1BAD")]).unwrap_err(),
            Error::InvalidNcNameIdentifier(String::from("1BAD"))
        );
    }

    #[test]
    fn group_id_deserialize_enforces_the_idtype_grammar() {
        // GroupId's Deserialize declares an inner `Raw = String` (it does `String::deserialize` then
        // `Self::new(..)`), and its transparent Serialize encodes as that bare string. postcard is
        // positional, so an off-grammar `String` (empty included) decodes into new(), which rejects
        // it, while a valid one round-trips.
        for bad in ["", "a b"] {
            let bytes = postcard::to_allocvec(&String::from(bad)).unwrap();
            assert!(postcard::from_bytes::<GroupId>(&bytes).is_err());
        }
        let group_id = GroupId::new(String::from("G1")).unwrap();
        assert_eq!(group_id.as_str(), "G1");
        crate::test_support::round_trip(&group_id);
    }

    #[test]
    fn measure_relationship_deserialize_enforces_list_and_item_grammar() {
        // MeasureRelationship's Deserialize declares an inner `Raw = Vec<String>` (it does
        // `Vec::<String>::deserialize` then `Self::new(..)`), and its transparent Serialize encodes
        // as that bare vector. postcard is positional, so an empty `Vec<String>` or an off-grammar
        // item decodes into new(), which rejects it, while a valid one round-trips.
        let relationship = MeasureRelationship::new(vec![String::from("OBS_VALUE")]).unwrap();
        assert_eq!(relationship.as_slice().len(), 1);
        crate::test_support::round_trip(&relationship);
        let empty: Vec<String> = Vec::new();
        assert!(
            postcard::from_bytes::<MeasureRelationship>(&postcard::to_allocvec(&empty).unwrap())
                .is_err()
        );
        let off_grammar = vec![String::from("1BAD")];
        assert!(
            postcard::from_bytes::<MeasureRelationship>(
                &postcard::to_allocvec(&off_grammar).unwrap()
            )
            .is_err()
        );
    }

    #[test]
    fn attribute_relationship_round_trips_a_data_carrying_variant() {
        // The derived enum Deserialize delegates to the newtypes' custom impls; confirm the success
        // side, that a valid Dimensions relationship reconstructs (not only that empty is rejected).
        let relationship = AttributeRelationship::dimensions(vec![
            DimensionRef::new(String::from("FREQ"), Some(true)).unwrap(),
        ])
        .unwrap();
        crate::test_support::round_trip(&relationship);
    }

    #[test]
    fn attribute_id_is_stated_else_inherited() {
        let stated = basic_attribute(None);
        assert_eq!(stated.id(), "OBS_STATUS");
        assert_eq!(stated.stated_id(), Some("OBS_STATUS"));

        let inherited = Attribute::new(
            metadata(None),
            concept("CONCEPT_STATUS"),
            Vec::new(),
            None,
            AttributeRelationship::Observation,
            None,
            None,
        )
        .unwrap();
        assert_eq!(inherited.id(), "CONCEPT_STATUS");
        assert_eq!(inherited.stated_id(), None);
    }

    #[test]
    fn attribute_new_validates_the_basic_representation_rule() {
        let repr = Representation {
            choice: RepresentationChoice::TextFormat(TextFormat {
                text_type: Some(DataType::KeyValues), // outside the Basic subset
                is_sequence: None,
                interval: None,
                start_value: None,
                end_value: None,
                time_interval: None,
                start_time: None,
                end_time: None,
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                decimals: None,
                pattern: None,
                is_multi_lingual: None,
            }),
            min_occurs: None,
            max_occurs: None,
        };
        assert_eq!(
            Attribute::new(
                metadata(Some("OBS_STATUS")),
                concept("OBS_STATUS"),
                Vec::new(),
                Some(repr),
                AttributeRelationship::Observation,
                None,
                None,
            )
            .unwrap_err(),
            Error::InvalidTextTypeForComponent {
                component: String::from("Attribute"),
                text_type: String::from("KeyValues")
            }
        );
    }

    #[test]
    fn attribute_usage_default_is_a_layer_two_view() {
        // Absent statedness -> effective Optional; the stored None is preserved for the writer path.
        let absent = basic_attribute(None);
        assert_eq!(absent.usage(), None);
        assert_eq!(absent.effective_usage(), Usage::Optional);

        let stated = basic_attribute(Some(Usage::Mandatory));
        assert_eq!(stated.usage(), Some(Usage::Mandatory));
        assert_eq!(stated.effective_usage(), Usage::Mandatory);
    }

    #[test]
    fn attribute_exposes_relationship_and_measure_relationship() {
        let relationship = AttributeRelationship::dimensions(vec![
            DimensionRef::new(String::from("FREQ"), None).unwrap(),
        ])
        .unwrap();
        let attribute = Attribute::new(
            metadata(Some("OBS_STATUS")),
            concept("OBS_STATUS"),
            vec![concept("STATUS_ROLE")],
            None,
            relationship,
            Some(MeasureRelationship::new(vec![String::from("OBS_VALUE")]).unwrap()),
            None,
        )
        .unwrap();
        assert!(matches!(attribute.relationship(), AttributeRelationship::Dimensions(_)));
        assert_eq!(attribute.measure_relationship().map(|m| m.as_slice().len()), Some(1));
        assert!(attribute.representation().is_none());
        assert_eq!(attribute.concept().id, "OBS_STATUS");
        assert_eq!(attribute.concept_roles().len(), 1);
        assert_eq!(attribute.concept_roles()[0].id, "STATUS_ROLE");
    }

    #[test]
    fn attribute_deserialize_round_trips() {
        // The empty-Dimensions invariant lives on DimensionRefs and is proven on the wire in
        // dimension_ids_reject_empty; AttributeRelationship::Dimensions wraps it, so serde bubbles
        // that nested rejection up through Attribute's Deserialize. Attribute's own rule (the Basic
        // representation subset) is covered by the next test.
        let attribute = basic_attribute(Some(Usage::Mandatory));
        crate::test_support::round_trip(&attribute);
    }

    #[test]
    fn attribute_deserialize_enforces_the_basic_representation_rule() {
        // Attribute's Deserialize declares
        // `Raw { metadata: ComponentMetadata, concept: ConceptReference,
        //        concept_roles: Vec<ConceptReference>, representation: Option<Representation>,
        //        relationship: AttributeRelationship,
        //        measure_relationship: Option<MeasureRelationship>, usage: Option<Usage> }`
        // and routes through new(), whose validate_basic_representation rejects a textType outside
        // the Basic subset. KeyValues is a valid DataType token (so Raw deserialises), but is outside
        // the Basic subset, so new() rejects it. postcard is positional, so a tuple of those field
        // types carrying that representation proves the wire path re-runs the check rather than
        // letting it slip past.
        // A valid tuple of the same field types decodes — guards this proof's shape against Raw drift.
        let ok_repr = Representation {
            choice: RepresentationChoice::TextFormat(TextFormat {
                text_type: Some(DataType::String), // inside the Basic subset
                ..TextFormat::default()
            }),
            min_occurs: None,
            max_occurs: None,
        };
        let ok = (
            metadata(Some("OBS_STATUS")),
            concept("OBS_STATUS"),
            Vec::<ConceptReference>::new(),
            Some(ok_repr),
            AttributeRelationship::Observation,
            None::<MeasureRelationship>,
            None::<Usage>,
        );
        assert!(postcard::from_bytes::<Attribute>(&postcard::to_allocvec(&ok).unwrap()).is_ok());
        let repr = Representation {
            choice: RepresentationChoice::TextFormat(TextFormat {
                text_type: Some(DataType::KeyValues), // outside the Basic subset
                is_sequence: None,
                interval: None,
                start_value: None,
                end_value: None,
                time_interval: None,
                start_time: None,
                end_time: None,
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                decimals: None,
                pattern: None,
                is_multi_lingual: None,
            }),
            min_occurs: None,
            max_occurs: None,
        };
        let raw = (
            metadata(Some("OBS_STATUS")),
            concept("OBS_STATUS"),
            Vec::<ConceptReference>::new(),
            Some(repr),
            AttributeRelationship::Observation,
            None::<MeasureRelationship>,
            None::<Usage>,
        );
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<Attribute>(&bytes).is_err());
    }

    #[test]
    fn attribute_forwards_identifiable_accessors() {
        use crate::annotation::{Annotation, AnnotationUrl, Link};
        let full = ComponentMetadata::new(
            Some(String::from("OBS_STATUS")),
            Some(String::from("uri")),
            Some(String::from("urn:x")),
            vec![Annotation {
                id: Some(String::from("a1")),
                annotation_type: None,
                annotation_title: None,
                annotation_urls: vec![AnnotationUrl {
                    url: String::from("https://example.com"),
                    lang: Some(String::from("en")),
                }],
                annotation_value: None,
                texts: None,
            }],
            vec![Link {
                rel: String::from("self"),
                url: String::from("https://example.com/x"),
                urn: None,
                link_type: None,
            }],
        )
        .unwrap();
        let attribute = Attribute::new(
            full,
            concept("OBS_STATUS"),
            Vec::new(),
            None,
            AttributeRelationship::Observation,
            None,
            None,
        )
        .unwrap();
        assert_eq!(attribute.urn(), Some("urn:x"));
        assert_eq!(attribute.uri(), Some("uri"));
        assert_eq!(attribute.annotations().len(), 1);
        assert_eq!(attribute.links().len(), 1);
    }

    #[test]
    fn metadata_attribute_usage_validates_the_local_ref() {
        let usage = MetadataAttributeUsage::new(
            String::from("CONTACT"),
            Vec::new(),
            None,
            Some(String::from("urn:x")),
            Some(String::from("https://example.com/usage")),
            AttributeRelationship::Dataflow,
        )
        .unwrap();
        assert_eq!(usage.metadata_attribute_ref(), "CONTACT");
        assert!(usage.annotations().is_empty());
        assert!(usage.link().is_none());
        assert_eq!(usage.urn(), Some("urn:x"));
        assert_eq!(usage.uri(), Some("https://example.com/usage"));
        assert!(matches!(usage.relationship(), AttributeRelationship::Dataflow));
        // NCNameIDType: empty and off-grammar local references are rejected.
        for bad in ["", "1BAD", "@X"] {
            assert_eq!(
                MetadataAttributeUsage::new(
                    String::from(bad),
                    Vec::new(),
                    None,
                    None,
                    None,
                    AttributeRelationship::Dataflow,
                )
                .unwrap_err(),
                Error::InvalidNcNameIdentifier(String::from(bad))
            );
        }
    }

    #[test]
    fn metadata_attribute_usage_deserialize_enforces_the_ncname_grammar() {
        // MetadataAttributeUsage's Deserialize declares
        // `Raw { metadata_attribute_ref: String, annotations: Vec<Annotation>, link: Option<Link>,
        //        urn: Option<String>, uri: Option<String>, relationship: AttributeRelationship }`
        // and routes through new(). postcard is positional, so a tuple of those field types carrying
        // an off-grammar local reference proves the wire path re-runs the check.
        let raw = (
            String::from("1BAD"),
            Vec::<Annotation>::new(),
            None::<Link>,
            None::<String>,
            None::<String>,
            AttributeRelationship::Dataflow,
        );
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<MetadataAttributeUsage>(&bytes).is_err());
    }

    #[test]
    fn metadata_attribute_usage_and_list_member_round_trip() {
        let usage = MetadataAttributeUsage::new(
            String::from("CONTACT"),
            Vec::new(),
            None,
            Some(String::from("urn:x")),
            Some(String::from("https://example.com/usage")),
            AttributeRelationship::Dataflow,
        )
        .unwrap();
        let member = AttributeListMember::MetadataAttributeUsage(usage);
        crate::test_support::round_trip(&member);

        let attribute_member = AttributeListMember::Attribute(basic_attribute(None));
        crate::test_support::round_trip(&attribute_member);
    }

    #[test]
    fn dimension_ids_try_from_rejects_empty() {
        assert_eq!(
            DimensionRefs::try_from(Vec::new()).unwrap_err(),
            Error::EmptyAttributeDimensions
        );
    }

    #[test]
    fn measure_relationship_try_from_rejects_empty() {
        assert_eq!(
            MeasureRelationship::try_from(Vec::new()).unwrap_err(),
            Error::EmptyMeasureRelationship
        );
    }

    #[test]
    fn newtype_into_inner_and_from() {
        let g = GroupId::new(String::from("G")).unwrap();
        assert_eq!(g.clone().into_inner(), "G");
        assert_eq!(String::from(g), "G");
        let refs = vec![DimensionRef::new(String::from("D"), None).unwrap()];
        assert_eq!(DimensionRefs::new(refs.clone()).unwrap().into_inner(), refs);
        assert_eq!(Vec::from(DimensionRefs::new(refs.clone()).unwrap()), refs);
        let m = vec![String::from("M")];
        assert_eq!(MeasureRelationship::new(m.clone()).unwrap().into_inner(), m);
        assert_eq!(Vec::from(MeasureRelationship::new(m.clone()).unwrap()), m);
    }

    #[test]
    fn newtype_iteration() {
        let refs = vec![
            DimensionRef::new(String::from("FREQ"), None).unwrap(),
            DimensionRef::new(String::from("REF_AREA"), None).unwrap(),
        ];
        let dims = DimensionRefs::new(refs.clone()).unwrap();
        // `iter`, borrowed `IntoIterator`, and owned `IntoIterator` all yield stored order.
        assert!(dims.iter().eq(refs.iter()));
        assert!(IntoIterator::into_iter(&dims).eq(refs.iter()));
        assert_eq!(dims.into_iter().collect::<Vec<_>>(), refs);

        let ids = vec![String::from("OBS"), String::from("OBS_VALUE")];
        let measures = MeasureRelationship::new(ids.clone()).unwrap();
        // A borrowed `for` loop and an owned consuming collect agree on the elements.
        let mut borrowed = Vec::new();
        for id in &measures {
            borrowed.push(id.clone());
        }
        assert_eq!(borrowed, ids);
        assert_eq!(measures.iter().map(String::as_str).collect::<Vec<_>>(), ["OBS", "OBS_VALUE"]);
        assert_eq!(measures.into_iter().collect::<Vec<_>>(), ids);
    }

    // Property tests: the internal serde round-trip over generated position-valid
    // attributes (see `test_strategy`); wasm32 is excluded with the rest of the property
    // suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::attribute;

        proptest! {
            #[test]
            fn attribute_round_trips(value in attribute()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
