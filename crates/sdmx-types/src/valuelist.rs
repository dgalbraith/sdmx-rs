//! Value lists and their items.
//!
//! A [`ValueList`] is a maintainable artefact holding a closed set of [`ValueItem`]s for a
//! dimension, measure, or attribute. It is deliberately not an item scheme: a value item extends
//! the annotable base directly (not the item base), its name is optional, and its id is a plain
//! `xs:string`, the fourth, unrestricted id tier.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`ValueList` is MAINTAINABLE (D-0047) but NOT an item scheme, so it deliberately does not use
`ItemScheme<I>`: `ValueItemType` extends `AnnotableType` directly (not `ItemType`), its name is
optional, and its id is plain `xs:string`, the fourth id tier (unrestricted: none of the D-0023
validators applies, and there is nothing mechanical for Layer 1 to reject). Its own scheme id stays
`IDType` (no NCName tighten, unlike `Codelist`).

`items` is a `Vec`, NOT a keyed map: value-item ids carry no uniqueness constraint and official
material exhibits duplicates (the same symbol under two ids), so keying by id would silently destroy
schema-valid wire. The element list is held verbatim, order included (D-0031). An empty value list
is schema-valid (a plain `Vec`, no newtype). Both types are invariant-free pub-field carriers with
derived `Deserialize` (§7).

Decisions: D-0031, D-0033, D-0047.
"#
)]

use alloc::{string::String, vec::Vec};

use chrono::{DateTime, FixedOffset};

use crate::{
    annotation::{Annotation, Link},
    artefact::{IdentifiableArtefact, MaintainableArtefact, NameableArtefact, VersionableArtefact},
    lexical::SdmxVersion,
    localised::LocalisedString,
    metadata::MaintainableMetadata,
};

// ---------------------------------------------------------------------------
// ValueItem
// ---------------------------------------------------------------------------

/// A single value in a [`ValueList`].
///
/// ## Specification
/// - **Type**: `ValueItemType`
/// - **Element**: `<ValueItem>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ValueItemType.md"))]
///
/// Invariant-free pub-field carrier. Its id is a plain `xs:string`, deliberately unvalidated (the
/// fourth id tier): symbols such as `$`, `€`, `¥`, even `""`, are mechanically schema-valid. Its
/// name is optional, and it carries annotations directly (it extends the annotable base).
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ValueItem {
    /// The value's id (plain `xs:string`, required, unvalidated).
    pub id: String,
    /// The value's localised names; `None` ⟺ no names (the name is optional).
    pub names: Option<LocalisedString>,
    /// The value's localised descriptions; `None` ⟺ absent.
    pub descriptions: Option<LocalisedString>,
    /// The value's annotations; empty ⟺ none.
    pub annotations: Vec<Annotation>,
}

// ---------------------------------------------------------------------------
// ValueList
// ---------------------------------------------------------------------------

/// A maintainable closed set of [`ValueItem`]s.
///
/// ## Specification
/// - **Type**: `ValueListType`
/// - **Element**: `<ValueList>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ValueListType.md"))]
///
/// Invariant-free pub-field carrier delegating the artefact hierarchy to its metadata. The items
/// are held in wire order, duplicates preserved; an empty value list is schema-valid.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     IdentifiableMetadata, LocalisedString, LocalisedText, MaintainableArtefact,
///     MaintainableMetadata, NameableMetadata, ValueItem, ValueList, VersionableMetadata,
/// };
///
/// let names = LocalisedString::new(vec![LocalisedText {
///     language: Some("en".to_string()),
///     text: "Currencies".to_string(),
/// }])?;
/// let identifiable = IdentifiableMetadata::new("VL_CUR".to_string(), None, None, vec![], vec![])?;
/// let versionable = VersionableMetadata::new(
///     NameableMetadata::new(identifiable, names, None),
///     None,
///     None,
///     None,
/// );
/// let metadata =
///     MaintainableMetadata::new(versionable, "SDMX".to_string(), None, None, None, None)?;
///
/// let value_list = ValueList {
///     metadata,
///     items: vec![ValueItem {
///         id: "EUR".to_string(),
///         names: None,
///         descriptions: None,
///         annotations: vec![],
///     }],
/// };
/// assert_eq!(value_list.agency(), "SDMX");
/// assert_eq!(value_list.items.len(), 1);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ValueList {
    /// The maintenance metadata shared by every maintainable artefact.
    pub metadata: MaintainableMetadata,
    /// The value items in wire order; duplicates preserved, empty ⟺ none.
    pub items: Vec<ValueItem>,
}

impl IdentifiableArtefact for ValueList {
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

impl NameableArtefact for ValueList {
    fn names(&self) -> &LocalisedString {
        self.metadata.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.metadata.descriptions()
    }
}

impl VersionableArtefact for ValueList {
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

impl MaintainableArtefact for ValueList {
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
    use alloc::vec;

    use super::*;
    use crate::{
        localised::LocalisedText,
        metadata::{IdentifiableMetadata, NameableMetadata, VersionableMetadata},
    };

    fn metadata(id: &str) -> MaintainableMetadata {
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "Currencies".into(),
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

    fn value_item(id: &str) -> ValueItem {
        ValueItem { id: id.into(), names: None, descriptions: None, annotations: vec![] }
    }

    #[test]
    fn empty_value_list_is_valid_and_delegates() {
        let value_list = ValueList { metadata: metadata("VL_X"), items: vec![] };
        assert_eq!(value_list.id(), "VL_X");
        assert_eq!(value_list.agency(), "SDMX");
        assert!(value_list.items.is_empty());
    }

    #[test]
    fn duplicate_ids_and_unrestricted_ids_are_held_verbatim() {
        // The fourth id tier is unrestricted, and ids carry no uniqueness constraint, so a symbol
        // id and a duplicate are both stored faithfully.
        let value_list = ValueList {
            metadata: metadata("VL_CUR"),
            items: vec![value_item("¥"), value_item("¥"), value_item("")],
        };
        let ids: alloc::vec::Vec<&str> = value_list.items.iter().map(|v| v.id.as_str()).collect();
        assert_eq!(ids, vec!["¥", "¥", ""]);
    }

    #[test]
    fn deserialize_round_trips() {
        let value_list = ValueList { metadata: metadata("VL_CUR"), items: vec![value_item("EUR")] };
        let json = serde_json::to_string(&value_list).unwrap();
        assert_eq!(serde_json::from_str::<ValueList>(&json).unwrap(), value_list);
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
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "Currencies".into(),
        }])
        .unwrap();
        let descriptions = LocalisedString::new(vec![LocalisedText {
            language: Some("en".into()),
            text: "ISO codes".into(),
        }])
        .unwrap();
        let version = SdmxVersion::new("1.2.3".into()).unwrap();
        let valid_from = DateTime::parse_from_rfc3339("2024-01-01T00:00:00+00:00").unwrap();
        let identifiable = IdentifiableMetadata::new(
            "VL_CUR".into(),
            Some("uri".into()),
            Some("urn:x".into()),
            vec![annotation],
            vec![link],
        )
        .unwrap();
        let metadata = MaintainableMetadata::new(
            VersionableMetadata::new(
                NameableMetadata::new(identifiable, names, Some(descriptions)),
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
        let value_list = ValueList { metadata, items: vec![] };

        assert_eq!(value_list.id(), "VL_CUR");
        assert_eq!(value_list.urn(), Some("urn:x"));
        assert_eq!(value_list.uri(), Some("uri"));
        assert_eq!(value_list.annotations().len(), 1);
        assert_eq!(value_list.links().len(), 1);
        assert_eq!(value_list.names().first(), "Currencies");
        assert_eq!(value_list.descriptions().map(LocalisedString::first), Some("ISO codes"));
        assert_eq!(value_list.version().map(SdmxVersion::as_str), Some("1.2.3"));
        assert_eq!(value_list.valid_from(), Some(&valid_from));
        assert_eq!(value_list.valid_to(), None);
        assert_eq!(value_list.agency(), "SDMX");
        assert!(value_list.is_partial_language());
        assert!(value_list.is_external_reference());
        assert_eq!(value_list.service_url(), Some("https://service"));
        assert_eq!(value_list.structure_url(), Some("https://structure"));
    }
}
