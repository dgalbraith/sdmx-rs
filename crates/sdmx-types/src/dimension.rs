//! Dimensions and the time dimension.
//!
//! A [`Dimension`] is a key component of a data structure: it takes its identity from a concept and
//! its values from a codelist or an uncoded text format, held to the `Simple` position rules. A
//! [`TimeDimension`] is the structure's single time slot, its id fixed to `TIME_PERIOD` and its
//! representation a mandatory time text format.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Both are components (D-0028/D-0057): validated-item types with a [`ComponentMetadata`] leaf, a
[`ConceptReference`] identity, private fields, a validated `new()`, and a custom `Deserialize`.
`IdentifiableArtefact::id()` is the EFFECTIVE identity (Layer 2): a dimension's stated id, else its
concept's id; a time dimension's fixed `TIME_PERIOD`. `urn`/`annotations`/`links` delegate to the
leaf.

`Dimension::position` is stored verbatim as `Option<i32>` (D-0022/D-0031): `None` ⟺ the wire
omitted it, `Some(n)` holds even a meaningless negative (the spec types it `xs:int`; coherence is a
Layer-2 lint, not a `new()` rejection). `effective_position` is the Layer-2 view, 1-based per D-0056
(the derived fallback is `list_index + 1`); a `Dimension` does not know its own position, so the
enclosing `DimensionList` passes the index. `TimeDimension` is NOT a member of the ordered key, so
it has no position; its representation is mandatory (no `Option`), with the time text-format rule
enforced at `new()` (D-0048).

Decisions: D-0022, D-0028, D-0031, D-0048, D-0056, D-0057.
"#
)]

use crate::{
    annotation::{Annotation, Link},
    artefact::IdentifiableArtefact,
    component::ComponentMetadata,
    error::{Error, to_de_error},
    reference::ConceptReference,
    representation::{
        Representation, validate_dimension_representation, validate_time_representation,
    },
    validate::validate_fixed,
};

// ---------------------------------------------------------------------------
// Dimension
// ---------------------------------------------------------------------------

/// A key component of a data structure.
///
/// ## Specification
/// - **Type**: `DimensionType`
/// - **Element**: `<Dimension>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DimensionType.md"))]
///
/// A validated component: [`new`](Self::new) holds the representation to the dimension position
/// rules (codelist-only enumeration, the `Simple` `textType` subset, no `isMultiLingual` or
/// representation-level `maxOccurs`), and the fields are private. The id is optional, inherited from
/// the concept when absent.
///
/// # Examples
///
/// ```
/// use sdmx_types::{ComponentMetadata, ConceptReference, Dimension, IdentifiableArtefact};
///
/// let metadata = ComponentMetadata::new(Some("FREQ".to_string()), None, None, vec![], vec![])?;
/// let concept = ConceptReference {
///     agency: "SDMX".to_string(),
///     scheme_id: "CS_FREQ".to_string(),
///     version: "1.0.0".parse().unwrap(),
///     id: "FREQ".to_string(),
/// };
/// let dimension = Dimension::new(metadata, concept, None, Some(1))?;
/// assert_eq!(dimension.id(), "FREQ");
/// assert_eq!(dimension.effective_position(0), 1);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Dimension {
    metadata: ComponentMetadata,
    concept: ConceptReference,
    representation: Option<Representation>,
    position: Option<i32>,
}

impl Dimension {
    /// Builds a dimension, validating its representation against the dimension position rules.
    /// The stated id, if any, was already validated by the [`ComponentMetadata`] leaf.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ValueListEnumerationNotAllowed`], [`Error::ProhibitedRepresentationFacet`],
    /// or [`Error::InvalidTextTypeForComponent`] if the representation breaks a dimension position
    /// rule.
    pub fn new(
        metadata: ComponentMetadata,
        concept: ConceptReference,
        representation: Option<Representation>,
        position: Option<i32>,
    ) -> Result<Self, Error> {
        validate_dimension_representation(representation.as_ref())?;
        Ok(Self { metadata, concept, representation, position })
    }

    /// Stated: the id exactly as the wire carried it. `None` means the id was absent and the
    /// dimension inherits its concept's identity.
    #[must_use]
    pub fn stated_id(&self) -> Option<&str> {
        self.metadata.stated_id()
    }

    /// The concept this dimension takes its identity from.
    #[must_use]
    pub const fn concept(&self) -> &ConceptReference {
        &self.concept
    }

    /// The dimension's local representation, if any. `None` means it inherits its concept's.
    #[must_use]
    pub const fn representation(&self) -> Option<&Representation> {
        self.representation.as_ref()
    }

    /// Stated: the position exactly as the wire carried it. `None` means it was omitted.
    #[must_use]
    pub const fn position(&self) -> Option<i32> {
        self.position
    }

    /// Effective: the 1-based position. This is the stated value if present,
    /// otherwise it is derived from the dimension's `list_index` in the enclosing
    /// [`DimensionList`](crate::DimensionList).
    /// The index is passed in because a `Dimension` alone does not know its position in the parent.
    #[must_use]
    pub fn effective_position(&self, list_index: usize) -> i32 {
        self.position
            .unwrap_or_else(|| i32::try_from(list_index.saturating_add(1)).unwrap_or(i32::MAX))
    }
}

impl IdentifiableArtefact for Dimension {
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

impl<'de> serde::Deserialize<'de> for Dimension {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            metadata: ComponentMetadata,
            concept: ConceptReference,
            representation: Option<Representation>,
            position: Option<i32>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.metadata, raw.concept, raw.representation, raw.position).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// TimeDimension
// ---------------------------------------------------------------------------

/// The single time slot of a data structure.
///
/// ## Specification
/// - **Type**: `TimeDimensionType`
/// - **Element**: `<TimeDimension>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/TimeDimensionType.md"))]
///
/// A validated component whose id is fixed to `TIME_PERIOD`: a stated id differing from it is
/// rejected, and the effective [`id`](IdentifiableArtefact::id) is always `TIME_PERIOD`. Its
/// representation is mandatory and must be a time text format ([`new`](Self::new) enforces both, the
/// time position rules). Unlike a [`Dimension`], it is not a member of the ordered key, so
/// it has no position.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     ComponentMetadata, ConceptReference, DataType, IdentifiableArtefact, Representation,
///     RepresentationChoice, TextFormat, TimeDimension,
/// };
///
/// let metadata = ComponentMetadata::new(None, None, None, vec![], vec![])?;
/// let concept = ConceptReference {
///     agency: "SDMX".to_string(),
///     scheme_id: "CS_TIME".to_string(),
///     version: "1.0.0".parse().unwrap(),
///     id: "TIME_PERIOD".to_string(),
/// };
/// // `TextFormat` derives `Default`, so only the facets that matter need naming.
/// let representation = Representation {
///     choice: RepresentationChoice::TextFormat(TextFormat {
///         text_type: Some(DataType::ObservationalTimePeriod),
///         ..Default::default()
///     }),
///     min_occurs: None,
///     max_occurs: None,
/// };
/// let time_dimension = TimeDimension::new(metadata, concept, representation)?;
/// assert_eq!(time_dimension.id(), "TIME_PERIOD");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct TimeDimension {
    metadata: ComponentMetadata,
    concept: ConceptReference,
    representation: Representation,
}

impl TimeDimension {
    /// Builds a time dimension, rejecting a stated id other than `TIME_PERIOD` and validating its
    /// mandatory representation against the time position rules.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FixedAttributeMismatch`] if a stated id differs from `TIME_PERIOD`, or
    /// [`Error::EnumerationNotAllowed`], [`Error::InvalidTextTypeForComponent`], or
    /// [`Error::ProhibitedRepresentationFacet`] if the representation breaks a time position rule.
    pub fn new(
        metadata: ComponentMetadata,
        concept: ConceptReference,
        representation: Representation,
    ) -> Result<Self, Error> {
        validate_fixed("id", metadata.stated_id(), "TIME_PERIOD")?;
        validate_time_representation(&representation)?;
        Ok(Self { metadata, concept, representation })
    }

    /// Stated: the id exactly as the wire carried it. `None` means it was absent; the effective id
    /// is `TIME_PERIOD` either way.
    #[must_use]
    pub fn stated_id(&self) -> Option<&str> {
        self.metadata.stated_id()
    }

    /// The concept this time dimension takes its identity from.
    #[must_use]
    pub const fn concept(&self) -> &ConceptReference {
        &self.concept
    }

    /// The time dimension's mandatory time representation.
    #[must_use]
    pub const fn representation(&self) -> &Representation {
        &self.representation
    }
}

impl IdentifiableArtefact for TimeDimension {
    // The trait fixes the signature as `&self -> &str`; the effective id is the fixed literal
    // regardless of `self`, so the return cannot be narrowed to `&'static str` without breaking
    // the trait (D-0057).
    #[allow(clippy::unnecessary_literal_bound)]
    fn id(&self) -> &str {
        "TIME_PERIOD"
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

impl<'de> serde::Deserialize<'de> for TimeDimension {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            metadata: ComponentMetadata,
            concept: ConceptReference,
            representation: Representation,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.metadata, raw.concept, raw.representation).map_err(to_de_error)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::representation::{DataType, RepresentationChoice, TextFormat};

    fn concept(id: &str) -> ConceptReference {
        ConceptReference {
            agency: "SDMX".into(),
            scheme_id: "CS_FREQ".into(),
            version: "1.0.0".parse().unwrap(),
            id: id.into(),
        }
    }

    fn metadata(id: Option<&str>) -> ComponentMetadata {
        ComponentMetadata::new(id.map(Into::into), None, None, vec![], vec![]).unwrap()
    }

    fn time_text_format() -> Representation {
        Representation {
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
        }
    }

    #[test]
    fn dimension_id_is_stated_else_inherited_from_concept() {
        // A stated id is the effective id.
        let stated =
            Dimension::new(metadata(Some("FREQ")), concept("CONCEPT_FREQ"), None, None).unwrap();
        assert_eq!(stated.id(), "FREQ");
        assert_eq!(stated.stated_id(), Some("FREQ"));

        // An absent id inherits the concept's id as the effective identity.
        let inherited =
            Dimension::new(metadata(None), concept("CONCEPT_FREQ"), None, None).unwrap();
        assert_eq!(inherited.id(), "CONCEPT_FREQ");
        assert_eq!(inherited.stated_id(), None);
    }

    #[test]
    fn dimension_new_validates_the_representation_position_rule() {
        let value_list = Representation {
            choice: RepresentationChoice::Enumeration {
                enumeration: crate::representation::EnumerationReference::ValueList(
                    crate::reference::ValueListReference {
                        agency: "SDMX".into(),
                        id: "VL".into(),
                        version: "1.0.0".parse().unwrap(),
                    },
                ),
                format: None,
            },
            min_occurs: None,
            max_occurs: None,
        };
        assert_eq!(
            Dimension::new(metadata(Some("FREQ")), concept("FREQ"), Some(value_list), None)
                .unwrap_err(),
            Error::ValueListEnumerationNotAllowed("Dimension".into())
        );
    }

    #[test]
    fn dimension_effective_position_is_stated_else_one_based_index() {
        let stated =
            Dimension::new(metadata(Some("FREQ")), concept("FREQ"), None, Some(5)).unwrap();
        assert_eq!(stated.effective_position(0), 5);
        assert_eq!(stated.position(), Some(5));

        let derived = Dimension::new(metadata(Some("FREQ")), concept("FREQ"), None, None).unwrap();
        assert_eq!(derived.effective_position(0), 1); // first dimension -> position 1
        assert_eq!(derived.effective_position(3), 4);
        assert_eq!(derived.position(), None);
    }

    #[test]
    fn dimension_deserialize_round_trips() {
        let dimension =
            Dimension::new(metadata(Some("FREQ")), concept("CONCEPT_FREQ"), None, Some(1)).unwrap();
        crate::test_support::round_trip(&dimension);
    }

    #[test]
    fn dimension_deserialize_rejects_a_value_list_enumeration() {
        // Dimension's Deserialize declares `Raw { metadata, concept, representation, position }` and
        // routes through new(), which forbids a ValueList enumeration on a dimension. postcard is
        // positional, so a tuple of those field types carrying a well-formed ValueList
        // representation (so Raw deserialises, but rejected by the codelist-only rule in new())
        // proves the wire path re-runs the check.
        let value_list = Representation {
            choice: RepresentationChoice::Enumeration {
                enumeration: crate::representation::EnumerationReference::ValueList(
                    crate::reference::ValueListReference {
                        agency: "SDMX".into(),
                        id: "VL".into(),
                        version: "1.0.0".parse().unwrap(),
                    },
                ),
                format: None,
            },
            min_occurs: None,
            max_occurs: None,
        };
        // A valid tuple of the same field types decodes — guards this proof's shape against Raw drift.
        let ok = (metadata(Some("FREQ")), concept("FREQ"), None::<Representation>, None::<i32>);
        assert!(postcard::from_bytes::<Dimension>(&postcard::to_allocvec(&ok).unwrap()).is_ok());
        let raw = (metadata(Some("FREQ")), concept("FREQ"), Some(value_list), None::<i32>);
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<Dimension>(&bytes).is_err());
    }

    #[test]
    fn time_dimension_id_is_always_time_period() {
        // An absent id, and the explicitly stated fixed value, both yield TIME_PERIOD.
        let absent =
            TimeDimension::new(metadata(None), concept("TIME"), time_text_format()).unwrap();
        assert_eq!(absent.id(), "TIME_PERIOD");
        assert_eq!(absent.stated_id(), None); // the wire omitted it; the effective id still holds
        let stated =
            TimeDimension::new(metadata(Some("TIME_PERIOD")), concept("TIME"), time_text_format())
                .unwrap();
        assert_eq!(stated.id(), "TIME_PERIOD");
        assert_eq!(stated.stated_id(), Some("TIME_PERIOD"));
    }

    #[test]
    fn time_dimension_rejects_a_mismatched_fixed_id() {
        assert_eq!(
            TimeDimension::new(metadata(Some("OBS_TIME")), concept("TIME"), time_text_format())
                .unwrap_err(),
            Error::FixedAttributeMismatch { attribute: "id".into(), value: "OBS_TIME".into() }
        );
    }

    #[test]
    fn time_dimension_validates_the_time_representation_rule() {
        // A non-time textType is outside the Time subset.
        let mut bad = time_text_format();
        if let RepresentationChoice::TextFormat(text_format) = &mut bad.choice {
            text_format.text_type = Some(DataType::String);
        }
        assert_eq!(
            TimeDimension::new(metadata(None), concept("TIME"), bad).unwrap_err(),
            Error::InvalidTextTypeForComponent {
                component: "TimeDimension".into(),
                text_type: "String".into()
            }
        );
    }

    #[test]
    fn time_dimension_deserialize_round_trips() {
        let time_dimension =
            TimeDimension::new(metadata(None), concept("TIME"), time_text_format()).unwrap();
        crate::test_support::round_trip(&time_dimension);
    }

    #[test]
    fn time_dimension_deserialize_rejects_a_mismatched_fixed_id() {
        // TimeDimension's Deserialize declares `Raw { metadata, concept, representation }` and routes
        // through new(), which checks the stated id against the fixed literal "TIME_PERIOD".
        // postcard is positional, so a tuple of those field types carrying a stated id other than
        // TIME_PERIOD (a valid IDType, so Raw deserialises, with a valid time representation so only
        // the fixed-id check fires) proves the wire path re-runs the check.
        // A valid tuple of the same field types decodes — guards this proof's shape against Raw drift.
        let ok = (metadata(None), concept("TIME"), time_text_format());
        assert!(
            postcard::from_bytes::<TimeDimension>(&postcard::to_allocvec(&ok).unwrap()).is_ok()
        );
        let raw = (metadata(Some("OBS_TIME")), concept("TIME"), time_text_format());
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<TimeDimension>(&bytes).is_err());
    }

    /// Component metadata with the URN, an annotation, and a link populated, to exercise the
    /// `IdentifiableArtefact` delegation down to the leaf.
    fn full_metadata(id: Option<&str>) -> ComponentMetadata {
        use crate::annotation::{Annotation, AnnotationUrl, Link};
        let annotation = Annotation {
            id: Some("a1".into()),
            annotation_type: None,
            annotation_title: None,
            annotation_urls: vec![AnnotationUrl {
                url: "https://example.com".into(),
                lang: Some("en".into()),
            }],
            annotation_value: None,
            texts: None,
        };
        let link = Link {
            rel: "self".into(),
            url: "https://example.com/x".into(),
            urn: None,
            link_type: None,
        };
        ComponentMetadata::new(
            id.map(Into::into),
            Some("uri".into()),
            Some("urn:x".into()),
            vec![annotation],
            vec![link],
        )
        .unwrap()
    }

    #[test]
    fn dimension_forwards_identifiable_accessors_and_exposes_its_fields() {
        let concept_ref = concept("CONCEPT_FREQ");
        let dimension =
            Dimension::new(full_metadata(Some("FREQ")), concept_ref.clone(), None, None).unwrap();
        // urn/annotations/links delegate to the ComponentMetadata leaf.
        assert_eq!(dimension.urn(), Some("urn:x"));
        assert_eq!(dimension.uri(), Some("uri"));
        assert_eq!(dimension.annotations().len(), 1);
        assert_eq!(dimension.links().len(), 1);
        // The component's own field accessors.
        assert_eq!(dimension.concept(), &concept_ref);
        assert!(dimension.representation().is_none());
    }

    #[test]
    fn time_dimension_forwards_identifiable_accessors_and_exposes_its_fields() {
        let concept_ref = concept("TIME");
        let time_dimension =
            TimeDimension::new(full_metadata(None), concept_ref.clone(), time_text_format())
                .unwrap();
        assert_eq!(time_dimension.urn(), Some("urn:x"));
        assert_eq!(time_dimension.uri(), Some("uri"));
        assert_eq!(time_dimension.annotations().len(), 1);
        assert_eq!(time_dimension.links().len(), 1);
        assert_eq!(time_dimension.concept(), &concept_ref);
        assert!(matches!(
            time_dimension.representation().choice,
            RepresentationChoice::TextFormat(_)
        ));
    }

    // Property tests: the internal serde round-trip over generated position-valid
    // components (see `test_strategy`); wasm32 is excluded with the rest of the property
    // suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::{dimension, time_dimension};

        proptest! {
            #[test]
            fn dimension_round_trips(value in dimension()) {
                crate::test_support::round_trip(&value);
            }

            #[test]
            fn time_dimension_round_trips(value in time_dimension()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
