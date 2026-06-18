//! Measures.
//!
//! A [`Measure`] is a component carrying the observed values of a data structure. SDMX 3.x models
//! multiple measures (replacing the 2.1-era single `PrimaryMeasure`), each taking its identity from
//! a concept and, optionally, a representation; unlike an attribute, a measure has no relationship.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A component (D-0025/D-0028/D-0057): a validated-item type over a [`ComponentMetadata`] leaf with a
[`ConceptReference`] identity, its representation held to the Basic position rules. Its shape mirrors
[`Attribute`](crate::Attribute) minus the relationship. `usage` is stored as `Option<Usage>`
(statedness, D-0052); the schema default `optional` is the `effective_usage()` view.
`IdentifiableArtefact::id()` is the effective identity (the stated id, else the concept's), and
`urn`/`annotations`/`links` delegate to the leaf.

Decisions: D-0025, D-0028, D-0048, D-0052, D-0057.
"#
)]

use crate::{
    annotation::{Annotation, Link},
    artefact::IdentifiableArtefact,
    component::{ComponentMetadata, Usage},
    error::{Error, to_de_error},
    reference::ConceptReference,
    representation::{Representation, validate_basic_representation},
};

/// A component carrying the observed values of a data structure.
///
/// ## Specification
/// - **Type**: `MeasureType`
/// - **Element**: `<Measure>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/MeasureType.md"))]
///
/// A validated component: [`new`](Self::new) holds the representation to the Basic position rules,
/// and the fields are private. The id is optional, inherited from the concept when absent. `usage`
/// stores statedness; its schema default (`optional`) is the [`effective_usage`](Self::effective_usage)
/// view.
///
/// # Examples
///
/// ```
/// use sdmx_types::{ComponentMetadata, ConceptReference, IdentifiableArtefact, Measure};
///
/// let metadata =
///     ComponentMetadata::new(Some("OBS_VALUE".to_string()), None, None, vec![], vec![])?;
/// let concept = ConceptReference {
///     agency: "SDMX".to_string(),
///     scheme_id: "CS".to_string(),
///     id: "OBS_VALUE".to_string(),
/// };
/// let measure = Measure::new(metadata, concept, None, None)?;
/// assert_eq!(measure.id(), "OBS_VALUE");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Measure {
    metadata: ComponentMetadata,
    concept: ConceptReference,
    representation: Option<Representation>,
    usage: Option<Usage>,
}

impl Measure {
    /// Builds a measure, validating its representation against the Basic position rules. The stated
    /// id, if any, was already validated by the [`ComponentMetadata`] leaf.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTextTypeForComponent`] if the representation states a `textType`
    /// outside the Basic subset.
    pub fn new(
        metadata: ComponentMetadata,
        concept: ConceptReference,
        representation: Option<Representation>,
        usage: Option<Usage>,
    ) -> Result<Self, Error> {
        validate_basic_representation("Measure", representation.as_ref())?;
        Ok(Self { metadata, concept, representation, usage })
    }

    /// Stated: the id exactly as the wire carried it. `None` means the id was absent and the
    /// measure inherits its concept's identity.
    #[must_use]
    pub fn stated_id(&self) -> Option<&str> {
        self.metadata.stated_id()
    }

    /// The concept this measure takes its identity from.
    #[must_use]
    pub const fn concept(&self) -> &ConceptReference {
        &self.concept
    }

    /// The measure's local representation, if any.
    #[must_use]
    pub const fn representation(&self) -> Option<&Representation> {
        self.representation.as_ref()
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

impl IdentifiableArtefact for Measure {
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

impl<'de> serde::Deserialize<'de> for Measure {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            metadata: ComponentMetadata,
            concept: ConceptReference,
            representation: Option<Representation>,
            usage: Option<Usage>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.metadata, raw.concept, raw.representation, raw.usage).map_err(to_de_error)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::representation::{DataType, RepresentationChoice, TextFormat};

    fn concept(id: &str) -> ConceptReference {
        ConceptReference { agency: "SDMX".into(), scheme_id: "CS".into(), id: id.into() }
    }

    fn metadata(id: Option<&str>) -> ComponentMetadata {
        ComponentMetadata::new(id.map(Into::into), None, None, vec![], vec![]).unwrap()
    }

    fn text_format(text_type: DataType) -> Representation {
        Representation {
            choice: RepresentationChoice::TextFormat(TextFormat {
                text_type: Some(text_type),
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
        }
    }

    #[test]
    fn measure_id_is_stated_else_inherited() {
        let stated =
            Measure::new(metadata(Some("OBS_VALUE")), concept("OBS_VALUE"), None, None).unwrap();
        assert_eq!(stated.id(), "OBS_VALUE");
        assert_eq!(stated.stated_id(), Some("OBS_VALUE"));

        let inherited = Measure::new(metadata(None), concept("CONCEPT_VALUE"), None, None).unwrap();
        assert_eq!(inherited.id(), "CONCEPT_VALUE");
        assert_eq!(inherited.stated_id(), None);
    }

    #[test]
    fn measure_new_validates_the_basic_representation_rule() {
        // KeyValues is outside the Basic subset.
        assert_eq!(
            Measure::new(
                metadata(Some("OBS_VALUE")),
                concept("OBS_VALUE"),
                Some(text_format(DataType::KeyValues)),
                None,
            )
            .unwrap_err(),
            Error::InvalidTextTypeForComponent("Measure".into(), "KeyValues".into())
        );
        // A Basic textType (and a measure can be coded/typed, D-0028) is accepted.
        assert!(
            Measure::new(
                metadata(Some("OBS_VALUE")),
                concept("OBS_VALUE"),
                Some(text_format(DataType::Double)),
                None,
            )
            .is_ok()
        );
    }

    #[test]
    fn measure_usage_default_is_a_layer_two_view() {
        let absent =
            Measure::new(metadata(Some("OBS_VALUE")), concept("OBS_VALUE"), None, None).unwrap();
        assert_eq!(absent.usage(), None);
        assert_eq!(absent.effective_usage(), Usage::Optional);

        let stated = Measure::new(
            metadata(Some("OBS_VALUE")),
            concept("OBS_VALUE"),
            None,
            Some(Usage::Mandatory),
        )
        .unwrap();
        assert_eq!(stated.usage(), Some(Usage::Mandatory));
        assert_eq!(stated.effective_usage(), Usage::Mandatory);
    }

    #[test]
    fn measure_exposes_concept_and_representation() {
        let measure = Measure::new(
            metadata(Some("OBS_VALUE")),
            concept("OBS_VALUE"),
            Some(text_format(DataType::Double)),
            None,
        )
        .unwrap();
        assert_eq!(measure.concept().id, "OBS_VALUE");
        assert!(measure.representation().is_some());
    }

    #[test]
    fn measure_forwards_identifiable_accessors() {
        use crate::annotation::{Annotation, AnnotationUrl, Link};
        let full = ComponentMetadata::new(
            Some("OBS_VALUE".into()),
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
        let measure = Measure::new(full, concept("OBS_VALUE"), None, None).unwrap();
        assert_eq!(measure.urn(), Some("urn:x"));
        assert_eq!(measure.annotations().len(), 1);
        assert_eq!(measure.links().len(), 1);
    }

    #[test]
    fn measure_deserialize_round_trips() {
        let measure = Measure::new(
            metadata(Some("OBS_VALUE")),
            concept("OBS_VALUE"),
            Some(text_format(DataType::Double)),
            Some(Usage::Mandatory),
        )
        .unwrap();
        let json = serde_json::to_string(&measure).unwrap();
        assert_eq!(serde_json::from_str::<Measure>(&json).unwrap(), measure);
    }

    #[test]
    fn measure_deserialize_enforces_the_rule() {
        // A non-Basic textType is rejected on the wire, routing through new().
        let measure = Measure::new(
            metadata(Some("OBS_VALUE")),
            concept("OBS_VALUE"),
            Some(text_format(DataType::String)),
            None,
        )
        .unwrap();
        let json = serde_json::to_string(&measure).unwrap();
        let bad = json.replace("String", "KeyValues");
        assert!(serde_json::from_str::<Measure>(&bad).is_err());
    }
}
