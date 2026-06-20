//! The crate-wide [`Error`] type.
//!
//! `sdmx-types` exposes a single error enum, path-disambiguated as `sdmx_types::Error`. It is
//! deliberately **exhaustive** (no `#[non_exhaustive]`): the domain's validation failures are
//! knowable from the fixed SDMX specification, so consumers can write a complete `match` with no
//! catch-all arm, and any future variant is a deliberate, surfacing breaking change.
//!
//! Every variant is reachable: the enum carries no placeholder cases, so it lists exactly the
//! failures the crate can produce.

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

Decisions: D-0021, D-0023, D-0027, D-0031, D-0036, D-0038, D-0040, D-0048, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An identifier failed the SDMX `IDType` grammar (`[A-Za-z0-9_@$\-]+`).
    ///
    /// This is the loosest of the three identifier tiers: the base check
    /// every identifiable artefact shares (codes, generic ids, and the maintainable
    /// artefacts whose ids the spec leaves at `IDType`).
    #[error("Invalid artefact identifier: {0}. Must match SDMX IDType format.")]
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
    /// [`ConceptScheme::new`](crate::ConceptScheme::new) (their scheme ids). The component leaf
    /// [`ComponentMetadata::new`](crate::ComponentMetadata::new) validates a stated component id.
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
        "Empty localised string. Artefact name or description must contain at least one language variant."
    )]
    EmptyLocalisation,

    /// A code selection in a codelist extension was constructed with an empty member-value
    /// list. The schema requires at least one `MemberValue` per selection (`MemberValue+`),
    /// so an empty list is mechanically schema-invalid. Produced by
    /// [`MemberValues::new`](crate::MemberValues::new).
    #[error("Invalid codelist extension: a code selection must contain at least one member value.")]
    EmptyMemberValues,

    /// A cube-region dimension selection was constructed with an empty value list. A chosen value
    /// arm requires at least one value (`Value+`), so an empty list is mechanically schema-invalid.
    /// Produced by [`CubeKeyValues::new`](crate::CubeKeyValues::new).
    #[error("Invalid cube region: a dimension value selection must contain at least one value.")]
    EmptyCubeKeyValues,

    /// A cube-region component selection was constructed with an empty value list. A chosen value
    /// arm requires at least one value (`Value+`), so an empty list is mechanically schema-invalid.
    /// Produced by [`SimpleComponentValues::new`](crate::SimpleComponentValues::new).
    #[error("Invalid cube region: a component value selection must contain at least one value.")]
    EmptySimpleComponentValues,

    /// A cube-region list was constructed with more than two regions. `DataConstraintType` caps the
    /// count at `maxOccurs="2"`, so a third region is mechanically schema-invalid. Produced by
    /// [`CubeRegions::new`](crate::CubeRegions::new).
    #[error("Invalid data constraint: a cube-region list may contain at most two regions.")]
    TooManyCubeRegions,

    /// An `AttributeRelationship::Dimensions` was constructed with an empty dimension list. The
    /// schema requires at least one dimension reference (`Dimension+`), so an empty list is
    /// mechanically schema-invalid. Produced by [`DimensionIds::new`](crate::DimensionIds::new).
    #[error(
        "Invalid attribute relationship: an AttributeRelationship::Dimensions must reference at least one dimension id."
    )]
    EmptyAttributeDimensions,

    /// An `AttributeRelationship::Group` was constructed with an empty group id. The schema requires
    /// a non-empty group reference, so an empty id is mechanically schema-invalid. Produced by
    /// [`GroupId::new`](crate::GroupId::new).
    #[error(
        "Invalid attribute relationship: an AttributeRelationship::Group must reference a non-empty group id."
    )]
    EmptyGroupId,

    /// A measure relationship was constructed with an empty measure list. The schema requires at
    /// least one measure reference, so an empty list is mechanically schema-invalid. Produced by
    /// [`MeasureRelationship::new`](crate::MeasureRelationship::new).
    #[error(
        "Invalid measure relationship: a MeasureRelationship must reference at least one measure."
    )]
    EmptyMeasureRelationship,

    /// A dimension list was constructed empty. `DimensionListType` requires at least one dimension
    /// (`Dimension+`), so an empty list is mechanically schema-invalid. Produced by
    /// [`DimensionList::new`](crate::DimensionList::new).
    #[error("Invalid dimension list: a DimensionList must contain at least one dimension.")]
    EmptyDimensionList,

    /// A group was constructed with an empty dimension list. The schema requires at least one
    /// `GroupDimension`, so an empty list is mechanically schema-invalid. Produced by
    /// [`GroupDimensions::new`](crate::GroupDimensions::new).
    #[error("Invalid group: a Group must reference at least one dimension.")]
    EmptyGroupDimensions,

    /// A present attribute list was constructed empty. The schema's member choice is
    /// `minOccurs="1"`, so a present `AttributeList` holds at least one attribute or metadata
    /// attribute usage (a structure with no attributes omits the descriptor entirely). Produced by
    /// [`AttributeList::new`](crate::AttributeList::new).
    #[error(
        "Invalid attribute list: a present AttributeList must contain at least one attribute or metadata attribute usage."
    )]
    EmptyAttributeList,

    /// A present measure list was constructed empty. `MeasureListType` requires at least one measure
    /// (`Measure+`), so an empty list is mechanically schema-invalid (a measure-less structure omits
    /// the descriptor entirely). Produced by [`MeasureList::new`](crate::MeasureList::new).
    #[error("Invalid measure list: a present MeasureList must contain at least one measure.")]
    EmptyMeasureList,

    /// A dimension constraint was constructed empty. `DimensionConstraintType` requires at least one
    /// dimension reference, so an empty list is mechanically schema-invalid. Produced by
    /// [`DimensionConstraint::new`](crate::DimensionConstraint::new).
    #[error(
        "Invalid dimension constraint: a DimensionConstraint must reference at least one dimension id."
    )]
    EmptyDimensionConstraint,

    /// A component's representation states a `textType` outside the subset its position allows.
    /// The first field names the component kind (for example `"Concept"`), the second the
    /// offending `textType`. This is a mechanical XSD restriction: each position
    /// restricts the base `DataType` enumeration to a tier-specific subset. Produced by the
    /// position-rule validators: the Basic-position validator (the core-representation check shared
    /// by [`Concept::new`](crate::Concept::new), [`Attribute::new`](crate::Attribute::new), and
    /// [`Measure::new`](crate::Measure::new)) and the dimension- and time-position validators
    /// ([`Dimension::new`](crate::Dimension::new) and
    /// [`TimeDimension::new`](crate::TimeDimension::new)).
    #[error(
        "Invalid representation for {0}: textType '{1}' is outside this position's allowed subset."
    )]
    InvalidTextTypeForComponent(String, String),

    /// A dimension's representation uses a `ValueList` enumeration, which the dimension position
    /// prohibits: a dimension admits a codelist enumeration only. The field names
    /// the component kind. Produced by [`Dimension::new`](crate::Dimension::new).
    #[error(
        "Invalid representation for {0}: a ValueList enumeration is not allowed at this position (codelist-only)."
    )]
    ValueListEnumerationNotAllowed(String),

    /// A time dimension's representation uses an enumeration, which the time position prohibits: it
    /// is `TextFormat`-only. The field names the component kind. Produced by
    /// [`TimeDimension::new`](crate::TimeDimension::new).
    #[error(
        "Invalid representation for {0}: an Enumeration is not allowed (TextFormat-only position)."
    )]
    EnumerationNotAllowed(String),

    /// A component's representation sets a facet its position prohibits: a dimension may not set
    /// `isMultiLingual` or a representation-level `minOccurs`/`maxOccurs`, and a time dimension may
    /// set only `textType`, `startTime`, and `endTime` and prohibits a representation-level
    /// `minOccurs`/`maxOccurs`. The first field names the component kind, the second
    /// the prohibited facet. Produced by [`Dimension::new`](crate::Dimension::new) and
    /// [`TimeDimension::new`](crate::TimeDimension::new).
    #[error("Invalid representation for {0}: facet '{1}' is prohibited at this position.")]
    ProhibitedRepresentationFacet(String, String),

    /// A stated value contradicts an XSD `fixed` value, which an XSD validator would
    /// itself reject. The first field names the attribute or site, the second the
    /// offending stated value. Produced by [`FixedInclude::new`](crate::FixedInclude::new),
    /// [`AgencyScheme::new`](crate::AgencyScheme::new) (the `fixed="AGENCIES"` scheme id),
    /// [`TimeDimension::new`](crate::TimeDimension::new) (the fixed `TIME_PERIOD` id), and the
    /// fixed-id descriptors [`DimensionList::new`](crate::DimensionList::new),
    /// [`AttributeList::new`](crate::AttributeList::new), and
    /// [`MeasureList::new`](crate::MeasureList::new).
    #[error("Invalid fixed attribute {0}: stated value '{1}' differs from the schema-fixed value.")]
    FixedAttributeMismatch(String, String),
}

/// Maps a validation [`Error`] onto a deserializer's own error type, preserving the message.
///
/// The custom `Deserialize` impls throughout the crate route a rejected value through their
/// type's validated `new()` and then through this helper, so a schema-invalid document fails
/// deserialisation with the same diagnostic a direct constructor call would produce.
pub fn to_de_error<E: serde::de::Error>(e: Error) -> E {
    E::custom(e)
}
