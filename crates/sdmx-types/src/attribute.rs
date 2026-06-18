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

The data-carrying relationship variants wrap private-field newtypes so the non-empty invariant is
enforced by construction, not convention: an `AttributeRelationship::Dimensions` cannot be built
without a [`DimensionIds`], whose validating constructor rejects an empty list, and `Group` needs a
non-empty [`GroupId`]. The enum itself merely composes unit variants and already-valid newtypes, so
it carries a derived `Deserialize` that delegates to those newtypes' custom impls (§7). Referential
integrity (do the referenced ids name real components?) is NOT checked here: that is a cross-object
concern above the type level (D-0020).

[`Attribute`] is a component (D-0028/D-0057): a validated-item type over a [`ComponentMetadata`] leaf
with a [`ConceptReference`] identity, its representation held to the Basic position rules. `usage` is
stored as `Option<Usage>` (statedness, D-0052); the schema default `optional` is the
`effective_usage()` view. [`MetadataAttributeUsage`] (D-0050) has no id and excludes
concept-identity and representation, so it is an invariant-free pub-field carrier; the wire admits at
most one `Link` (so `Option`, not `Vec`).

Decisions: D-0012, D-0020, D-0025, D-0028, D-0048, D-0050, D-0051, D-0052, D-0057, D-0058.
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
};

// ---------------------------------------------------------------------------
// GroupId
// ---------------------------------------------------------------------------

/// A non-empty reference to a [`Group`](crate::Group) by its id.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// The id a [`AttributeRelationship::Group`] points at. A structural reference, not re-validated as
/// an identifier; the constructor rejects only an empty id.
///
/// ## Guarantees
///
/// Always holds a non-empty id.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct GroupId(String);

impl GroupId {
    /// Builds a group reference.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyGroupId`] if `id` is empty.
    pub fn new(id: String) -> Result<Self, Error> {
        if id.is_empty() {
            return Err(Error::EmptyGroupId);
        }
        Ok(Self(id))
    }

    /// The referenced group's id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for GroupId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// DimensionRef, DimensionIds
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
/// The id is a structural reference, not re-validated. `optional` carries statedness: `None` means
/// the wire omitted it, and the schema default (`false`) is the
/// [`effective_optional`](Self::effective_optional) view.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DimensionRef {
    /// The referenced dimension's id.
    pub id: String,
    /// Whether the attribute's value may vary when this dimension is wildcarded. `None` ⟺ absent.
    pub optional: Option<bool>,
}

impl DimensionRef {
    /// Effective: the `optional` flag, applying the schema default of `false`.
    #[must_use]
    pub fn effective_optional(&self) -> bool {
        self.optional.unwrap_or(false)
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct DimensionIds(Vec<DimensionRef>);

impl DimensionIds {
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
}

impl<'de> serde::Deserialize<'de> for DimensionIds {
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
/// let group = AttributeRelationship::group("SIBLING".to_string())?;
/// let dimensions = AttributeRelationship::dimensions(vec![DimensionRef {
///     id: "FREQ".to_string(),
///     optional: None,
/// }])?;
/// assert!(matches!(group, AttributeRelationship::Group(_)));
/// assert!(matches!(dimensions, AttributeRelationship::Dimensions(_)));
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AttributeRelationship {
    /// The value attaches to the whole dataflow.
    Dataflow,
    /// The value attaches to each observation.
    Observation,
    /// The value attaches to a named [`Group`](crate::Group).
    Group(GroupId),
    /// The value attaches to a set of dimensions.
    Dimensions(DimensionIds),
}

impl AttributeRelationship {
    /// Builds a [`Group`](Self::Group) relationship from a group id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyGroupId`] if `id` is empty.
    pub fn group(id: String) -> Result<Self, Error> {
        Ok(Self::Group(GroupId::new(id)?))
    }

    /// Builds a [`Dimensions`](Self::Dimensions) relationship from dimension references.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyAttributeDimensions`] if `refs` is empty.
    pub fn dimensions(refs: Vec<DimensionRef>) -> Result<Self, Error> {
        Ok(Self::Dimensions(DimensionIds::new(refs)?))
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
/// A non-empty list of local measure-id references. The ids are structural references, not
/// re-validated.
///
/// ## Guarantees
///
/// Always holds at least one measure id.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct MeasureRelationship(Vec<String>);

impl MeasureRelationship {
    /// Builds a measure relationship.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyMeasureRelationship`] if `measure_ids` is empty.
    pub fn new(measure_ids: Vec<String>) -> Result<Self, Error> {
        if measure_ids.is_empty() {
            return Err(Error::EmptyMeasureRelationship);
        }
        Ok(Self(measure_ids))
    }

    /// The measure ids, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        &self.0
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
/// and the fields are private. The id is optional, inherited from the concept when absent. `usage`
/// stores statedness; its schema default (`optional`) is the [`effective_usage`](Self::effective_usage)
/// view.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     Attribute, AttributeRelationship, ComponentMetadata, ConceptReference, IdentifiableArtefact,
/// };
///
/// let metadata =
///     ComponentMetadata::new(Some("OBS_STATUS".to_string()), None, None, vec![], vec![])?;
/// let concept = ConceptReference {
///     agency: "SDMX".to_string(),
///     scheme_id: "CS".to_string(),
///     id: "OBS_STATUS".to_string(),
/// };
/// let attribute =
///     Attribute::new(metadata, concept, None, AttributeRelationship::Observation, None, None)?;
/// assert_eq!(attribute.id(), "OBS_STATUS");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Attribute {
    metadata: ComponentMetadata,
    concept: ConceptReference,
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
        representation: Option<Representation>,
        relationship: AttributeRelationship,
        measure_relationship: Option<MeasureRelationship>,
        usage: Option<Usage>,
    ) -> Result<Self, Error> {
        validate_basic_representation("Attribute", representation.as_ref())?;
        Ok(Self { metadata, concept, representation, relationship, measure_relationship, usage })
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
            representation: Option<Representation>,
            relationship: AttributeRelationship,
            measure_relationship: Option<MeasureRelationship>,
            usage: Option<Usage>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(
            raw.metadata,
            raw.concept,
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
/// Has no id, and excludes concept identity and local representation: what remains is the
/// local reference into the metadata structure plus a full [`AttributeRelationship`]. The wire
/// admits at most one `Link`, so it is an `Option`, not a `Vec`. An invariant-free pub-field
/// carrier.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MetadataAttributeUsage {
    /// The local reference to the metadata attribute (a structural reference).
    pub metadata_attribute_ref: String,
    /// The level the usage attaches to.
    pub relationship: AttributeRelationship,
    /// Annotations carried on the usage.
    pub annotations: Vec<Annotation>,
    /// The single optional link (the wire admits at most one here).
    pub link: Option<Link>,
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
        ConceptReference { agency: "SDMX".into(), scheme_id: "CS".into(), id: id.into() }
    }

    fn metadata(id: Option<&str>) -> ComponentMetadata {
        ComponentMetadata::new(id.map(Into::into), None, None, vec![], vec![]).unwrap()
    }

    fn basic_attribute(usage: Option<Usage>) -> Attribute {
        Attribute::new(
            metadata(Some("OBS_STATUS")),
            concept("OBS_STATUS"),
            None,
            AttributeRelationship::Observation,
            None,
            usage,
        )
        .unwrap()
    }

    #[test]
    fn group_id_rejects_empty() {
        assert_eq!(GroupId::new("G1".into()).unwrap().as_str(), "G1");
        assert_eq!(GroupId::new(String::new()).unwrap_err(), Error::EmptyGroupId);
    }

    #[test]
    fn dimension_ref_effective_optional_defaults_false() {
        assert!(!DimensionRef { id: "FREQ".into(), optional: None }.effective_optional());
        assert!(DimensionRef { id: "FREQ".into(), optional: Some(true) }.effective_optional());
    }

    #[test]
    fn dimension_ids_reject_empty() {
        let refs = vec![DimensionRef { id: "FREQ".into(), optional: None }];
        assert_eq!(DimensionIds::new(refs).unwrap().as_slice().len(), 1);
        assert_eq!(DimensionIds::new(Vec::new()).unwrap_err(), Error::EmptyAttributeDimensions);
    }

    #[test]
    fn relationship_ergonomic_constructors_enforce_non_empty() {
        assert!(matches!(
            AttributeRelationship::group("G1".into()).unwrap(),
            AttributeRelationship::Group(_)
        ));
        assert_eq!(AttributeRelationship::group(String::new()).unwrap_err(), Error::EmptyGroupId);
        assert!(matches!(
            AttributeRelationship::dimensions(vec![DimensionRef {
                id: "FREQ".into(),
                optional: None
            }])
            .unwrap(),
            AttributeRelationship::Dimensions(_)
        ));
        assert_eq!(
            AttributeRelationship::dimensions(Vec::new()).unwrap_err(),
            Error::EmptyAttributeDimensions
        );
    }

    #[test]
    fn measure_relationship_rejects_empty() {
        assert_eq!(MeasureRelationship::new(vec!["OBS_VALUE".into()]).unwrap().as_slice().len(), 1);
        assert_eq!(
            MeasureRelationship::new(Vec::new()).unwrap_err(),
            Error::EmptyMeasureRelationship
        );
    }

    #[test]
    fn group_id_deserialize_enforces_non_empty() {
        // The custom Deserialize routes through new(), so an empty id is rejected on the wire.
        assert_eq!(serde_json::from_str::<GroupId>(r#""G1""#).unwrap().as_str(), "G1");
        assert!(serde_json::from_str::<GroupId>(r#""""#).is_err());
    }

    #[test]
    fn measure_relationship_deserialize_enforces_non_empty() {
        assert_eq!(
            serde_json::from_str::<MeasureRelationship>(r#"["OBS_VALUE"]"#)
                .unwrap()
                .as_slice()
                .len(),
            1
        );
        assert!(serde_json::from_str::<MeasureRelationship>("[]").is_err());
    }

    #[test]
    fn attribute_relationship_round_trips_a_data_carrying_variant() {
        // The derived enum Deserialize delegates to the newtypes' custom impls; confirm the success
        // side, that a valid Dimensions relationship reconstructs (not only that empty is rejected).
        let relationship = AttributeRelationship::dimensions(vec![DimensionRef {
            id: "FREQ".into(),
            optional: Some(true),
        }])
        .unwrap();
        let json = serde_json::to_string(&relationship).unwrap();
        assert_eq!(serde_json::from_str::<AttributeRelationship>(&json).unwrap(), relationship);
    }

    #[test]
    fn attribute_id_is_stated_else_inherited() {
        let stated = basic_attribute(None);
        assert_eq!(stated.id(), "OBS_STATUS");
        assert_eq!(stated.stated_id(), Some("OBS_STATUS"));

        let inherited = Attribute::new(
            metadata(None),
            concept("CONCEPT_STATUS"),
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
                Some(repr),
                AttributeRelationship::Observation,
                None,
                None,
            )
            .unwrap_err(),
            Error::InvalidTextTypeForComponent("Attribute".into(), "KeyValues".into())
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
        let relationship = AttributeRelationship::dimensions(vec![DimensionRef {
            id: "FREQ".into(),
            optional: None,
        }])
        .unwrap();
        let attribute = Attribute::new(
            metadata(Some("OBS_STATUS")),
            concept("OBS_STATUS"),
            None,
            relationship,
            Some(MeasureRelationship::new(vec!["OBS_VALUE".into()]).unwrap()),
            None,
        )
        .unwrap();
        assert!(matches!(attribute.relationship(), AttributeRelationship::Dimensions(_)));
        assert_eq!(attribute.measure_relationship().map(|m| m.as_slice().len()), Some(1));
        assert!(attribute.representation().is_none());
        assert_eq!(attribute.concept().id, "OBS_STATUS");
    }

    #[test]
    fn attribute_deserialize_round_trips_and_rejects_an_empty_relationship() {
        let attribute = basic_attribute(Some(Usage::Mandatory));
        let json = serde_json::to_string(&attribute).unwrap();
        assert_eq!(serde_json::from_str::<Attribute>(&json).unwrap(), attribute);

        // An empty Dimensions relationship is rejected on the wire, routing through the newtype's
        // custom Deserialize that the enum delegates to. (This guards the relationship newtype, not
        // Attribute::new's representation rule, which the next test covers.)
        let bad = r#"{"metadata":{"id":"X","uri":null,"urn":null,"annotations":[],"links":[]},"concept":{"agency":"SDMX","scheme_id":"CS","id":"X"},"representation":null,"relationship":{"Dimensions":[]},"measure_relationship":null,"usage":null}"#;
        assert!(serde_json::from_str::<Attribute>(bad).is_err());
    }

    #[test]
    fn attribute_deserialize_enforces_the_basic_representation_rule() {
        // A valid String textType, flipped on the wire to a non-Basic KeyValues: the custom
        // Deserialize routes through Attribute::new, so validate_basic_representation rejects it
        // rather than letting it slip past the derive. (String appears only as the textType token.)
        let repr = Representation {
            choice: RepresentationChoice::TextFormat(TextFormat {
                text_type: Some(DataType::String),
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
        let attribute = Attribute::new(
            metadata(Some("OBS_STATUS")),
            concept("OBS_STATUS"),
            Some(repr),
            AttributeRelationship::Observation,
            None,
            None,
        )
        .unwrap();
        let json = serde_json::to_string(&attribute).unwrap();
        let bad = json.replace("String", "KeyValues");
        assert!(serde_json::from_str::<Attribute>(&bad).is_err());
    }

    #[test]
    fn attribute_forwards_identifiable_accessors() {
        use crate::annotation::{Annotation, AnnotationUrl, Link};
        let full = ComponentMetadata::new(
            Some("OBS_STATUS".into()),
            Some("uri".into()),
            Some("urn:x".into()),
            vec![Annotation {
                id: Some("a1".into()),
                annotation_type: None,
                annotation_title: None,
                annotation_urls: vec![AnnotationUrl {
                    url: "https://example.com".into(),
                    lang: Some("en".into()),
                }],
                annotation_value: None,
                texts: None,
            }],
            vec![Link {
                rel: "self".into(),
                url: "https://example.com/x".into(),
                urn: None,
                link_type: None,
            }],
        )
        .unwrap();
        let attribute = Attribute::new(
            full,
            concept("OBS_STATUS"),
            None,
            AttributeRelationship::Observation,
            None,
            None,
        )
        .unwrap();
        assert_eq!(attribute.urn(), Some("urn:x"));
        assert_eq!(attribute.annotations().len(), 1);
        assert_eq!(attribute.links().len(), 1);
    }

    #[test]
    fn metadata_attribute_usage_and_list_member_round_trip() {
        let usage = MetadataAttributeUsage {
            metadata_attribute_ref: "CONTACT".into(),
            relationship: AttributeRelationship::Dataflow,
            annotations: vec![],
            link: None,
        };
        let member = AttributeListMember::MetadataAttributeUsage(usage);
        let json = serde_json::to_string(&member).unwrap();
        assert_eq!(serde_json::from_str::<AttributeListMember>(&json).unwrap(), member);

        let attribute_member = AttributeListMember::Attribute(basic_attribute(None));
        let json = serde_json::to_string(&attribute_member).unwrap();
        assert_eq!(serde_json::from_str::<AttributeListMember>(&json).unwrap(), attribute_member);
    }
}
