//! The component descriptors of a data structure.
//!
//! A data structure organises its components into three descriptors: a [`DimensionList`] (the
//! ordered key), an [`AttributeList`], and a [`MeasureList`]. Each is identifiable, carries a
//! fixed descriptor id, and owns a non-empty collection of its components in wire order. A
//! [`Group`] is a named selection of dimensions an attribute can attach to.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The component containers are identifiable (`ComponentListType` extends `IdentifiableType`): each
carries `annotations`/`links`/`urn` as public fields. Their ids are *optional with a fixed value*
("DimensionDescriptor"/"AttributeDescriptor"/"MeasureDescriptor"), so statedness is stored as a
private `Option<String>` with a mismatch rejected at `new()` and a `stated_id()` accessor (D-0049 as
amended by D-0052; private per ADR-0021, since mutation could break the fixed-id invariant). Each
descriptor owns its own mechanical non-empty invariant (the type owning the invariant enforces it,
D-0019), takes its initial collection at `new()`, and exposes a `push()` surface for additions.

The collections are `Vec`, not maps (D-0051): wire order is preserved and lookup is a first-match
view, not a key. The attribute list stores one interleaved `Vec<AttributeListMember>` because the
wire is a single repeated `Attribute | MetadataAttributeUsage` choice; `attributes()`/`usages()` are
filtered views over it. `Group`'s id is required and user-chosen, so it carries full
`IdentifiableMetadata` and is a derived pub-field carrier (both fields self-validate, §7). Identical
across 3.0 and 3.1 (D-0046).

Decisions: D-0019, D-0020, D-0029, D-0046, D-0049, D-0050, D-0051, D-0052.
"#
)]

use alloc::{string::String, vec::Vec};

use crate::{
    annotation::{Annotation, Link},
    artefact::IdentifiableArtefact,
    attribute::{Attribute, AttributeListMember, MetadataAttributeUsage},
    dimension::{Dimension, TimeDimension},
    error::{Error, to_de_error},
    measure::Measure,
    metadata::IdentifiableMetadata,
    validate::validate_fixed,
};

// ---------------------------------------------------------------------------
// GroupDimensions, Group
// ---------------------------------------------------------------------------

/// A non-empty list of dimension-id references for a [`Group`].
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps the `GroupDimension+` of a group. The schema requires at least one dimension reference, so
/// the constructor rejects an empty list. The ids are structural references, not re-validated.
///
/// ## Guarantees
///
/// Always holds at least one dimension id.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct GroupDimensions(Vec<String>);

impl GroupDimensions {
    /// Builds a group's dimension-reference list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyGroupDimensions`] if `dimension_ids` is empty.
    pub fn new(dimension_ids: Vec<String>) -> Result<Self, Error> {
        if dimension_ids.is_empty() {
            return Err(Error::EmptyGroupDimensions);
        }
        Ok(Self(dimension_ids))
    }

    /// The dimension ids, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<String> {
        self.0
    }
}

impl From<GroupDimensions> for Vec<String> {
    fn from(value: GroupDimensions) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<String>> for GroupDimensions {
    type Error = Error;

    fn try_from(dimension_ids: Vec<String>) -> Result<Self, Error> {
        Self::new(dimension_ids)
    }
}

impl<'de> serde::Deserialize<'de> for GroupDimensions {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(Vec::<String>::deserialize(deserializer)?).map_err(to_de_error)
    }
}

/// A named selection of dimensions an attribute can attach to.
///
/// ## Specification
/// - **Type**: `GroupType`
/// - **Element**: `<Group>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/GroupType.md"))]
///
/// Unlike the list descriptors, a group's id is required and user-chosen, so it carries full
/// [`IdentifiableMetadata`]. Both fields self-validate, so this is a pub-field carrier with derived
/// `Deserialize`.
///
/// # Examples
///
/// ```
/// use sdmx_types::{Group, GroupDimensions, IdentifiableArtefact, IdentifiableMetadata};
///
/// let group = Group {
///     metadata: IdentifiableMetadata::new(
///         String::from("SIBLING"),
///         None,
///         None,
///         Vec::new(),
///         Vec::new(),
///     )?,
///     dimensions: GroupDimensions::new(vec![String::from("FREQ"), String::from("CURRENCY")])?,
/// };
/// assert_eq!(group.id(), "SIBLING");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Group {
    /// The group's identity (a required, user-chosen id).
    pub metadata: IdentifiableMetadata,
    /// The dimensions the group selects.
    pub dimensions: GroupDimensions,
}

impl IdentifiableArtefact for Group {
    fn id(&self) -> &str {
        self.metadata.id()
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

// ---------------------------------------------------------------------------
// DimensionList
// ---------------------------------------------------------------------------

/// The ordered key descriptor of a data structure.
///
/// ## Specification
/// - **Type**: `DimensionListType`
/// - **Element**: `<DimensionList>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DimensionListType.md"))]
///
/// Holds the ordered [`Dimension`]s of the key plus an optional [`TimeDimension`] in a separate slot
/// (the time dimension is not a member of the ordered key). The descriptor id is fixed to
/// `DimensionDescriptor`; a stated value differing from it is rejected. A present list holds at least
/// one dimension.
///
/// ## Guarantees
///
/// Always holds at least one [`Dimension`].
///
/// # Examples
///
/// ```
/// use sdmx_types::{ComponentMetadata, ConceptReference, Dimension, DimensionList};
///
/// let metadata =
///     ComponentMetadata::new(Some(String::from("FREQ")), None, None, Vec::new(), Vec::new())?;
/// let concept = ConceptReference {
///     agency: String::from("SDMX"),
///     scheme_id: String::from("CS_FREQ"),
///     version: "1.0.0".parse().unwrap(),
///     id: String::from("FREQ"),
/// };
/// let dimension = Dimension::new(metadata, concept, None, Some(1))?;
/// // A `None` id defaults to the fixed `DimensionDescriptor`.
/// let dimension_list =
///     DimensionList::new(None, vec![dimension], None, Vec::new(), Vec::new(), None)?;
/// assert_eq!(dimension_list.dimensions().len(), 1);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct DimensionList {
    id: Option<String>,
    /// Annotations carried on the descriptor.
    pub annotations: Vec<Annotation>,
    /// Links carried on the descriptor.
    pub links: Vec<Link>,
    /// The descriptor's URN, if any.
    pub urn: Option<String>,
    dimensions: Vec<Dimension>,
    /// The optional time dimension (a separate slot, never a member of the ordered key).
    pub time_dimension: Option<TimeDimension>,
}

impl DimensionList {
    /// Builds a dimension list, rejecting a stated id other than `DimensionDescriptor` and an empty
    /// dimension list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FixedAttributeMismatch`] if a stated id differs from `DimensionDescriptor`,
    /// or [`Error::EmptyDimensionList`] if `dimensions` is empty.
    pub fn new(
        id: Option<String>,
        dimensions: Vec<Dimension>,
        time_dimension: Option<TimeDimension>,
        annotations: Vec<Annotation>,
        links: Vec<Link>,
        urn: Option<String>,
    ) -> Result<Self, Error> {
        validate_fixed("id", id.as_deref(), "DimensionDescriptor")?;
        if dimensions.is_empty() {
            return Err(Error::EmptyDimensionList);
        }
        Ok(Self { id, annotations, links, urn, dimensions, time_dimension })
    }

    /// Stated: the descriptor id as the wire carried it.
    #[must_use]
    pub fn stated_id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// The key dimensions, in wire order (always at least one).
    #[must_use]
    pub fn dimensions(&self) -> &[Dimension] {
        &self.dimensions
    }

    /// Appends a dimension, preserving wire order.
    pub fn push(&mut self, dimension: Dimension) {
        self.dimensions.push(dimension);
    }

    /// The first dimension whose effective id equals `id`, in wire order (a first-match view).
    /// The time dimension is a separate slot and is not searched.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Dimension> {
        self.dimensions.iter().find(|dimension| dimension.id() == id)
    }

    /// Iterates the key dimensions in wire order (always at least one). The time dimension is a
    /// separate slot and is not yielded.
    pub fn iter(&self) -> impl Iterator<Item = &Dimension> {
        self.dimensions.iter()
    }
}

impl<'de> serde::Deserialize<'de> for DimensionList {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            id: Option<String>,
            annotations: Vec<Annotation>,
            links: Vec<Link>,
            urn: Option<String>,
            dimensions: Vec<Dimension>,
            time_dimension: Option<TimeDimension>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.id, raw.dimensions, raw.time_dimension, raw.annotations, raw.links, raw.urn)
            .map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// AttributeList
// ---------------------------------------------------------------------------

/// The attribute descriptor of a data structure.
///
/// ## Specification
/// - **Type**: `AttributeListType`
/// - **Element**: `<AttributeList>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/AttributeListType.md"))]
///
/// Holds the interleaved [`Attribute`] and [`MetadataAttributeUsage`] members in wire order. The
/// descriptor id is fixed to `AttributeDescriptor`. A *present* list holds at least one member of
/// either kind; "no attributes at all" is the absent descriptor (the DSD field is `None`).
///
/// ## Guarantees
///
/// Always holds at least one [`AttributeListMember`].
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     Attribute, AttributeList, AttributeListMember, AttributeRelationship, ComponentMetadata,
///     ConceptReference,
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
/// let attribute =
///     Attribute::new(metadata, concept, None, AttributeRelationship::Observation, None, None)?;
/// let attribute_list = AttributeList::new(
///     None,
///     vec![AttributeListMember::Attribute(attribute)],
///     Vec::new(),
///     Vec::new(),
///     None,
/// )?;
/// assert_eq!(attribute_list.members().len(), 1);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct AttributeList {
    id: Option<String>,
    /// Annotations carried on the descriptor.
    pub annotations: Vec<Annotation>,
    /// Links carried on the descriptor.
    pub links: Vec<Link>,
    /// The descriptor's URN, if any.
    pub urn: Option<String>,
    members: Vec<AttributeListMember>,
}

impl AttributeList {
    /// Builds an attribute list, rejecting a stated id other than `AttributeDescriptor` and an empty
    /// member list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FixedAttributeMismatch`] if a stated id differs from `AttributeDescriptor`,
    /// or [`Error::EmptyAttributeList`] if `members` is empty.
    pub fn new(
        id: Option<String>,
        members: Vec<AttributeListMember>,
        annotations: Vec<Annotation>,
        links: Vec<Link>,
        urn: Option<String>,
    ) -> Result<Self, Error> {
        validate_fixed("id", id.as_deref(), "AttributeDescriptor")?;
        if members.is_empty() {
            return Err(Error::EmptyAttributeList);
        }
        Ok(Self { id, annotations, links, urn, members })
    }

    /// Stated: the descriptor id as the wire carried it.
    #[must_use]
    pub fn stated_id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// The list members, in wire order (always at least one).
    #[must_use]
    pub fn members(&self) -> &[AttributeListMember] {
        &self.members
    }

    /// Appends a member, preserving wire order.
    pub fn push(&mut self, member: AttributeListMember) {
        self.members.push(member);
    }

    /// A filtered view over the [`Attribute`] members, in wire order.
    pub fn attributes(&self) -> impl Iterator<Item = &Attribute> {
        self.members.iter().filter_map(|member| match member {
            AttributeListMember::Attribute(attribute) => Some(attribute),
            AttributeListMember::MetadataAttributeUsage(_) => None,
        })
    }

    /// A filtered view over the [`MetadataAttributeUsage`] members, in wire order.
    pub fn usages(&self) -> impl Iterator<Item = &MetadataAttributeUsage> {
        self.members.iter().filter_map(|member| match member {
            AttributeListMember::MetadataAttributeUsage(usage) => Some(usage),
            AttributeListMember::Attribute(_) => None,
        })
    }

    /// The first attribute whose effective id equals `id`, in wire order (a first-match view).
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Attribute> {
        self.attributes().find(|attribute| attribute.id() == id)
    }
}

impl<'de> serde::Deserialize<'de> for AttributeList {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            id: Option<String>,
            annotations: Vec<Annotation>,
            links: Vec<Link>,
            urn: Option<String>,
            members: Vec<AttributeListMember>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.id, raw.members, raw.annotations, raw.links, raw.urn).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// MeasureList
// ---------------------------------------------------------------------------

/// The measure descriptor of a data structure.
///
/// ## Specification
/// - **Type**: `MeasureListType`
/// - **Element**: `<MeasureList>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/MeasureListType.md"))]
///
/// Holds the [`Measure`]s in wire order (a `Vec`, not a map; lookup is a first-match view).
/// The descriptor id is fixed to `MeasureDescriptor`. A *present* list holds at least one measure;
/// a measure-less data structure is the absent descriptor (the DSD field is `None`).
///
/// ## Guarantees
///
/// Always holds at least one [`Measure`].
///
/// # Examples
///
/// ```
/// use sdmx_types::{ComponentMetadata, ConceptReference, Measure, MeasureList};
///
/// let metadata = ComponentMetadata::new(
///     Some(String::from("OBS_VALUE")),
///     None,
///     None,
///     Vec::new(),
///     Vec::new(),
/// )?;
/// let concept = ConceptReference {
///     agency: String::from("SDMX"),
///     scheme_id: String::from("CS"),
///     version: "1.0.0".parse().unwrap(),
///     id: String::from("OBS_VALUE"),
/// };
/// let measure = Measure::new(metadata, concept, None, None)?;
/// let measure_list = MeasureList::new(None, vec![measure], Vec::new(), Vec::new(), None)?;
/// assert_eq!(measure_list.measures().len(), 1);
/// assert!(measure_list.get("OBS_VALUE").is_some());
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct MeasureList {
    id: Option<String>,
    /// Annotations carried on the descriptor.
    pub annotations: Vec<Annotation>,
    /// Links carried on the descriptor.
    pub links: Vec<Link>,
    /// The descriptor's URN, if any.
    pub urn: Option<String>,
    measures: Vec<Measure>,
}

impl MeasureList {
    /// Builds a measure list, rejecting a stated id other than `MeasureDescriptor` and an empty
    /// measure list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FixedAttributeMismatch`] if a stated id differs from `MeasureDescriptor`, or
    /// [`Error::EmptyMeasureList`] if `measures` is empty.
    pub fn new(
        id: Option<String>,
        measures: Vec<Measure>,
        annotations: Vec<Annotation>,
        links: Vec<Link>,
        urn: Option<String>,
    ) -> Result<Self, Error> {
        validate_fixed("id", id.as_deref(), "MeasureDescriptor")?;
        if measures.is_empty() {
            return Err(Error::EmptyMeasureList);
        }
        Ok(Self { id, annotations, links, urn, measures })
    }

    /// Stated: the descriptor id as the wire carried it.
    #[must_use]
    pub fn stated_id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// The measures, in wire order (always at least one).
    #[must_use]
    pub fn measures(&self) -> &[Measure] {
        &self.measures
    }

    /// Appends a measure, preserving wire order.
    pub fn push(&mut self, measure: Measure) {
        self.measures.push(measure);
    }

    /// The first measure whose effective id equals `id`, in wire order (a first-match view).
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Measure> {
        self.measures.iter().find(|measure| measure.id() == id)
    }

    /// Iterates the measures in wire order (always at least one).
    pub fn iter(&self) -> impl Iterator<Item = &Measure> {
        self.measures.iter()
    }
}

impl<'de> serde::Deserialize<'de> for MeasureList {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            id: Option<String>,
            annotations: Vec<Annotation>,
            links: Vec<Link>,
            urn: Option<String>,
            measures: Vec<Measure>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.id, raw.measures, raw.annotations, raw.links, raw.urn).map_err(to_de_error)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::*;
    use crate::{
        attribute::AttributeRelationship,
        component::ComponentMetadata,
        metadata::IdentifiableMetadata,
        reference::ConceptReference,
        representation::{DataType, Representation, RepresentationChoice, TextFormat},
    };

    fn concept(id: &str) -> ConceptReference {
        ConceptReference {
            agency: String::from("SDMX"),
            scheme_id: String::from("CS"),
            version: "1.0.0".parse().unwrap(),
            id: id.into(),
        }
    }

    fn component_metadata(id: &str) -> ComponentMetadata {
        ComponentMetadata::new(Some(id.into()), None, None, Vec::new(), Vec::new()).unwrap()
    }

    fn dimension(id: &str) -> Dimension {
        Dimension::new(component_metadata(id), concept(id), None, None).unwrap()
    }

    fn time_dimension() -> TimeDimension {
        let representation = Representation {
            choice: RepresentationChoice::TextFormat(TextFormat {
                text_type: Some(DataType::ObservationalTimePeriod),
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
        TimeDimension::new(component_metadata("TIME_PERIOD"), concept("TIME"), representation)
            .unwrap()
    }

    fn attribute(id: &str) -> Attribute {
        Attribute::new(
            component_metadata(id),
            concept(id),
            None,
            AttributeRelationship::Observation,
            None,
            None,
        )
        .unwrap()
    }

    fn measure(id: &str) -> Measure {
        Measure::new(component_metadata(id), concept(id), None, None).unwrap()
    }

    fn group_metadata(id: &str) -> IdentifiableMetadata {
        use crate::annotation::{Annotation, AnnotationUrl, Link};
        IdentifiableMetadata::new(
            id.into(),
            None,
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
        .unwrap()
    }

    // --- GroupDimensions / Group ---

    #[test]
    fn group_dimensions_reject_empty() {
        assert_eq!(GroupDimensions::new(vec![String::from("FREQ")]).unwrap().as_slice().len(), 1);
        assert_eq!(GroupDimensions::new(Vec::new()).unwrap_err(), Error::EmptyGroupDimensions);
    }

    #[test]
    fn group_dimensions_deserialize_enforces_non_empty() {
        // GroupDimensions' Deserialize declares an inner `Raw = Vec<String>` (it does
        // `Vec::<String>::deserialize` then `Self::new(..)`), and its transparent Serialize encodes
        // as that bare vector. postcard is positional, so an empty `Vec<String>` decodes into new(),
        // which rejects it, while a non-empty one round-trips.
        let empty: Vec<String> = Vec::new();
        assert!(
            postcard::from_bytes::<GroupDimensions>(&postcard::to_allocvec(&empty).unwrap())
                .is_err()
        );
        let dimensions = GroupDimensions::new(vec![String::from("FREQ")]).unwrap();
        assert_eq!(dimensions.as_slice().len(), 1);
        crate::test_support::round_trip(&dimensions);
    }

    #[test]
    fn group_forwards_identifiable_and_round_trips() {
        let group = Group {
            metadata: group_metadata("SIBLING"),
            dimensions: GroupDimensions::new(vec![String::from("FREQ"), String::from("CURRENCY")])
                .unwrap(),
        };
        // IdentifiableArtefact delegates every accessor to the metadata leaf.
        assert_eq!(group.id(), "SIBLING");
        assert_eq!(group.urn(), Some("urn:x"));
        assert_eq!(group.uri(), None);
        assert_eq!(group.annotations().len(), 1);
        assert_eq!(group.links().len(), 1);
        assert_eq!(group.dimensions.as_slice().len(), 2);
        crate::test_support::round_trip(&group);
    }

    // --- DimensionList ---

    #[test]
    fn dimension_list_validates_fixed_id_and_non_empty_with_lookup() {
        // The stated fixed `DimensionDescriptor` id passes, and the list keeps wire order.
        let list = DimensionList::new(
            Some(String::from("DimensionDescriptor")),
            vec![dimension("FREQ"), dimension("CURRENCY")],
            Some(time_dimension()),
            Vec::new(),
            Vec::new(),
            None,
        )
        .unwrap();
        assert_eq!(list.stated_id(), Some("DimensionDescriptor"));
        assert_eq!(list.dimensions().len(), 2);
        assert_eq!(list.dimensions()[0].id(), "FREQ");
        assert!(list.time_dimension.is_some());

        // get/iter range over the key dimensions, in wire order.
        assert_eq!(list.iter().count(), 2);
        assert_eq!(list.iter().next().unwrap().id(), "FREQ");
        assert_eq!(list.get("FREQ").map(IdentifiableArtefact::id), Some("FREQ"));
        assert!(list.get("MISSING").is_none());
        // Neither get nor iter surfaces the time dimension (id TIME_PERIOD).
        assert!(list.get("TIME_PERIOD").is_none());

        // A mismatched fixed id is rejected.
        assert_eq!(
            DimensionList::new(
                Some(String::from("Wrong")),
                vec![dimension("FREQ")],
                None,
                Vec::new(),
                Vec::new(),
                None
            )
            .unwrap_err(),
            Error::FixedAttributeMismatch {
                attribute: String::from("id"),
                value: String::from("Wrong")
            }
        );
        // An empty dimension list is rejected.
        assert_eq!(
            DimensionList::new(None, Vec::new(), None, Vec::new(), Vec::new(), None).unwrap_err(),
            Error::EmptyDimensionList
        );
    }

    #[test]
    fn dimension_list_push_and_deserialize() {
        let mut list =
            DimensionList::new(None, vec![dimension("FREQ")], None, Vec::new(), Vec::new(), None)
                .unwrap();
        list.push(dimension("CURRENCY"));
        assert_eq!(list.dimensions().len(), 2);
        crate::test_support::round_trip(&list);

        // DimensionList's Deserialize declares
        // `Raw { id: Option<String>, annotations: Vec<Annotation>, links: Vec<Link>,
        //        urn: Option<String>, dimensions: Vec<Dimension>,
        //        time_dimension: Option<TimeDimension> }`
        // and routes through new(), which rejects an empty dimension list. postcard is positional, so
        // a tuple of those field types carrying an empty `Vec<Dimension>` (every field valid, so Raw
        // deserialises, but rejected by the non-empty rule in new()) proves the wire path re-runs the
        // check.
        // A valid tuple of the same field types decodes — guards this proof's shape against Raw drift.
        let ok = (
            None::<String>,
            Vec::<Annotation>::new(),
            Vec::<Link>::new(),
            None::<String>,
            vec![dimension("FREQ")],
            None::<TimeDimension>,
        );
        assert!(
            postcard::from_bytes::<DimensionList>(&postcard::to_allocvec(&ok).unwrap()).is_ok()
        );
        let raw = (
            None::<String>,
            Vec::<Annotation>::new(),
            Vec::<Link>::new(),
            None::<String>,
            Vec::<Dimension>::new(),
            None::<TimeDimension>,
        );
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<DimensionList>(&bytes).is_err());
    }

    #[test]
    fn dimension_list_get_returns_first_match() {
        // Duplicate ids are held verbatim; get returns the first in wire order.
        let list = DimensionList::new(
            None,
            vec![dimension("FREQ"), dimension("FREQ")],
            None,
            Vec::new(),
            Vec::new(),
            None,
        )
        .unwrap();
        assert_eq!(list.iter().count(), 2);
        assert!(core::ptr::eq(list.get("FREQ").unwrap(), &raw const list.dimensions()[0]));
    }

    // --- AttributeList ---

    #[test]
    fn attribute_list_validates_fixed_id_and_non_empty_with_filtered_views() {
        let usage = MetadataAttributeUsage {
            metadata_attribute_ref: String::from("CONTACT"),
            relationship: AttributeRelationship::Dataflow,
            annotations: Vec::new(),
            link: None,
        };
        let list = AttributeList::new(
            Some(String::from("AttributeDescriptor")),
            vec![
                AttributeListMember::Attribute(attribute("OBS_STATUS")),
                AttributeListMember::MetadataAttributeUsage(usage),
            ],
            Vec::new(),
            Vec::new(),
            None,
        )
        .unwrap();
        assert_eq!(list.stated_id(), Some("AttributeDescriptor"));
        assert_eq!(list.members().len(), 2);
        assert_eq!(list.attributes().count(), 1); // filtered view
        assert_eq!(list.usages().count(), 1);
        assert_eq!(list.get("OBS_STATUS").map(IdentifiableArtefact::id), Some("OBS_STATUS"));
        assert!(list.get("MISSING").is_none());

        assert_eq!(
            AttributeList::new(
                Some(String::from("Wrong")),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                None
            )
            .unwrap_err(),
            Error::FixedAttributeMismatch {
                attribute: String::from("id"),
                value: String::from("Wrong")
            }
        );
        assert_eq!(
            AttributeList::new(None, Vec::new(), Vec::new(), Vec::new(), None).unwrap_err(),
            Error::EmptyAttributeList
        );
    }

    #[test]
    fn attribute_list_push_and_deserialize() {
        let mut list = AttributeList::new(
            None,
            vec![AttributeListMember::Attribute(attribute("OBS_STATUS"))],
            Vec::new(),
            Vec::new(),
            None,
        )
        .unwrap();
        list.push(AttributeListMember::Attribute(attribute("CONF_STATUS")));
        assert_eq!(list.members().len(), 2);
        crate::test_support::round_trip(&list);

        // AttributeList's Deserialize declares
        // `Raw { id: Option<String>, annotations: Vec<Annotation>, links: Vec<Link>,
        //        urn: Option<String>, members: Vec<AttributeListMember> }`
        // and routes through new(), which rejects an empty member list. postcard is positional, so a
        // tuple of those field types carrying an empty `Vec<AttributeListMember>` decodes into new(),
        // which rejects it on the wire.
        // A valid tuple of the same field types decodes — guards this proof's shape against Raw drift.
        let ok = (
            None::<String>,
            Vec::<Annotation>::new(),
            Vec::<Link>::new(),
            None::<String>,
            vec![AttributeListMember::Attribute(attribute("OBS_STATUS"))],
        );
        assert!(
            postcard::from_bytes::<AttributeList>(&postcard::to_allocvec(&ok).unwrap()).is_ok()
        );
        let raw = (
            None::<String>,
            Vec::<Annotation>::new(),
            Vec::<Link>::new(),
            None::<String>,
            Vec::<AttributeListMember>::new(),
        );
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<AttributeList>(&bytes).is_err());
    }

    // --- MeasureList ---

    #[test]
    fn measure_list_validates_fixed_id_and_non_empty_with_lookup() {
        let list = MeasureList::new(
            Some(String::from("MeasureDescriptor")),
            vec![measure("OBS_VALUE"), measure("LOWER_BOUND")],
            Vec::new(),
            Vec::new(),
            None,
        )
        .unwrap();
        assert_eq!(list.stated_id(), Some("MeasureDescriptor"));
        assert_eq!(list.measures().len(), 2);
        assert_eq!(list.measures()[0].id(), "OBS_VALUE");
        assert_eq!(list.iter().count(), 2);
        assert_eq!(list.get("OBS_VALUE").map(IdentifiableArtefact::id), Some("OBS_VALUE"));
        assert!(list.get("MISSING").is_none());

        assert_eq!(
            MeasureList::new(Some(String::from("Wrong")), Vec::new(), Vec::new(), Vec::new(), None)
                .unwrap_err(),
            Error::FixedAttributeMismatch {
                attribute: String::from("id"),
                value: String::from("Wrong")
            }
        );
        assert_eq!(
            MeasureList::new(None, Vec::new(), Vec::new(), Vec::new(), None).unwrap_err(),
            Error::EmptyMeasureList
        );
    }

    #[test]
    fn measure_list_push_and_deserialize() {
        let mut list =
            MeasureList::new(None, vec![measure("OBS_VALUE")], Vec::new(), Vec::new(), None)
                .unwrap();
        list.push(measure("LOWER_BOUND"));
        assert_eq!(list.iter().count(), 2);
        crate::test_support::round_trip(&list);

        // MeasureList's Deserialize declares
        // `Raw { id: Option<String>, annotations: Vec<Annotation>, links: Vec<Link>,
        //        urn: Option<String>, measures: Vec<Measure> }`
        // and routes through new(), which rejects an empty measure list. postcard is positional, so a
        // tuple of those field types carrying an empty `Vec<Measure>` decodes into new(), which
        // rejects it on the wire.
        // A valid tuple of the same field types decodes — guards this proof's shape against Raw drift.
        let ok = (
            None::<String>,
            Vec::<Annotation>::new(),
            Vec::<Link>::new(),
            None::<String>,
            vec![measure("OBS_VALUE")],
        );
        assert!(postcard::from_bytes::<MeasureList>(&postcard::to_allocvec(&ok).unwrap()).is_ok());
        let raw = (
            None::<String>,
            Vec::<Annotation>::new(),
            Vec::<Link>::new(),
            None::<String>,
            Vec::<Measure>::new(),
        );
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<MeasureList>(&bytes).is_err());
    }

    #[test]
    fn group_dimensions_try_from_rejects_empty() {
        assert_eq!(GroupDimensions::try_from(Vec::new()).unwrap_err(), Error::EmptyGroupDimensions);
    }

    #[test]
    fn group_dimensions_into_inner_and_from() {
        let v = vec![String::from("FREQ")];
        assert_eq!(GroupDimensions::new(v.clone()).unwrap().into_inner(), v);
        assert_eq!(Vec::from(GroupDimensions::new(v.clone()).unwrap()), v);
    }

    // Property tests: the internal serde round-trip over the generated descriptor
    // families (see `test_strategy`); wasm32 is excluded with the rest of the property
    // suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::{attribute_list, dimension_list, group, measure_list};

        proptest! {
            #[test]
            fn dimension_list_round_trips(value in dimension_list()) {
                crate::test_support::round_trip(&value);
            }

            #[test]
            fn attribute_list_round_trips(value in attribute_list()) {
                crate::test_support::round_trip(&value);
            }

            #[test]
            fn measure_list_round_trips(value in measure_list()) {
                crate::test_support::round_trip(&value);
            }

            #[test]
            fn group_round_trips(value in group()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
