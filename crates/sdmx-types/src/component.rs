//! Shared foundations for the DSD components.
//!
//! [`ComponentMetadata`] is the identity leaf every component (dimension, attribute, measure)
//! embeds, and [`Usage`] is the mandatory-or-optional flag attributes and measures share. They
//! live here, apart from the component types themselves, because more than one component builds on
//! each: keeping them together avoids any one component owning state its siblings also need.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A component's id is `use="optional"` (`ComponentBaseType`, both editions): when absent, the
component's identity is inherited from its concept identity, so [`ComponentMetadata`] cannot reuse
`IdentifiableMetadata`, whose id is mechanically required. The leaf stores the id's *statedness*
exactly (`None` ⟺ inherited; `Some` is NCName-validated, the D-0023 component tier made
conditional-on-stated) and exposes only `stated_id()`: the effective `id()` view lives on each
component, which resolves the inherited value against its concept reference, so the trait is the
domain boundary (D-0057).

`Usage` is the spec's single `UsageType`; it is set positionally in the component constructors, so
it is an enum rather than a bare bool (the D-0018 bool-versus-enum rule), and is stored as
`Option<Usage>` to preserve statedness (D-0052).

Decisions: D-0018, D-0023, D-0052, D-0057.
"#
)]

use alloc::{string::String, vec::Vec};

use crate::{
    annotation::{Annotation, Link},
    error::{Error, to_de_error},
    validate::validate_ncname,
};

// ---------------------------------------------------------------------------
// ComponentMetadata
// ---------------------------------------------------------------------------

/// The component identity leaf: an optional, conditionally validated id plus optional URI, URN,
/// annotations, and links.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// The storage leaf bundling the `ComponentBaseType` identity attributes every component embeds.
/// Unlike [`IdentifiableMetadata`](crate::IdentifiableMetadata), the id is optional: a component id
/// is `use="optional"`, and an absent id means the component inherits its concept's identity.
///
/// # Examples
///
/// ```
/// use sdmx_types::ComponentMetadata;
///
/// // A stated id is validated as an NCName.
/// let meta = ComponentMetadata::new(Some("OBS_VALUE".to_string()), None, None, vec![], vec![])?;
/// assert_eq!(meta.stated_id(), Some("OBS_VALUE"));
///
/// // An absent id is allowed: the component inherits its concept's identity.
/// let inherited = ComponentMetadata::new(None, None, None, vec![], vec![])?;
/// assert_eq!(inherited.stated_id(), None);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ComponentMetadata {
    id: Option<String>,
    uri: Option<String>,
    urn: Option<String>,
    annotations: Vec<Annotation>,
    links: Vec<Link>,
}

impl ComponentMetadata {
    /// Builds component metadata, validating `id` against SDMX `NCNameIDType` **when it is stated**.
    /// A component id is `use="optional"`, so `None` (the identity is inherited from the concept)
    /// has nothing to validate.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNcNameIdentifier`] if `id` is `Some` and not a valid `NCNameIDType`.
    pub fn new(
        id: Option<String>,
        uri: Option<String>,
        urn: Option<String>,
        annotations: Vec<Annotation>,
        links: Vec<Link>,
    ) -> Result<Self, Error> {
        if let Some(id) = &id {
            validate_ncname(id)?;
        }
        Ok(Self { id, uri, urn, annotations, links })
    }

    /// Stated: the id exactly as the wire carried it. `None` means the id was absent and the
    /// component's identity is inherited from its concept.
    #[must_use]
    pub fn stated_id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// The component's URN, if any. A component delegates its `IdentifiableArtefact::urn` here.
    #[must_use]
    pub fn urn(&self) -> Option<&str> {
        self.urn.as_deref()
    }

    /// The component's URI, if any. A component delegates its `IdentifiableArtefact::uri` here.
    #[must_use]
    pub fn uri(&self) -> Option<&str> {
        self.uri.as_deref()
    }

    /// The annotations carried on the component. A component delegates its
    /// `IdentifiableArtefact::annotations` here.
    #[must_use]
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// The links carried on the component. A component delegates its `IdentifiableArtefact::links`
    /// here.
    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }
}

impl<'de> serde::Deserialize<'de> for ComponentMetadata {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            id: Option<String>,
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
// Usage
// ---------------------------------------------------------------------------

/// Whether a component's value is mandatory or optional.
///
/// ## Specification
/// - **Type**: `UsageType`
/// - **Element**: N/A (Simple Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/UsageType.md"))]
///
/// The spec's single `UsageType` (`mandatory | optional`), the usage flag attributes and measures
/// share. It is stored as `Option<Usage>` on those components so the schema default (`optional`)
/// stays an effective view, never baked into the store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Usage {
    /// The component's value must be present.
    Mandatory,
    /// The component's value may be omitted.
    Optional,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn new_validates_stated_id_as_ncname() {
        // A stated id must be a valid NCName.
        assert!(
            ComponentMetadata::new(Some("OBS_VALUE".into()), None, None, vec![], vec![]).is_ok()
        );
        // A dotted id is not an NCName at the component tier.
        assert_eq!(
            ComponentMetadata::new(Some("a.b".into()), None, None, vec![], vec![]).unwrap_err(),
            Error::InvalidNcNameIdentifier("a.b".into())
        );
    }

    #[test]
    fn absent_id_skips_validation_and_reads_as_none() {
        // None means inherited identity: nothing to validate, stated_id() reports the absence.
        let meta = ComponentMetadata::new(None, None, None, vec![], vec![]).unwrap();
        assert_eq!(meta.stated_id(), None);
    }

    #[test]
    fn deserialize_round_trips_and_enforces_the_id_tier() {
        let meta = ComponentMetadata::new(
            Some("FREQ".into()),
            None,
            Some("urn:sdmx:freq".into()),
            vec![],
            vec![],
        )
        .unwrap();
        crate::test_support::round_trip(&meta);

        // The serde path routes through new(): a Raw (id, uri, urn, annotations, links) carrying
        // an invalid id decodes into new(), which rejects it.
        // A valid tuple of the same field types decodes — guards this proof's shape against Raw drift.
        let ok = (
            Some(String::from("OBS_VALUE")),
            None::<String>,
            None::<String>,
            Vec::<crate::Annotation>::new(),
            Vec::<crate::Link>::new(),
        );
        assert!(
            postcard::from_bytes::<ComponentMetadata>(&postcard::to_allocvec(&ok).unwrap()).is_ok()
        );
        let bad = (
            Some(String::from("a.b")),
            None::<String>,
            None::<String>,
            Vec::<crate::Annotation>::new(),
            Vec::<crate::Link>::new(),
        );
        let bytes = postcard::to_allocvec(&bad).unwrap();
        assert!(postcard::from_bytes::<ComponentMetadata>(&bytes).is_err());
    }

    #[test]
    fn usage_round_trips_both_variants() {
        for usage in [Usage::Mandatory, Usage::Optional] {
            crate::test_support::round_trip(&usage);
        }
    }

    // Property tests: the internal serde round-trip over generated values (see
    // `test_strategy`); wasm32 is excluded with the rest of the property suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::component_metadata;

        proptest! {
            #[test]
            fn component_metadata_round_trips(value in component_metadata()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
