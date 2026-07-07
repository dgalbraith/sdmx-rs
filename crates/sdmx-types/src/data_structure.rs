//! The data structure definition (DSD).
//!
//! A [`DataStructureDefinition`] is the maintainable artefact that ties the component descriptors
//! together: the dimension list (the key), optional attribute and measure lists, and any groups. It
//! is the structure a [`Dataflow`](crate::Dataflow) describes and a dataset conforms to.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The DSD composes self-enforcing descriptors and owns no cross-field invariant of its own, so by §7's
test it is a pub-field carrier with DERIVED `Deserialize` (the non-empty-dimensions invariant lives
in `DimensionList::new`). `None` for `attribute_list`/`measure_list` is the wire's *absent*
descriptor (a measure-less DSD is an absent measure list, not an empty one, D-0025 as revised by
D-0049). `groups` is a `Vec`, not a map: it preserves wire order and stays uniform with the
descriptor model; `get_group` is a first-match lookup view. `evolving_structure` is a 3.1-only
attribute whose statedness is stored (D-0045/D-0052). The artefact trait hierarchy delegates to the
`metadata` leaf, as on every maintainable.

Decisions: D-0025, D-0045, D-0049, D-0052.
"#
)]

use alloc::vec::Vec;

use chrono::{DateTime, FixedOffset};

use crate::{
    annotation::{Annotation, Link},
    artefact::{IdentifiableArtefact, MaintainableArtefact, NameableArtefact, VersionableArtefact},
    descriptor::{AttributeList, DimensionList, Group, MeasureList},
    lexical::SdmxVersion,
    localised::LocalisedString,
    metadata::MaintainableMetadata,
};

/// A maintainable definition of a data structure: its dimensions, attributes, measures, and groups.
///
/// ## Specification
/// - **Type**: `DataStructureType`
/// - **Element**: `<DataStructure>`
/// - **Editions**: SDMX 3.0 and 3.1 (Divergent)
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataStructureType.3.0.md"))]
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataStructureType.3.1.md"))]
#[cfg_attr(design_docs, doc = "")]
///
/// A pub-field carrier: it composes already-validated descriptors and owns no cross-field invariant,
/// so it derives `Deserialize`. The `dimension_list` is required; `attribute_list` and
/// `measure_list` are `None` when the wire omits the descriptor (a measure-less DSD has no measure
/// list).
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     ComponentMetadata, ConceptReference, DataStructureDefinition, Dimension, DimensionList,
///     IdentifiableArtefact, IdentifiableMetadata, LocalisedString, LocalisedText,
///     MaintainableArtefact, MaintainableMetadata, NameableMetadata, VersionableMetadata,
/// };
///
/// let dimension = Dimension::new(
///     ComponentMetadata::new(Some(String::from("FREQ")), None, None, Vec::new(), Vec::new())?,
///     ConceptReference {
///         agency: String::from("SDMX"),
///         scheme_id: String::from("CS"),
///         version: "1.0.0".parse().unwrap(),
///         id: String::from("FREQ"),
///     },
///     Vec::new(),
///     None,
///     None,
/// )?;
/// let dimension_list =
///     DimensionList::new(None, vec![dimension], None, Vec::new(), Vec::new(), None, None)?;
///
/// let names = LocalisedString::new(vec![LocalisedText {
///     language: Some(String::from("en")),
///     text: String::from("Exchange rates"),
/// }])?;
/// let identifiable =
///     IdentifiableMetadata::new(String::from("ECB_EXR"), None, None, Vec::new(), Vec::new())?;
/// let metadata = MaintainableMetadata::new(
///     VersionableMetadata::new(
///         NameableMetadata::new(identifiable, names, None),
///         None,
///         None,
///         None,
///     ),
///     String::from("ECB"),
///     None,
///     None,
///     None,
///     None,
/// )?;
///
/// let dsd = DataStructureDefinition {
///     metadata,
///     dimension_list,
///     groups: Vec::new(),
///     attribute_list: None,
///     measure_list: None,
///     evolving_structure: None,
/// };
/// assert_eq!(dsd.id(), "ECB_EXR");
/// assert_eq!(dsd.agency(), "ECB");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataStructureDefinition {
    /// The maintainable identity of the data structure.
    pub metadata: MaintainableMetadata,
    /// The ordered key descriptor (required).
    pub dimension_list: DimensionList,
    /// The groups of dimensions, in wire order (a `Vec`, not a map).
    pub groups: Vec<Group>,
    /// The attribute descriptor, or `None` when the wire omits it.
    pub attribute_list: Option<AttributeList>,
    /// The measure descriptor, or `None` for a measure-less structure.
    pub measure_list: Option<MeasureList>,
    /// `evolvingStructure` (3.1-only): statedness stored; `None` ⟺ absent.
    pub evolving_structure: Option<bool>,
}

impl DataStructureDefinition {
    /// Resolves the [`Group`] an [`AttributeRelationship::Group`](crate::AttributeRelationship::Group)
    /// names: a first-match lookup view over the groups in wire order. A duplicate group id is
    /// schema-valid but dubious (a catalogued lint, not a construction error).
    #[must_use]
    pub fn get_group(&self, id: &str) -> Option<&Group> {
        self.groups.iter().find(|group| group.id() == id)
    }
}

impl IdentifiableArtefact for DataStructureDefinition {
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

impl NameableArtefact for DataStructureDefinition {
    fn names(&self) -> &LocalisedString {
        self.metadata.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.metadata.descriptions()
    }
}

impl VersionableArtefact for DataStructureDefinition {
    fn version(&self) -> Option<&SdmxVersion> {
        self.metadata.version()
    }
    fn valid_from(&self) -> Option<&DateTime<FixedOffset>> {
        self.metadata.valid_from()
    }
    fn valid_to(&self) -> Option<&DateTime<FixedOffset>> {
        self.metadata.valid_to()
    }
}

impl MaintainableArtefact for DataStructureDefinition {
    fn agency(&self) -> &str {
        self.metadata.agency()
    }
    fn is_partial_language(&self) -> bool {
        self.metadata.is_partial_language()
    }
    fn is_external_reference(&self) -> bool {
        self.metadata.is_external_reference()
    }
    fn service_url(&self) -> Option<&str> {
        self.metadata.service_url()
    }
    fn structure_url(&self) -> Option<&str> {
        self.metadata.structure_url()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::{string::String, vec};

    use super::*;
    use crate::{
        GroupDimensions,
        attribute::{Attribute, AttributeListMember, AttributeRelationship},
        component::ComponentMetadata,
        dimension::{Dimension, TimeDimension},
        localised::LocalisedText,
        measure::Measure,
        metadata::{IdentifiableMetadata, NameableMetadata, VersionableMetadata},
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
        Dimension::new(component_metadata(id), concept(id), Vec::new(), None, None).unwrap()
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

    fn maintainable(id: &str, agency: &str) -> MaintainableMetadata {
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some(String::from("en")),
            text: String::from("Exchange rates"),
        }])
        .unwrap();
        let identifiable =
            IdentifiableMetadata::new(id.into(), None, None, Vec::new(), Vec::new()).unwrap();
        MaintainableMetadata::new(
            VersionableMetadata::new(
                NameableMetadata::new(identifiable, names, None),
                None,
                None,
                None,
            ),
            agency.into(),
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    /// A complete DSD with every descriptor populated, the milestone exit gate's subject.
    fn complete_dsd() -> DataStructureDefinition {
        let dimension_list = DimensionList::new(
            None,
            vec![dimension("FREQ"), dimension("CURRENCY")],
            Some(time_dimension()),
            Vec::new(),
            Vec::new(),
            None,
            None,
        )
        .unwrap();
        let attribute_list = AttributeList::new(
            None,
            vec![AttributeListMember::Attribute(
                Attribute::new(
                    component_metadata("OBS_STATUS"),
                    concept("OBS_STATUS"),
                    Vec::new(),
                    None,
                    AttributeRelationship::Observation,
                    None,
                    None,
                )
                .unwrap(),
            )],
            Vec::new(),
            Vec::new(),
            None,
            None,
        )
        .unwrap();
        let measure_list = MeasureList::new(
            None,
            vec![
                Measure::new(
                    component_metadata("OBS_VALUE"),
                    concept("OBS_VALUE"),
                    Vec::new(),
                    None,
                    None,
                )
                .unwrap(),
            ],
            Vec::new(),
            Vec::new(),
            None,
            None,
        )
        .unwrap();
        let group = Group {
            metadata: IdentifiableMetadata::new(
                String::from("SIBLING"),
                None,
                None,
                Vec::new(),
                Vec::new(),
            )
            .unwrap(),
            dimensions: GroupDimensions::new(vec![String::from("CURRENCY")]).unwrap(),
        };
        DataStructureDefinition {
            metadata: maintainable("ECB_EXR", "ECB"),
            dimension_list,
            groups: vec![group],
            attribute_list: Some(attribute_list),
            measure_list: Some(measure_list),
            evolving_structure: None,
        }
    }

    /// A maintainable leaf with every optional field populated, for the delegation matrix.
    fn full_maintainable(id: &str, agency: &str) -> MaintainableMetadata {
        use crate::annotation::{Annotation, AnnotationUrl, Link};
        let annotation = Annotation {
            id: Some(String::from("a1")),
            annotation_type: None,
            annotation_title: None,
            annotation_urls: vec![AnnotationUrl {
                url: String::from("https://example.com"),
                lang: Some(String::from("en")),
            }],
            annotation_value: None,
            texts: None,
        };
        let link = Link {
            rel: String::from("self"),
            url: String::from("https://example.com/x"),
            urn: None,
            link_type: None,
        };
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some(String::from("en")),
            text: String::from("Exchange rates"),
        }])
        .unwrap();
        let descriptions = LocalisedString::new(vec![LocalisedText {
            language: Some(String::from("en")),
            text: String::from("How often"),
        }])
        .unwrap();
        let version = SdmxVersion::new(String::from("1.2.3")).unwrap();
        let valid_from = DateTime::parse_from_rfc3339("2024-01-01T00:00:00+00:00").unwrap();
        let identifiable = IdentifiableMetadata::new(
            id.into(),
            Some(String::from("uri")),
            Some(String::from("urn:x")),
            vec![annotation],
            vec![link],
        )
        .unwrap();
        MaintainableMetadata::new(
            VersionableMetadata::new(
                NameableMetadata::new(identifiable, names, Some(descriptions)),
                Some(version),
                Some(valid_from),
                None,
            ),
            agency.into(),
            Some(true),
            Some(true),
            Some(String::from("https://service")),
            Some(String::from("https://structure")),
        )
        .unwrap()
    }

    #[test]
    fn artefact_hierarchy_forwards_every_accessor() {
        let dimension_list = DimensionList::new(
            None,
            vec![dimension("FREQ")],
            None,
            Vec::new(),
            Vec::new(),
            None,
            None,
        )
        .unwrap();
        let dsd = DataStructureDefinition {
            metadata: full_maintainable("ECB_EXR", "ECB"),
            dimension_list,
            groups: Vec::new(),
            attribute_list: None,
            measure_list: None,
            evolving_structure: None,
        };
        assert_eq!(dsd.id(), "ECB_EXR");
        assert_eq!(dsd.urn(), Some("urn:x"));
        assert_eq!(dsd.uri(), Some("uri"));
        assert_eq!(dsd.annotations().len(), 1);
        assert_eq!(dsd.links().len(), 1);
        assert_eq!(dsd.names().first(), "Exchange rates");
        assert_eq!(dsd.descriptions().map(LocalisedString::first), Some("How often"));
        assert_eq!(dsd.version().map(alloc::string::ToString::to_string).as_deref(), Some("1.2.3"));
        assert!(dsd.valid_from().is_some());
        assert_eq!(dsd.valid_to(), None);
        assert_eq!(dsd.agency(), "ECB");
        assert!(dsd.is_partial_language());
        assert!(dsd.is_external_reference());
        assert_eq!(dsd.service_url(), Some("https://service"));
        assert_eq!(dsd.structure_url(), Some("https://structure"));
    }

    #[test]
    fn complete_dsd_constructs_and_exposes_its_artefact_identity() {
        let dsd = complete_dsd();
        assert_eq!(dsd.id(), "ECB_EXR");
        assert_eq!(dsd.names().first(), "Exchange rates");
        assert_eq!(dsd.agency(), "ECB");
        assert_eq!(dsd.dimension_list.dimensions().len(), 2);
        assert!(dsd.dimension_list.time_dimension.is_some());
        assert_eq!(dsd.attribute_list.as_ref().unwrap().attributes().count(), 1);
        assert_eq!(dsd.measure_list.as_ref().unwrap().iter().count(), 1);
    }

    #[test]
    fn get_group_is_a_first_match_view() {
        let dsd = complete_dsd();
        assert_eq!(dsd.get_group("SIBLING").map(IdentifiableArtefact::id), Some("SIBLING"));
        assert!(dsd.get_group("MISSING").is_none());
    }

    #[test]
    fn complete_dsd_round_trips() {
        let dsd = complete_dsd();
        crate::test_support::round_trip(&dsd);
    }

    #[test]
    fn measure_less_dsd_has_no_measure_list() {
        let mut dsd = complete_dsd();
        dsd.measure_list = None;
        let bytes = postcard::to_allocvec(&dsd).unwrap();
        let restored: DataStructureDefinition = postcard::from_bytes(&bytes).unwrap();
        assert!(restored.measure_list.is_none());
        assert_eq!(restored, dsd);
    }

    #[test]
    fn dsd_deserialize_bubbles_the_empty_dimension_list_rejection() {
        // Bubbling demonstration, not a composite-own proof: the non-empty-dimensions invariant is
        // DimensionList's (its source-level proof lives in descriptor.rs). The DSD derives
        // Deserialize over its fields in declaration order (metadata, dimension_list, groups,
        // attribute_list, measure_list, evolving_structure), and DimensionList has a custom
        // Deserialize routing an empty list through new(). Feeding the dimension_list position a
        // DimensionList Raw tuple (id, annotations, links, urn, uri, dimensions, time_dimension)
        // with an empty `dimensions` proves the DSD's derived Deserialize propagates that nested
        // rejection rather than swallowing it.
        let metadata = maintainable("ECB_EXR", "ECB");
        // A valid (non-empty) dimension list decodes — guards the shape against field-order drift.
        let ok = (
            metadata.clone(),
            (
                None::<String>,
                Vec::<Annotation>::new(),
                Vec::<Link>::new(),
                None::<String>,
                None::<String>,
                vec![dimension("FREQ")],
                None::<crate::TimeDimension>,
            ),
            Vec::<Group>::new(),
            None::<AttributeList>,
            None::<MeasureList>,
            None::<bool>,
        );
        assert!(
            postcard::from_bytes::<DataStructureDefinition>(&postcard::to_allocvec(&ok).unwrap())
                .is_ok()
        );
        let raw = (
            metadata,
            (
                None::<String>,
                Vec::<Annotation>::new(),
                Vec::<Link>::new(),
                None::<String>,
                None::<String>,
                Vec::<crate::Dimension>::new(),
                None::<crate::TimeDimension>,
            ),
            Vec::<Group>::new(),
            None::<AttributeList>,
            None::<MeasureList>,
            None::<bool>,
        );
        assert!(
            postcard::from_bytes::<DataStructureDefinition>(&postcard::to_allocvec(&raw).unwrap())
                .is_err()
        );
    }

    // Property tests: the internal serde round-trip over full generated data structures
    // (see `test_strategy`); wasm32 is excluded with the rest of the property suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::data_structure_definition;

        proptest! {
            #[test]
            fn data_structure_definition_round_trips(value in data_structure_definition()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
