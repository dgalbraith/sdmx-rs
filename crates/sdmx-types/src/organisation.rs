//! Agencies and the agency scheme.
//!
//! An [`Agency`] is a maintenance organisation: a validated scheme item (its id is `NCNameIDType`)
//! that additionally carries [`Contact`]s. The [`AgencyScheme`] is the maintainable scheme of
//! agencies, whose id is the fixed literal `"AGENCIES"`.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`Agency` follows the validated-item pattern (id is `NCNameIDType`): private fields, a fallible
`new()` calling `validate_ncname`, and a custom `Deserialize`. It additionally carries
`contacts: Vec<Contact>` (`0..unbounded`, empty ⟺ absent, D-0055), an invariant-free field threaded
through `new()` with no extra check. The contact detail elements (Telephone/Fax/X400/URI/Email) are
ONE repeated wire choice, so the store is ONE interleaved `Vec` in wire order (the D-0051
precedent). The localisable Name/Department/Role triple reuses `LocalisedString`.

`AgencyScheme` is the asymmetric wrapper (D-0023, verified): its id is `IDType` with
`use="required" fixed="AGENCIES"`, NOT `NCNameIDType`. So `new()` is fallible via `validate_fixed`
(a stated value differing from the fixed one is a mechanical mismatch), but it does NOT re-validate
NCName. This asymmetry is the spec's, not an oversight, so it is not "consistency-fixed".

Decisions: D-0023, D-0052, D-0055.
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
    metadata::{MaintainableMetadata, NameableMetadata},
    scheme::{ItemScheme, SchemeItem},
    validate::{validate_fixed, validate_ncname},
};

// ---------------------------------------------------------------------------
// Contact
// ---------------------------------------------------------------------------

/// A single contact-detail entry: a typed contact endpoint.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Projects the repeated contact-detail choice (`<Telephone>`, `<Fax>`, `<X400>`, `<URI>`,
/// `<Email>`) into a single Rust enum so a [`Contact`] can store the entries in one interleaved
/// list, preserving wire order. Exhaustive: exactly these five kinds.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ContactDetail {
    /// A telephone number.
    Telephone(String),
    /// A fax number.
    Fax(String),
    /// An X.400 address.
    X400(String),
    /// A URI (`xs:anyURI`), stored verbatim and not validated.
    Uri(String),
    /// An email address.
    Email(String),
}

/// A point of contact for an organisation.
///
/// ## Specification
/// - **Type**: `ContactType`
/// - **Element**: `<Contact>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ContactType.md"))]
///
/// Invariant-free pub-field carrier. The localisable Name/Department/Role triple are each optional
/// (`minOccurs="0"`); the detail endpoints are one interleaved list in wire order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Contact {
    /// The contact's localised names; `None` ⟺ no names.
    pub names: Option<LocalisedString>,
    /// The contact's localised departments; `None` ⟺ absent.
    pub departments: Option<LocalisedString>,
    /// The contact's localised roles; `None` ⟺ absent.
    pub roles: Option<LocalisedString>,
    /// The contact endpoints, in wire order; empty ⟺ none.
    pub details: Vec<ContactDetail>,
}

// ---------------------------------------------------------------------------
// Agency
// ---------------------------------------------------------------------------

/// A maintenance organisation.
///
/// ## Specification
/// - **Type**: `AgencyType`
/// - **Element**: `<Agency>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/AgencyType.md"))]
///
/// A validated item: its id is `NCNameIDType`, so [`new`](Self::new) re-validates it and is
/// fallible, and the fields are private. It additionally carries [`Contact`]s.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     Agency, IdentifiableArtefact, IdentifiableMetadata, LocalisedString, LocalisedText,
///     NameableMetadata,
/// };
///
/// let names = LocalisedString::new(vec![LocalisedText {
///     language: Some("en".to_string()),
///     text: "Eurostat".to_string(),
/// }])?;
/// let identifiable =
///     IdentifiableMetadata::new("ESTAT".to_string(), None, None, Vec::new(), Vec::new())?;
/// let agency = Agency::new(NameableMetadata::new(identifiable, names, None), Vec::new())?;
/// assert_eq!(agency.id(), "ESTAT");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Agency {
    metadata: NameableMetadata,
    contacts: Vec<Contact>,
}

impl Agency {
    /// Builds an agency, re-validating its id against SDMX `NCNameIDType`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNcNameIdentifier`] if the id is not a valid `NCNameIDType`.
    pub fn new(metadata: NameableMetadata, contacts: Vec<Contact>) -> Result<Self, Error> {
        validate_ncname(metadata.id())?;
        Ok(Self { metadata, contacts })
    }

    /// The agency's contacts (empty slice if none).
    #[must_use]
    pub fn contacts(&self) -> &[Contact] {
        &self.contacts
    }
}

impl IdentifiableArtefact for Agency {
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

impl NameableArtefact for Agency {
    fn names(&self) -> &LocalisedString {
        self.metadata.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.metadata.descriptions()
    }
}

impl SchemeItem for Agency {}

impl<'de> serde::Deserialize<'de> for Agency {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            metadata: NameableMetadata,
            contacts: Vec<Contact>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.metadata, raw.contacts).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// AgencyScheme
// ---------------------------------------------------------------------------

/// The maintainable scheme of [`Agency`]s.
///
/// ## Specification
/// - **Type**: `AgencySchemeType`
/// - **Element**: `<AgencyScheme>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/AgencySchemeType.md"))]
///
/// Wraps an [`ItemScheme<Agency>`](ItemScheme). Its scheme id is fixed to `"AGENCIES"`, so
/// [`new`](Self::new) rejects any other stated id (unlike [`Codelist`](crate::Codelist), it does
/// not apply the `NCName` check: the spec types this id as `IDType` with `fixed="AGENCIES"`).
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     AgencyScheme, Error, IdentifiableMetadata, LocalisedString, LocalisedText,
///     MaintainableMetadata, NameableMetadata, VersionableMetadata,
/// };
///
/// fn scheme(id: &str) -> Result<AgencyScheme, Error> {
///     let names = LocalisedString::new(vec![LocalisedText {
///         language: Some("en".to_string()),
///         text: "Agencies".to_string(),
///     }])?;
///     let identifiable =
///         IdentifiableMetadata::new(id.to_string(), None, None, Vec::new(), Vec::new())?;
///     let versionable = VersionableMetadata::new(
///         NameableMetadata::new(identifiable, names, None),
///         None,
///         None,
///         None,
///     );
///     let metadata =
///         MaintainableMetadata::new(versionable, "SDMX".to_string(), None, None, None, None)?;
///     AgencyScheme::new(metadata, None)
/// }
///
/// assert!(scheme("AGENCIES").is_ok());
/// // Any other id contradicts the fixed value.
/// assert!(scheme("OTHER").is_err());
/// # Ok::<(), Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct AgencyScheme {
    scheme: ItemScheme<Agency>,
}

impl AgencyScheme {
    /// Builds an empty agency scheme, checking the scheme id against the fixed literal `"AGENCIES"`.
    /// Agencies are added with [`push`](Self::push).
    ///
    /// # Errors
    ///
    /// Returns [`Error::FixedAttributeMismatch`] if the scheme id is stated as anything other than
    /// `"AGENCIES"`.
    pub fn new(metadata: MaintainableMetadata, is_partial: Option<bool>) -> Result<Self, Error> {
        validate_fixed("id", Some(metadata.id()), "AGENCIES")?;
        Ok(Self { scheme: ItemScheme::new(metadata, is_partial) })
    }

    /// Appends an agency, preserving wire order.
    pub fn push(&mut self, agency: Agency) {
        self.scheme.push(agency);
    }

    /// The first agency whose id equals `id`, in wire order (a first-match view).
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Agency> {
        self.scheme.get(id)
    }

    /// Iterates the agencies in wire order.
    pub fn iter(&self) -> impl Iterator<Item = &Agency> {
        self.scheme.iter()
    }

    /// The effective value of the scheme's `isPartial` flag (schema default `false`).
    #[must_use]
    pub fn is_partial(&self) -> bool {
        self.scheme.is_partial()
    }
}

impl IdentifiableArtefact for AgencyScheme {
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

impl NameableArtefact for AgencyScheme {
    fn names(&self) -> &LocalisedString {
        self.scheme.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.scheme.descriptions()
    }
}

impl VersionableArtefact for AgencyScheme {
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

impl MaintainableArtefact for AgencyScheme {
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

impl<'de> serde::Deserialize<'de> for AgencyScheme {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            scheme: ItemScheme<Agency>,
        }
        let raw = Raw::deserialize(deserializer)?;
        // Route through new() so the fixed-id invariant is enforced, then restore the items.
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
    };

    fn nameable(id: &str) -> NameableMetadata {
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "Eurostat".into(),
        }])
        .unwrap();
        let identifiable =
            IdentifiableMetadata::new(id.into(), None, None, Vec::new(), Vec::new()).unwrap();
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

    #[test]
    fn agency_new_validates_id_as_ncname() {
        assert!(Agency::new(nameable("ESTAT"), Vec::new()).is_ok());
        assert_eq!(
            Agency::new(nameable("1ESTAT"), Vec::new()).unwrap_err(),
            Error::InvalidNcNameIdentifier("1ESTAT".into())
        );
    }

    fn contact() -> Contact {
        Contact {
            names: Some(
                LocalisedString::new(vec![LocalisedText {
                    language: Some("en".into()),
                    text: "Helpdesk".into(),
                }])
                .unwrap(),
            ),
            departments: None,
            roles: None,
            details: vec![
                ContactDetail::Email("info@example.com".into()),
                ContactDetail::Uri("https://example.com".into()),
            ],
        }
    }

    #[test]
    fn agency_carries_contacts() {
        let agency = Agency::new(nameable("ESTAT"), vec![contact()]).unwrap();
        assert_eq!(agency.contacts().len(), 1);
        assert_eq!(agency.contacts()[0].details.len(), 2);
    }

    #[test]
    fn agency_deserialize_round_trips() {
        let agency = Agency::new(nameable("ESTAT"), vec![contact()]).unwrap();
        crate::test_support::round_trip(&agency);
    }

    #[test]
    fn agency_new_enforces_ncname_id() {
        // The validated-item NCName tightening is Agency::new's (composite over the nested nameable
        // id): a leading-digit id passes IDType but fails NCName.
        assert!(Agency::new(nameable("9ESTAT"), Vec::new()).is_err());
    }

    #[test]
    fn agency_deserialize_rejects_non_ncname_id() {
        // Agency's Deserialize declares `Raw { metadata, contacts }` and routes through new().
        // postcard is positional, so a tuple of those field types carrying a leading-digit id
        // (valid IDType, so the nested metadata deserialises, but rejected by the NCName tightening
        // in Agency::new) proves the wire path re-runs the check.
        // A valid tuple of the same field types decodes — guards this proof's shape against Raw drift.
        let ok = (nameable("ESTAT"), Vec::<Contact>::new());
        assert!(postcard::from_bytes::<Agency>(&postcard::to_allocvec(&ok).unwrap()).is_ok());
        let raw = (nameable("9ESTAT"), Vec::<Contact>::new());
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<Agency>(&bytes).is_err());
    }

    #[test]
    fn agency_scheme_rejects_non_agencies_id() {
        // The scheme id is fixed to "AGENCIES": any other stated id is a mechanical mismatch.
        assert!(AgencyScheme::new(scheme_metadata("AGENCIES"), None).is_ok());
        assert_eq!(
            AgencyScheme::new(scheme_metadata("OTHER"), None).unwrap_err(),
            Error::FixedAttributeMismatch { attribute: "id".into(), value: "OTHER".into() }
        );
    }

    #[test]
    fn agency_scheme_forwards_item_access() {
        let mut scheme = AgencyScheme::new(scheme_metadata("AGENCIES"), None).unwrap();
        scheme.push(Agency::new(nameable("ESTAT"), Vec::new()).unwrap());
        assert_eq!(scheme.get("ESTAT").map(IdentifiableArtefact::id), Some("ESTAT"));
        assert_eq!(scheme.iter().count(), 1);
        assert_eq!(scheme.agency(), "SDMX");
    }

    #[test]
    fn agency_scheme_deserialize_round_trips() {
        let mut scheme = AgencyScheme::new(scheme_metadata("AGENCIES"), None).unwrap();
        scheme.push(Agency::new(nameable("ESTAT"), Vec::new()).unwrap());
        crate::test_support::round_trip(&scheme);
    }

    #[test]
    fn agency_scheme_deserialize_rejects_non_agencies_id() {
        // AgencyScheme's Deserialize declares `Raw { scheme: ItemScheme<Agency> }` and routes the
        // metadata through new(), which checks the id against the fixed literal "AGENCIES".
        // postcard is positional and a single-field struct encodes exactly as its one field, so an
        // ItemScheme carrying any other stated id (a valid IDType, so ItemScheme deserialises) is
        // rejected by new()'s fixed-id check.
        // A valid ItemScheme decodes — guards the single-field shape if the Raw grows a second field.
        let ok = ItemScheme::<Agency>::new(scheme_metadata("AGENCIES"), None);
        assert!(postcard::from_bytes::<AgencyScheme>(&postcard::to_allocvec(&ok).unwrap()).is_ok());
        let scheme = ItemScheme::<Agency>::new(scheme_metadata("OTHER"), None);
        let bytes = postcard::to_allocvec(&scheme).unwrap();
        assert!(postcard::from_bytes::<AgencyScheme>(&bytes).is_err());
    }

    /// A nameable leaf with every optional field populated, for the delegation matrix.
    fn full_nameable(id: &str) -> NameableMetadata {
        use crate::annotation::AnnotationUrl;
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
            text: "Eurostat".into(),
        }])
        .unwrap();
        let descriptions = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "Statistical office".into(),
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
            VersionableMetadata::new(
                full_nameable("AGENCIES"),
                Some(version),
                Some(valid_from),
                None,
            ),
            "SDMX".into(),
            Some(true),
            Some(true),
            Some("https://service".into()),
            Some("https://structure".into()),
        )
        .unwrap();
        let scheme = AgencyScheme::new(metadata, Some(true)).unwrap();

        assert_eq!(scheme.id(), "AGENCIES");
        assert_eq!(scheme.urn(), Some("urn:x"));
        assert_eq!(scheme.uri(), Some("uri"));
        assert_eq!(scheme.annotations().len(), 1);
        assert_eq!(scheme.links().len(), 1);
        assert_eq!(scheme.names().first(), "Eurostat");
        assert_eq!(scheme.descriptions().map(LocalisedString::first), Some("Statistical office"));
        assert_eq!(
            scheme.version().map(alloc::string::ToString::to_string).as_deref(),
            Some("1.2.3")
        );
        assert_eq!(scheme.valid_from(), Some(&valid_from));
        assert_eq!(scheme.valid_to(), None);
        assert_eq!(scheme.agency(), "SDMX");
        assert!(scheme.is_partial_language());
        assert!(scheme.is_external_reference());
        assert_eq!(scheme.service_url(), Some("https://service"));
        assert_eq!(scheme.structure_url(), Some("https://structure"));
        assert!(scheme.is_partial());

        // The Agency carrier forwards its identifiable and nameable accessors to its metadata.
        let agency = Agency::new(full_nameable("ESTAT"), Vec::new()).unwrap();
        assert_eq!(agency.id(), "ESTAT");
        assert_eq!(agency.urn(), Some("urn:x"));
        assert_eq!(agency.uri(), Some("uri"));
        assert_eq!(agency.annotations().len(), 1);
        assert_eq!(agency.links().len(), 1);
        assert_eq!(agency.names().first(), "Eurostat");
        assert_eq!(agency.descriptions().map(LocalisedString::first), Some("Statistical office"));
    }

    #[test]
    fn contact_default_is_all_absent() {
        let contact = Contact::default();
        assert!(contact.names.is_none());
        assert!(contact.departments.is_none());
        assert!(contact.roles.is_none());
        assert!(contact.details.is_empty());

        // Struct-update sets only the stated field; the rest fall back to the default.
        let with_email =
            Contact { details: vec![ContactDetail::Email("x@y".into())], ..Default::default() };
        assert_eq!(with_email.details.len(), 1);
        assert!(with_email.names.is_none());
    }

    // Property tests: the internal serde round-trip over generated agency schemes, which
    // compose agencies and contacts (see `test_strategy`); wasm32 is excluded with the
    // rest of the property suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::agency_scheme;

        proptest! {
            #[test]
            fn agency_scheme_round_trips(value in agency_scheme()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
