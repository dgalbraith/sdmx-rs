//! Dataflows.
//!
//! A [`Dataflow`] is the maintainable artefact a dataset is reported against: it names the
//! [`DataStructureDefinition`](crate::DataStructureDefinition) the data conforms to and, in SDMX
//! 3.1, may pin the subset of that structure's dimensions it uses via a [`DimensionConstraint`].
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A pub-field carrier with derived `Deserialize` (it composes a reference and a validated newtype, no
cross-field invariant). `dsd` is `Option` because `Structure` is `minOccurs="0"` in both editions
(D-0053): `None` is a schema-valid stub, typically an external reference whose full definition lives
elsewhere; the "must reference a DSD unless defined externally" rule is prose, so it is a catalogued
lint, not a construction rejection. `dimension_constraint` is a 3.1-only addition (D-0045); a 3.0
payload never produces `Some`. The artefact trait hierarchy delegates to the `metadata` leaf.

Decisions: D-0020, D-0030, D-0045, D-0053.
"#
)]

use alloc::{string::String, vec::Vec};

use chrono::{DateTime, FixedOffset};

use crate::{
    annotation::{Annotation, Link},
    artefact::{IdentifiableArtefact, MaintainableArtefact, NameableArtefact, VersionableArtefact},
    error::{Error, to_de_error},
    lexical::SdmxVersion,
    localised::LocalisedString,
    metadata::MaintainableMetadata,
    reference::DsdReference,
};

// ---------------------------------------------------------------------------
// DimensionConstraint
// ---------------------------------------------------------------------------

/// A non-empty subset of a data structure's dimensions a dataflow constrains itself to (3.1-only).
///
/// ## Specification
/// - **Type**: `DimensionConstraintType`
/// - **Element**: `<DimensionConstraint>`
/// - **Editions**: SDMX 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DimensionConstraintType.md"))]
///
/// Pins the subset of the referenced structure's dimensions this dataflow uses. The ids are
/// structural references, not re-validated. 3.1-only: a 3.0 payload never produces
/// one.
///
/// ## Guarantees
///
/// Always holds at least one dimension id.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct DimensionConstraint(Vec<String>);

impl DimensionConstraint {
    /// Builds a dimension constraint.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyDimensionConstraint`] if `dimension_ids` is empty.
    pub fn new(dimension_ids: Vec<String>) -> Result<Self, Error> {
        if dimension_ids.is_empty() {
            return Err(Error::EmptyDimensionConstraint);
        }
        Ok(Self(dimension_ids))
    }

    /// The constrained dimension ids, in order (always at least one).
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

impl From<DimensionConstraint> for Vec<String> {
    fn from(value: DimensionConstraint) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<String>> for DimensionConstraint {
    type Error = Error;

    fn try_from(dimension_ids: Vec<String>) -> Result<Self, Error> {
        Self::new(dimension_ids)
    }
}

impl<'de> serde::Deserialize<'de> for DimensionConstraint {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(Vec::<String>::deserialize(deserializer)?).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// Dataflow
// ---------------------------------------------------------------------------

/// A maintainable artefact a dataset is reported against.
///
/// ## Specification
/// - **Type**: `DataflowType`
/// - **Element**: `<Dataflow>`
/// - **Editions**: SDMX 3.0 and 3.1 (Divergent)
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataflowType.3.0.md"))]
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataflowType.3.1.md"))]
#[cfg_attr(design_docs, doc = "")]
///
/// A pub-field carrier. `dsd` is `None` for a schema-valid stub (an external reference);
/// `dimension_constraint` is a 3.1-only refinement.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     Dataflow, DsdReference, IdentifiableArtefact, IdentifiableMetadata, LocalisedString,
///     LocalisedText, MaintainableArtefact, MaintainableMetadata, NameableMetadata,
///     VersionableMetadata,
/// };
///
/// let names = LocalisedString::new(vec![LocalisedText {
///     language: Some("en".to_string()),
///     text: "Exchange rates".to_string(),
/// }])?;
/// let identifiable =
///     IdentifiableMetadata::new("ECB_EXR_FLOW".to_string(), None, None, vec![], vec![])?;
/// let metadata = MaintainableMetadata::new(
///     VersionableMetadata::new(
///         NameableMetadata::new(identifiable, names, None),
///         None,
///         None,
///         None,
///     ),
///     "ECB".to_string(),
///     None,
///     None,
///     None,
///     None,
/// )?;
///
/// let dataflow = Dataflow {
///     metadata,
///     dsd: Some(DsdReference {
///         agency: "ECB".to_string(),
///         id: "ECB_EXR".to_string(),
///         version: "1.0.0".parse().unwrap(),
///     }),
///     dimension_constraint: None,
/// };
/// assert_eq!(dataflow.id(), "ECB_EXR_FLOW");
/// assert_eq!(dataflow.agency(), "ECB");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Dataflow {
    /// The maintainable identity of the dataflow.
    pub metadata: MaintainableMetadata,
    /// The structure the data conforms to, or `None` for a stub (an external reference).
    pub dsd: Option<DsdReference>,
    /// The 3.1-only dimension subset this dataflow pins itself to.
    pub dimension_constraint: Option<DimensionConstraint>,
}

impl IdentifiableArtefact for Dataflow {
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

impl NameableArtefact for Dataflow {
    fn names(&self) -> &LocalisedString {
        self.metadata.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.metadata.descriptions()
    }
}

impl VersionableArtefact for Dataflow {
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

impl MaintainableArtefact for Dataflow {
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
    use alloc::{string::ToString, vec, vec::Vec};

    use super::*;
    use crate::{
        localised::LocalisedText,
        metadata::{IdentifiableMetadata, NameableMetadata, VersionableMetadata},
    };

    fn maintainable(id: &str, agency: &str) -> MaintainableMetadata {
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "Exchange rates".into(),
        }])
        .unwrap();
        let identifiable =
            IdentifiableMetadata::new(id.into(), None, None, vec![], vec![]).unwrap();
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

    fn dsd_reference() -> DsdReference {
        DsdReference {
            agency: "ECB".into(),
            id: "ECB_EXR".into(),
            version: "1.0.0".parse().unwrap(),
        }
    }

    #[test]
    fn dimension_constraint_rejects_empty() {
        assert_eq!(DimensionConstraint::new(vec!["FREQ".into()]).unwrap().as_slice().len(), 1);
        assert_eq!(
            DimensionConstraint::new(Vec::new()).unwrap_err(),
            Error::EmptyDimensionConstraint
        );
    }

    #[test]
    fn dimension_constraint_deserialize_enforces_non_empty() {
        // DimensionConstraint's Deserialize declares an inner `Raw = Vec<String>` (it does
        // `Vec::<String>::deserialize` then `Self::new(..)`), and its transparent Serialize encodes
        // as that bare vector. postcard is positional, so an empty `Vec<String>` decodes into new(),
        // which rejects it, while a non-empty one round-trips.
        let empty: Vec<String> = Vec::new();
        assert!(
            postcard::from_bytes::<DimensionConstraint>(&postcard::to_allocvec(&empty).unwrap())
                .is_err()
        );
        let constraint = DimensionConstraint::new(vec!["FREQ".into()]).unwrap();
        assert_eq!(constraint.as_slice().len(), 1);
        crate::test_support::round_trip(&constraint);
    }

    /// A maintainable leaf with every optional field populated, for the delegation matrix.
    fn full_maintainable(id: &str, agency: &str) -> MaintainableMetadata {
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
            text: "Exchange rates".into(),
        }])
        .unwrap();
        let descriptions = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "How often".into(),
        }])
        .unwrap();
        let version = SdmxVersion::new("1.2.3".into()).unwrap();
        let valid_from = DateTime::parse_from_rfc3339("2024-01-01T00:00:00+00:00").unwrap();
        let identifiable = IdentifiableMetadata::new(
            id.into(),
            Some("uri".into()),
            Some("urn:x".into()),
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
            Some("https://service".into()),
            Some("https://structure".into()),
        )
        .unwrap()
    }

    #[test]
    fn artefact_hierarchy_forwards_every_accessor() {
        let dataflow = Dataflow {
            metadata: full_maintainable("ECB_EXR_FLOW", "ECB"),
            dsd: Some(dsd_reference()),
            dimension_constraint: None,
        };
        assert_eq!(dataflow.id(), "ECB_EXR_FLOW");
        assert_eq!(dataflow.urn(), Some("urn:x"));
        assert_eq!(dataflow.uri(), Some("uri"));
        assert_eq!(dataflow.annotations().len(), 1);
        assert_eq!(dataflow.links().len(), 1);
        assert_eq!(dataflow.names().first(), "Exchange rates");
        assert_eq!(dataflow.descriptions().map(LocalisedString::first), Some("How often"));
        assert_eq!(
            dataflow.version().map(alloc::string::ToString::to_string).as_deref(),
            Some("1.2.3")
        );
        assert!(dataflow.valid_from().is_some());
        assert_eq!(dataflow.valid_to(), None);
        assert_eq!(dataflow.agency(), "ECB");
        assert!(dataflow.is_partial_language());
        assert!(dataflow.is_external_reference());
        assert_eq!(dataflow.service_url(), Some("https://service"));
        assert_eq!(dataflow.structure_url(), Some("https://structure"));
    }

    #[test]
    fn dataflow_with_dsd_reference_exposes_its_artefact_identity() {
        let dataflow = Dataflow {
            metadata: maintainable("ECB_EXR_FLOW", "ECB"),
            dsd: Some(dsd_reference()),
            dimension_constraint: Some(DimensionConstraint::new(vec!["FREQ".into()]).unwrap()),
        };
        assert_eq!(dataflow.id(), "ECB_EXR_FLOW");
        assert_eq!(dataflow.names().first(), "Exchange rates");
        assert_eq!(dataflow.agency(), "ECB");
        assert_eq!(dataflow.dsd.as_ref().unwrap().id, "ECB_EXR");
    }

    #[test]
    fn dataflow_round_trips_with_and_without_a_dsd() {
        let with_dsd = Dataflow {
            metadata: maintainable("ECB_EXR_FLOW", "ECB"),
            dsd: Some(dsd_reference()),
            dimension_constraint: Some(DimensionConstraint::new(vec!["FREQ".into()]).unwrap()),
        };
        crate::test_support::round_trip(&with_dsd);

        // A stub dataflow (no DSD) is schema-valid and round-trips (D-0053).
        let stub = Dataflow {
            metadata: maintainable("STUB_FLOW", "ECB"),
            dsd: None,
            dimension_constraint: None,
        };
        let bytes = postcard::to_allocvec(&stub).unwrap();
        let restored: Dataflow = postcard::from_bytes(&bytes).unwrap();
        assert!(restored.dsd.is_none());
        assert_eq!(restored, stub);
    }

    #[test]
    fn dataflow_deserialize_enforces_the_dimension_constraint_invariant() {
        // Bubbling demonstration, not a composite-own proof: the empty-constraint invariant is
        // DimensionConstraint's (its source-level proof is above). Dataflow's derived Deserialize
        // reads its fields in declaration order
        // `(metadata: MaintainableMetadata, dsd: Option<DsdReference>,
        //   dimension_constraint: Option<DimensionConstraint>)` and delegates the last field to
        // DimensionConstraint's custom impl. DimensionConstraint is #[serde(transparent)] over
        // Vec<String>, so its Option is byte-identical to Option<Vec<String>>; postcard is
        // positional, so a tuple of those types carrying `Some(empty vec)` decodes into
        // DimensionConstraint::new(), which rejects the empty constraint, and Dataflow's derive
        // propagates it.
        // A valid tuple of the same field types decodes — guards this proof's shape against field-order drift.
        let ok = (
            maintainable("ECB_EXR_FLOW", "ECB"),
            Some(dsd_reference()),
            Some(vec![String::from("FREQ")]),
        );
        assert!(postcard::from_bytes::<Dataflow>(&postcard::to_allocvec(&ok).unwrap()).is_ok());
        let raw = (
            maintainable("ECB_EXR_FLOW", "ECB"),
            Some(dsd_reference()),
            Some(Vec::<String>::new()),
        );
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<Dataflow>(&bytes).is_err());
    }

    #[test]
    fn dimension_constraint_try_from_rejects_empty() {
        assert_eq!(
            DimensionConstraint::try_from(vec![]).unwrap_err(),
            Error::EmptyDimensionConstraint
        );
    }

    #[test]
    fn dimension_constraint_into_inner_and_from() {
        let v = vec!["FREQ".to_string()];
        assert_eq!(DimensionConstraint::new(v.clone()).unwrap().into_inner(), v);
        assert_eq!(Vec::from(DimensionConstraint::new(v.clone()).unwrap()), v);
    }

    // Property tests: the internal serde round-trip over generated values (see
    // `test_strategy`); wasm32 is excluded with the rest of the property suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::dataflow;

        proptest! {
            #[test]
            fn dataflow_round_trips(value in dataflow()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
