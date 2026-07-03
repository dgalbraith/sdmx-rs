//! Component representation: how a component is typed and valued.
//!
//! A representation is either an *enumeration* (a reference to a [`Codelist`](crate::Codelist) or
//! [`ValueList`](crate::ValueList), optionally refined by an [`EnumerationFormat`]) or an
//! *uncoded* [`TextFormat`] (a bundle of facets), plus the `minOccurs`/`maxOccurs` on the
//! representation node. The single stored [`Representation`] is the superset of every position's
//! shape; which arms, facets, and `textType`s a given component position admits is enforced at the
//! component constructors, not by this type. It provides the position validators those constructors
//! call: [`validate_basic_representation`] (the Basic tier), [`validate_dimension_representation`],
//! and [`validate_time_representation`].
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The store is the SUPERSET of every position's shape (§5.6.1, D-0028/D-0048); the per-position
mechanical restrictions live on the component constructors (the D-0023 owns-its-own-check pattern),
exposed here only as the `DataType` subset predicates and `validate_basic_representation`. This
replaces the earlier ad-hoc coded-only `codelist` field on components: a representation models both
the coded and uncoded cases faithfully.

`TextFormat`, `EnumerationFormat`, `Representation`, and the enums are invariant-free pub-field
carriers (derived `Deserialize`); only the position validators carry logic, and they are free
functions the component constructors call. `DataType`'s subset predicates (`is_basic`, `is_simple`,
`is_code`, `is_time`) are the Layer-2 views the validators check against; the tiers form a
restriction chain (Basic ⊃ Simple ⊃ Code), so `is_code` composes `is_simple`.

Decisions: D-0021, D-0027, D-0028, D-0046, D-0047, D-0048.
"#
)]

use alloc::{format, string::String};

use crate::{
    error::Error,
    lexical::{SdmxDecimal, SdmxInteger, SdmxTimePeriod},
    reference::{CodelistReference, ValueListReference},
};

// ---------------------------------------------------------------------------
// DataType
// ---------------------------------------------------------------------------

/// The SDMX `textType` facet: the data type of a component's values.
///
/// ## Specification
/// - **Type**: `DataType`
/// - **Element**: N/A (Simple Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataType.md"))]
///
/// All 44 enumerated values, modelled exhaustively: the set is fixed by the specification, so a
/// new value would be a real spec event a consumer should be made to handle. The schema default
/// (applied at a component position, never baked into the store) is [`DataType::String`].
///
/// Each position admits only a subset of these values; the subset-membership predicates
/// ([`is_basic`](Self::is_basic), [`is_code`](Self::is_code)) are what the component
/// constructors check against.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

ONE wide enum is the store; the spec's per-position subsets (Basic = the full set minus
`{DataSetReference, IdentifiableReference, KeyValues}`; Code = Simple minus
`{DateTime, Decimal, Double, Float, GeospatialInformation, Time, TimeRange}`) are subset-membership
predicates and are ENFORCED at the component constructors (D-0048), not by four parallel enums.

Decisions: D-0021, D-0048.
"#
)]
// The all-caps `URI` and `XHTML` variants reproduce the spec's enumeration tokens verbatim so the
// derived serde representation matches the wire; the lint that would lowercase them is waived here.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DataType {
    /// A character string (`xs:string`); the schema default.
    String,
    /// Alphabetic characters only.
    Alpha,
    /// Alphabetic and numeric characters only.
    AlphaNumeric,
    /// Numeric characters only (digits as text, not a number type).
    Numeric,
    /// An arbitrary-precision integer (`xs:integer`).
    BigInteger,
    /// A signed 32-bit integer (`xs:int`).
    Integer,
    /// A signed 64-bit integer (`xs:long`).
    Long,
    /// A signed 16-bit integer (`xs:short`).
    Short,
    /// A decimal number (`xs:decimal`).
    Decimal,
    /// A single-precision floating-point number (`xs:float`).
    Float,
    /// A double-precision floating-point number (`xs:double`).
    Double,
    /// A boolean (`xs:boolean`).
    Boolean,
    /// A URI (`xs:anyURI`).
    URI,
    /// A monotonically increasing integer count.
    Count,
    /// A value within an inclusive numeric range.
    InclusiveValueRange,
    /// A value within an exclusive numeric range.
    ExclusiveValueRange,
    /// A value that increments by a fixed interval.
    Incremental,
    /// Any observational time period (standard or reporting).
    ObservationalTimePeriod,
    /// A standard (Gregorian or date-time) time period.
    StandardTimePeriod,
    /// A basic Gregorian or date-time period.
    BasicTimePeriod,
    /// A Gregorian calendar period.
    GregorianTimePeriod,
    /// A Gregorian year (`xs:gYear`).
    GregorianYear,
    /// A Gregorian year and month (`xs:gYearMonth`).
    GregorianYearMonth,
    /// A Gregorian date (`xs:date`).
    GregorianDay,
    /// Any reporting time period.
    ReportingTimePeriod,
    /// A reporting year.
    ReportingYear,
    /// A reporting semester.
    ReportingSemester,
    /// A reporting trimester.
    ReportingTrimester,
    /// A reporting quarter.
    ReportingQuarter,
    /// A reporting month.
    ReportingMonth,
    /// A reporting week.
    ReportingWeek,
    /// A reporting day.
    ReportingDay,
    /// A date and time (`xs:dateTime`).
    DateTime,
    /// A range between two time points.
    TimeRange,
    /// A month (`xs:gMonth`).
    Month,
    /// A month and day (`xs:gMonthDay`).
    MonthDay,
    /// A day of the month (`xs:gDay`).
    Day,
    /// A time of day (`xs:time`).
    Time,
    /// A duration (`xs:duration`).
    Duration,
    /// Geospatial information.
    GeospatialInformation,
    /// XHTML markup content.
    XHTML,
    /// A set of key values.
    KeyValues,
    /// A reference to an identifiable artefact.
    IdentifiableReference,
    /// A reference to a data set.
    DataSetReference,
}

impl DataType {
    /// `true` if this value is admitted at a Basic-tier position (a concept core representation,
    /// an attribute, or a measure): every value except the three reference/key types
    /// (`KeyValues`, `IdentifiableReference`, `DataSetReference`).
    #[must_use]
    pub const fn is_basic(self) -> bool {
        !matches!(self, Self::KeyValues | Self::IdentifiableReference | Self::DataSetReference)
    }

    /// `true` if this value is admitted at a Dimension position (the `SimpleDataType` subset):
    /// the Basic set minus `XHTML`.
    #[must_use]
    pub const fn is_simple(self) -> bool {
        self.is_basic() && !matches!(self, Self::XHTML)
    }

    /// `true` if this value is admitted at a coded position (the `textType` subset of
    /// `CodedTextFormatType`): the Simple set, further minus the seven uncoded-only types
    /// (`DateTime`, `Decimal`, `Double`, `Float`, `GeospatialInformation`, `Time`, `TimeRange`).
    #[must_use]
    pub const fn is_code(self) -> bool {
        // The spec's tiers are a restriction chain (Basic ⊃ Simple ⊃ Code), expressed here as the
        // composed predicate now that `is_simple` exists.
        self.is_simple()
            && !matches!(
                self,
                Self::DateTime
                    | Self::Decimal
                    | Self::Double
                    | Self::Float
                    | Self::GeospatialInformation
                    | Self::Time
                    | Self::TimeRange
            )
    }

    /// `true` if this value is admitted at a `TimeDimension` position (the `TimeDataType` subset):
    /// the 17 time-period values.
    #[must_use]
    pub const fn is_time(self) -> bool {
        matches!(
            self,
            Self::ObservationalTimePeriod
                | Self::StandardTimePeriod
                | Self::BasicTimePeriod
                | Self::GregorianTimePeriod
                | Self::GregorianYear
                | Self::GregorianYearMonth
                | Self::GregorianDay
                | Self::ReportingTimePeriod
                | Self::ReportingYear
                | Self::ReportingSemester
                | Self::ReportingTrimester
                | Self::ReportingQuarter
                | Self::ReportingMonth
                | Self::ReportingWeek
                | Self::ReportingDay
                | Self::DateTime
                | Self::TimeRange
        )
    }
}

// ---------------------------------------------------------------------------
// MaxOccurs
// ---------------------------------------------------------------------------

/// A representation-level `maxOccurs`: a finite count or the literal `unbounded`.
///
/// ## Specification
/// - **Type**: `OccurenceType`
/// - **Element**: N/A (Simple Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/OccurenceType.md"))]
///
/// The spec's `OccurenceType` is a number or the literal `"unbounded"`. A `u32` cannot hold the
/// literal, so it gets its own [`Unbounded`](Self::Unbounded) arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MaxOccurs {
    /// A finite upper bound (`xs:nonNegativeInteger`).
    Count(u32),
    /// The literal `"unbounded"`: no upper bound.
    Unbounded,
}

// ---------------------------------------------------------------------------
// TextFormat
// ---------------------------------------------------------------------------

/// The uncoded facet bundle of a representation (`TextFormatType`).
///
/// ## Specification
/// - **Type**: `TextFormatType`
/// - **Element**: `<TextFormat>`
/// - **Editions**: SDMX 3.0 and 3.1 (Divergent)
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/TextFormatType.3.0.md"))]
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/TextFormatType.3.1.md"))]
#[cfg_attr(design_docs, doc = "")]
///
/// The superset of the spec's `TextFormat` tier chain. Every facet is optional, so the type mirrors
/// the wire one-to-one; which facets and `textType`s a given component position permits is the
/// component constructor's check.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`TextFormatType` diverges across editions: the `isMultiLingual` default flips (3.0 `true`, 3.1
`false`), so `is_multi_lingual` is `Option<bool>` (statedness stored, D-0046/D-0052) and the
effective value is a version-aware view supplied at the component level, never baked into the store.
Numeric facets are `xs:decimal` (lossless `SdmxDecimal`, D-0027); time facets are
`StandardTimePeriodType` (`SdmxTimePeriod`). `time_interval` is an `xs:duration` lexical form whose
grammar is deferred to the parser.

Decisions: D-0027, D-0046, D-0048, D-0052.
"#
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TextFormat {
    /// The data type of the values (`textType`); `None` ⟺ absent, the default applied as a
    /// position-aware effective view.
    pub text_type: Option<DataType>,
    /// `isSequence`: whether the values form a sequence.
    pub is_sequence: Option<bool>,
    /// `interval`: the increment between sequence values.
    pub interval: Option<SdmxDecimal>,
    /// `startValue`: the first value of a sequence.
    pub start_value: Option<SdmxDecimal>,
    /// `endValue`: the last value of a sequence.
    pub end_value: Option<SdmxDecimal>,
    /// `timeInterval`: an `xs:duration` increment (lexical form; grammar deferred to the parser).
    pub time_interval: Option<String>,
    /// `startTime`: the first value of a time sequence.
    pub start_time: Option<SdmxTimePeriod>,
    /// `endTime`: the last value of a time sequence.
    pub end_time: Option<SdmxTimePeriod>,
    /// `minLength`: the minimum value length (`xs:positiveInteger`).
    pub min_length: Option<u32>,
    /// `maxLength`: the maximum value length.
    pub max_length: Option<u32>,
    /// `minValue`: the inclusive lower bound.
    pub min_value: Option<SdmxDecimal>,
    /// `maxValue`: the inclusive upper bound.
    pub max_value: Option<SdmxDecimal>,
    /// `decimals`: the number of fractional digits.
    pub decimals: Option<u32>,
    /// `pattern`: a regular expression the values must match.
    pub pattern: Option<String>,
    /// `isMultiLingual`: whether the values are localised. `None` ⟺ absent; the default flips
    /// between editions, so the effective value is a version-aware view.
    pub is_multi_lingual: Option<bool>,
}

// ---------------------------------------------------------------------------
// EnumerationFormat
// ---------------------------------------------------------------------------

/// The facet bundle refining a coded representation (`CodedTextFormatType`).
///
/// ## Specification
/// - **Type**: `CodedTextFormatType`
/// - **Element**: `<EnumerationFormat>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/CodedTextFormatType.md"))]
///
/// A near-subset of [`TextFormat`] applied to an enumeration: numeric facets are `xs:integer`
/// (so a coded interval cannot be fractional), `decimals` is prohibited, and `isMultiLingual` is
/// prohibited. The `textType` is restricted to the Code subset
/// ([`DataType::is_code`](DataType::is_code)), enforced at the component constructor.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Numeric facets are `xs:integer` → `SdmxInteger` (D-0027): a coded interval cannot be fractional.
The `textType` re-declares without a schema default (the restriction replaces the base declaration),
so absent means "unrestricted" and no effective-view default applies here (report-5 V-8). `pattern`
is NOT the coded type's distinguishing mark (it exists uncoded too, D-0048); the real coded deltas
are integer numerics, no `decimals`, and the Code `textType` subset.

Decisions: D-0027, D-0048.
"#
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EnumerationFormat {
    /// The data type of the values (`textType`), restricted to the Code subset. No schema default:
    /// `None` means unrestricted.
    pub text_type: Option<DataType>,
    /// `isSequence`: whether the values form a sequence.
    pub is_sequence: Option<bool>,
    /// `interval`: the integer increment between sequence values.
    pub interval: Option<SdmxInteger>,
    /// `startValue`: the first value of a sequence.
    pub start_value: Option<SdmxInteger>,
    /// `endValue`: the last value of a sequence.
    pub end_value: Option<SdmxInteger>,
    /// `timeInterval`: an `xs:duration` increment (lexical form; grammar deferred to the parser).
    pub time_interval: Option<String>,
    /// `startTime`: the first value of a time sequence.
    pub start_time: Option<SdmxTimePeriod>,
    /// `endTime`: the last value of a time sequence.
    pub end_time: Option<SdmxTimePeriod>,
    /// `minLength`: the minimum value length.
    pub min_length: Option<u32>,
    /// `maxLength`: the maximum value length.
    pub max_length: Option<u32>,
    /// `minValue`: the inclusive integer lower bound.
    pub min_value: Option<SdmxInteger>,
    /// `maxValue`: the inclusive integer upper bound.
    pub max_value: Option<SdmxInteger>,
    /// `pattern`: a regular expression the values must match.
    pub pattern: Option<String>,
}

// ---------------------------------------------------------------------------
// EnumerationReference, RepresentationChoice, Representation
// ---------------------------------------------------------------------------

/// The target of an enumerated representation: a codelist or a value list.
///
/// ## Specification
/// - **Type**: `AnyCodelistReferenceType`
/// - **Element**: `<Enumeration>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/AnyCodelistReferenceType.md"))]
///
/// The base representation admits either reference; a dimension position narrows this to
/// codelist-only, enforced at the dimension constructor.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EnumerationReference {
    /// A reference to a codelist.
    Codelist(CodelistReference),
    /// A reference to a value list (admitted at concept, attribute, and measure positions).
    ValueList(ValueListReference),
}

/// The representation's choice: an enumeration (with an optional format) or an uncoded text format.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Projects the spec's `RepresentationType` choice into a Rust enum. Exhaustive: exactly these two
/// arms.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RepresentationChoice {
    /// An enumerated representation: a reference to a codelist or value list, optionally refined by
    /// an [`EnumerationFormat`].
    Enumeration {
        /// The codelist or value list the values are drawn from.
        enumeration: EnumerationReference,
        /// The optional facet refinement.
        format: Option<EnumerationFormat>,
    },
    /// An uncoded representation: a bundle of [`TextFormat`] facets.
    TextFormat(TextFormat),
}

/// How a component is typed and valued (`RepresentationType`).
///
/// ## Specification
/// - **Type**: `RepresentationType`
/// - **Element**: N/A (Base Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/RepresentationType.md"))]
///
/// A choice (enumeration or text format) plus the `minOccurs`/`maxOccurs` on the representation
/// node. Both occurrence attributes store statedness; their schema defaults are position-aware
/// effective views applied at the component level.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Representation {
    /// The enumeration-or-text-format choice.
    pub choice: RepresentationChoice,
    /// `minOccurs` (`xs:nonNegativeInteger`, schema default 1); `None` ⟺ absent.
    pub min_occurs: Option<u32>,
    /// `maxOccurs`; `None` ⟺ absent, the default position-dependent.
    pub max_occurs: Option<MaxOccurs>,
}

// ---------------------------------------------------------------------------
// Position-rule validators
// ---------------------------------------------------------------------------

/// Validates a representation against the Basic-tier position rules: a concept core representation,
/// an attribute, or a measure. The Basic position admits either enumeration target
/// (codelist or value list) and the full Basic `textType` subset; a refining
/// [`EnumerationFormat`] is held to the Code subset. No facet is prohibited at this tier.
///
/// `component` names the component kind for the diagnostic (for example `"Concept"`). A `None`
/// representation is always valid: the component inherits its representation from its concept.
///
/// # Errors
///
/// Returns [`Error::InvalidTextTypeForComponent`] if a stated `textType` falls outside the subset
/// its position allows: the Basic subset for an uncoded [`TextFormat`], the Code subset for an
/// enumeration's [`EnumerationFormat`].
pub fn validate_basic_representation(
    component: &str,
    representation: Option<&Representation>,
) -> Result<(), Error> {
    let Some(representation) = representation else {
        return Ok(());
    };
    match &representation.choice {
        RepresentationChoice::TextFormat(text_format) => {
            if let Some(text_type) = text_format.text_type
                && !text_type.is_basic()
            {
                return Err(invalid_text_type(component, text_type));
            }
        }
        // Both enumeration targets are admitted at the Basic position, so the reference itself is
        // unchecked here. Only the optional format's textType is restricted (to the Code subset).
        RepresentationChoice::Enumeration { format, .. } => {
            if let Some(format) = format
                && let Some(text_type) = format.text_type
                && !text_type.is_code()
            {
                return Err(invalid_text_type(component, text_type));
            }
        }
    }
    Ok(())
}

/// Builds an [`Error::InvalidTextTypeForComponent`], rendering the `textType` via its `Debug`
/// token, which is identical to the wire enumeration value (the variants reproduce the spec
/// tokens verbatim).
fn invalid_text_type(component: &str, text_type: DataType) -> Error {
    Error::InvalidTextTypeForComponent {
        component: String::from(component),
        text_type: format!("{text_type:?}"),
    }
}

/// Builds an [`Error::ProhibitedRepresentationFacet`] naming the component kind and the facet.
fn prohibited_facet(component: &str, facet: &str) -> Error {
    Error::ProhibitedRepresentationFacet {
        component: String::from(component),
        facet: String::from(facet),
    }
}

/// Validates a representation against the Dimension position rules. A dimension
/// admits a codelist enumeration only (no value list), holds an uncoded `textType` to the `Simple`
/// subset and a refining [`EnumerationFormat`] to the `Code` subset, and prohibits `isMultiLingual`
/// and a representation-level `minOccurs`/`maxOccurs`. A `None` representation is always valid: the
/// dimension inherits its representation from its concept.
///
/// # Errors
///
/// Returns [`Error::ValueListEnumerationNotAllowed`] for a value-list enumeration,
/// [`Error::ProhibitedRepresentationFacet`] for `isMultiLingual` or a representation-level
/// `minOccurs`/`maxOccurs`, or [`Error::InvalidTextTypeForComponent`] for a `textType` outside the
/// subset its position allows (`Simple` for an uncoded text format, `Code` for an enumeration
/// format).
pub fn validate_dimension_representation(
    representation: Option<&Representation>,
) -> Result<(), Error> {
    let Some(representation) = representation else {
        return Ok(());
    };
    // The representation-level `minOccurs`/`maxOccurs` live on the representation node, so they are
    // prohibited regardless of which choice arm follows.
    if representation.min_occurs.is_some() {
        return Err(prohibited_facet("Dimension", "minOccurs"));
    }
    if representation.max_occurs.is_some() {
        return Err(prohibited_facet("Dimension", "maxOccurs"));
    }
    match &representation.choice {
        RepresentationChoice::TextFormat(text_format) => {
            if let Some(text_type) = text_format.text_type
                && !text_type.is_simple()
            {
                return Err(invalid_text_type("Dimension", text_type));
            }
            if text_format.is_multi_lingual.is_some() {
                return Err(prohibited_facet("Dimension", "isMultiLingual"));
            }
        }
        RepresentationChoice::Enumeration { enumeration, format } => {
            if matches!(enumeration, EnumerationReference::ValueList(_)) {
                return Err(Error::ValueListEnumerationNotAllowed(String::from("Dimension")));
            }
            if let Some(format) = format
                && let Some(text_type) = format.text_type
                && !text_type.is_code()
            {
                return Err(invalid_text_type("Dimension", text_type));
            }
        }
    }
    Ok(())
}

/// Validates a representation against the `TimeDimension` position rules. The time
/// position is `TextFormat`-only (no enumeration), restricts the `textType` to the `Time` subset,
/// permits only the `textType`, `startTime`, and `endTime` facets, and prohibits a
/// representation-level `minOccurs`/`maxOccurs`; every other facet is prohibited. A time dimension's
/// representation is mandatory, so it is taken by reference.
///
/// # Errors
///
/// Returns [`Error::EnumerationNotAllowed`] for an enumeration,
/// [`Error::InvalidTextTypeForComponent`] for a `textType` outside the `Time` subset, or
/// [`Error::ProhibitedRepresentationFacet`] for any facet other than `textType`, `startTime`, and
/// `endTime`, or a representation-level `minOccurs`/`maxOccurs`.
pub fn validate_time_representation(representation: &Representation) -> Result<(), Error> {
    // The representation-level `minOccurs`/`maxOccurs` live on the representation node and are
    // prohibited at the time position (`TimeDimensionRepresentationType` restricts both away).
    if representation.min_occurs.is_some() {
        return Err(prohibited_facet("TimeDimension", "minOccurs"));
    }
    if representation.max_occurs.is_some() {
        return Err(prohibited_facet("TimeDimension", "maxOccurs"));
    }
    let text_format = match &representation.choice {
        RepresentationChoice::TextFormat(text_format) => text_format,
        RepresentationChoice::Enumeration { .. } => {
            return Err(Error::EnumerationNotAllowed(String::from("TimeDimension")));
        }
    };
    if let Some(text_type) = text_format.text_type
        && !text_type.is_time()
    {
        return Err(invalid_text_type("TimeDimension", text_type));
    }
    // Only `textType`, `startTime`, and `endTime` may be set at the time position; flag the first
    // other facet that is present.
    let prohibited: [(bool, &str); 12] = [
        (text_format.is_sequence.is_some(), "isSequence"),
        (text_format.interval.is_some(), "interval"),
        (text_format.start_value.is_some(), "startValue"),
        (text_format.end_value.is_some(), "endValue"),
        (text_format.time_interval.is_some(), "timeInterval"),
        (text_format.min_length.is_some(), "minLength"),
        (text_format.max_length.is_some(), "maxLength"),
        (text_format.min_value.is_some(), "minValue"),
        (text_format.max_value.is_some(), "maxValue"),
        (text_format.decimals.is_some(), "decimals"),
        (text_format.pattern.is_some(), "pattern"),
        (text_format.is_multi_lingual.is_some(), "isMultiLingual"),
    ];
    for (is_set, facet) in prohibited {
        if is_set {
            return Err(prohibited_facet("TimeDimension", facet));
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn datatype_basic_excludes_only_reference_and_key_types() {
        assert!(DataType::String.is_basic());
        assert!(DataType::XHTML.is_basic());
        assert!(!DataType::KeyValues.is_basic());
        assert!(!DataType::IdentifiableReference.is_basic());
        assert!(!DataType::DataSetReference.is_basic());
    }

    #[test]
    fn datatype_code_excludes_uncoded_only_types() {
        assert!(DataType::String.is_code());
        assert!(DataType::Count.is_code());
        // Uncoded-only types are outside the Code subset.
        for excluded in [
            DataType::XHTML,
            DataType::DateTime,
            DataType::Decimal,
            DataType::Double,
            DataType::Float,
            DataType::GeospatialInformation,
            DataType::Time,
            DataType::TimeRange,
        ] {
            assert!(!excluded.is_code(), "{excluded:?} must not be in the Code subset");
        }
    }

    #[test]
    fn datatype_round_trips() {
        // The projection is internal (D-0068); postcard encodes variants positionally, so no test
        // pins field/variant names or the spec-token wire form (a future rename changes the
        // disclaimed projection, not the wire) — token fidelity is the writer crates' concern. This
        // asserts only that the round-trip is lossless, not the shape.
        crate::test_support::round_trip(&DataType::URI);
        crate::test_support::round_trip(&DataType::XHTML);
        crate::test_support::round_trip(&DataType::ObservationalTimePeriod);
    }

    fn text_format(text_type: Option<DataType>) -> Representation {
        Representation {
            choice: RepresentationChoice::TextFormat(TextFormat {
                text_type,
                is_sequence: None,
                interval: None,
                start_value: None,
                end_value: None,
                time_interval: None,
                start_time: None,
                end_time: None,
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                decimals: None,
                pattern: None,
                is_multi_lingual: None,
            }),
            min_occurs: None,
            max_occurs: None,
        }
    }

    #[test]
    fn basic_representation_accepts_none_and_basic_text_type() {
        assert!(validate_basic_representation("Concept", None).is_ok());
        assert!(validate_basic_representation("Concept", Some(&text_format(None))).is_ok());
        assert!(
            validate_basic_representation("Concept", Some(&text_format(Some(DataType::String))))
                .is_ok()
        );
    }

    #[test]
    fn basic_representation_rejects_non_basic_text_type() {
        let repr = text_format(Some(DataType::KeyValues));
        assert_eq!(
            validate_basic_representation("Concept", Some(&repr)),
            Err(Error::InvalidTextTypeForComponent {
                component: "Concept".into(),
                text_type: "KeyValues".into()
            })
        );
    }

    #[test]
    fn basic_representation_rejects_non_code_enumeration_format() {
        // An enumeration's optional format textType is restricted to the Code subset (tighter than
        // the TextFormat arm's Basic subset), so a basic-but-not-coded type is rejected even though
        // the enumeration reference itself is admitted at the Basic position.
        let repr = Representation {
            choice: RepresentationChoice::Enumeration {
                enumeration: EnumerationReference::Codelist(CodelistReference {
                    agency: "SDMX".into(),
                    id: "CL_FREQ".into(),
                    version: "1.0.0".parse().unwrap(),
                }),
                format: Some(EnumerationFormat {
                    text_type: Some(DataType::Double), // basic, but outside the Code subset
                    ..EnumerationFormat::default()
                }),
            },
            min_occurs: None,
            max_occurs: None,
        };
        assert!(matches!(
            validate_basic_representation("Concept", Some(&repr)),
            Err(Error::InvalidTextTypeForComponent { .. })
        ));
    }

    #[test]
    fn basic_representation_holds_enumeration_format_to_code_subset() {
        let codelist = CodelistReference {
            agency: "SDMX".into(),
            id: "CL_FREQ".into(),
            version: "1.0.0".parse().unwrap(),
        };
        let format = EnumerationFormat {
            text_type: Some(DataType::XHTML), // XHTML is not in the Code subset
            is_sequence: None,
            interval: None,
            start_value: None,
            end_value: None,
            time_interval: None,
            start_time: None,
            end_time: None,
            min_length: None,
            max_length: None,
            min_value: None,
            max_value: None,
            pattern: None,
        };
        let repr = Representation {
            choice: RepresentationChoice::Enumeration {
                enumeration: EnumerationReference::Codelist(codelist),
                format: Some(format),
            },
            min_occurs: None,
            max_occurs: None,
        };
        assert_eq!(
            validate_basic_representation("Concept", Some(&repr)),
            Err(Error::InvalidTextTypeForComponent {
                component: "Concept".into(),
                text_type: "XHTML".into()
            })
        );
    }

    #[test]
    fn datatype_simple_excludes_only_xhtml_from_basic() {
        assert!(DataType::String.is_simple());
        // XHTML is Basic but not Simple; the Basic exclusions stay out too.
        assert!(!DataType::XHTML.is_simple());
        assert!(!DataType::KeyValues.is_simple());
    }

    #[test]
    fn datatype_time_is_the_seventeen_time_values() {
        // The full TimeDataType subset, pinned value by value (17 of 44).
        let time = [
            DataType::ObservationalTimePeriod,
            DataType::StandardTimePeriod,
            DataType::BasicTimePeriod,
            DataType::GregorianTimePeriod,
            DataType::GregorianYear,
            DataType::GregorianYearMonth,
            DataType::GregorianDay,
            DataType::ReportingTimePeriod,
            DataType::ReportingYear,
            DataType::ReportingSemester,
            DataType::ReportingTrimester,
            DataType::ReportingQuarter,
            DataType::ReportingMonth,
            DataType::ReportingWeek,
            DataType::ReportingDay,
            DataType::DateTime,
            DataType::TimeRange,
        ];
        assert_eq!(time.len(), 17);
        for value in time {
            assert!(value.is_time(), "{value:?} must be in the Time subset");
        }
        // Gregorian-adjacent values outside TimeDataType, and a plain string, are not time values.
        for non_time in
            [DataType::String, DataType::Month, DataType::Day, DataType::Time, DataType::Duration]
        {
            assert!(!non_time.is_time(), "{non_time:?} must not be in the Time subset");
        }
    }

    fn codelist_enumeration() -> Representation {
        Representation {
            choice: RepresentationChoice::Enumeration {
                enumeration: EnumerationReference::Codelist(CodelistReference {
                    agency: "SDMX".into(),
                    id: "CL_FREQ".into(),
                    version: "1.0.0".parse().unwrap(),
                }),
                format: None,
            },
            min_occurs: None,
            max_occurs: None,
        }
    }

    #[test]
    fn dimension_representation_accepts_none_codelist_and_simple_text_type() {
        assert!(validate_dimension_representation(None).is_ok());
        assert!(validate_dimension_representation(Some(&codelist_enumeration())).is_ok());
        assert!(
            validate_dimension_representation(Some(&text_format(Some(DataType::String)))).is_ok()
        );
    }

    #[test]
    fn dimension_representation_rejects_value_list_enumeration() {
        let repr = Representation {
            choice: RepresentationChoice::Enumeration {
                enumeration: EnumerationReference::ValueList(ValueListReference {
                    agency: "SDMX".into(),
                    id: "VL_CURRENCY".into(),
                    version: "1.0.0".parse().unwrap(),
                }),
                format: None,
            },
            min_occurs: None,
            max_occurs: None,
        };
        assert_eq!(
            validate_dimension_representation(Some(&repr)),
            Err(Error::ValueListEnumerationNotAllowed("Dimension".into()))
        );
    }

    #[test]
    fn dimension_representation_holds_enumeration_format_to_code_subset() {
        // A codelist enumeration is fine, but its refining format's textType is held to the Code
        // subset (DateTime is a Simple-but-not-Code type).
        let repr = Representation {
            choice: RepresentationChoice::Enumeration {
                enumeration: EnumerationReference::Codelist(CodelistReference {
                    agency: "SDMX".into(),
                    id: "CL_FREQ".into(),
                    version: "1.0.0".parse().unwrap(),
                }),
                format: Some(EnumerationFormat {
                    text_type: Some(DataType::DateTime),
                    is_sequence: None,
                    interval: None,
                    start_value: None,
                    end_value: None,
                    time_interval: None,
                    start_time: None,
                    end_time: None,
                    min_length: None,
                    max_length: None,
                    min_value: None,
                    max_value: None,
                    pattern: None,
                }),
            },
            min_occurs: None,
            max_occurs: None,
        };
        assert_eq!(
            validate_dimension_representation(Some(&repr)),
            Err(Error::InvalidTextTypeForComponent {
                component: "Dimension".into(),
                text_type: "DateTime".into()
            })
        );
    }

    #[test]
    fn dimension_representation_rejects_non_simple_text_type() {
        // XHTML is admitted at the Basic position but not at a dimension's Simple position.
        assert_eq!(
            validate_dimension_representation(Some(&text_format(Some(DataType::XHTML)))),
            Err(Error::InvalidTextTypeForComponent {
                component: "Dimension".into(),
                text_type: "XHTML".into()
            })
        );
    }

    #[test]
    fn dimension_representation_rejects_prohibited_facets() {
        // isMultiLingual is prohibited at the Simple position.
        let mut multi_lingual = text_format(None);
        if let RepresentationChoice::TextFormat(text_format) = &mut multi_lingual.choice {
            text_format.is_multi_lingual = Some(true);
        }
        assert_eq!(
            validate_dimension_representation(Some(&multi_lingual)),
            Err(Error::ProhibitedRepresentationFacet {
                component: "Dimension".into(),
                facet: "isMultiLingual".into()
            })
        );

        // A representation-level minOccurs or maxOccurs is prohibited regardless of the choice arm.
        let mut min_bounded = codelist_enumeration();
        min_bounded.min_occurs = Some(1);
        assert_eq!(
            validate_dimension_representation(Some(&min_bounded)),
            Err(Error::ProhibitedRepresentationFacet {
                component: "Dimension".into(),
                facet: "minOccurs".into()
            })
        );
        let mut bounded = codelist_enumeration();
        bounded.max_occurs = Some(MaxOccurs::Unbounded);
        assert_eq!(
            validate_dimension_representation(Some(&bounded)),
            Err(Error::ProhibitedRepresentationFacet {
                component: "Dimension".into(),
                facet: "maxOccurs".into()
            })
        );
    }

    #[test]
    fn time_representation_accepts_a_time_text_type() {
        assert!(
            validate_time_representation(&text_format(Some(DataType::ObservationalTimePeriod)))
                .is_ok()
        );
    }

    #[test]
    fn time_representation_accepts_start_and_end_time() {
        // startTime and endTime are the only facets besides textType the time position permits, so
        // a representation that sets them must be accepted (the "no more than the rule" side).
        let mut repr = text_format(Some(DataType::ObservationalTimePeriod));
        if let RepresentationChoice::TextFormat(text_format) = &mut repr.choice {
            text_format.start_time =
                Some(SdmxTimePeriod::new("2024-05-01T09:30:00".into()).unwrap());
            text_format.end_time = Some(SdmxTimePeriod::new("2024-05-01T09:30:00".into()).unwrap());
        }
        assert!(validate_time_representation(&repr).is_ok());
    }

    #[test]
    fn time_representation_rejects_node_level_occurs() {
        // minOccurs and maxOccurs live on the representation node (not the TextFormat) and are both
        // prohibited at the time position.
        let mut min_bounded = text_format(Some(DataType::ObservationalTimePeriod));
        min_bounded.min_occurs = Some(1);
        assert_eq!(
            validate_time_representation(&min_bounded),
            Err(Error::ProhibitedRepresentationFacet {
                component: "TimeDimension".into(),
                facet: "minOccurs".into()
            })
        );
        let mut max_bounded = text_format(Some(DataType::ObservationalTimePeriod));
        max_bounded.max_occurs = Some(MaxOccurs::Unbounded);
        assert_eq!(
            validate_time_representation(&max_bounded),
            Err(Error::ProhibitedRepresentationFacet {
                component: "TimeDimension".into(),
                facet: "maxOccurs".into()
            })
        );
    }

    #[test]
    fn time_representation_rejects_enumeration() {
        assert_eq!(
            validate_time_representation(&codelist_enumeration()),
            Err(Error::EnumerationNotAllowed("TimeDimension".into()))
        );
    }

    #[test]
    fn time_representation_rejects_non_time_text_type() {
        assert_eq!(
            validate_time_representation(&text_format(Some(DataType::String))),
            Err(Error::InvalidTextTypeForComponent {
                component: "TimeDimension".into(),
                text_type: "String".into()
            })
        );
    }

    #[test]
    fn time_representation_rejects_a_prohibited_facet() {
        let mut repr = text_format(Some(DataType::ObservationalTimePeriod));
        if let RepresentationChoice::TextFormat(text_format) = &mut repr.choice {
            text_format.pattern = Some("[0-9]+".into());
        }
        assert_eq!(
            validate_time_representation(&repr),
            Err(Error::ProhibitedRepresentationFacet {
                component: "TimeDimension".into(),
                facet: "pattern".into()
            })
        );
    }

    // Property tests: the internal serde round-trip over position-valid generated
    // representations (see `test_strategy`); wasm32 is excluded with the rest of the
    // property suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::{
            basic_representation, dimension_representation, time_representation,
        };

        proptest! {
            #[test]
            fn basic_representation_round_trips(value in basic_representation()) {
                crate::test_support::round_trip(&value);
            }

            #[test]
            fn dimension_representation_round_trips(value in dimension_representation()) {
                crate::test_support::round_trip(&value);
            }

            #[test]
            fn time_representation_round_trips(value in time_representation()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
