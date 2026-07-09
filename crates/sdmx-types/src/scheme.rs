//! The generic item-scheme framework.
//!
//! An item scheme is a maintainable collection of items (codes, concepts, agencies). [`ItemScheme`]
//! is the shared carrier: maintenance metadata, the `isPartial` flag, and the ordered items. The
//! [`SchemeItem`] marker says which types may be scheme items; it is implemented explicitly per
//! item type, so membership is a deliberate opt-in rather than a blanket impl.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`ItemScheme` is a transparent pub-field carrier with derived `Serialize` AND `Deserialize` (D-0051):
with items stored as an ordered `Vec` there is no derived map key and so no key/id desync to defend
against, which is exactly §7's sharper test for the derive. Items are a `Vec`, not a keyed map: wire
order and any duplicates are preserved (a duplicate id is schema-invalid under the relevant
`xs:unique`, so a non-conformant document's duplicate is held verbatim and flagged by a catalogued
lint, never collapsed). `get` is a first-match Layer-2 view.

`SchemeItem` is implemented explicitly per item type (no blanket impl) so scheme membership is a
deliberate opt-in; the marker is sealed through the crate-private `sealed::Sealed` supertrait
(D-0078), so it can grow with the spec with no external implementation to break, while staying
fully usable in downstream bounds and calls.

`ItemScheme` implements the full artefact hierarchy by delegating to its own `metadata`; the
concrete wrappers (`Codelist`, ...) delegate through THESE trait methods, not the private metadata
field, so a wrapper need not share a module with `ItemScheme`.

Decisions: D-0032, D-0051, D-0052, D-0078.
"#
)]

use alloc::vec::Vec;

use crate::{
    annotation::{Annotation, Link},
    artefact::{IdentifiableArtefact, MaintainableArtefact, NameableArtefact, VersionableArtefact},
    lexical::{SdmxDateTime, SdmxVersion},
    localised::LocalisedString,
    metadata::MaintainableMetadata,
    sealed,
};

// ---------------------------------------------------------------------------
// SchemeItem
// ---------------------------------------------------------------------------

/// The marker for a type that may be an item in an [`ItemScheme`].
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// A supertrait of [`IdentifiableArtefact`] with no added methods: it records the deliberate
/// decision that a type is a scheme item. Implemented explicitly for [`Code`](crate::Code),
/// [`Concept`](crate::Concept), and [`Agency`](crate::Agency); there is no blanket impl.
///
/// Sealed (D-0078): usable in downstream bounds and calls like any trait, but implementable only
/// within `sdmx-types`.
pub trait SchemeItem: IdentifiableArtefact + sealed::Sealed {}

// ---------------------------------------------------------------------------
// ItemScheme
// ---------------------------------------------------------------------------

/// A maintainable collection of items, generic over the item type.
///
/// ## Specification
/// - **Type**: `ItemSchemeType`
/// - **Element**: N/A (Abstract Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ItemSchemeType.md"))]
///
/// Carries the maintenance metadata, the `isPartial` flag, and the items in wire order. The
/// concrete schemes ([`Codelist`](crate::Codelist), [`ConceptScheme`](crate::ConceptScheme),
/// [`AgencyScheme`](crate::AgencyScheme)) wrap this and forward to it.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     Code, IdentifiableArtefact, IdentifiableMetadata, ItemScheme, LocalisedString,
///     LocalisedText, MaintainableMetadata, NameableMetadata, VersionableMetadata,
/// };
///
/// let names = LocalisedString::new(vec![LocalisedText {
///     language: Some(String::from("en")),
///     text: String::from("Frequency"),
/// }])?;
/// let identifiable =
///     IdentifiableMetadata::new(String::from("FREQ"), None, None, Vec::new(), Vec::new())?;
/// let versionable = VersionableMetadata::new(
///     NameableMetadata::new(identifiable, names, None),
///     None,
///     None,
///     None,
/// );
/// let metadata =
///     MaintainableMetadata::new(versionable, String::from("SDMX"), None, None, None, None)?;
///
/// let mut scheme: ItemScheme<Code> = ItemScheme::new(metadata, None);
/// let code_names = LocalisedString::new(vec![LocalisedText {
///     language: Some(String::from("en")),
///     text: String::from("Annual"),
/// }])?;
/// let code_id = IdentifiableMetadata::new(String::from("A"), None, None, Vec::new(), Vec::new())?;
/// scheme.push(Code::new(NameableMetadata::new(code_id, code_names, None), None)?);
/// assert_eq!(scheme.get("A").map(IdentifiableArtefact::id), Some("A"));
/// assert!(!scheme.is_partial());
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ItemScheme<I: SchemeItem> {
    /// The maintenance metadata shared by every maintainable artefact.
    pub metadata: MaintainableMetadata,
    /// `isPartial` (`xs:boolean`, schema default `false`): `None` ⟺ absent. Distinct from
    /// [`MaintainableMetadata`]'s `isPartialLanguage`: this flags an incomplete set of *items*,
    /// that one an incomplete set of *languages*. The effective value is the
    /// [`is_partial`](Self::is_partial) view.
    pub is_partial: Option<bool>,
    /// The scheme's items in wire order; duplicates preserved.
    pub items: Vec<I>,
}

impl<I: SchemeItem> ItemScheme<I> {
    /// Builds an empty item scheme. Items are added with [`push`](Self::push). Infallible: the
    /// scheme-id invariant (where one exists) is the concrete wrapper's, not the carrier's.
    #[must_use]
    pub const fn new(metadata: MaintainableMetadata, is_partial: Option<bool>) -> Self {
        Self { metadata, is_partial, items: Vec::new() }
    }

    /// Appends an item, preserving wire order.
    pub fn push(&mut self, item: I) {
        self.items.push(item);
    }

    /// The first item whose effective id equals `id`, in wire order.
    /// When an id repeats, the first wins; later items stay reachable through [`iter`](Self::iter).
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&I> {
        self.items.iter().find(|item| item.id() == id)
    }

    /// Iterates the items in wire order.
    pub fn iter(&self) -> impl Iterator<Item = &I> {
        self.items.iter()
    }

    /// The effective value of `isPartial` (schema default `false`): `true` if only the relevant
    /// portion of the scheme is communicated.
    #[must_use]
    pub fn is_partial(&self) -> bool {
        self.is_partial.unwrap_or(false)
    }
}

impl<I: SchemeItem> IdentifiableArtefact for ItemScheme<I> {
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

impl<I: SchemeItem> NameableArtefact for ItemScheme<I> {
    fn names(&self) -> &LocalisedString {
        self.metadata.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.metadata.descriptions()
    }
}

impl<I: SchemeItem> VersionableArtefact for ItemScheme<I> {
    fn version(&self) -> Option<&SdmxVersion> {
        self.metadata.version()
    }
    fn valid_from(&self) -> Option<&SdmxDateTime> {
        self.metadata.valid_from()
    }
    fn valid_to(&self) -> Option<&SdmxDateTime> {
        self.metadata.valid_to()
    }
}

impl<I: SchemeItem> MaintainableArtefact for ItemScheme<I> {
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
    use alloc::{
        string::{String, ToString},
        vec,
    };

    use super::*;
    use crate::{
        codelist::Code,
        localised::LocalisedText,
        metadata::{IdentifiableMetadata, NameableMetadata, VersionableMetadata},
    };

    fn metadata(id: &str) -> MaintainableMetadata {
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some(String::from("en")),
            text: String::from("Frequency"),
        }])
        .unwrap();
        let identifiable =
            IdentifiableMetadata::new(id.into(), None, None, Vec::new(), Vec::new()).unwrap();
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable, names, None),
            None,
            None,
            None,
        );
        MaintainableMetadata::new(versionable, String::from("SDMX"), None, None, None, None)
            .unwrap()
    }

    fn code(id: &str) -> Code {
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some(String::from("en")),
            text: id.to_string(),
        }])
        .unwrap();
        let identifiable =
            IdentifiableMetadata::new(id.into(), None, None, Vec::new(), Vec::new()).unwrap();
        Code::new(NameableMetadata::new(identifiable, names, None), None).unwrap()
    }

    #[test]
    fn push_preserves_order_and_get_is_first_match() {
        let mut scheme: ItemScheme<Code> = ItemScheme::new(metadata("CL_FREQ"), None);
        scheme.push(code("A"));
        scheme.push(code("M"));
        // A second "A" is held verbatim (a duplicate is schema-invalid but not collapsed).
        scheme.push(code("A"));

        let ids: alloc::vec::Vec<&str> = scheme.iter().map(IdentifiableArtefact::id).collect();
        assert_eq!(ids, vec!["A", "M", "A"]);
        // get is a first-match view.
        assert_eq!(scheme.get("A").map(IdentifiableArtefact::id), Some("A"));
        assert_eq!(scheme.get("M").map(IdentifiableArtefact::id), Some("M"));
        assert_eq!(scheme.get("Z"), None);
    }

    #[test]
    fn is_partial_is_an_effective_view() {
        let absent: ItemScheme<Code> = ItemScheme::new(metadata("CL_FREQ"), None);
        assert!(!absent.is_partial()); // default false
        let stated: ItemScheme<Code> = ItemScheme::new(metadata("CL_FREQ"), Some(true));
        assert!(stated.is_partial());
    }

    #[test]
    fn delegation_matrix_forwards_every_accessor() {
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
            text: String::from("Frequency"),
        }])
        .unwrap();
        let descriptions = LocalisedString::new(vec![LocalisedText {
            language: Some(String::from("en")),
            text: String::from("How often"),
        }])
        .unwrap();
        let version = SdmxVersion::new(String::from("1.2.3")).unwrap();
        let valid_from = SdmxDateTime::new(String::from("2024-01-01T00:00:00+00:00")).unwrap();
        let identifiable = IdentifiableMetadata::new(
            String::from("CL_FREQ"),
            Some(String::from("uri")),
            Some(String::from("urn:x")),
            vec![annotation],
            vec![link],
        )
        .unwrap();
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable, names, Some(descriptions)),
            Some(version),
            Some(valid_from.clone()),
            None,
        );
        let metadata = MaintainableMetadata::new(
            versionable,
            String::from("ESTAT"),
            Some(true),
            Some(true),
            Some(String::from("https://service")),
            Some(String::from("https://structure")),
        )
        .unwrap();
        let scheme: ItemScheme<Code> = ItemScheme::new(metadata, None);

        assert_eq!(scheme.id(), "CL_FREQ");
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
    }

    #[test]
    fn deserialize_round_trips() {
        let mut scheme: ItemScheme<Code> = ItemScheme::new(metadata("CL_FREQ"), Some(false));
        scheme.push(code("A"));
        crate::test_support::round_trip(&scheme);
    }
}
