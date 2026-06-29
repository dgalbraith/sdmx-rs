//! SDMX annotation and link types.
//!
//! These attach to identifiable SDMX artefacts: [`Annotation`] carries free-form metadata
//! (optionally with [`AnnotationUrl`] links), and [`Link`] is a typed association to another
//! resource.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Invariant-free pub-field carriers: every field enforces its own (non-)constraints, so they use
derived `Serialize`/`Deserialize`. `Annotation` and `Link` are `IdentifiableType` members,
carried on the `IdentifiableMetadata` leaf so every identifiable artefact inherits them.

Decisions: D-0011, D-0035.
"#
)]

use alloc::{string::String, vec::Vec};

use crate::localised::LocalisedString;

// ---------------------------------------------------------------------------
// AnnotationUrl
// ---------------------------------------------------------------------------

/// An SDMX `AnnotationURL` element: a single URL with an optional language tag.
///
/// ## Specification
/// - **Type**: `AnnotationURLType`
/// - **Element**: `<AnnotationURL>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/AnnotationURLType.md"))]
///
/// The element carries one URL and at most one `xml:lang`. When the language tag is absent,
/// the specification treats the resource as not localised.
///
/// # Examples
///
/// ```
/// use sdmx_types::AnnotationUrl;
///
/// // All fields are public, so you can construct one directly.
/// let url = AnnotationUrl {
///     url: "https://example.com/guidelines".to_string(),
///     lang: Some("en".to_string()),
/// };
/// assert_eq!(url.lang.as_deref(), Some("en"));
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A single `xml:lang` attaches to each `AnnotationURL`, so modelling this as a
[`LocalisedString`] map would imply a one-to-many structure the wire does not have.

Decisions: D-0011.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AnnotationUrl {
    /// The URL payload. Maps directly to the `xs:anyURI` element content.
    ///
    /// This is stored verbatim and is not mechanically validated.
    pub url: String,

    /// The language tag. Maps directly to the optional `xml:lang` attribute.
    ///
    /// This is stored verbatim and is not mechanically validated.
    pub lang: Option<String>,
}

// ---------------------------------------------------------------------------
// Annotation
// ---------------------------------------------------------------------------

/// An SDMX `Annotation`: free-form metadata attached to any annotable artefact.
///
/// ## Specification
/// - **Type**: `AnnotationType`
/// - **Element**: `<Annotation>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/AnnotationType.md"))]
///
/// An annotation attaches non-normative notes to an artefact: an optional id, type, title,
/// value, localised text, and any number of associated URLs. Every field is optional or a
/// possibly-empty collection, so it mirrors the wire one-to-one.
///
/// # Examples
///
/// ```
/// use sdmx_types::{Annotation, AnnotationUrl};
///
/// // All fields are public, so you can construct one directly.
/// let note = Annotation {
///     id: Some("note-1".to_string()),
///     annotation_type: Some("source".to_string()),
///     annotation_title: Some("Data source".to_string()),
///     annotation_urls: vec![AnnotationUrl {
///         url: "https://example.com/source".to_string(),
///         lang: None,
///     }],
///     annotation_value: None,
///     texts: None,
/// };
/// assert_eq!(note.annotation_urls.len(), 1);
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Invariant-free pub-field carrier: every field is optional or a possibly-empty collection, so
there is no construction invariant. `annotation_type` stays a bare `String` rather than an enum
because the spec leaves the type open (adhere, don't invent).

Decisions: D-0011.
"#
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Annotation {
    /// An optional id that disambiguates the annotation (the `id` attribute).
    pub id: Option<String>,
    /// An optional, free-form type label (`AnnotationType`).
    pub annotation_type: Option<String>,
    /// An optional title (`AnnotationTitle`).
    pub annotation_title: Option<String>,
    /// Any number of associated URLs (`AnnotationURL`); empty when absent.
    pub annotation_urls: Vec<AnnotationUrl>,
    /// An optional non-localised value (`AnnotationValue`).
    pub annotation_value: Option<String>,
    /// Optional localised annotation text (`AnnotationText`); `None` when absent.
    pub texts: Option<LocalisedString>,
}

// ---------------------------------------------------------------------------
// Link
// ---------------------------------------------------------------------------

/// An SDMX `Link`: a typed association from an identifiable artefact to another resource.
///
/// ## Specification
/// - **Type**: `LinkType`
/// - **Element**: `<Link>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/LinkType.md"))]
///
/// A link carries more than a bare URL: a relationship type, the target URL, an optional SDMX
/// registry URN of the target, and an optional media-type hint.
///
/// # Examples
///
/// ```
/// use sdmx_types::Link;
///
/// // All fields are public, so you can construct one directly.
/// let link = Link {
///     rel: "metadata".to_string(),
///     url: "https://example.com/report".to_string(),
///     urn: None,
///     link_type: Some("PDF".to_string()),
/// };
/// assert_eq!(link.rel.as_str(), "metadata");
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Invariant-free pub-field carrier. `rel` and `link_type` stay `String` rather than enums because
the spec gives no enumeration for either (the `type` examples "PDF, text, HTML" are illustrative,
not a closed set), so an enum would invent a constraint the wire does not impose.

Decisions: D-0035.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Link {
    /// Identifies the kind of object being linked to (required `rel` attribute).
    pub rel: String,
    /// The target URL (required `url` attribute, `xs:anyURI`).
    ///
    /// This is stored verbatim and is not mechanically validated.
    pub url: String,
    /// The optional SDMX registry URN of the linked object (`urn` attribute, `xs:anyURI`).
    ///
    /// This is stored verbatim and is not mechanically validated.
    pub urn: Option<String>,
    /// An optional hint at the link's format (the spec's `type` attribute, for example `"PDF"`),
    /// renamed to avoid the Rust keyword.
    pub link_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annotation_default_is_all_absent() {
        let note = Annotation::default();
        assert!(note.id.is_none());
        assert!(note.annotation_type.is_none());
        assert!(note.annotation_title.is_none());
        assert!(note.annotation_urls.is_empty());
        assert!(note.annotation_value.is_none());
        assert!(note.texts.is_none());

        // Struct-update sets only the stated field; the rest fall back to the default.
        let with_value = Annotation { annotation_value: Some("x".into()), ..Default::default() };
        assert_eq!(with_value.annotation_value.as_deref(), Some("x"));
        assert!(with_value.id.is_none());
    }
}
