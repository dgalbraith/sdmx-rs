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
downstream implementation, while staying fully usable in downstream bounds and calls. What each
trait adds, and why, is recorded on that trait rather than here.

Decisions: D-0031, D-0052, D-0078.
"#
)]

use crate::{
    annotation::{Annotation, Link},
    lexical::{SdmxDateTime, SdmxVersion, VersionDisplay},
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
/// Sealed: usable in downstream bounds and calls like any trait, but implementable only
/// within `sdmx-types`.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The base of the hierarchy and the only place the identity members are declared. Concrete artefacts
compose an `IdentifiableMetadata` leaf and delegate these accessors to it rather than inheriting a
struct, so one identity block serves every artefact.

`links()` is a sibling of `annotations()` because `LinkType` sits on `IdentifiableType` itself and
is persisted in the structure message. A link is therefore producer-supplied domain content the
canonical superset round-trips, not the transport-layer affordance an earlier reading took it for
(D-0035).

Sealed through the crate-private `sealed::Sealed` supertrait (D-0078), so the accessor set can grow
with the spec without breaking a downstream implementation.

Decisions: D-0035, D-0078.
"#
)]
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
/// Sealed: usable in downstream bounds and calls like any trait, but implementable only
/// within `sdmx-types`.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Adds the localised members and no invariant of its own. `names()` is guaranteed non-empty because
`LocalisedString::new` rejects an empty entry list (D-0016), so the accessor surfaces a value the
metadata leaf has already validated; `descriptions()` is optional because `NameableType` declares
`Name` required and `Description` `minOccurs="0"`.

Sealed through the crate-private `sealed::Sealed` supertrait (D-0078), so the accessor set can grow
with the spec without breaking a downstream implementation.

Decisions: D-0016, D-0078.
"#
)]
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
/// Sealed: usable in downstream bounds and calls like any trait, but implementable only
/// within `sdmx-types`.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`version()` returns an `Option` because an artefact with no `version` is *un-versioned*, a distinct
spec state rather than a synonym for `1.0`; collapsing absent onto `1.0` was rejected as lossy
(D-0024). The `<unversioned>` sentinel therefore lives on the `VersionDisplay` adapter reached
through `version_display()`, never on `SdmxVersion`'s own `Display`, and its angle brackets sit
outside every SDMX version lexeme, so a sentinel that reached a writer would fail validation loudly
instead of passing as a version. Declaring `version_display()` as a provided method gives every
delegating artefact the display path with no per-impl boilerplate.

The validity windows store the stated `xs:dateTime` lexeme rather than an instant (D-0079), so a
schema-valid offsetless value and the distinct `Z` and `+00:00` spellings all survive, equality is
lexeme identity, and same-moment comparison is the explicit `instant()` view.

Sealed through the crate-private `sealed::Sealed` supertrait (D-0078), so the accessor set can grow
with the spec without breaking a downstream implementation.

Decisions: D-0024, D-0078, D-0079.
"#
)]
pub trait VersionableArtefact: NameableArtefact + sealed::Sealed {
    /// The artefact's version. `None` is the spec's "un-versioned" state, distinct from any
    /// version value.
    fn version(&self) -> Option<&SdmxVersion>;
    /// The start of the artefact's validity window, if any.
    ///
    /// The datum is the stored `xs:dateTime` lexeme: preserved and round-tripped verbatim, so
    /// a schema-valid offsetless value and the distinct `Z` and `+00:00` spellings all survive.
    /// Identity is that stored text, so two windows whose lexemes differ are distinct even at
    /// the same instant. The written date-time and stated offset are the type's value views
    /// ([`SdmxDateTime::date_time`], [`SdmxDateTime::offset`]); same-moment comparison is the
    /// explicit [`SdmxDateTime::instant`] view, never `Eq`.
    fn valid_from(&self) -> Option<&SdmxDateTime>;
    /// The end of the artefact's validity window, if any. Carries the same lexeme-storage and
    /// identity contract as [`valid_from`](Self::valid_from).
    fn valid_to(&self) -> Option<&SdmxDateTime>;

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
///
/// Sealed: usable in downstream bounds and calls like any trait, but implementable only
/// within `sdmx-types`.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`MaintainableType` diverges across editions: SDMX 3.1 adds the `isPartialLanguage` attribute
(`use="optional" default="false"`), absent in 3.0, surfaced here as
[`is_partial_language`](Self::is_partial_language). The two fragments above show each edition's
verbatim contract; the attribute is carried unconditionally as a superset member, its `false`
default applying to a 3.0 payload exactly as to an absent 3.1 attribute.

Sealed through the crate-private `sealed::Sealed` supertrait (D-0078), so the accessor set can grow
with the spec without breaking a downstream implementation.

Decisions: D-0010, D-0046, D-0078.
"#
)]
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
