//! Concepts and concept schemes.
//!
//! A [`Concept`] is the validated-item exemplar: its id is `NCNameIDType`, stricter than the base
//! `IDType`, so it owns a fallible constructor and private fields. A [`ConceptScheme`] is the
//! maintainable scheme of concepts, with the same `NCName` scheme-id invariant as a codelist.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`Concept` is the VALIDATED-ITEM exemplar (D-0023): its id is `NCNameIDType`, stricter than the base
`IDType`, so it re-validates its own id, owns a validated `new()`, has private fields, and carries a
custom `Deserialize` (§7). The re-validation is a harmless redundancy (every `NCName` is a valid
`IDType`); the two-layer errors are intentional (an `@`-id reports `InvalidIdentifier` from the
base, a `1abc`-id reports `InvalidNcNameIdentifier` here). `parent_id` is a reference, structural
only (D-0020), not NCName-validated.

The `CoreRepresentation` is modelled with the same `Representation` type as components (D-0028), and
its position rules are the Basic tier (D-0048): the same `validate_basic_representation` the
attribute and measure constructors use.

`ConceptScheme` follows `Codelist`'s wrapper shape: a private `scheme`, a fallible `new()` that
re-validates the scheme id as `NCNameIDType`, and a custom `Deserialize` routing through it.

Decisions: D-0020, D-0023, D-0028, D-0048.
"#
)]

use alloc::string::String;

use chrono::{DateTime, FixedOffset};

use crate::{
    annotation::{Annotation, Link},
    artefact::{IdentifiableArtefact, MaintainableArtefact, NameableArtefact, VersionableArtefact},
    error::{Error, to_de_error},
    lexical::SdmxVersion,
    localised::LocalisedString,
    metadata::{MaintainableMetadata, NameableMetadata},
    representation::{Representation, validate_basic_representation},
    scheme::{ItemScheme, SchemeItem},
    validate::validate_ncname,
};

// ---------------------------------------------------------------------------
// Concept
// ---------------------------------------------------------------------------

/// A concept: a unit of meaning a component can take its identity and representation from.
///
/// ## Specification
/// - **Type**: `ConceptType`
/// - **Element**: `<Concept>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ConceptType.md"))]
///
/// A validated item: its id is `NCNameIDType`, so [`new`](Self::new) re-validates it and is
/// fallible, and the fields are private. The optional core representation declares the concept's
/// default data type or enumeration, held to the Basic-tier position rules.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     Concept, IdentifiableArtefact, IdentifiableMetadata, LocalisedString, LocalisedText,
///     NameableMetadata,
/// };
///
/// let names = LocalisedString::new(vec![LocalisedText {
///     language: Some("en".to_string()),
///     text: "Frequency".to_string(),
/// }])?;
/// let identifiable = IdentifiableMetadata::new("FREQ".to_string(), None, None, vec![], vec![])?;
/// let concept = Concept::new(NameableMetadata::new(identifiable, names, None), None, None)?;
/// assert_eq!(concept.id(), "FREQ");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Concept {
    metadata: NameableMetadata,
    parent_id: Option<String>,
    core_representation: Option<Representation>,
}

impl Concept {
    /// Builds a concept, re-validating its id against SDMX `NCNameIDType` and its core
    /// representation against the Basic-tier position rules.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNcNameIdentifier`] if the id is not a valid `NCNameIDType`, or
    /// [`Error::InvalidTextTypeForComponent`] if the core representation states a `textType`
    /// outside the Basic subset.
    pub fn new(
        metadata: NameableMetadata,
        parent_id: Option<String>,
        core_representation: Option<Representation>,
    ) -> Result<Self, Error> {
        validate_ncname(metadata.id())?;
        validate_basic_representation("Concept", core_representation.as_ref())?;
        Ok(Self { metadata, parent_id, core_representation })
    }

    /// The id of the parent concept in a hierarchy, if any. A structural reference, not
    /// re-validated.
    #[must_use]
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_deref()
    }

    /// The concept's core representation, if any.
    #[must_use]
    pub const fn core_representation(&self) -> Option<&Representation> {
        self.core_representation.as_ref()
    }
}

impl IdentifiableArtefact for Concept {
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

impl NameableArtefact for Concept {
    fn names(&self) -> &LocalisedString {
        self.metadata.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.metadata.descriptions()
    }
}

impl SchemeItem for Concept {}

impl<'de> serde::Deserialize<'de> for Concept {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            metadata: NameableMetadata,
            parent_id: Option<String>,
            core_representation: Option<Representation>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.metadata, raw.parent_id, raw.core_representation).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// ConceptScheme
// ---------------------------------------------------------------------------

/// A maintainable scheme of [`Concept`]s.
///
/// ## Specification
/// - **Type**: `ConceptSchemeType`
/// - **Element**: `<ConceptScheme>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ConceptSchemeType.md"))]
///
/// Wraps an [`ItemScheme<Concept>`](ItemScheme). Its scheme id is `NCNameIDType` (the same
/// invariant as [`Codelist`](crate::Codelist)), so [`new`](Self::new) re-validates and is fallible.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     ConceptScheme, IdentifiableMetadata, LocalisedString, LocalisedText, MaintainableArtefact,
///     MaintainableMetadata, NameableMetadata, VersionableMetadata,
/// };
///
/// let names = LocalisedString::new(vec![LocalisedText {
///     language: Some("en".to_string()),
///     text: "Concepts".to_string(),
/// }])?;
/// let identifiable = IdentifiableMetadata::new("CS_X".to_string(), None, None, vec![], vec![])?;
/// let versionable = VersionableMetadata::new(
///     NameableMetadata::new(identifiable, names, None),
///     None,
///     None,
///     None,
/// );
/// let metadata =
///     MaintainableMetadata::new(versionable, "SDMX".to_string(), None, None, None, None)?;
///
/// let scheme = ConceptScheme::new(metadata, None)?;
/// assert_eq!(scheme.agency(), "SDMX");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ConceptScheme {
    scheme: ItemScheme<Concept>,
}

impl ConceptScheme {
    /// Builds an empty concept scheme, validating the scheme id against SDMX `NCNameIDType`.
    /// Concepts are added with [`push`](Self::push).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNcNameIdentifier`] if the scheme id is not a valid `NCNameIDType`.
    pub fn new(metadata: MaintainableMetadata, is_partial: Option<bool>) -> Result<Self, Error> {
        validate_ncname(metadata.id())?;
        Ok(Self { scheme: ItemScheme::new(metadata, is_partial) })
    }

    /// Appends a concept, preserving wire order.
    pub fn push(&mut self, concept: Concept) {
        self.scheme.push(concept);
    }

    /// The first concept whose id equals `id`, in wire order (a first-match view).
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Concept> {
        self.scheme.get(id)
    }

    /// Iterates the concepts in wire order.
    pub fn iter(&self) -> impl Iterator<Item = &Concept> {
        self.scheme.iter()
    }

    /// The effective value of the scheme's `isPartial` flag (schema default `false`).
    #[must_use]
    pub fn is_partial(&self) -> bool {
        self.scheme.is_partial()
    }
}

impl IdentifiableArtefact for ConceptScheme {
    fn id(&self) -> &str {
        self.scheme.id()
    }
    fn urn(&self) -> Option<&str> {
        self.scheme.urn()
    }
    fn uri(&self) -> Option<&str> {
        self.scheme.uri()
    }
    fn annotations(&self) -> &[Annotation] {
        self.scheme.annotations()
    }
    fn links(&self) -> &[Link] {
        self.scheme.links()
    }
}

impl NameableArtefact for ConceptScheme {
    fn names(&self) -> &LocalisedString {
        self.scheme.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.scheme.descriptions()
    }
}

impl VersionableArtefact for ConceptScheme {
    fn version(&self) -> Option<&SdmxVersion> {
        self.scheme.version()
    }
    fn valid_from(&self) -> Option<&DateTime<FixedOffset>> {
        self.scheme.valid_from()
    }
    fn valid_to(&self) -> Option<&DateTime<FixedOffset>> {
        self.scheme.valid_to()
    }
}

impl MaintainableArtefact for ConceptScheme {
    fn agency(&self) -> &str {
        self.scheme.agency()
    }
    fn is_partial_language(&self) -> bool {
        self.scheme.is_partial_language()
    }
    fn is_external_reference(&self) -> bool {
        self.scheme.is_external_reference()
    }
    fn service_url(&self) -> Option<&str> {
        self.scheme.service_url()
    }
    fn structure_url(&self) -> Option<&str> {
        self.scheme.structure_url()
    }
}

impl<'de> serde::Deserialize<'de> for ConceptScheme {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            scheme: ItemScheme<Concept>,
        }
        let raw = Raw::deserialize(deserializer)?;
        // Route through new() so the NCName scheme-id invariant is enforced, then restore the items.
        let ItemScheme { metadata, is_partial, items } = raw.scheme;
        let mut scheme = Self::new(metadata, is_partial).map_err(to_de_error)?;
        scheme.scheme.items = items;
        Ok(scheme)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::{
        localised::LocalisedText,
        metadata::{IdentifiableMetadata, VersionableMetadata},
        representation::{DataType, RepresentationChoice, TextFormat},
    };

    fn nameable(id: &str) -> NameableMetadata {
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "Frequency".into(),
        }])
        .unwrap();
        let identifiable =
            IdentifiableMetadata::new(id.into(), None, None, vec![], vec![]).unwrap();
        NameableMetadata::new(identifiable, names, None)
    }

    fn scheme_metadata(id: &str) -> MaintainableMetadata {
        MaintainableMetadata::new(
            VersionableMetadata::new(nameable(id), None, None, None),
            "SDMX".into(),
            None,
            None,
            None,
            None,
        )
        .unwrap()
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
    fn concept_new_validates_id_as_ncname() {
        assert!(Concept::new(nameable("FREQ"), None, None).is_ok());
        // A leading-digit id is a valid IDType but not an NCNameIDType.
        assert_eq!(
            Concept::new(nameable("1FREQ"), None, None).unwrap_err(),
            Error::InvalidNcNameIdentifier("1FREQ".into())
        );
    }

    #[test]
    fn concept_new_validates_core_representation_at_basic_tier() {
        // A Basic textType is fine; a reference/key type is outside the Basic subset.
        assert!(Concept::new(nameable("FREQ"), None, Some(text_format(DataType::String))).is_ok());
        assert_eq!(
            Concept::new(nameable("FREQ"), None, Some(text_format(DataType::KeyValues)))
                .unwrap_err(),
            Error::InvalidTextTypeForComponent {
                component: "Concept".into(),
                text_type: "KeyValues".into()
            }
        );
    }

    #[test]
    fn concept_exposes_parent_and_core_representation() {
        let concept = Concept::new(
            nameable("FREQ"),
            Some("PARENT".into()),
            Some(text_format(DataType::String)),
        )
        .unwrap();
        assert_eq!(concept.parent_id(), Some("PARENT"));
        assert!(concept.core_representation().is_some());
    }

    #[test]
    fn concept_deserialize_round_trips() {
        let concept = Concept::new(
            nameable("FREQ"),
            Some("PARENT".into()),
            Some(text_format(DataType::String)),
        )
        .unwrap();
        crate::test_support::round_trip(&concept);
    }

    #[test]
    fn concept_new_enforces_ncname_id() {
        // The NCName id tightening is Concept::new's (composite over the nested nameable id): a
        // leading-digit id passes IDType but fails NCName.
        assert!(Concept::new(nameable("1FREQ"), None, None).is_err());
    }

    #[test]
    fn concept_deserialize_rejects_non_ncname_id() {
        // Concept's Deserialize declares `Raw { metadata, parent_id, core_representation }` and
        // routes through new(). postcard is positional, so a tuple of those field types carrying a
        // leading-digit id (valid IDType, so the nested metadata deserialises, but rejected by the
        // NCName tightening in Concept::new) proves the wire path re-runs the check.
        // A valid tuple of the same field types decodes — guards this proof's shape against Raw drift.
        let ok = (nameable("FREQ"), Option::<String>::None, Option::<Representation>::None);
        assert!(postcard::from_bytes::<Concept>(&postcard::to_allocvec(&ok).unwrap()).is_ok());
        let raw = (nameable("1FREQ"), Option::<String>::None, Option::<Representation>::None);
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<Concept>(&bytes).is_err());
    }

    #[test]
    fn concept_scheme_validates_id_and_forwards() {
        assert!(ConceptScheme::new(scheme_metadata("CS_X"), None).is_ok());
        // A leading-digit id passes IDType but fails the NCName tightening.
        assert_eq!(
            ConceptScheme::new(scheme_metadata("9CS"), None).unwrap_err(),
            Error::InvalidNcNameIdentifier("9CS".into())
        );

        let mut scheme = ConceptScheme::new(scheme_metadata("CS_X"), None).unwrap();
        scheme.push(Concept::new(nameable("FREQ"), None, None).unwrap());
        assert_eq!(scheme.get("FREQ").map(IdentifiableArtefact::id), Some("FREQ"));
        assert_eq!(scheme.iter().count(), 1);
    }

    #[test]
    fn concept_scheme_deserialize_round_trips() {
        let mut scheme = ConceptScheme::new(scheme_metadata("CS_X"), None).unwrap();
        scheme.push(Concept::new(nameable("FREQ"), None, None).unwrap());
        crate::test_support::round_trip(&scheme);
    }

    #[test]
    fn concept_scheme_deserialize_rejects_non_ncname_id() {
        // ConceptScheme's Deserialize declares `Raw { scheme: ItemScheme<Concept> }` and routes the
        // metadata through new(), which applies the NCName tightening. postcard is positional and a
        // single-field struct encodes exactly as its one field, so an ItemScheme carrying a
        // leading-digit scheme id (valid IDType, so ItemScheme deserialises) is rejected by new().
        // A valid ItemScheme decodes — guards the single-field shape if the Raw grows a second field.
        let ok = ItemScheme::<Concept>::new(scheme_metadata("CS_X"), None);
        assert!(
            postcard::from_bytes::<ConceptScheme>(&postcard::to_allocvec(&ok).unwrap()).is_ok()
        );
        let scheme = ItemScheme::<Concept>::new(scheme_metadata("9CS"), None);
        let bytes = postcard::to_allocvec(&scheme).unwrap();
        assert!(postcard::from_bytes::<ConceptScheme>(&bytes).is_err());
    }

    /// A nameable leaf with every optional field populated, for the delegation matrix.
    fn full_nameable(id: &str) -> NameableMetadata {
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
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "Frequency".into(),
        }])
        .unwrap();
        let descriptions = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "How often".into(),
        }])
        .unwrap();
        let identifiable = IdentifiableMetadata::new(
            id.into(),
            Some("uri".into()),
            Some("urn:x".into()),
            vec![annotation],
            vec![link],
        )
        .unwrap();
        NameableMetadata::new(identifiable, names, Some(descriptions))
    }

    #[test]
    fn delegation_matrix_forwards_every_accessor() {
        let version = SdmxVersion::new("1.2.3".into()).unwrap();
        let valid_from = DateTime::parse_from_rfc3339("2024-01-01T00:00:00+00:00").unwrap();
        let metadata = MaintainableMetadata::new(
            VersionableMetadata::new(full_nameable("CS_X"), Some(version), Some(valid_from), None),
            "ESTAT".into(),
            Some(true),
            Some(true),
            Some("https://service".into()),
            Some("https://structure".into()),
        )
        .unwrap();
        let scheme = ConceptScheme::new(metadata, Some(true)).unwrap();

        assert_eq!(scheme.id(), "CS_X");
        assert_eq!(scheme.urn(), Some("urn:x"));
        assert_eq!(scheme.uri(), Some("uri"));
        assert_eq!(scheme.annotations().len(), 1);
        assert_eq!(scheme.links().len(), 1);
        assert_eq!(scheme.names().first(), "Frequency");
        assert_eq!(scheme.descriptions().map(LocalisedString::first), Some("How often"));
        assert_eq!(
            scheme.version().map(alloc::string::ToString::to_string).as_deref(),
            Some("1.2.3")
        );
        assert_eq!(scheme.valid_from(), Some(&valid_from));
        assert_eq!(scheme.valid_to(), None);
        assert_eq!(scheme.agency(), "ESTAT");
        assert!(scheme.is_partial_language());
        assert!(scheme.is_external_reference());
        assert_eq!(scheme.service_url(), Some("https://service"));
        assert_eq!(scheme.structure_url(), Some("https://structure"));
        assert!(scheme.is_partial());

        // The Concept carrier forwards its identifiable and nameable accessors to its metadata.
        let concept = Concept::new(full_nameable("FREQ"), None, None).unwrap();
        assert_eq!(concept.id(), "FREQ");
        assert_eq!(concept.urn(), Some("urn:x"));
        assert_eq!(concept.uri(), Some("uri"));
        assert_eq!(concept.annotations().len(), 1);
        assert_eq!(concept.links().len(), 1);
        assert_eq!(concept.names().first(), "Frequency");
        assert_eq!(concept.descriptions().map(LocalisedString::first), Some("How often"));
    }

    // Property tests: the internal serde round-trip over generated concepts and schemes
    // (see `test_strategy`); wasm32 is excluded with the rest of the property suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::{concept, concept_scheme};

        proptest! {
            #[test]
            fn concept_round_trips(value in concept()) {
                crate::test_support::round_trip(&value);
            }

            #[test]
            fn concept_scheme_round_trips(value in concept_scheme()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
