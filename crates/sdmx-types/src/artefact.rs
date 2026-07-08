//! The SDMX artefact trait hierarchy.
//!
//! The SDMX information model layers `Identifiable → Nameable → Versionable → Maintainable`.
//! Rust has no class inheritance, so the shared interface is expressed as four supertrait-linked
//! traits providing standard accessors. Concrete domain types implement them by delegating to a
//! composed metadata leaf. The accessors are **effective views**: where the schema assigns a
//! default to an absent attribute, the trait applies it over the stored value.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The hierarchy mirrors the abstract artefact types of the SDMX information model (§5.3); concrete
types compose a metadata leaf (§5.4) and delegate to it. Traits rather than trait objects: the
accessors monomorphise, so there is no vtable or heap cost. The defaults applied here are the
Layer-2 effective views over the statedness the metadata leaves store (Layer 1).

The four traits are sealed through the crate-private `sealed::Sealed` supertrait (D-0078): only
`sdmx-types` implements them, so they grow with the spec's artefact members without breaking any
downstream implementation, while staying fully usable in downstream bounds and calls.

Decisions: D-0024, D-0031, D-0035, D-0052, D-0078.
"#
)]

use chrono::{DateTime, FixedOffset};

use crate::{
    annotation::{Annotation, Link},
    lexical::{SdmxVersion, VersionDisplay},
    localised::LocalisedString,
    sealed,
};

/// An identifiable artefact: it has an id and may carry a URN, a URI, annotations, and links.
///
/// ## Specification
/// - **Schema**: `SDMXCommon.xsd`
/// - **Type**: `IdentifiableType`
/// - **Element**: N/A (Abstract Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/IdentifiableType.md"))]
///
/// The base of the artefact hierarchy: every identifiable SDMX artefact exposes these accessors.
///
/// Sealed (D-0078): usable in downstream bounds and calls like any trait, but implementable only
/// within `sdmx-types`.
pub trait IdentifiableArtefact: sealed::Sealed {
    /// The artefact's effective id.
    fn id(&self) -> &str;
    /// The artefact's registry URN, if any.
    fn urn(&self) -> Option<&str>;
    /// The artefact's human-navigable URI, if any.
    fn uri(&self) -> Option<&str>;
    /// The artefact's annotations (empty slice if none).
    fn annotations(&self) -> &[Annotation];
    /// The artefact's links; empty slice if none. Sibling of
    /// [`annotations`](Self::annotations): both ride on `IdentifiableType`.
    fn links(&self) -> &[Link];
}

/// A nameable artefact: an identifiable artefact that additionally carries localised names and
/// optional localised descriptions.
///
/// ## Specification
/// - **Schema**: `SDMXCommon.xsd`
/// - **Type**: `NameableType`
/// - **Element**: N/A (Abstract Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/NameableType.md"))]
///
/// Sealed (D-0078): usable in downstream bounds and calls like any trait, but implementable only
/// within `sdmx-types`.
pub trait NameableArtefact: IdentifiableArtefact + sealed::Sealed {
    /// The artefact's localised names (guaranteed non-empty).
    fn names(&self) -> &LocalisedString;
    /// The artefact's localised descriptions, if any.
    fn descriptions(&self) -> Option<&LocalisedString>;
}

/// A versionable artefact: a nameable artefact that additionally carries version and validity
/// information.
///
/// ## Specification
/// - **Schema**: `SDMXCommon.xsd`
/// - **Type**: `VersionableType`
/// - **Element**: N/A (Abstract Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/VersionableType.md"))]
///
/// Sealed (D-0078): usable in downstream bounds and calls like any trait, but implementable only
/// within `sdmx-types`.
pub trait VersionableArtefact: NameableArtefact + sealed::Sealed {
    /// The artefact's version. `None` is the spec's "un-versioned" state, distinct from any
    /// version value.
    fn version(&self) -> Option<&SdmxVersion>;
    /// The start of the artefact's validity window, if any.
    ///
    /// The preserved datum is the XSD `dateTime` value: the instant *and* the stated
    /// numeric offset. The stated offset is data — a document stating `+05:00` and its UTC
    /// equivalent are distinct — and survives the round-trip, even though
    /// `DateTime<FixedOffset>` equality compares the instant alone. The spelling
    /// distinctions XSD's own lexical-to-value mapping collapses (`Z` versus `+00:00`,
    /// fractional-second zero padding) are deliberately not preserved.
    fn valid_from(&self) -> Option<&DateTime<FixedOffset>>;
    /// The end of the artefact's validity window, if any. Carries the same stated-offset
    /// contract as [`valid_from`](Self::valid_from).
    fn valid_to(&self) -> Option<&DateTime<FixedOffset>>;

    /// A `Display` adapter for the version that renders `<unversioned>` when absent. Every
    /// versionable artefact inherits this display path for free; it is for display and logging
    /// only and must never be round-tripped (the sentinel is un-roundtrippable by design).
    fn version_display(&self) -> VersionDisplay<'_> {
        VersionDisplay(self.version())
    }
}

/// A maintainable artefact: a versionable artefact owned by a maintenance agency, optionally a
/// stub whose full definition is resolved elsewhere.
///
/// ## Specification
/// - **Schema**: `SDMXCommon.xsd`
/// - **Type**: `MaintainableType`
/// - **Element**: N/A (Abstract Type)
/// - **Editions**: SDMX 3.0 and 3.1 (Divergent)
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/MaintainableType.3.0.md"))]
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/MaintainableType.3.1.md"))]
#[cfg_attr(design_docs, doc = "")]
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`MaintainableType` diverges across editions: SDMX 3.1 adds the `isPartialLanguage` attribute
(`use="optional" default="false"`), absent in 3.0, surfaced here as
[`is_partial_language`](Self::is_partial_language). The two fragments above show each edition's
verbatim contract; the attribute is carried unconditionally as a superset member, its `false`
default applying to a 3.0 payload exactly as to an absent 3.1 attribute.

Decisions: D-0010, D-0046.
"#
)]
///
/// Sealed (D-0078): usable in downstream bounds and calls like any trait, but implementable only
/// within `sdmx-types`.
pub trait MaintainableArtefact: VersionableArtefact + sealed::Sealed {
    /// The maintenance agency id (`agencyID`).
    fn agency(&self) -> &str;
    /// `true` if this artefact carries only a *subset* of the localisations its agency
    /// maintains (the spec's `isPartialLanguage`, SDMX 3.1 only). `false` (the default, and the
    /// value for a 3.0 payload or an absent attribute) asserts the localisations are complete.
    fn is_partial_language(&self) -> bool;
    /// `true` if this artefact is a stub whose full definition lives elsewhere (resolve via the
    /// service or structure URL); `false` (the default) means it is defined inline.
    fn is_external_reference(&self) -> bool;
    /// `serviceURL`: an SDMX web-service endpoint the artefact can be retrieved from.
    fn service_url(&self) -> Option<&str>;
    /// `structureURL`: a structure message (same version) containing the artefact.
    fn structure_url(&self) -> Option<&str>;
}
