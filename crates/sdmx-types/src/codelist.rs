//! Codes, codelists, and codelist extensions.
//!
//! A [`Codelist`] is a maintainable scheme of [`Code`]s. A code is the carrier exemplar: its id is
//! the loosest `IDType`, already enforced by the base metadata, so it adds no invariant and stays a
//! transparent pub-field carrier. A codelist may also *extend* other codelists, selecting members
//! to include or exclude ([`CodelistExtension`]).
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Two scheme-item patterns, decided by the item's spec id lexical type (D-0023). `Code` is the CARRIER
exemplar: its id is `IDType`, the loosest tier already enforced by `IdentifiableMetadata::new()`, so
there is nothing stricter to add and it stays a pub-field carrier with derived `Deserialize` and no
constructor of its own. (`Concept`/`Agency`, whose ids are `NCNameIDType`, are the validated-item
pattern instead.)

`Codelist`'s `scheme` field is private not for item storage (invariant-free) but because the WRAPPER
owns the NCName scheme-id invariant: `CodelistBaseType` restricts the id to `NCNameIDType`, stricter
than the base `IDType`, so `new()` re-validates and is fallible, and `Codelist` carries a custom
`Deserialize` routing through `new()`. The invariant-free `extensions` stay pub and are assigned
directly after `new()` (report-5 V-12). `ConceptScheme` tightens identically; `AgencyScheme` does
NOT (its id is `IDType` `fixed="AGENCIES"`).

`Cascade` is the spec's `CascadeSelectionType` (`boolean | "excluderoot"`), a tri-state, so an enum
(D-0018). It lives here because the codelist-extension member values are its first consumer; the
constraint types reuse it.

Decisions: D-0018, D-0023, D-0054.
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
    reference::CodelistReference,
    scheme::{ItemScheme, SchemeItem},
    validate::validate_ncname,
};

// ---------------------------------------------------------------------------
// Cascade
// ---------------------------------------------------------------------------

/// How a member-value selection cascades through a code hierarchy.
///
/// ## Specification
/// - **Type**: `CascadeSelectionType`
/// - **Element**: N/A (Simple Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/CascadeSelectionType.md"))]
///
/// The spec's `CascadeSelectionType` is `boolean | "excluderoot"`, a tri-state, so it is an enum:
/// `false` is [`None`](Self::None), `true` is [`IncludeChildren`](Self::IncludeChildren), and the
/// literal `"excluderoot"` is [`ExcludeRoot`](Self::ExcludeRoot).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Cascade {
    /// `false` (the default): just this value, no children.
    None,
    /// `true`: this value plus all child codes in a simple hierarchy.
    IncludeChildren,
    /// `"excluderoot"`: child codes only, not this value itself.
    ExcludeRoot,
}

// ---------------------------------------------------------------------------
// MemberValue, MemberValues, CodeSelection
// ---------------------------------------------------------------------------

/// A single value in a codelist-extension selection.
///
/// ## Specification
/// - **Type**: `MemberValueType`
/// - **Element**: `<MemberValue>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/MemberValueType.md"))]
///
/// The value content may contain wildcards (the spec stores them as content), so it is held
/// verbatim. An empty or off-pattern value is mechanically schema-invalid (the
/// `WildcardedMemberValueType` pattern forbids it) but is still held verbatim; its well-formedness
/// is a catalogued lint, not a construction error. The `cascade` attribute is optional with no
/// schema default.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A pub-field carrier, stored verbatim: member-value well-formedness (non-empty plus the
`WildcardedMemberValueType` character pattern) is a Layer-2 lint, not a `new()` rejection, applying
the ADR-0023 ceiling-not-mandate principle exactly as the `LocalisedString` language key does. The
pattern reads `$-%` as a range excluding the literal `-`, which `IDType` code ids permit, so a strict
check would refuse valid references; holding verbatim avoids that.

Decisions: D-0054, D-0061.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct MemberValue {
    /// The member value content (wildcards are part of the content), stored verbatim.
    pub value: String,
    /// How the selection cascades through the hierarchy; `None` ⟺ absent (no schema default).
    pub cascade: Option<Cascade>,
}

/// A non-empty list of [`MemberValue`]s.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps the `MemberValue+` of a code selection. The schema requires at least one member value, so
/// the constructor rejects an empty list.
///
/// ## Guarantees
///
/// Always holds at least one [`MemberValue`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct MemberValues(Vec<MemberValue>);

impl MemberValues {
    /// Builds a member-value list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyMemberValues`] if `values` is empty (the schema requires
    /// `MemberValue+`).
    pub fn new(values: Vec<MemberValue>) -> Result<Self, Error> {
        if values.is_empty() {
            return Err(Error::EmptyMemberValues);
        }
        Ok(Self(values))
    }

    /// The member values, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[MemberValue] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<MemberValue> {
        self.0
    }
}

impl From<MemberValues> for Vec<MemberValue> {
    fn from(value: MemberValues) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<MemberValue>> for MemberValues {
    type Error = Error;

    fn try_from(values: Vec<MemberValue>) -> Result<Self, Error> {
        Self::new(values)
    }
}

impl<'de> serde::Deserialize<'de> for MemberValues {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = Vec::<MemberValue>::deserialize(deserializer)?;
        Self::new(values).map_err(to_de_error)
    }
}

/// A code selection in a codelist extension: include the listed members, or exclude them.
///
/// ## Specification
/// - **Type**: `CodeSelectionType`
/// - **Element**: N/A (Base Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/CodeSelectionType.md"))]
///
/// The spec distinguishes the two senses by element name (`InclusiveCodeSelection` versus
/// `ExclusiveCodeSelection`), both of type `CodeSelectionType`; the distinction is modelled here
/// as the enum variant. Exhaustive: exactly these two arms.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CodeSelection {
    /// Include the listed members.
    Inclusive(MemberValues),
    /// Exclude the listed members.
    Exclusive(MemberValues),
}

// ---------------------------------------------------------------------------
// CodelistExtension
// ---------------------------------------------------------------------------

/// A reference to another codelist whose codes this codelist incorporates, optionally filtered.
///
/// ## Specification
/// - **Type**: `CodelistExtensionType`
/// - **Element**: `<CodelistExtension>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/CodelistExtensionType.md"))]
///
/// Invariant-free pub-field carrier: the reference self-validates structurally, the selection
/// composes the validated [`MemberValues`] newtype, and `prefix` is an unconstrained string.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CodelistExtension {
    /// The codelist being extended.
    pub codelist: CodelistReference,
    /// The optional member selection (`minOccurs="0"`); `None` ⟺ the whole codelist is included.
    pub selection: Option<CodeSelection>,
    /// The optional prefix the selection's `removePrefix` flag refers to.
    pub prefix: Option<String>,
}

// ---------------------------------------------------------------------------
// Code
// ---------------------------------------------------------------------------

/// A single code in a [`Codelist`].
///
/// ## Specification
/// - **Type**: `CodeType`
/// - **Element**: `<Code>`
/// - **Editions**: SDMX 3.0 and 3.1 (Divergent)
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/CodeType.3.0.md"))]
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/CodeType.3.1.md"))]
#[cfg_attr(design_docs, doc = "")]
///
/// The carrier exemplar: a code's id is `IDType` (the loosest tier, already validated by the inner
/// [`NameableMetadata`]), so it adds no invariant and exposes public fields with a derived
/// `Deserialize`. `parent_id` references another code in the same list for a simple hierarchy.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     Code, IdentifiableArtefact, IdentifiableMetadata, LocalisedString, LocalisedText,
///     NameableMetadata,
/// };
///
/// let names = LocalisedString::new(vec![LocalisedText {
///     language: Some("en".to_string()),
///     text: "Annual".to_string(),
/// }])?;
/// let identifiable = IdentifiableMetadata::new("A".to_string(), None, None, vec![], vec![])?;
/// let code = Code { metadata: NameableMetadata::new(identifiable, names, None), parent_id: None };
/// assert_eq!(code.id(), "A");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`CodeType` diverges across editions only in the `Parent` element's declared type
(`SingleNCNameIDType` in 3.0, `IDType` in 3.1), the two fragments above. The store is unaffected:
`parent_id` is held as an unvalidated structural reference (`Option<String>`, D-0020) either way, so
the superset carrier needs no per-edition branch.

Decisions: D-0020, D-0023.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Code {
    /// The code's nameable metadata (id, names, descriptions, annotations, links).
    pub metadata: NameableMetadata,
    /// The id of the parent code in a simple hierarchy, if any. A structural reference, not
    /// re-validated.
    pub parent_id: Option<String>,
}

impl IdentifiableArtefact for Code {
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

impl NameableArtefact for Code {
    fn names(&self) -> &LocalisedString {
        self.metadata.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.metadata.descriptions()
    }
}

impl SchemeItem for Code {}

// ---------------------------------------------------------------------------
// Codelist
// ---------------------------------------------------------------------------

/// A maintainable scheme of [`Code`]s.
///
/// ## Specification
/// - **Type**: `CodelistType`
/// - **Element**: `<Codelist>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/CodelistType.md"))]
///
/// Wraps an [`ItemScheme<Code>`](ItemScheme) and adds the optional codelist extensions. Its scheme
/// id is `NCNameIDType` (stricter than the base `IDType`), so [`new`](Self::new) re-validates and
/// is fallible. The item-access methods forward to the inner scheme, keeping it encapsulated.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     Codelist, IdentifiableMetadata, LocalisedString, LocalisedText, MaintainableArtefact,
///     MaintainableMetadata, NameableMetadata, VersionableMetadata,
/// };
///
/// let names = LocalisedString::new(vec![LocalisedText {
///     language: Some("en".to_string()),
///     text: "Frequency".to_string(),
/// }])?;
/// let identifiable =
///     IdentifiableMetadata::new("CL_FREQ".to_string(), None, None, vec![], vec![])?;
/// let versionable = VersionableMetadata::new(
///     NameableMetadata::new(identifiable, names, None),
///     None,
///     None,
///     None,
/// );
/// let metadata =
///     MaintainableMetadata::new(versionable, "SDMX".to_string(), None, None, None, None)?;
///
/// let codelist = Codelist::new(metadata, None)?;
/// assert_eq!(codelist.agency(), "SDMX");
/// assert!(!codelist.is_partial());
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Codelist {
    scheme: ItemScheme<Code>,
    /// The codelists this one extends (`0..unbounded`); empty ⟺ absent. Invariant-free, so public
    /// even though `scheme` is private: the `NCName` invariant rides on the scheme id, not here.
    pub extensions: Vec<CodelistExtension>,
}

impl Codelist {
    /// Builds an empty codelist, validating the scheme id against SDMX `NCNameIDType`. Codes are
    /// added with [`push`](Self::push); extensions are assigned to the public field.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNcNameIdentifier`] if the scheme id is not a valid `NCNameIDType`.
    pub fn new(metadata: MaintainableMetadata, is_partial: Option<bool>) -> Result<Self, Error> {
        validate_ncname(metadata.id())?;
        Ok(Self { scheme: ItemScheme::new(metadata, is_partial), extensions: Vec::new() })
    }

    /// Appends a code, preserving wire order.
    pub fn push(&mut self, code: Code) {
        self.scheme.push(code);
    }

    /// The first code whose id equals `id`, in wire order (a first-match view).
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Code> {
        self.scheme.get(id)
    }

    /// Iterates the codes in wire order.
    pub fn iter(&self) -> impl Iterator<Item = &Code> {
        self.scheme.iter()
    }

    /// The effective value of the scheme's `isPartial` flag (schema default `false`).
    #[must_use]
    pub fn is_partial(&self) -> bool {
        self.scheme.is_partial()
    }
}

impl IdentifiableArtefact for Codelist {
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

impl NameableArtefact for Codelist {
    fn names(&self) -> &LocalisedString {
        self.scheme.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.scheme.descriptions()
    }
}

impl VersionableArtefact for Codelist {
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

impl MaintainableArtefact for Codelist {
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

impl<'de> serde::Deserialize<'de> for Codelist {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            scheme: ItemScheme<Code>,
            extensions: Vec<CodelistExtension>,
        }
        let raw = Raw::deserialize(deserializer)?;
        // Route through new() so the NCName scheme-id invariant is enforced, then assign the
        // invariant-free fields directly (the items the scheme carried, and the extensions).
        let ItemScheme { metadata, is_partial, items } = raw.scheme;
        let mut codelist = Self::new(metadata, is_partial).map_err(to_de_error)?;
        codelist.scheme.items = items;
        codelist.extensions = raw.extensions;
        Ok(codelist)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::{string::ToString, vec};

    use super::*;
    use crate::{
        localised::LocalisedText,
        metadata::{IdentifiableMetadata, VersionableMetadata},
    };

    fn metadata(id: &str) -> MaintainableMetadata {
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "Frequency".into(),
        }])
        .unwrap();
        let identifiable =
            IdentifiableMetadata::new(id.into(), None, None, vec![], vec![]).unwrap();
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable, names, None),
            None,
            None,
            None,
        );
        MaintainableMetadata::new(versionable, "SDMX".into(), None, None, None, None).unwrap()
    }

    fn code(id: &str) -> Code {
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: id.to_string(),
        }])
        .unwrap();
        let identifiable =
            IdentifiableMetadata::new(id.into(), None, None, vec![], vec![]).unwrap();
        Code { metadata: NameableMetadata::new(identifiable, names, None), parent_id: None }
    }

    /// A nameable leaf with every optional field populated, for the delegation matrices.
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
        let versionable = VersionableMetadata::new(
            full_nameable("CL_FREQ"),
            Some(version),
            Some(valid_from),
            None,
        );
        let metadata = MaintainableMetadata::new(
            versionable,
            "ESTAT".into(),
            Some(true),
            Some(true),
            Some("https://service".into()),
            Some("https://structure".into()),
        )
        .unwrap();
        let codelist = Codelist::new(metadata, Some(true)).unwrap();

        // Every forwarded accessor on the wrapper resolves through the inner scheme.
        assert_eq!(codelist.id(), "CL_FREQ");
        assert_eq!(codelist.urn(), Some("urn:x"));
        assert_eq!(codelist.uri(), Some("uri"));
        assert_eq!(codelist.annotations().len(), 1);
        assert_eq!(codelist.links().len(), 1);
        assert_eq!(codelist.names().first(), "Frequency");
        assert_eq!(codelist.descriptions().map(LocalisedString::first), Some("How often"));
        assert_eq!(
            codelist.version().map(alloc::string::ToString::to_string).as_deref(),
            Some("1.2.3")
        );
        assert_eq!(codelist.valid_from(), Some(&valid_from));
        assert_eq!(codelist.valid_to(), None);
        assert_eq!(codelist.agency(), "ESTAT");
        assert!(codelist.is_partial_language());
        assert!(codelist.is_external_reference());
        assert_eq!(codelist.service_url(), Some("https://service"));
        assert_eq!(codelist.structure_url(), Some("https://structure"));

        // The Code carrier forwards its identifiable and nameable accessors to its own metadata.
        let carrier = Code { metadata: full_nameable("A"), parent_id: Some("ROOT".into()) };
        assert_eq!(carrier.id(), "A");
        assert_eq!(carrier.urn(), Some("urn:x"));
        assert_eq!(carrier.uri(), Some("uri"));
        assert_eq!(carrier.annotations().len(), 1);
        assert_eq!(carrier.links().len(), 1);
        assert_eq!(carrier.names().first(), "Frequency");
        assert_eq!(carrier.descriptions().map(LocalisedString::first), Some("How often"));
    }

    #[test]
    fn new_validates_scheme_id_as_ncname() {
        // A leading-digit scheme id is a valid IDType but not an NCNameIDType (a dot would not even
        // pass the base IDType check), so the NCName tightening is what rejects it here.
        assert!(Codelist::new(metadata("CL_FREQ"), None).is_ok());
        assert_eq!(
            Codelist::new(metadata("9FREQ"), None).unwrap_err(),
            Error::InvalidNcNameIdentifier("9FREQ".into())
        );
    }

    #[test]
    fn forwards_item_access_and_partial_view() {
        let mut codelist = Codelist::new(metadata("CL_FREQ"), Some(true)).unwrap();
        codelist.push(code("A"));
        codelist.push(code("M"));
        assert_eq!(codelist.get("A").map(IdentifiableArtefact::id), Some("A"));
        assert_eq!(codelist.iter().count(), 2);
        assert!(codelist.is_partial());
        assert_eq!(codelist.agency(), "SDMX");
    }

    #[test]
    fn member_values_rejects_empty() {
        assert_eq!(MemberValues::new(vec![]).unwrap_err(), Error::EmptyMemberValues);
        let ok = MemberValues::new(vec![MemberValue { value: "A".into(), cascade: None }]).unwrap();
        assert_eq!(ok.as_slice().len(), 1);
    }

    #[test]
    fn extension_round_trips_through_serde() {
        let extension = CodelistExtension {
            codelist: CodelistReference {
                agency: "SDMX".into(),
                id: "CL_BASE".into(),
                version: "1.0.0".parse().unwrap(),
            },
            selection: Some(CodeSelection::Inclusive(
                MemberValues::new(vec![MemberValue {
                    value: "A".into(),
                    cascade: Some(Cascade::IncludeChildren),
                }])
                .unwrap(),
            )),
            prefix: Some("X_".into()),
        };
        crate::test_support::round_trip(&extension);
    }

    #[test]
    fn deserialize_round_trips_and_restores_items() {
        let mut codelist = Codelist::new(metadata("CL_FREQ"), None).unwrap();
        codelist.push(code("A"));
        // The round-trip restores the items, which new() alone would not carry.
        crate::test_support::round_trip(&codelist);
    }

    #[test]
    fn deserialize_rejects_non_ncname_scheme_id() {
        // Codelist's Deserialize declares `Raw { scheme: ItemScheme<Code>, extensions }` and routes
        // the metadata through new(), which applies the NCName tightening. postcard is positional,
        // so a tuple of those field types carrying a leading-digit scheme id (valid IDType, so the
        // ItemScheme deserialises, but rejected by new()) proves the wire path re-runs the check.
        // A valid tuple of the same field types decodes — guards this proof's shape against Raw drift.
        let ok =
            (ItemScheme::<Code>::new(metadata("CL_FREQ"), None), Vec::<CodelistExtension>::new());
        assert!(postcard::from_bytes::<Codelist>(&postcard::to_allocvec(&ok).unwrap()).is_ok());
        let raw =
            (ItemScheme::<Code>::new(metadata("9FREQ"), None), Vec::<CodelistExtension>::new());
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<Codelist>(&bytes).is_err());
    }

    #[test]
    fn member_values_try_from_rejects_empty() {
        assert_eq!(MemberValues::try_from(vec![]).unwrap_err(), Error::EmptyMemberValues);
    }

    #[test]
    fn member_values_into_inner_and_from() {
        let v = vec![MemberValue { value: "V".to_string(), cascade: None }];
        assert_eq!(MemberValues::new(v.clone()).unwrap().into_inner(), v);
        assert_eq!(Vec::from(MemberValues::new(v.clone()).unwrap()), v);
    }

    // Property tests: the headline internal serde round-trip (D-0031/D-0063/D-0068) over
    // the full generated spine — a codelist composes every metadata leaf and its codes —
    // via `test_support::round_trip`. wasm32 is excluded with the rest of the property
    // suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::{code, codelist};

        proptest! {
            #[test]
            fn code_round_trips(value in code()) {
                crate::test_support::round_trip(&value);
            }

            #[test]
            fn codelist_round_trips(value in codelist()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
