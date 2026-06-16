//! The crate-wide [`Error`] type.
//!
//! `sdmx-types` exposes a single error enum, path-disambiguated as `sdmx_types::Error`. It is
//! deliberately **exhaustive** (no `#[non_exhaustive]`): the domain's validation failures are
//! knowable from the fixed SDMX specification, so consumers can write a complete `match` with no
//! catch-all arm, and any future variant is a deliberate, surfacing breaking change.
//!
//! A variant exists only once a producer for it lands, so the enum grows milestone by milestone.
//! The variants present here are those whose producers exist in the foundation layer; variants
//! whose producers arrive later (for example an NCName-identifier failure, the `Empty*` collection
//! family, and the representation-rule failures) join alongside them.

use alloc::string::String;

/// Errors returned by the fallible constructors and validators of `sdmx-types`.
///
/// Every variant corresponds to a schema-mechanical rejection: input an XSD
/// validator could itself reject (an off-pattern identifier, a malformed lexeme,
/// a missing-but-required element, a value that contradicts a schema-fixed one).
/// Constraints the SDMX specification states only in prose are **not** represented
/// here; they are non-destructive catalogued lints, not construction errors.
///
/// The enum derives `Clone`, `PartialEq`, and `Eq` so that consumers can assert on
/// specific failures (for example `assert_eq!(result, Err(Error::EmptyLocalisation))`);
/// every variant is either a unit or carries only `String`s, so all three are free.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A single error enum rather than per-module error types (ADR-0006), exhaustive by design: the
closed SDMX schema makes the failure set knowable, so exhaustiveness lets consumers match without a
catch-all and turns every new variant into a surfacing breaking change. The no-producerless-variants
policy keeps the set honest: a variant lands only with its first producer, and an absent one rejoins
on a later minor bump.

The identifier tiers back the distinct identifier-failure variants (`IDType` for `InvalidIdentifier`,
`NestedNCNameIDType` for `InvalidAgencyIdentifier`); the lexical newtypes back the
`Invalid{Decimal,Integer,Version,TimePeriod}` variants.

Decisions: D-0021, D-0023, D-0027, D-0031, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An identifier failed the SDMX `IDType` grammar (`[A-Za-z0-9_@$\-]+`).
    ///
    /// This is the loosest of the three identifier tiers: the base check
    /// every identifiable artefact shares (codes, generic ids, and the maintainable
    /// artefacts whose ids the spec leaves at `IDType`).
    #[error("Invalid artifact identifier: {0}. Must match SDMX IDType format.")]
    InvalidIdentifier(String),

    /// An `agencyID` failed the SDMX `NestedNCNameIDType` grammar (dotted `NCName`:
    /// `[A-Za-z][A-Za-z0-9_\-]*(\.[A-Za-z][A-Za-z0-9_\-]*)*`).
    ///
    /// Produced by [`MaintainableMetadata::new`](crate::MaintainableMetadata::new),
    /// the only owner of the agency-identifier check.
    #[error("Invalid agency identifier: {0}. Must match SDMX NestedNCNameIDType format.")]
    InvalidAgencyIdentifier(String),

    /// An identifier failed the SDMX `NCNameIDType` grammar (`[A-Za-z][A-Za-z0-9_\-]*`):
    /// the middle tier, stricter than `IDType` (a leading digit, `@`, `$`, or `.` are all
    /// rejected here even though `IDType` permits them).
    ///
    /// Produced by the constructors whose ids the spec types as `NCNameIDType`: the validated
    /// scheme items [`Concept::new`](crate::Concept::new) and [`Agency::new`](crate::Agency::new)
    /// (their own ids), and the `NCName` scheme wrappers [`Codelist::new`](crate::Codelist::new) and
    /// [`ConceptScheme::new`](crate::ConceptScheme::new) (their scheme ids). The component-id
    /// producers join in a later milestone.
    #[error("Invalid NCName identifier: {0}. Must match SDMX NCNameIDType format.")]
    InvalidNcNameIdentifier(String),

    /// A value failed the `xs:decimal` lexical grammar (optional sign, digits, at
    /// most one decimal point, no exponent). Produced by
    /// [`SdmxDecimal::new`](crate::SdmxDecimal::new).
    #[error("Invalid xs:decimal value: {0}.")]
    InvalidDecimal(String),

    /// A value failed the `xs:integer` lexical grammar (optional sign followed by
    /// digits only). Produced by [`SdmxInteger::new`](crate::SdmxInteger::new).
    #[error("Invalid xs:integer value: {0}.")]
    InvalidInteger(String),

    /// A value failed the SDMX `VersionType` grammar, either semantic
    /// (`major.minor.patch[-extension]`) or legacy (`major[.minor]`). Produced by
    /// [`SdmxVersion::new`](crate::SdmxVersion::new).
    #[error(
        "Invalid SDMX version: {0}. Must match VersionType (semantic major.minor.patch[-ext] or legacy major[.minor])."
    )]
    InvalidVersion(String),

    /// A value failed the SDMX `StandardTimePeriodType` grammar (a Gregorian period,
    /// `xs:dateTime`, or a reporting period). Produced by
    /// [`SdmxTimePeriod::new`](crate::SdmxTimePeriod::new).
    #[error("Invalid SDMX time period: {0}. Must match StandardTimePeriodType.")]
    InvalidTimePeriod(String),

    /// A localised string was constructed with an empty entry list. The parent
    /// elements (`Name`, `Description`) require at least one entry, so an empty list
    /// is mechanically schema-invalid. Produced by
    /// [`LocalisedString::new`](crate::LocalisedString::new).
    #[error(
        "Empty localized string. Artifact name or description must contain at least one language variant."
    )]
    EmptyLocalisation,

    /// A code selection in a codelist extension was constructed with an empty member-value
    /// list. The schema requires at least one `MemberValue` per selection (`MemberValue+`),
    /// so an empty list is mechanically schema-invalid. Produced by
    /// [`MemberValues::new`](crate::MemberValues::new).
    #[error("Invalid codelist extension: a code selection must contain at least one member value.")]
    EmptyMemberValues,

    /// A stated value contradicts an XSD `fixed` value, which an XSD validator would
    /// itself reject. The first field names the attribute or site, the
    /// second the offending stated value. Produced by
    /// [`FixedInclude::new`](crate::FixedInclude::new) in the foundation layer; later
    /// milestones add the descriptor-id and `AgencyScheme` producers.
    #[error("Invalid fixed attribute {0}: stated value '{1}' differs from the schema-fixed value.")]
    FixedAttributeMismatch(String, String),
}

/// Maps a validation [`Error`] onto a deserializer's own error type, preserving the message.
///
/// The custom `Deserialize` impls throughout the crate route a rejected value through their
/// type's validated `new()` and then through this helper, so a schema-invalid document fails
/// deserialization with the same diagnostic a direct constructor call would produce.
pub fn to_de_error<E: serde::de::Error>(e: Error) -> E {
    E::custom(e)
}
