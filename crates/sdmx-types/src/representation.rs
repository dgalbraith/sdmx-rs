//! Component representation: how a component is typed and valued.
//!
//! A representation is either an *enumeration* (a reference to a [`Codelist`](crate::Codelist) or
//! [`ValueList`](crate::ValueList), optionally refined by an [`EnumerationFormat`]) or an
//! *uncoded* [`TextFormat`] (a bundle of facets), plus the `minOccurs`/`maxOccurs` on the
//! representation node. The single stored [`Representation`] is the superset of every position's
//! shape; which arms, facets, and `textType`s a given component position admits is enforced at the
//! component constructors, not by this type. This milestone provides the Basic-position validator
//! ([`validate_basic_representation`]) for a concept's core representation; the dimension- and
//! time-position validators arrive with their callers.
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
functions the component constructors call. `DataType`'s subset predicates (`is_basic`, `is_code`)
are Layer-2 views; the `is_simple`/`is_time` views join with the dimension/time validators.

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
/// ([`is_basic`](Self::is_basic), [`is_code`](Self::is_code)) are the Layer-2 views the component
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

    /// `true` if this value is admitted at a coded position (the `textType` subset of
    /// `CodedTextFormatType`): the Basic set, further minus the uncoded-only types
    /// (`XHTML`, `DateTime`, `Decimal`, `Double`, `Float`, `GeospatialInformation`, `Time`,
    /// `TimeRange`).
    #[must_use]
    pub const fn is_code(self) -> bool {
        // Expressed flat on purpose. The spec's tiers are a restriction chain
        // (Basic ⊃ Simple ⊃ Code), so the faithful form is `is_simple() && !matches!(…7…)`; but
        // `is_simple` is deferred until its first caller, the dimension/time validators, lands at
        // M3 (build-at-first-caller). Until then this lists the full exclusion (the three
        // reference/key types from Basic, plus `XHTML` from Simple, plus the seven uncoded-only
        // types). Refactor to the composed chain when `is_simple` is introduced.
        !matches!(
            self,
            Self::KeyValues
                | Self::IdentifiableReference
                | Self::DataSetReference
                | Self::XHTML
                | Self::DateTime
                | Self::Decimal
                | Self::Double
                | Self::Float
                | Self::GeospatialInformation
                | Self::Time
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    /// between editions, so the effective value is a version-aware view (D-0046).
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
/// an attribute, or a measure (§5.6.1). The Basic position admits either enumeration target
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
    Error::InvalidTextTypeForComponent(String::from(component), format!("{text_type:?}"))
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
    fn datatype_serialises_to_spec_token() {
        // The wire value is the verbatim spec token, including the all-caps acronyms.
        assert_eq!(serde_json::to_string(&DataType::URI).unwrap(), "\"URI\"");
        assert_eq!(serde_json::to_string(&DataType::XHTML).unwrap(), "\"XHTML\"");
        assert_eq!(
            serde_json::to_string(&DataType::ObservationalTimePeriod).unwrap(),
            "\"ObservationalTimePeriod\""
        );
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
            Err(Error::InvalidTextTypeForComponent("Concept".into(), "KeyValues".into()))
        );
    }

    #[test]
    fn basic_representation_holds_enumeration_format_to_code_subset() {
        let codelist = CodelistReference {
            agency: "SDMX".into(),
            id: "CL_FREQ".into(),
            version: "1.0.0".into(),
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
            Err(Error::InvalidTextTypeForComponent("Concept".into(), "XHTML".into()))
        );
    }
}
