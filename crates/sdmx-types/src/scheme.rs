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
deliberate opt-in and the marker stays sealable in a later phase if needed.

`ItemScheme` implements the full artefact hierarchy by delegating to its own `metadata`; the
concrete wrappers (`Codelist`, ...) delegate through THESE trait methods, not the private metadata
field, so a wrapper need not share a module with `ItemScheme`.

Decisions: D-0032, D-0051, D-0052.
"#
)]

use alloc::vec::Vec;

use chrono::{DateTime, FixedOffset};

use crate::{
    annotation::{Annotation, Link},
    artefact::{IdentifiableArtefact, MaintainableArtefact, NameableArtefact, VersionableArtefact},
    lexical::SdmxVersion,
    localised::LocalisedString,
    metadata::MaintainableMetadata,
};

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
pub trait SchemeItem: IdentifiableArtefact {}

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
///     MaintainableMetadata, NameableMetadata, VersionableMetadata,
/// };
///
/// let names = LocalisedString::new(vec![(Some("en".to_string()), "Frequency".to_string())])?;
/// let identifiable = IdentifiableMetadata::new("FREQ".to_string(), None, None, vec![], vec![])?;
/// let versionable = VersionableMetadata::new(
///     NameableMetadata::new(identifiable, names, None),
///     None,
///     None,
///     None,
/// );
/// let metadata =
///     MaintainableMetadata::new(versionable, "SDMX".to_string(), None, None, None, None)?;
///
/// let mut scheme: ItemScheme<Code> = ItemScheme::new(metadata, None);
/// let code_names = LocalisedString::new(vec![(Some("en".to_string()), "Annual".to_string())])?;
/// let code_id = IdentifiableMetadata::new("A".to_string(), None, None, vec![], vec![])?;
/// scheme.insert(Code {
///     metadata: NameableMetadata::new(code_id, code_names, None),
///     parent_id: None,
/// });
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
    /// Builds an empty item scheme. Items are added with [`insert`](Self::insert). Infallible: the
    /// scheme-id invariant (where one exists) is the concrete wrapper's, not the carrier's.
    #[must_use]
    pub const fn new(metadata: MaintainableMetadata, is_partial: Option<bool>) -> Self {
        Self { metadata, is_partial, items: Vec::new() }
    }

    /// Appends an item, preserving wire order.
    pub fn insert(&mut self, item: I) {
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
    fn valid_from(&self) -> Option<&DateTime<FixedOffset>> {
        self.metadata.valid_from()
    }
    fn valid_to(&self) -> Option<&DateTime<FixedOffset>> {
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
    use alloc::{string::ToString, vec};

    use super::*;
    use crate::{
        codelist::Code,
        metadata::{IdentifiableMetadata, NameableMetadata, VersionableMetadata},
    };

    fn metadata(id: &str) -> MaintainableMetadata {
        let names = LocalisedString::new(vec![(Some("en".into()), "Frequency".into())]).unwrap();
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
        let names = LocalisedString::new(vec![(Some("en".into()), id.to_string())]).unwrap();
        let identifiable =
            IdentifiableMetadata::new(id.into(), None, None, vec![], vec![]).unwrap();
        Code { metadata: NameableMetadata::new(identifiable, names, None), parent_id: None }
    }

    #[test]
    fn insert_preserves_order_and_get_is_first_match() {
        let mut scheme: ItemScheme<Code> = ItemScheme::new(metadata("CL_FREQ"), None);
        scheme.insert(code("A"));
        scheme.insert(code("M"));
        // A second "A" is held verbatim (a duplicate is schema-invalid but not collapsed).
        scheme.insert(code("A"));

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
        let names = LocalisedString::new(vec![(Some("en".into()), "Frequency".into())]).unwrap();
        let descriptions =
            LocalisedString::new(vec![(Some("en".into()), "How often".into())]).unwrap();
        let version = SdmxVersion::new("1.2.3".into()).unwrap();
        let valid_from = DateTime::parse_from_rfc3339("2024-01-01T00:00:00+00:00").unwrap();
        let identifiable = IdentifiableMetadata::new(
            "CL_FREQ".into(),
            Some("uri".into()),
            Some("urn:x".into()),
            vec![annotation],
            vec![link],
        )
        .unwrap();
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable, names, Some(descriptions)),
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
        let scheme: ItemScheme<Code> = ItemScheme::new(metadata, None);

        assert_eq!(scheme.id(), "CL_FREQ");
        assert_eq!(scheme.urn(), Some("urn:x"));
        assert_eq!(scheme.annotations().len(), 1);
        assert_eq!(scheme.links().len(), 1);
        assert_eq!(scheme.names().first(), "Frequency");
        assert_eq!(scheme.descriptions().map(LocalisedString::first), Some("How often"));
        assert_eq!(scheme.version().map(SdmxVersion::as_str), Some("1.2.3"));
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
        scheme.insert(code("A"));
        let json = serde_json::to_string(&scheme).unwrap();
        assert_eq!(serde_json::from_str::<ItemScheme<Code>>(&json).unwrap(), scheme);
    }
}
