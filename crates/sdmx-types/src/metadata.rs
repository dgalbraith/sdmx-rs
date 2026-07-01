//! The four metadata composition leaves.
//!
//! `IdentifiableMetadata → NameableMetadata → VersionableMetadata → MaintainableMetadata` nest
//! progressively, each composing its predecessor. They are the storage projection of the SDMX
//! abstract artefact hierarchy: the [`IdentifiableArtefact`] trait family exposes the interface,
//! these leaves hold the data. All four use private fields and `Result`-returning constructors as
//! the single write path, with a custom `Deserialize` that routes through that constructor so
//! identifier validation cannot be bypassed.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Composition for storage (§5.2 / §5.4): the abstract artefact hierarchy is realised as nesting
structs rather than inheritance, the same content factored once and reused by every concrete domain
type. `Serialize` is derived (it reads fields directly), so the stored statedness round-trips
faithfully; only `Deserialize` is hand-written, to route construction through the validated `new()`.

Two-layer model: the store is a precise image of the wire (every XSD-defaulted attribute is an
`Option`, preserving absent-versus-stated), and the trait accessors are the effective views that
apply the schema defaults.

Decisions: D-0031, D-0052.
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
    validate::{validate_id, validate_nested_ncname},
};

// ---------------------------------------------------------------------------
// IdentifiableMetadata
// ---------------------------------------------------------------------------

/// The identifiable metadata leaf: a validated id plus optional URI, URN, annotations, and links.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// The storage leaf bundling the `IdentifiableType` attributes (id, URI, URN, annotations, links)
/// that the [`IdentifiableArtefact`] trait exposes.
///
/// # Examples
///
/// ```
/// use sdmx_types::{IdentifiableArtefact, IdentifiableMetadata};
///
/// let meta = IdentifiableMetadata::new("CL_FREQ".to_string(), None, None, vec![], vec![])?;
/// assert_eq!(meta.id(), "CL_FREQ");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct IdentifiableMetadata {
    id: String,
    uri: Option<String>,
    urn: Option<String>,
    annotations: Vec<Annotation>,
    // `Link` is on `IdentifiableType` (0..*), so it lives on this leaf, the same single
    // chokepoint as `annotations` (D-0035). Empty ⟺ absent.
    links: Vec<Link>,
}

impl IdentifiableMetadata {
    /// Builds identifiable metadata, validating `id` against SDMX `IDType`, the loosest tier
    /// every identifiable artefact shares. NCName-tier types tighten the check in their own
    /// constructors.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidIdentifier`] if `id` is not a valid `IDType`.
    pub fn new(
        id: String,
        uri: Option<String>,
        urn: Option<String>,
        annotations: Vec<Annotation>,
        links: Vec<Link>,
    ) -> Result<Self, Error> {
        validate_id(&id)?;
        Ok(Self { id, uri, urn, annotations, links })
    }
}

impl IdentifiableArtefact for IdentifiableMetadata {
    fn id(&self) -> &str {
        &self.id
    }
    fn urn(&self) -> Option<&str> {
        self.urn.as_deref()
    }
    fn uri(&self) -> Option<&str> {
        self.uri.as_deref()
    }
    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
    fn links(&self) -> &[Link] {
        &self.links
    }
}

impl<'de> serde::Deserialize<'de> for IdentifiableMetadata {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            id: String,
            uri: Option<String>,
            urn: Option<String>,
            annotations: Vec<Annotation>,
            links: Vec<Link>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.id, raw.uri, raw.urn, raw.annotations, raw.links).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// NameableMetadata
// ---------------------------------------------------------------------------

/// The nameable metadata leaf: identifiable metadata plus localised names and optional
/// descriptions.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// The storage leaf bundling the `NameableType` content (names and descriptions) that the
/// [`NameableArtefact`] trait exposes.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct NameableMetadata {
    identifiable: IdentifiableMetadata,
    names: LocalisedString,
    descriptions: Option<LocalisedString>,
}

impl NameableMetadata {
    /// Composes nameable metadata. Infallible: all invariant enforcement is delegated to the
    /// nested constructors ([`IdentifiableMetadata::new`] and [`LocalisedString::new`]).
    #[must_use]
    pub const fn new(
        identifiable: IdentifiableMetadata,
        names: LocalisedString,
        descriptions: Option<LocalisedString>,
    ) -> Self {
        Self { identifiable, names, descriptions }
    }
}

impl IdentifiableArtefact for NameableMetadata {
    fn id(&self) -> &str {
        self.identifiable.id()
    }
    fn urn(&self) -> Option<&str> {
        self.identifiable.urn()
    }
    fn uri(&self) -> Option<&str> {
        self.identifiable.uri()
    }
    fn annotations(&self) -> &[Annotation] {
        self.identifiable.annotations()
    }
    fn links(&self) -> &[Link] {
        self.identifiable.links()
    }
}

impl NameableArtefact for NameableMetadata {
    fn names(&self) -> &LocalisedString {
        &self.names
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.descriptions.as_ref()
    }
}

impl<'de> serde::Deserialize<'de> for NameableMetadata {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            identifiable: IdentifiableMetadata,
            names: LocalisedString,
            descriptions: Option<LocalisedString>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(Self::new(raw.identifiable, raw.names, raw.descriptions))
    }
}

// ---------------------------------------------------------------------------
// VersionableMetadata
// ---------------------------------------------------------------------------

/// The versionable metadata leaf: nameable metadata plus an optional version and validity window.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// The storage leaf bundling the `VersionableType` content (version and validity window) that the
/// [`VersionableArtefact`] trait exposes.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct VersionableMetadata {
    nameable: NameableMetadata,
    // `version` is `Option`: the spec marks it optional with "if not supplied, the artefact is
    // un-versioned" and assigns no default, so `None` is a distinct state, not a synonym for a
    // version value (D-0024).
    version: Option<SdmxVersion>,
    valid_from: Option<DateTime<FixedOffset>>,
    valid_to: Option<DateTime<FixedOffset>>,
}

impl VersionableMetadata {
    /// Composes versionable metadata. Infallible: it delegates all invariant enforcement to its
    /// nested constructors.
    #[must_use]
    pub const fn new(
        nameable: NameableMetadata,
        version: Option<SdmxVersion>,
        valid_from: Option<DateTime<FixedOffset>>,
        valid_to: Option<DateTime<FixedOffset>>,
    ) -> Self {
        Self { nameable, version, valid_from, valid_to }
    }
}

impl IdentifiableArtefact for VersionableMetadata {
    fn id(&self) -> &str {
        self.nameable.id()
    }
    fn urn(&self) -> Option<&str> {
        self.nameable.urn()
    }
    fn uri(&self) -> Option<&str> {
        self.nameable.uri()
    }
    fn annotations(&self) -> &[Annotation] {
        self.nameable.annotations()
    }
    fn links(&self) -> &[Link] {
        self.nameable.links()
    }
}

impl NameableArtefact for VersionableMetadata {
    fn names(&self) -> &LocalisedString {
        self.nameable.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.nameable.descriptions()
    }
}

impl VersionableArtefact for VersionableMetadata {
    fn version(&self) -> Option<&SdmxVersion> {
        self.version.as_ref()
    }
    fn valid_from(&self) -> Option<&DateTime<FixedOffset>> {
        self.valid_from.as_ref()
    }
    fn valid_to(&self) -> Option<&DateTime<FixedOffset>> {
        self.valid_to.as_ref()
    }
}

impl<'de> serde::Deserialize<'de> for VersionableMetadata {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            nameable: NameableMetadata,
            version: Option<SdmxVersion>,
            valid_from: Option<DateTime<FixedOffset>>,
            valid_to: Option<DateTime<FixedOffset>>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(Self::new(raw.nameable, raw.version, raw.valid_from, raw.valid_to))
    }
}

// ---------------------------------------------------------------------------
// MaintainableMetadata
// ---------------------------------------------------------------------------

/// The maintainable metadata leaf: versionable metadata plus the maintenance agency and the
/// external-reference triple.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// The storage leaf bundling the `MaintainableType` content (the maintenance agency and the
/// external-reference triple) that the [`MaintainableArtefact`] trait exposes.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     IdentifiableMetadata, LocalisedString, LocalisedText, MaintainableArtefact,
///     MaintainableMetadata, NameableMetadata, VersionableMetadata,
/// };
///
/// let names = LocalisedString::new(vec![LocalisedText {
///     language: Some("en".to_string()),
///     text: "Frequency".to_string(),
/// }])?;
/// let identifiable = IdentifiableMetadata::new("FREQ".to_string(), None, None, vec![], vec![])?;
/// let nameable = NameableMetadata::new(identifiable, names, None);
/// let versionable = VersionableMetadata::new(nameable, None, None, None);
/// let maintainable =
///     MaintainableMetadata::new(versionable, "ESTAT".to_string(), None, None, None, None)?;
/// assert_eq!(maintainable.agency(), "ESTAT");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct MaintainableMetadata {
    versionable: VersionableMetadata,
    agency: String,
    // Statedness stored (D-0052): `None` ⟺ the attribute was absent; the schema default is
    // applied by the trait view, never baked into the store.
    is_partial_language: Option<bool>,
    // The `isExternalReference` + serviceURL/structureURL triple (D-0030). Stored verbatim:
    // every schema-valid combination round-trips, including the dubious `false` with a URL
    // present. Collapsing it into a `Local | External` enum or rejecting that combination would
    // destroy a schema-valid wire shape (D-0031); coherence is a Layer-2 lint, not an invariant.
    is_external_reference: Option<bool>,
    service_url: Option<String>,
    structure_url: Option<String>,
}

impl MaintainableMetadata {
    /// Builds maintainable metadata, validating `agency` against SDMX `NestedNCNameIDType`. This
    /// is the only reason the constructor is fallible: the external-reference triple adds no
    /// rejection (its schema-valid combinations are all stored).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidAgencyIdentifier`] if `agency` is not a valid `NestedNCNameIDType`.
    pub fn new(
        versionable: VersionableMetadata,
        agency: String,
        is_partial_language: Option<bool>,
        is_external_reference: Option<bool>,
        service_url: Option<String>,
        structure_url: Option<String>,
    ) -> Result<Self, Error> {
        validate_nested_ncname(&agency)?;
        Ok(Self {
            versionable,
            agency,
            is_partial_language,
            is_external_reference,
            service_url,
            structure_url,
        })
    }
}

impl IdentifiableArtefact for MaintainableMetadata {
    fn id(&self) -> &str {
        self.versionable.id()
    }
    fn urn(&self) -> Option<&str> {
        self.versionable.urn()
    }
    fn uri(&self) -> Option<&str> {
        self.versionable.uri()
    }
    fn annotations(&self) -> &[Annotation] {
        self.versionable.annotations()
    }
    fn links(&self) -> &[Link] {
        self.versionable.links()
    }
}

impl NameableArtefact for MaintainableMetadata {
    fn names(&self) -> &LocalisedString {
        self.versionable.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.versionable.descriptions()
    }
}

impl VersionableArtefact for MaintainableMetadata {
    fn version(&self) -> Option<&SdmxVersion> {
        self.versionable.version()
    }
    fn valid_from(&self) -> Option<&DateTime<FixedOffset>> {
        self.versionable.valid_from()
    }
    fn valid_to(&self) -> Option<&DateTime<FixedOffset>> {
        self.versionable.valid_to()
    }
}

impl MaintainableArtefact for MaintainableMetadata {
    fn agency(&self) -> &str {
        &self.agency
    }
    // The trait accessors are effective views (Layer 2): the schema default is applied here,
    // over the stored statedness (D-0031 / D-0052).
    fn is_partial_language(&self) -> bool {
        self.is_partial_language.unwrap_or(false)
    }
    fn is_external_reference(&self) -> bool {
        self.is_external_reference.unwrap_or(false)
    }
    fn service_url(&self) -> Option<&str> {
        self.service_url.as_deref()
    }
    fn structure_url(&self) -> Option<&str> {
        self.structure_url.as_deref()
    }
}

impl<'de> serde::Deserialize<'de> for MaintainableMetadata {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            versionable: VersionableMetadata,
            agency: String,
            is_partial_language: Option<bool>,
            is_external_reference: Option<bool>,
            service_url: Option<String>,
            structure_url: Option<String>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(
            raw.versionable,
            raw.agency,
            raw.is_partial_language,
            raw.is_external_reference,
            raw.service_url,
            raw.structure_url,
        )
        .map_err(to_de_error)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::localised::LocalisedText;

    fn names() -> LocalisedString {
        LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "Frequency".into(),
        }])
        .unwrap()
    }

    fn identifiable(id: &str) -> Result<IdentifiableMetadata, Error> {
        IdentifiableMetadata::new(id.into(), None, None, vec![], vec![])
    }

    #[test]
    fn identifiable_validates_id_tier() {
        assert!(identifiable("FREQ").is_ok());
        assert!(identifiable("EUR$").is_ok()); // IDType permits $
        assert_eq!(identifiable("a.b").unwrap_err(), Error::InvalidIdentifier("a.b".into()));
    }

    #[test]
    fn delegation_climbs_the_hierarchy() {
        let nameable = NameableMetadata::new(identifiable("FREQ").unwrap(), names(), None);
        let versionable = VersionableMetadata::new(
            nameable,
            Some(SdmxVersion::new("1.0.0".into()).unwrap()),
            None,
            None,
        );
        let maintainable =
            MaintainableMetadata::new(versionable, "ESTAT".into(), None, None, None, None).unwrap();

        // Accessors delegate down to the identifiable leaf and across the hierarchy.
        assert_eq!(maintainable.id(), "FREQ");
        assert_eq!(maintainable.names().first(), "Frequency");
        assert_eq!(maintainable.version().map(SdmxVersion::as_str), Some("1.0.0"));
        assert_eq!(maintainable.agency(), "ESTAT");
    }

    #[test]
    fn maintainable_validates_agency_as_nested_ncname() {
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable("FREQ").unwrap(), names(), None),
            None,
            None,
            None,
        );
        // A dotted agency id is valid NestedNCName.
        assert!(
            MaintainableMetadata::new(
                versionable.clone(),
                "ORG.SUB".into(),
                None,
                None,
                None,
                None
            )
            .is_ok()
        );
        // A leading-digit agency id is not.
        assert_eq!(
            MaintainableMetadata::new(versionable, "1ORG".into(), None, None, None, None)
                .unwrap_err(),
            Error::InvalidAgencyIdentifier("1ORG".into())
        );
    }

    #[test]
    fn external_reference_defaults_are_effective_views() {
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable("FREQ").unwrap(), names(), None),
            None,
            None,
            None,
        );
        // Absent statedness -> effective false; stored None is preserved for the writer path.
        let m =
            MaintainableMetadata::new(versionable, "ESTAT".into(), None, None, None, None).unwrap();
        assert!(!m.is_partial_language());
        assert!(!m.is_external_reference());
    }

    #[test]
    fn delegation_matrix_forwards_every_accessor_on_every_leaf() {
        use crate::annotation::{Annotation, AnnotationUrl, Link};

        let annotation = Annotation {
            id: Some("a1".into()),
            annotation_type: None,
            annotation_title: None,
            annotation_urls: vec![AnnotationUrl {
                url: "https://x".into(),
                lang: Some("en".into()),
            }],
            annotation_value: None,
            texts: None,
        };
        let link = Link {
            rel: "self".into(),
            url: "https://example/cl".into(),
            urn: None,
            link_type: None,
        };
        let descriptions = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "How often".into(),
        }])
        .unwrap();
        let version = SdmxVersion::new("1.2.3".into()).unwrap();
        let valid_from = DateTime::parse_from_rfc3339("2024-01-01T00:00:00+00:00").unwrap();

        // Build the full chain with every optional field populated.
        let identifiable = IdentifiableMetadata::new(
            "FREQ".into(),
            Some("urn:x".into()),
            Some("urn:sdmx:freq".into()),
            vec![annotation],
            vec![link],
        )
        .unwrap();
        let nameable = NameableMetadata::new(identifiable, names(), Some(descriptions));
        let versionable =
            VersionableMetadata::new(nameable.clone(), Some(version), Some(valid_from), None);
        let maintainable = MaintainableMetadata::new(
            versionable.clone(),
            "ESTAT".into(),
            Some(true),
            Some(true),
            Some("https://service".into()),
            Some("https://structure".into()),
        )
        .unwrap();

        // Identifiable-tier accessors are identical through every leaf that delegates them.
        for leaf in [
            &nameable as &dyn IdentifiableArtefact,
            &versionable as &dyn IdentifiableArtefact,
            &maintainable as &dyn IdentifiableArtefact,
        ] {
            assert_eq!(leaf.id(), "FREQ");
            assert_eq!(leaf.urn(), Some("urn:sdmx:freq"));
            assert_eq!(leaf.uri(), Some("urn:x"));
            assert_eq!(leaf.annotations().len(), 1);
            assert_eq!(leaf.links().len(), 1);
        }
        // Nameable-tier through the versionable and maintainable leaves.
        for leaf in [&versionable as &dyn NameableArtefact, &maintainable as &dyn NameableArtefact]
        {
            assert_eq!(leaf.names().first(), "Frequency");
            assert_eq!(leaf.descriptions().map(LocalisedString::first), Some("How often"));
        }
        // Versionable tier (incl. the version_display default method) on both carriers.
        for leaf in
            [&versionable as &dyn VersionableArtefact, &maintainable as &dyn VersionableArtefact]
        {
            assert_eq!(leaf.version().map(SdmxVersion::as_str), Some("1.2.3"));
            assert_eq!(leaf.valid_from(), Some(&valid_from));
            assert_eq!(leaf.valid_to(), None);
            assert_eq!(alloc::format!("{}", leaf.version_display()), "1.2.3");
        }
        // Maintainable tier: agency plus the external-reference triple as effective views.
        assert_eq!(maintainable.agency(), "ESTAT");
        assert!(maintainable.is_partial_language());
        assert!(maintainable.is_external_reference());
        assert_eq!(maintainable.service_url(), Some("https://service"));
        assert_eq!(maintainable.structure_url(), Some("https://structure"));
    }

    #[test]
    fn version_display_renders_unversioned_when_absent() {
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable("FREQ").unwrap(), names(), None),
            None,
            None,
            None,
        );
        assert_eq!(alloc::format!("{}", versionable.version_display()), "<unversioned>");
    }

    #[test]
    fn deserialize_round_trips_through_the_chain() {
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable("FREQ").unwrap(), names(), None),
            Some(SdmxVersion::new("1.0.0".into()).unwrap()),
            None,
            None,
        );
        let maintainable =
            MaintainableMetadata::new(versionable, "ESTAT".into(), Some(false), None, None, None)
                .unwrap();
        crate::test_support::round_trip(&maintainable);
    }

    #[test]
    fn deserialize_routes_through_new_at_the_id_tier() {
        // The id constraint is IdentifiableMetadata::new's, so prove the Raw -> new() routing
        // here, at its source. postcard is positional: a Raw (id, uri, urn, annotations, links)
        // carrying a non-IDType id decodes into new(), which rejects it. Composite types inherit
        // this on the wire because serde bubbles the nested failure up; their own re-validation
        // (e.g. MaintainableMetadata's agency) has its own wire proof below.
        // A valid tuple of the same field types decodes — guards this proof's shape against Raw drift.
        let ok = (
            String::from("OBS_VALUE"),
            None::<String>,
            None::<String>,
            Vec::<crate::Annotation>::new(),
            Vec::<crate::Link>::new(),
        );
        assert!(
            postcard::from_bytes::<IdentifiableMetadata>(&postcard::to_allocvec(&ok).unwrap())
                .is_ok()
        );
        let raw = (
            String::from("a.b"),
            None::<String>,
            None::<String>,
            Vec::<crate::Annotation>::new(),
            Vec::<crate::Link>::new(),
        );
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<IdentifiableMetadata>(&bytes).is_err());
    }

    #[test]
    fn maintainable_metadata_deserialize_rejects_bad_agency() {
        // The agency tier is MaintainableMetadata::new's own invariant (NestedNCName), enforced by
        // no nested type, so it needs its own wire proof: the custom Deserialize reads Raw
        // (versionable, agency, is_partial_language, is_external_reference, service_url,
        // structure_url) and routes through new(). postcard is positional, so a tuple of those field
        // types carrying a leading-digit agency decodes at the field level but is rejected by new().
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable("FREQ").unwrap(), names(), None),
            None,
            None,
            None,
        );
        // A valid tuple of the same field types decodes — guards this proof's shape against Raw drift.
        let ok = (
            versionable.clone(),
            String::from("ESTAT"),
            None::<bool>,
            None::<bool>,
            None::<String>,
            None::<String>,
        );
        assert!(
            postcard::from_bytes::<MaintainableMetadata>(&postcard::to_allocvec(&ok).unwrap())
                .is_ok()
        );
        let raw = (
            versionable,
            String::from("1ORG"),
            None::<bool>,
            None::<bool>,
            None::<String>,
            None::<String>,
        );
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<MaintainableMetadata>(&bytes).is_err());
    }
}
