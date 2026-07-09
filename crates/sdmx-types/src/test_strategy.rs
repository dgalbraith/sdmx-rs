//! Shared `proptest` strategies, compiled only under `cfg(test)` and never for wasm32
//! (the property suite verifies platform-independent invariants; see `docs/dev/testing.md`).
//!
//! Every strategy here emits a *grammar-valid lexeme* (or the components one is formatted
//! from), and the property owning it constructs the value through the type's validated
//! `new()`/`from_str()` — the same single write path production code uses, so the generated
//! set is exactly the legal set. A generator that bypassed the constructor would have the
//! same defect as a `Deserialize` that did (design 0010 §7). Strategies for deliberately off-grammar
//! input are the clearly named `invalid_*` family at the end of the module.

// Strategies are `pub(crate)` so property tests in sibling modules can compose them; the
// enclosing module is private, which makes clippy's nursery `redundant_pub_crate` fire, but
// the crate-scoped visibility is the intent (the same allowance as `test_support`).
// `expect_used` is allowed as in `test_support`: a strategy that emits an invalid lexeme is
// a generator bug, and panicking there is the correct failure mode for test code.
#![allow(clippy::redundant_pub_crate, clippy::expect_used)]

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::num::NonZeroU32;

use proptest::prelude::*;

use crate::{
    Agency, AgencyScheme, Annotation, AnnotationUrl, Attribute, AttributeList, AttributeListMember,
    AttributeRelationship, AvailabilityConstraint, AvailabilityConstraintAttachment, Cascade, Code,
    CodeSelection, Codelist, CodelistExtension, CodelistReference, ComponentMetadata,
    ComponentSelection, ComponentValueSet, Concept, ConceptReference, ConceptScheme,
    ConstraintModel, ConstraintRole, Contact, ContactDetail, CubeKeyValue, CubeKeyValues,
    CubeRegion, CubeRegionKey, CubeRegions, DataComponentSelection, DataComponentValue,
    DataComponentValueSet, DataComponentValues, DataConstraint, DataConstraintAttachment, DataKey,
    DataKeySet, DataKeyValue, DataKeys, DataProviderReference, DataStructureDefinition,
    DataStructureReference, DataStructureRefs, DataType, Dataflow, DataflowReference, DataflowRefs,
    Dimension, DimensionConstraint, DimensionList, DimensionRef, EnumerationFormat,
    EnumerationReference, FixedInclude, Group, GroupDimensions, IdentifiableMetadata,
    IsoConceptReference, KeyValueSelection, Link, LocalisedString, LocalisedText,
    MaintainableMetadata, MaxOccurs, Measure, MeasureList, MeasureRelationship, MemberValue,
    MemberValues, MetadataAttributeUsage, NameableMetadata, ObservationalTimePeriod,
    ProvisionAgreementReference, ProvisionAgreementRefs, QueryableDataSource, ReleaseCalendar,
    Representation, RepresentationChoice, SdmxDateTime, SdmxDecimal, SdmxDuration, SdmxInteger,
    SdmxTimePeriod, SdmxVersion, SimpleComponentValue, SimpleComponentValues, SimpleDataSources,
    SimpleKeyValues, TextFormat, TimeDimension, TimePeriodRange, TimeRange, TimeRangeKind, Usage,
    ValueItem, ValueList, ValueListReference, VersionRef, VersionableMetadata,
};

// ---------------------------------------------------------------------------
// xs:decimal / xs:integer (raw-backed, D-0027)
// ---------------------------------------------------------------------------

/// A valid `xs:decimal` lexeme: optional sign, decimal digits with at most one point, at
/// least one digit, no exponent. Covers the integer, point, leading-dot, and trailing-dot
/// shapes.
pub(crate) fn xs_decimal_lexeme() -> impl Strategy<Value = String> {
    prop_oneof![r"[+-]?[0-9]{1,18}", r"[+-]?[0-9]{0,9}\.[0-9]{1,9}", r"[+-]?[0-9]{1,9}\.",]
}

/// A valid `xs:integer` lexeme: optional sign followed by decimal digits only.
pub(crate) fn xs_integer_lexeme() -> impl Strategy<Value = String> {
    r"[+-]?[0-9]{1,18}"
}

// ---------------------------------------------------------------------------
// VersionType / WildcardVersionType (raw-free, D-0070/D-0071)
// ---------------------------------------------------------------------------

/// A single `SemVer` prerelease identifier: numeric (no leading zero) or alphanumeric over
/// `[A-Za-z0-9-]` with at least one non-digit (guaranteed by the mandatory non-digit char).
fn extension_identifier() -> impl Strategy<Value = String> {
    prop_oneof![any::<u32>().prop_map(|n| n.to_string()), r"[0-9]{0,3}[A-Za-z-][0-9A-Za-z-]{0,4}",]
}

/// A prerelease extension: one to three dot-separated identifiers.
fn version_extension() -> impl Strategy<Value = String> {
    proptest::collection::vec(extension_identifier(), 1..=3).prop_map(|ids| ids.join("."))
}

/// A valid `VersionType` lexeme: the legacy (`major[.minor]`) and semantic
/// (`major.minor.patch[-extension]`) forms. `u32` formatting yields `0|[1-9]\d*` components
/// by construction.
pub(crate) fn version_lexeme() -> impl Strategy<Value = String> {
    prop_oneof![
        any::<u32>().prop_map(|major| major.to_string()),
        (any::<u32>(), any::<u32>()).prop_map(|(major, minor)| format!("{major}.{minor}")),
        (any::<u32>(), any::<u32>(), any::<u32>())
            .prop_map(|(major, minor, patch)| format!("{major}.{minor}.{patch}")),
        (any::<u32>(), any::<u32>(), any::<u32>(), version_extension())
            .prop_map(|(major, minor, patch, ext)| format!("{major}.{minor}.{patch}-{ext}")),
    ]
}

/// A "latest available" reference lexeme: a full semantic triple with `+` on exactly one
/// component and no extension.
fn latest_version_lexeme() -> impl Strategy<Value = String> {
    (any::<u32>(), any::<u32>(), any::<u32>(), 0_usize..3).prop_map(|(major, minor, patch, at)| {
        let wild = |position: usize| if position == at { "+" } else { "" };
        format!("{major}{}.{minor}{}.{patch}{}", wild(0), wild(1), wild(2))
    })
}

/// A valid `WildcardVersionType` lexeme: an exact version, a `+`-wildcarded triple, or the
/// bare `*`.
pub(crate) fn version_ref_lexeme() -> impl Strategy<Value = String> {
    prop_oneof![version_lexeme(), latest_version_lexeme(), Just(String::from("*"))]
}

/// The version part a structural reference URN admits: an exact version or a `+`-wildcarded
/// triple, never the bare `*` (D-0073).
pub(crate) fn reference_version_lexeme() -> impl Strategy<Value = String> {
    prop_oneof![version_lexeme(), latest_version_lexeme()]
}

// ---------------------------------------------------------------------------
// StandardTimePeriodType / TimeRangeType / ObservationalTimePeriodType (D-0027/D-0072)
// ---------------------------------------------------------------------------

/// A calendar year: four digits, a longer non-zero-leading run, or a BC (`-`-prefixed) year.
/// Gregorian years floor at `0001`: XSD 1.0 prohibits year `0000` across the builtin date/time
/// types, so `is_year` rejects it, and a generator emitting `0000` (or `-0000`) would produce a
/// lexeme the classifier refuses. Reporting-period years keep their own `\d{4}` generator, whose
/// pattern grammar admits `0000`.
fn gregorian_year() -> impl Strategy<Value = String> {
    prop_oneof![
        6 => (1_u32..=9999).prop_map(|y| format!("{y:04}")),
        1 => r"[1-9][0-9]{4}",
        1 => (1_u32..=9999).prop_map(|y| format!("-{y:04}")),
    ]
}

/// A Gregorian core lexeme: `gYear`, `gYearMonth`, or full date (timezone added separately).
fn gregorian_core() -> impl Strategy<Value = String> {
    (gregorian_year(), proptest::option::of((1_u32..=12, proptest::option::of(1_u32..=31))))
        .prop_map(|(year, month_day)| match month_day {
            None => year,
            Some((month, None)) => format!("{year}-{month:02}"),
            Some((month, Some(day))) => format!("{year}-{month:02}-{day:02}"),
        })
}

/// A full `xs:date` core (`YYYY-MM-DD`, BC years included), as `TimeRangeType` starts require.
fn date_core() -> impl Strategy<Value = String> {
    (gregorian_year(), 1_u32..=12, 1_u32..=31)
        .prop_map(|(year, month, day)| format!("{year}-{month:02}-{day:02}"))
}

/// An `xs:dateTime` core: full date, `T`, `hh:mm:ss`, optional fractional seconds.
fn datetime_core() -> impl Strategy<Value = String> {
    (date_core(), 0_u32..=23, 0_u32..=59, 0_u32..=59, proptest::option::of(r"[0-9]{1,6}")).prop_map(
        |(date, hour, minute, second, fraction)| {
            fraction.map_or_else(
                || format!("{date}T{hour:02}:{minute:02}:{second:02}"),
                |fraction| format!("{date}T{hour:02}:{minute:02}:{second:02}.{fraction}"),
            )
        },
    )
}

/// A reporting-period core: a four-digit reporting year and an in-range period designator.
fn reporting_core() -> impl Strategy<Value = String> {
    let period = prop_oneof![
        Just(String::from("A1")),
        (1_u32..=2).prop_map(|n| format!("S{n}")),
        (1_u32..=3).prop_map(|n| format!("T{n}")),
        (1_u32..=4).prop_map(|n| format!("Q{n}")),
        (1_u32..=12).prop_map(|n| format!("M{n:02}")),
        (1_u32..=53).prop_map(|n| format!("W{n:02}")),
        (1_u32..=366).prop_map(|n| format!("D{n:03}")),
    ];
    // The reporting year is exactly four digits (`BaseReportPeriodType`'s `\d{4}`), not the
    // four-or-more of `gregorian_year`: a wider run is not a reporting period (see
    // `classify_reporting` and its disambiguation guard).
    (r"[0-9]{4}", period).prop_map(|(year, period)| format!("{year}-{period}"))
}

/// A timezone suffix: `Z` or `±hh:mm` within the `-14:00..+14:00` window.
fn timezone_lexeme() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::from("Z")),
        (r"[+-]", 0_u32..=13, 0_u32..=59)
            .prop_map(|(sign, hour, minute)| format!("{sign}{hour:02}:{minute:02}")),
        r"[+-]".prop_map(|sign| format!("{sign}14:00")),
    ]
}

/// Appends an optional timezone to a core lexeme (a quarter of generated values carry one).
fn with_optional_timezone(core: impl Strategy<Value = String>) -> impl Strategy<Value = String> {
    (core, proptest::option::weighted(0.25, timezone_lexeme())).prop_map(|(core, timezone)| {
        match timezone {
            None => core,
            Some(timezone) => format!("{core}{timezone}"),
        }
    })
}

/// A valid `StandardTimePeriodType` lexeme: a Gregorian period, an `xs:dateTime`, or a
/// reporting period, each with an optional timezone.
pub(crate) fn standard_time_period_lexeme() -> impl Strategy<Value = String> {
    with_optional_timezone(prop_oneof![gregorian_core(), datetime_core(), reporting_core()])
}

/// Formats an optional duration component as `<n><unit>`, or nothing when absent.
fn duration_component(value: Option<u32>, unit: char) -> String {
    value.map_or_else(String::new, |value| format!("{value}{unit}"))
}

/// The date half of an `xs:duration`: an ordered, non-empty subset of `nY nM nD`.
fn duration_date_part() -> impl Strategy<Value = String> {
    (
        proptest::option::of(0_u32..1_000),
        proptest::option::of(0_u32..1_000),
        proptest::option::of(0_u32..1_000),
    )
        .prop_filter("at least one date component", |(years, months, days)| {
            years.is_some() || months.is_some() || days.is_some()
        })
        .prop_map(|(years, months, days)| {
            format!(
                "{}{}{}",
                duration_component(years, 'Y'),
                duration_component(months, 'M'),
                duration_component(days, 'D')
            )
        })
}

/// The time half of an `xs:duration`: an ordered, non-empty subset of `nH nM n[.n]S`, the
/// fraction admitted only on the seconds component.
fn duration_time_part() -> impl Strategy<Value = String> {
    (
        proptest::option::of(0_u32..1_000),
        proptest::option::of(0_u32..1_000),
        proptest::option::of((0_u32..1_000, proptest::option::of(r"[0-9]{1,4}"))),
    )
        .prop_filter("at least one time component", |(hours, minutes, seconds)| {
            hours.is_some() || minutes.is_some() || seconds.is_some()
        })
        .prop_map(|(hours, minutes, seconds)| {
            let seconds = match seconds {
                None => String::new(),
                Some((value, None)) => format!("{value}S"),
                Some((value, Some(fraction))) => format!("{value}.{fraction}S"),
            };
            format!(
                "{}{}{}",
                duration_component(hours, 'H'),
                duration_component(minutes, 'M'),
                seconds
            )
        })
}

/// A valid `TimeRangeType` duration: `P` plus a date part, a time part, or both.
fn duration_lexeme() -> impl Strategy<Value = String> {
    prop_oneof![
        duration_date_part().prop_map(|date| format!("P{date}")),
        duration_time_part().prop_map(|time| format!("PT{time}")),
        (duration_date_part(), duration_time_part())
            .prop_map(|(date, time)| format!("P{date}T{time}")),
    ]
}

/// A valid `xs:duration` lexeme: the ordered-component grammar with the optional leading `-`
/// that plain `xs:duration` admits (and the `TimeRangeType` chain's duration half does not).
/// Boxed: a composition hub consumed by the format-facet strategies (see the module's
/// stack-depth note on boxing at hubs).
pub(crate) fn xs_duration_lexeme() -> BoxedStrategy<String> {
    (any::<bool>(), duration_lexeme())
        .prop_map(|(negative, body)| if negative { format!("-{body}") } else { body })
        .boxed()
}

/// A typed `SdmxDuration` over generated `xs:duration` lexemes.
fn sdmx_duration() -> BoxedStrategy<SdmxDuration> {
    xs_duration_lexeme()
        .prop_map(|lexeme| SdmxDuration::new(lexeme).expect("strategy emits valid xs:duration"))
        .boxed()
}

/// A valid `TimeRangeType` lexeme: a full date or date-time start (optional timezone), `/`,
/// then a duration.
pub(crate) fn time_range_lexeme() -> impl Strategy<Value = String> {
    let start = with_optional_timezone(prop_oneof![date_core(), datetime_core()]);
    (start, duration_lexeme()).prop_map(|(start, duration)| format!("{start}/{duration}"))
}

/// A valid `ObservationalTimePeriodType` lexeme: either union member.
pub(crate) fn observational_time_period_lexeme() -> impl Strategy<Value = String> {
    prop_oneof![standard_time_period_lexeme(), time_range_lexeme()]
}

// ---------------------------------------------------------------------------
// Reference URN components (D-0073)
// ---------------------------------------------------------------------------

/// An agency identifier: one to three dot-separated NCName-style segments (sub-agencies nest).
pub(crate) fn urn_agency() -> impl Strategy<Value = String> {
    proptest::collection::vec(r"[A-Za-z][A-Za-z0-9_-]{0,9}", 1..=3)
        .prop_map(|segments| segments.join("."))
}

/// A single identifier from the URN id character class.
pub(crate) fn urn_id() -> impl Strategy<Value = String> {
    r"[A-Za-z0-9_@$-]{1,10}"
}

/// An item tail: one to three dot-separated id segments (nested container paths are
/// wire-legal).
pub(crate) fn urn_item_path() -> impl Strategy<Value = String> {
    proptest::collection::vec(urn_id(), 1..=3).prop_map(|segments| segments.join("."))
}

// ---------------------------------------------------------------------------
// Identifier tiers (D-0023) — the URN component classes coincide with the id tiers.
// ---------------------------------------------------------------------------

/// A valid `IDType` identifier, the loosest tier (the URN id segment shares the class).
pub(crate) fn id_type_lexeme() -> impl Strategy<Value = String> {
    urn_id()
}

/// A valid `NCNameIDType` identifier: a single `NCName` segment (the scheme-id tier).
pub(crate) fn ncname_lexeme() -> impl Strategy<Value = String> {
    r"[A-Za-z][A-Za-z0-9_-]{0,9}"
}

/// A valid `NestedNCNameIDType` agency identifier (the URN agency shares the grammar).
pub(crate) fn nested_ncname_lexeme() -> impl Strategy<Value = String> {
    urn_agency()
}

// ---------------------------------------------------------------------------
// xs:dateTime (lexeme-backed, D-0079)
// ---------------------------------------------------------------------------

/// An `SdmxDateTime` lexeme spanning the spelling variety its round-trip contract must preserve
/// byte-for-byte: offsetless, `Z`, `+00:00`, and assorted numeric `±hh:mm` offsets, with an
/// optional fractional-seconds part. Identity is the stored text (D-0079), so the generator
/// deliberately varies spellings the value model collapses (`Z` versus `+00:00`).
pub(crate) fn sdmx_date_time() -> impl Strategy<Value = SdmxDateTime> {
    // Numeric offsets stay in `1..=13` hours to avoid the `±14:00` boundary (where the grammar
    // requires zero minutes); `Z` and `+00:00` cover the zero-offset spellings explicitly.
    let timezone = prop_oneof![
        Just(String::new()),
        Just(String::from("Z")),
        Just(String::from("+00:00")),
        (1..=13u32, 0..=59u32).prop_map(|(h, m)| format!("+{h:02}:{m:02}")),
        (1..=13u32, 0..=59u32).prop_map(|(h, m)| format!("-{h:02}:{m:02}")),
    ];
    let fraction = prop_oneof![Just(String::new()), (0..=999u32).prop_map(|f| format!(".{f:03}"))];
    (1900..=2100u32, 1..=12u32, 1..=28u32, 0..=23u32, 0..=59u32, 0..=59u32, fraction, timezone)
        .prop_map(|(year, month, day, hour, minute, second, fraction, timezone)| {
            let lexeme = format!(
                "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{fraction}{timezone}"
            );
            SdmxDateTime::new(lexeme).expect("generator emits a valid xs:dateTime")
        })
        .boxed()
}

// ---------------------------------------------------------------------------
// The metadata spine (LocalisedText → … → MaintainableMetadata → Codelist)
// ---------------------------------------------------------------------------

/// A `LocalisedText` entry: free text with an optional stated `xml:lang` tag. Both are
/// held verbatim by the store (well-formedness is a lint, not an invariant), so arbitrary
/// strings are the legal set.
pub(crate) fn localised_text() -> impl Strategy<Value = LocalisedText> {
    (proptest::option::of(any::<String>()), any::<String>())
        .prop_map(|(language, text)| LocalisedText { language, text })
        .boxed()
}

/// A `LocalisedString`: one to three entries (the non-empty invariant holds by construction).
pub(crate) fn localised_string() -> impl Strategy<Value = LocalisedString> {
    proptest::collection::vec(localised_text(), 1..=3)
        .prop_map(|entries| LocalisedString::new(entries).expect("entries are non-empty"))
        .boxed()
}

/// Identifiable metadata over the given id strategy, annotations and links included.
fn identifiable_metadata_from(
    id: impl Strategy<Value = String>,
) -> impl Strategy<Value = IdentifiableMetadata> {
    (
        id,
        proptest::option::of(any::<String>()),
        proptest::option::of(any::<String>()),
        annotations(),
        links(),
    )
        .prop_map(|(id, uri, urn, annotations, links)| {
            IdentifiableMetadata::new(id, uri, urn, annotations, links)
                .expect("strategy emits valid ids")
        })
}

/// Identifiable metadata over a valid `IDType` id.
pub(crate) fn identifiable_metadata() -> impl Strategy<Value = IdentifiableMetadata> {
    identifiable_metadata_from(id_type_lexeme())
}

/// Nameable metadata over the given id strategy.
fn nameable_metadata_from(
    id: impl Strategy<Value = String>,
) -> impl Strategy<Value = NameableMetadata> {
    (identifiable_metadata_from(id), localised_string(), proptest::option::of(localised_string()))
        .prop_map(|(identifiable, names, descriptions)| {
            NameableMetadata::new(identifiable, names, descriptions)
        })
}

/// Nameable metadata over a valid `IDType` id.
pub(crate) fn nameable_metadata() -> impl Strategy<Value = NameableMetadata> {
    nameable_metadata_from(id_type_lexeme())
}

/// A version lexeme parsed into the raw-free `SdmxVersion`.
pub(crate) fn sdmx_version() -> impl Strategy<Value = SdmxVersion> {
    version_lexeme()
        .prop_map(|lexeme| SdmxVersion::new(lexeme).expect("strategy emits valid version lexemes"))
}

/// Versionable metadata over the given id strategy.
fn versionable_metadata_from(
    id: impl Strategy<Value = String>,
) -> impl Strategy<Value = VersionableMetadata> {
    (
        nameable_metadata_from(id),
        proptest::option::of(sdmx_version()),
        proptest::option::of(sdmx_date_time()),
        proptest::option::of(sdmx_date_time()),
    )
        .prop_map(|(nameable, version, valid_from, valid_to)| {
            VersionableMetadata::new(nameable, version, valid_from, valid_to)
        })
}

/// Versionable metadata over a valid `IDType` id.
pub(crate) fn versionable_metadata() -> impl Strategy<Value = VersionableMetadata> {
    versionable_metadata_from(id_type_lexeme())
}

/// Maintainable metadata over the given id strategy: a valid agency, stated-or-absent
/// flags, and free-text URLs (held verbatim; coherence is a lint).
fn maintainable_metadata_from(
    id: impl Strategy<Value = String>,
) -> impl Strategy<Value = MaintainableMetadata> {
    (
        versionable_metadata_from(id),
        nested_ncname_lexeme(),
        proptest::option::of(any::<bool>()),
        proptest::option::of(any::<bool>()),
        proptest::option::of(any::<String>()),
        proptest::option::of(any::<String>()),
    )
        .prop_map(
            |(
                versionable,
                agency,
                is_partial_language,
                is_external,
                service_url,
                structure_url,
            )| {
                MaintainableMetadata::new(
                    versionable,
                    agency,
                    is_partial_language,
                    is_external,
                    service_url,
                    structure_url,
                )
                .expect("strategy emits valid agency identifiers")
            },
        )
}

/// Maintainable metadata over a valid `IDType` id.
pub(crate) fn maintainable_metadata() -> impl Strategy<Value = MaintainableMetadata> {
    maintainable_metadata_from(id_type_lexeme())
}

/// A `Code`: nameable metadata plus an optional parent reference. The parent draws from the full
/// `IDType` class (leading digits, `@`, `$` included), so generation covers the lexemes valid at
/// the editions' union tier but invalid as 3.0 `SingleNCNameIDType` — locking the union boundary.
pub(crate) fn code() -> impl Strategy<Value = Code> {
    (nameable_metadata(), proptest::option::of(id_type_lexeme()))
        .prop_map(|(metadata, parent_id)| {
            Code::new(metadata, parent_id).expect("strategy emits IDType-valid parent ids")
        })
        .boxed()
}

/// A `Codelist` over an `NCName` scheme id, carrying zero to three codes in wire order.
pub(crate) fn codelist() -> impl Strategy<Value = Codelist> {
    (
        maintainable_metadata_from(ncname_lexeme()),
        proptest::option::of(any::<bool>()),
        proptest::collection::vec(code(), 0..=3),
        proptest::collection::vec(codelist_extension(), 0..=1),
    )
        .prop_map(|(metadata, is_partial, codes, extensions)| {
            let mut list =
                Codelist::new(metadata, is_partial).expect("strategy emits NCName scheme ids");
            for code in codes {
                list.push(code);
            }
            list.extensions = extensions;
            list
        })
        .boxed()
}

// ---------------------------------------------------------------------------
// Annotations and links (verbatim carriers)
// ---------------------------------------------------------------------------

/// An `AnnotationUrl`: verbatim URL text with an optional language tag.
fn annotation_url() -> impl Strategy<Value = AnnotationUrl> {
    (any::<String>(), proptest::option::of(any::<String>()))
        .prop_map(|(url, lang)| AnnotationUrl { url, lang })
}

/// An `Annotation`: every field optional or possibly-empty, mirroring the wire.
pub(crate) fn annotation() -> impl Strategy<Value = Annotation> {
    (
        proptest::option::of(any::<String>()),
        proptest::option::of(any::<String>()),
        proptest::option::of(any::<String>()),
        proptest::collection::vec(annotation_url(), 0..=2),
        proptest::option::of(any::<String>()),
        proptest::option::of(localised_string()),
    )
        .prop_map(
            |(id, annotation_type, annotation_title, annotation_urls, annotation_value, texts)| {
                Annotation {
                    id,
                    annotation_type,
                    annotation_title,
                    annotation_urls,
                    annotation_value,
                    texts,
                }
            },
        )
        .boxed()
}

/// A `Link`: required rel and url, optional urn and type, all held verbatim.
pub(crate) fn link() -> impl Strategy<Value = Link> {
    (
        any::<String>(),
        any::<String>(),
        proptest::option::of(any::<String>()),
        proptest::option::of(any::<String>()),
    )
        .prop_map(|(rel, url, urn, link_type)| Link { rel, url, urn, link_type })
        .boxed()
}

/// Zero to two annotations, the collection shape shared by every annotable carrier.
fn annotations() -> impl Strategy<Value = Vec<Annotation>> {
    proptest::collection::vec(annotation(), 0..=2)
}

/// Zero to two links.
fn links() -> impl Strategy<Value = Vec<Link>> {
    proptest::collection::vec(link(), 0..=2)
}

// ---------------------------------------------------------------------------
// Typed lexical values and the fixed-include wrapper
// ---------------------------------------------------------------------------

/// A typed `SdmxDecimal` built from a valid lexeme.
pub(crate) fn sdmx_decimal() -> impl Strategy<Value = SdmxDecimal> {
    xs_decimal_lexeme()
        .prop_map(|s| SdmxDecimal::new(s).expect("strategy emits valid lexemes"))
        .boxed()
}

/// A typed `SdmxInteger` built from a valid lexeme.
pub(crate) fn sdmx_integer() -> impl Strategy<Value = SdmxInteger> {
    xs_integer_lexeme()
        .prop_map(|s| SdmxInteger::new(s).expect("strategy emits valid lexemes"))
        .boxed()
}

/// A typed `SdmxTimePeriod` built from a valid lexeme.
pub(crate) fn sdmx_time_period() -> impl Strategy<Value = SdmxTimePeriod> {
    standard_time_period_lexeme()
        .prop_map(|s| SdmxTimePeriod::new(s).expect("strategy emits valid lexemes"))
        .boxed()
}

/// A typed `ObservationalTimePeriod` built from a valid lexeme of either member.
pub(crate) fn observational_time_period() -> impl Strategy<Value = ObservationalTimePeriod> {
    observational_time_period_lexeme()
        .prop_map(|s| ObservationalTimePeriod::new(s).expect("strategy emits valid lexemes"))
        .boxed()
}

/// A `FixedInclude`: absent or the stated fixed value `true` (a stated `false` is the one
/// rejected input).
pub(crate) fn fixed_include() -> impl Strategy<Value = FixedInclude> {
    prop_oneof![Just(None), Just(Some(true))]
        .prop_map(|stated| FixedInclude::new(stated).expect("never the mismatching false"))
        .boxed()
}

// ---------------------------------------------------------------------------
// Representation (D-0048: the position subsets filter one wide DataType table)
// ---------------------------------------------------------------------------

/// Every `DataType` variant; the position strategies filter this table through the subset
/// predicates so each position generates exactly its admitted set.
const ALL_DATA_TYPES: [DataType; 44] = [
    DataType::String,
    DataType::Alpha,
    DataType::AlphaNumeric,
    DataType::Numeric,
    DataType::BigInteger,
    DataType::Integer,
    DataType::Long,
    DataType::Short,
    DataType::Decimal,
    DataType::Float,
    DataType::Double,
    DataType::Boolean,
    DataType::URI,
    DataType::Count,
    DataType::InclusiveValueRange,
    DataType::ExclusiveValueRange,
    DataType::Incremental,
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
    DataType::Month,
    DataType::MonthDay,
    DataType::Day,
    DataType::Time,
    DataType::Duration,
    DataType::GeospatialInformation,
    DataType::XHTML,
    DataType::KeyValues,
    DataType::IdentifiableReference,
    DataType::DataSetReference,
];

/// A `DataType` drawn from the subset the given predicate admits.
fn data_type_where(predicate: fn(DataType) -> bool) -> impl Strategy<Value = DataType> {
    let admitted: Vec<DataType> =
        ALL_DATA_TYPES.into_iter().filter(|data_type| predicate(*data_type)).collect();
    proptest::sample::select(admitted)
}

/// An `xs:positiveInteger` facet count: any non-zero `u32` (the D-0075 width).
fn facet_count() -> impl Strategy<Value = NonZeroU32> {
    (1..=u32::MAX).prop_map(|count| NonZeroU32::new(count).expect("range starts at one"))
}

/// A representation-level `maxOccurs`: a finite non-zero count or `unbounded`.
fn max_occurs() -> impl Strategy<Value = MaxOccurs> {
    prop_oneof![facet_count().prop_map(MaxOccurs::Count), Just(MaxOccurs::Unbounded)]
}

/// An uncoded facet bundle whose `textType` draws from the given position subset;
/// `is_multi_lingual` is supplied by the caller because some positions prohibit it.
fn text_format(
    text_type_subset: fn(DataType) -> bool,
    is_multi_lingual: impl Strategy<Value = Option<bool>>,
) -> impl Strategy<Value = TextFormat> {
    (
        (
            proptest::option::of(data_type_where(text_type_subset)),
            proptest::option::of(any::<bool>()),
            proptest::option::of(sdmx_decimal()),
            proptest::option::of(sdmx_decimal()),
            proptest::option::of(sdmx_decimal()),
        ),
        (
            proptest::option::of(sdmx_duration()),
            proptest::option::of(sdmx_time_period()),
            proptest::option::of(sdmx_time_period()),
            proptest::option::of(facet_count()),
            proptest::option::of(facet_count()),
        ),
        (
            proptest::option::of(sdmx_decimal()),
            proptest::option::of(sdmx_decimal()),
            proptest::option::of(facet_count()),
            proptest::option::of(any::<String>()),
            is_multi_lingual,
        ),
    )
        .prop_map(
            |(
                (text_type, is_sequence, interval, start_value, end_value),
                (time_interval, start_time, end_time, min_length, max_length),
                (min_value, max_value, decimals, pattern, is_multi_lingual),
            )| {
                TextFormat {
                    text_type,
                    is_sequence,
                    interval,
                    start_value,
                    end_value,
                    time_interval,
                    start_time,
                    end_time,
                    min_length,
                    max_length,
                    min_value,
                    max_value,
                    decimals,
                    pattern,
                    is_multi_lingual,
                }
            },
        )
}

/// The time-position facet bundle: only `textType` (the time subset), `startTime`, and
/// `endTime` may be set; every other facet is prohibited there.
fn time_text_format() -> impl Strategy<Value = TextFormat> {
    (
        proptest::option::of(data_type_where(DataType::is_time)),
        proptest::option::of(sdmx_time_period()),
        proptest::option::of(sdmx_time_period()),
    )
        .prop_map(|(text_type, start_time, end_time)| TextFormat {
            text_type,
            start_time,
            end_time,
            ..TextFormat::default()
        })
        .boxed()
}

/// A coded facet bundle: integer numerics, no `decimals`, `textType` from the Code subset.
fn enumeration_format() -> impl Strategy<Value = EnumerationFormat> {
    (
        (
            proptest::option::of(data_type_where(DataType::is_code)),
            proptest::option::of(any::<bool>()),
            proptest::option::of(sdmx_integer()),
            proptest::option::of(sdmx_integer()),
            proptest::option::of(sdmx_integer()),
        ),
        (
            proptest::option::of(sdmx_duration()),
            proptest::option::of(sdmx_time_period()),
            proptest::option::of(sdmx_time_period()),
        ),
        (
            proptest::option::of(facet_count()),
            proptest::option::of(facet_count()),
            proptest::option::of(sdmx_integer()),
            proptest::option::of(sdmx_integer()),
            proptest::option::of(any::<String>()),
        ),
    )
        .prop_map(
            |(
                (text_type, is_sequence, interval, start_value, end_value),
                (time_interval, start_time, end_time),
                (min_length, max_length, min_value, max_value, pattern),
            )| {
                EnumerationFormat {
                    text_type,
                    is_sequence,
                    interval,
                    start_value,
                    end_value,
                    time_interval,
                    start_time,
                    end_time,
                    min_length,
                    max_length,
                    min_value,
                    max_value,
                    pattern,
                }
            },
        )
        .boxed()
}

// ---------------------------------------------------------------------------
// Typed references
// ---------------------------------------------------------------------------

/// A typed reference version (exact or `+`-wildcarded, never the bare `*`).
fn reference_version() -> impl Strategy<Value = VersionRef> {
    reference_version_lexeme()
        .prop_map(|s| VersionRef::new(s).expect("strategy emits valid reference versions"))
}

/// A typed `CodelistReference` over generated URN components.
pub(crate) fn codelist_reference() -> impl Strategy<Value = CodelistReference> {
    (urn_agency(), urn_id(), reference_version())
        .prop_map(|(agency, id, version)| CodelistReference { agency, id, version })
}

/// A typed `ValueListReference`.
fn value_list_reference() -> impl Strategy<Value = ValueListReference> {
    (urn_agency(), urn_id(), reference_version())
        .prop_map(|(agency, id, version)| ValueListReference { agency, id, version })
}

/// A typed `DataStructureReference`.
pub(crate) fn dsd_reference() -> impl Strategy<Value = DataStructureReference> {
    (urn_agency(), urn_id(), reference_version())
        .prop_map(|(agency, id, version)| DataStructureReference { agency, id, version })
}

/// A typed `DataflowReference`.
fn dataflow_reference() -> impl Strategy<Value = DataflowReference> {
    (urn_agency(), urn_id(), reference_version())
        .prop_map(|(agency, id, version)| DataflowReference { agency, id, version })
}

/// A typed `ProvisionAgreementReference`.
fn provision_agreement_reference() -> impl Strategy<Value = ProvisionAgreementReference> {
    (urn_agency(), urn_id(), reference_version())
        .prop_map(|(agency, id, version)| ProvisionAgreementReference { agency, id, version })
}

/// A typed `DataProviderReference` (item-in-scheme shape).
fn data_provider_reference() -> impl Strategy<Value = DataProviderReference> {
    (urn_agency(), urn_id(), reference_version(), urn_id()).prop_map(
        |(agency, scheme_id, version, id)| DataProviderReference { agency, scheme_id, version, id },
    )
}

/// A typed `ConceptReference` (item-in-scheme shape).
pub(crate) fn concept_reference() -> impl Strategy<Value = ConceptReference> {
    (urn_agency(), urn_id(), reference_version(), urn_id()).prop_map(
        |(agency, scheme_id, version, id)| ConceptReference { agency, scheme_id, version, id },
    )
}

// ---------------------------------------------------------------------------
// Position-valid representations (D-0048)
// ---------------------------------------------------------------------------

/// A Basic-position representation (a concept core representation, an attribute, or a
/// measure): either enumeration target, the Basic `textType` subset, occurrence attributes
/// admitted.
pub(crate) fn basic_representation() -> impl Strategy<Value = Representation> {
    let enumeration = prop_oneof![
        codelist_reference().prop_map(EnumerationReference::Codelist),
        value_list_reference().prop_map(EnumerationReference::ValueList),
    ];
    let choice = prop_oneof![
        text_format(DataType::is_basic, proptest::option::of(any::<bool>()))
            .prop_map(RepresentationChoice::TextFormat),
        (enumeration, proptest::option::of(enumeration_format())).prop_map(
            |(enumeration, format)| RepresentationChoice::Enumeration { enumeration, format }
        ),
    ];
    (choice, proptest::option::of(any::<u32>()), proptest::option::of(max_occurs()))
        .prop_map(|(choice, min_occurs, max_occurs)| Representation {
            choice,
            min_occurs,
            max_occurs,
        })
        .boxed()
}

/// A Dimension-position representation: codelist enumeration only, the Simple `textType`
/// subset, no `isMultiLingual`, no occurrence attributes.
pub(crate) fn dimension_representation() -> impl Strategy<Value = Representation> {
    let choice = prop_oneof![
        text_format(DataType::is_simple, Just(None)).prop_map(RepresentationChoice::TextFormat),
        (
            codelist_reference().prop_map(EnumerationReference::Codelist),
            proptest::option::of(enumeration_format())
        )
            .prop_map(|(enumeration, format)| RepresentationChoice::Enumeration {
                enumeration,
                format
            }),
    ];
    choice.prop_map(|choice| Representation { choice, min_occurs: None, max_occurs: None }).boxed()
}

/// A TimeDimension-position representation: an uncoded time facet bundle only.
pub(crate) fn time_representation() -> impl Strategy<Value = Representation> {
    time_text_format()
        .prop_map(|text_format| Representation {
            choice: RepresentationChoice::TextFormat(text_format),
            min_occurs: None,
            max_occurs: None,
        })
        .boxed()
}

// ---------------------------------------------------------------------------
// Components and descriptors
// ---------------------------------------------------------------------------

/// Component metadata over the given optional-id strategy.
fn component_metadata_from(
    id: impl Strategy<Value = Option<String>>,
) -> impl Strategy<Value = ComponentMetadata> {
    (
        id,
        proptest::option::of(any::<String>()),
        proptest::option::of(any::<String>()),
        annotations(),
        links(),
    )
        .prop_map(|(id, uri, urn, annotations, links)| {
            ComponentMetadata::new(id, uri, urn, annotations, links)
                .expect("strategy emits valid component ids")
        })
}

/// Component metadata whose stated id, when present, is a valid `NCNameIDType`.
pub(crate) fn component_metadata() -> impl Strategy<Value = ComponentMetadata> {
    component_metadata_from(proptest::option::of(ncname_lexeme()))
}

/// A stated-or-absent `usage` flag value.
fn usage() -> impl Strategy<Value = Usage> {
    prop_oneof![Just(Usage::Mandatory), Just(Usage::Optional)]
}

/// Zero-or-more concept-role references (the schema admits `ConceptRole` unbounded).
fn concept_roles() -> impl Strategy<Value = Vec<ConceptReference>> {
    proptest::collection::vec(concept_reference(), 0..=2)
}

/// A `Dimension` with a position-valid representation.
pub(crate) fn dimension() -> impl Strategy<Value = Dimension> {
    (
        component_metadata(),
        concept_reference(),
        concept_roles(),
        proptest::option::of(dimension_representation()),
        proptest::option::of(any::<i32>()),
    )
        .prop_map(|(metadata, concept, concept_roles, representation, position)| {
            Dimension::new(metadata, concept, concept_roles, representation, position)
                .expect("strategy respects the dimension position rules")
        })
        .boxed()
}

/// A `TimeDimension`: the stated id is absent or the fixed `TIME_PERIOD`, and the mandatory
/// representation is time-position-valid.
pub(crate) fn time_dimension() -> impl Strategy<Value = TimeDimension> {
    let id = prop_oneof![Just(None), Just(Some(String::from("TIME_PERIOD")))];
    (component_metadata_from(id), concept_reference(), time_representation())
        .prop_map(|(metadata, concept, representation)| {
            TimeDimension::new(metadata, concept, representation)
                .expect("strategy respects the time position rules")
        })
        .boxed()
}

/// A `DimensionRef` for an attribute relationship (an `NCNameIDType` local reference).
fn dimension_ref() -> impl Strategy<Value = DimensionRef> {
    (ncname_lexeme(), proptest::option::of(any::<bool>())).prop_map(|(id, optional)| {
        DimensionRef::new(id, optional).expect("strategy emits NCName-valid dimension ids")
    })
}

/// An `AttributeRelationship` across all four arms. The group arm draws from the full `IDType`
/// class (its tier, looser than `NCName`).
pub(crate) fn attribute_relationship() -> impl Strategy<Value = AttributeRelationship> {
    prop_oneof![
        Just(AttributeRelationship::Dataflow),
        Just(AttributeRelationship::Observation),
        id_type_lexeme()
            .prop_map(|id| AttributeRelationship::group(id).expect("group ids are IDType-valid")),
        proptest::collection::vec(dimension_ref(), 1..=3).prop_map(|refs| {
            AttributeRelationship::dimensions(refs).expect("dimension refs are non-empty")
        }),
    ]
    .boxed()
}

/// A non-empty `MeasureRelationship` (its items are `NCNameIDType` local references).
fn measure_relationship() -> impl Strategy<Value = MeasureRelationship> {
    proptest::collection::vec(ncname_lexeme(), 1..=2)
        .prop_map(|ids| MeasureRelationship::new(ids).expect("measure ids are NCName-valid"))
}

/// An `Attribute` with a Basic-position representation.
pub(crate) fn attribute() -> impl Strategy<Value = Attribute> {
    (
        component_metadata(),
        concept_reference(),
        concept_roles(),
        proptest::option::of(basic_representation()),
        attribute_relationship(),
        proptest::option::of(measure_relationship()),
        proptest::option::of(usage()),
    )
        .prop_map(
            |(
                metadata,
                concept,
                concept_roles,
                representation,
                relationship,
                measure_relationship,
                usage,
            )| {
                Attribute::new(
                    metadata,
                    concept,
                    concept_roles,
                    representation,
                    relationship,
                    measure_relationship,
                    usage,
                )
                .expect("strategy respects the basic position rules")
            },
        )
        .boxed()
}

/// A `MetadataAttributeUsage` (its local reference is `NCNameIDType`).
fn metadata_attribute_usage() -> impl Strategy<Value = MetadataAttributeUsage> {
    (
        ncname_lexeme(),
        attribute_relationship(),
        annotations(),
        proptest::option::of(link()),
        proptest::option::of(any::<String>()),
        proptest::option::of(any::<String>()),
    )
        .prop_map(|(metadata_attribute_ref, relationship, annotations, link, urn, uri)| {
            MetadataAttributeUsage::new(
                metadata_attribute_ref,
                annotations,
                link,
                urn,
                uri,
                relationship,
            )
            .expect("strategy emits NCName-valid local references")
        })
        .boxed()
}

/// An attribute-list member: an attribute or a metadata attribute usage.
fn attribute_list_member() -> impl Strategy<Value = AttributeListMember> {
    prop_oneof![
        attribute().prop_map(AttributeListMember::Attribute),
        metadata_attribute_usage().prop_map(AttributeListMember::MetadataAttributeUsage),
    ]
    .boxed()
}

/// A `Measure` with a Basic-position representation.
pub(crate) fn measure() -> impl Strategy<Value = Measure> {
    (
        component_metadata(),
        concept_reference(),
        concept_roles(),
        proptest::option::of(basic_representation()),
        proptest::option::of(usage()),
    )
        .prop_map(|(metadata, concept, concept_roles, representation, usage)| {
            Measure::new(metadata, concept, concept_roles, representation, usage)
                .expect("strategy respects the basic position rules")
        })
        .boxed()
}

/// A descriptor id: absent or the schema-fixed literal.
fn descriptor_id(fixed: &'static str) -> impl Strategy<Value = Option<String>> {
    prop_oneof![Just(None), Just(Some(String::from(fixed)))]
}

/// A non-empty `DimensionList` with an optional time dimension.
pub(crate) fn dimension_list() -> impl Strategy<Value = DimensionList> {
    (
        descriptor_id("DimensionDescriptor"),
        proptest::collection::vec(dimension(), 1..=2),
        proptest::option::of(time_dimension()),
        annotations(),
        links(),
        proptest::option::of(any::<String>()),
        proptest::option::of(any::<String>()),
    )
        .prop_map(|(id, dimensions, time_dimension, annotations, links, urn, uri)| {
            DimensionList::new(id, dimensions, time_dimension, annotations, links, urn, uri)
                .expect("fixed id and non-empty dimensions hold")
        })
        .boxed()
}

/// A `Group` with a non-empty dimension selection (its items are `NCNameIDType` local references).
pub(crate) fn group() -> impl Strategy<Value = Group> {
    (identifiable_metadata(), proptest::collection::vec(ncname_lexeme(), 1..=2))
        .prop_map(|(metadata, ids)| Group {
            metadata,
            dimensions: GroupDimensions::new(ids).expect("group dimension ids are NCName-valid"),
        })
        .boxed()
}

/// A non-empty `AttributeList`.
pub(crate) fn attribute_list() -> impl Strategy<Value = AttributeList> {
    (
        descriptor_id("AttributeDescriptor"),
        proptest::collection::vec(attribute_list_member(), 1..=2),
        annotations(),
        links(),
        proptest::option::of(any::<String>()),
        proptest::option::of(any::<String>()),
    )
        .prop_map(|(id, members, annotations, links, urn, uri)| {
            AttributeList::new(id, members, annotations, links, urn, uri)
                .expect("fixed id and non-empty members hold")
        })
        .boxed()
}

/// A non-empty `MeasureList`.
pub(crate) fn measure_list() -> impl Strategy<Value = MeasureList> {
    (
        descriptor_id("MeasureDescriptor"),
        proptest::collection::vec(measure(), 1..=2),
        annotations(),
        links(),
        proptest::option::of(any::<String>()),
        proptest::option::of(any::<String>()),
    )
        .prop_map(|(id, measures, annotations, links, urn, uri)| {
            MeasureList::new(id, measures, annotations, links, urn, uri)
                .expect("fixed id and non-empty measures hold")
        })
        .boxed()
}

/// A full `DataStructureDefinition` composing every descriptor family.
pub(crate) fn data_structure_definition() -> impl Strategy<Value = DataStructureDefinition> {
    (
        maintainable_metadata(),
        dimension_list(),
        proptest::collection::vec(group(), 0..=1),
        proptest::option::of(attribute_list()),
        proptest::option::of(measure_list()),
        proptest::option::of(any::<bool>()),
    )
        .prop_map(|(metadata, dimension_list, groups, attribute_list, measure_list, evolving)| {
            DataStructureDefinition {
                metadata,
                dimension_list,
                groups,
                attribute_list,
                measure_list,
                evolving_structure: evolving,
            }
        })
        .boxed()
}

/// A `Dataflow` with an optional structure reference and 3.1 dimension constraint (its items are
/// `IDType` local references, drawn from the full class).
pub(crate) fn dataflow() -> impl Strategy<Value = Dataflow> {
    let constraint = proptest::collection::vec(id_type_lexeme(), 1..=2)
        .prop_map(|ids| DimensionConstraint::new(ids).expect("dimension ids are IDType-valid"));
    (
        maintainable_metadata(),
        proptest::option::of(dsd_reference()),
        proptest::option::of(constraint),
    )
        .prop_map(|(metadata, dsd, dimension_constraint)| Dataflow {
            metadata,
            dsd,
            dimension_constraint,
        })
        .boxed()
}

// ---------------------------------------------------------------------------
// Concept, organisation, and value-list schemes
// ---------------------------------------------------------------------------

/// An `IsoConceptReference` (three free-string children).
fn iso_concept_reference() -> impl Strategy<Value = IsoConceptReference> {
    (any::<String>(), any::<String>(), any::<String>()).prop_map(
        |(concept_agency, concept_scheme_id, concept_id)| IsoConceptReference {
            concept_agency,
            concept_scheme_id,
            concept_id,
        },
    )
}

/// A `Concept` over an `NCName` id with a Basic-position core representation. The parent is the
/// same `NCNameIDType` tier (the editions' `Parent` declarations share one pattern).
pub(crate) fn concept() -> impl Strategy<Value = Concept> {
    (
        nameable_metadata_from(ncname_lexeme()),
        proptest::option::of(ncname_lexeme()),
        proptest::option::of(basic_representation()),
        proptest::option::of(iso_concept_reference()),
    )
        .prop_map(|(metadata, parent_id, core_representation, iso_concept_reference)| {
            Concept::new(metadata, parent_id, core_representation, iso_concept_reference)
                .expect("strategy emits NCName ids and basic-valid representations")
        })
        .boxed()
}

/// A `ConceptScheme` over an `NCName` scheme id, carrying zero to two concepts.
pub(crate) fn concept_scheme() -> impl Strategy<Value = ConceptScheme> {
    (
        maintainable_metadata_from(ncname_lexeme()),
        proptest::option::of(any::<bool>()),
        proptest::collection::vec(concept(), 0..=2),
    )
        .prop_map(|(metadata, is_partial, concepts)| {
            let mut scheme =
                ConceptScheme::new(metadata, is_partial).expect("strategy emits NCName scheme ids");
            for concept in concepts {
                scheme.push(concept);
            }
            scheme
        })
        .boxed()
}

/// A `ContactDetail` across the five endpoint kinds.
fn contact_detail() -> impl Strategy<Value = ContactDetail> {
    prop_oneof![
        any::<String>().prop_map(ContactDetail::Telephone),
        any::<String>().prop_map(ContactDetail::Fax),
        any::<String>().prop_map(ContactDetail::X400),
        any::<String>().prop_map(ContactDetail::Uri),
        any::<String>().prop_map(ContactDetail::Email),
    ]
}

/// A `Contact` with optional localised triple and interleaved details.
fn contact() -> impl Strategy<Value = Contact> {
    (
        proptest::option::of(localised_string()),
        proptest::option::of(localised_string()),
        proptest::option::of(localised_string()),
        proptest::collection::vec(contact_detail(), 0..=2),
    )
        .prop_map(|(names, departments, roles, details)| Contact {
            names,
            departments,
            roles,
            details,
        })
        .boxed()
}

/// An `Agency` over an `NCName` id with zero to two contacts.
pub(crate) fn agency() -> impl Strategy<Value = Agency> {
    (nameable_metadata_from(ncname_lexeme()), proptest::collection::vec(contact(), 0..=2))
        .prop_map(|(metadata, contacts)| {
            Agency::new(metadata, contacts).expect("strategy emits NCName agency ids")
        })
        .boxed()
}

/// An `AgencyScheme`: the scheme id is the schema-fixed `AGENCIES`.
pub(crate) fn agency_scheme() -> impl Strategy<Value = AgencyScheme> {
    (
        maintainable_metadata_from(Just(String::from("AGENCIES"))),
        proptest::option::of(any::<bool>()),
        proptest::collection::vec(agency(), 0..=2),
    )
        .prop_map(|(metadata, is_partial, agencies)| {
            let mut scheme =
                AgencyScheme::new(metadata, is_partial).expect("the fixed AGENCIES id holds");
            for agency in agencies {
                scheme.push(agency);
            }
            scheme
        })
        .boxed()
}

/// A `ValueItem`: its id is the unvalidated fourth tier, so arbitrary text is legal.
fn value_item() -> impl Strategy<Value = ValueItem> {
    (
        any::<String>(),
        proptest::option::of(localised_string()),
        proptest::option::of(localised_string()),
        annotations(),
    )
        .prop_map(|(id, names, descriptions, annotations)| ValueItem {
            id,
            names,
            descriptions,
            annotations,
        })
        .boxed()
}

/// A `ValueList` carrying zero to three items in wire order.
pub(crate) fn value_list() -> impl Strategy<Value = ValueList> {
    (maintainable_metadata(), proptest::collection::vec(value_item(), 0..=3))
        .prop_map(|(metadata, items)| ValueList { metadata, items })
        .boxed()
}

// ---------------------------------------------------------------------------
// Codelist extensions
// ---------------------------------------------------------------------------

/// A `Cascade` selection across the tri-state.
pub(crate) fn cascade() -> impl Strategy<Value = Cascade> {
    prop_oneof![Just(Cascade::None), Just(Cascade::IncludeChildren), Just(Cascade::ExcludeRoot)]
}

/// A non-empty `MemberValues` list (member content is held verbatim).
fn member_values() -> impl Strategy<Value = MemberValues> {
    let member = (any::<String>(), proptest::option::of(cascade()))
        .prop_map(|(value, cascade)| MemberValue { value, cascade });
    proptest::collection::vec(member, 1..=2)
        .prop_map(|values| MemberValues::new(values).expect("member values are non-empty"))
        .boxed()
}

/// A `CodeSelection`: inclusive or exclusive member values.
fn code_selection() -> impl Strategy<Value = CodeSelection> {
    prop_oneof![
        member_values().prop_map(CodeSelection::Inclusive),
        member_values().prop_map(CodeSelection::Exclusive),
    ]
    .boxed()
}

/// A `CodelistExtension` with an optional filtered selection.
pub(crate) fn codelist_extension() -> impl Strategy<Value = CodelistExtension> {
    (
        codelist_reference(),
        proptest::option::of(code_selection()),
        proptest::option::of(any::<String>()),
    )
        .prop_map(|(codelist, selection, prefix)| CodelistExtension { codelist, selection, prefix })
        .boxed()
}

// ---------------------------------------------------------------------------
// The constraint model
// ---------------------------------------------------------------------------

/// A `CubeKeyValue` with optional cascade and validity window.
fn cube_key_value() -> impl Strategy<Value = CubeKeyValue> {
    (
        any::<String>(),
        proptest::option::of(cascade()),
        proptest::option::of(sdmx_time_period()),
        proptest::option::of(sdmx_time_period()),
    )
        .prop_map(|(value, cascade, valid_from, valid_to)| CubeKeyValue {
            value,
            cascade,
            valid_from,
            valid_to,
        })
        .boxed()
}

/// A `SimpleComponentValue` with optional cascade, language, and validity window.
fn simple_component_value() -> impl Strategy<Value = SimpleComponentValue> {
    (
        any::<String>(),
        proptest::option::of(cascade()),
        proptest::option::of(any::<String>()),
        proptest::option::of(sdmx_time_period()),
        proptest::option::of(sdmx_time_period()),
    )
        .prop_map(|(value, cascade, lang, valid_from, valid_to)| SimpleComponentValue {
            value,
            cascade,
            lang,
            valid_from,
            valid_to,
        })
        .boxed()
}

/// A `DataComponentValue` (the key-set counterpart: no validity window exists there).
fn data_component_value() -> impl Strategy<Value = DataComponentValue> {
    (any::<String>(), proptest::option::of(cascade()), proptest::option::of(any::<String>()))
        .prop_map(|(value, cascade, lang)| DataComponentValue { value, cascade, lang })
        .boxed()
}

/// A `TimePeriodRange` endpoint over the observational union.
fn time_period_range() -> impl Strategy<Value = TimePeriodRange> {
    (observational_time_period(), proptest::option::of(any::<bool>()))
        .prop_map(|(period, inclusive)| TimePeriodRange { period, inclusive })
        .boxed()
}

/// A `TimeRangeKind` across the before/after/between arms.
fn time_range_kind() -> impl Strategy<Value = TimeRangeKind> {
    prop_oneof![
        time_period_range().prop_map(TimeRangeKind::Before),
        time_period_range().prop_map(TimeRangeKind::After),
        (time_period_range(), time_period_range())
            .prop_map(|(start, end)| TimeRangeKind::Between { start, end }),
    ]
    .boxed()
}

/// A `TimeRange` with its own optional validity window.
pub(crate) fn time_range() -> impl Strategy<Value = TimeRange> {
    (
        time_range_kind(),
        proptest::option::of(sdmx_time_period()),
        proptest::option::of(sdmx_time_period()),
    )
        .prop_map(|(kind, valid_from, valid_to)| TimeRange { kind, valid_from, valid_to })
        .boxed()
}

/// A `KeyValueSelection`: enumerated values or a time range (no empty state exists).
fn key_value_selection() -> impl Strategy<Value = KeyValueSelection> {
    let values = proptest::collection::vec(cube_key_value(), 1..=2)
        .prop_map(|values| CubeKeyValues::new(values).expect("cube key values are non-empty"));
    prop_oneof![
        values.prop_map(KeyValueSelection::Values),
        time_range().prop_map(KeyValueSelection::TimeRange),
    ]
    .boxed()
}

/// A `ComponentSelection` across all three arms, `Empty` included.
fn component_selection() -> impl Strategy<Value = ComponentSelection> {
    let values = proptest::collection::vec(simple_component_value(), 1..=2).prop_map(|values| {
        SimpleComponentValues::new(values).expect("component values are non-empty")
    });
    prop_oneof![
        values.prop_map(ComponentSelection::Values),
        time_range().prop_map(ComponentSelection::TimeRange),
        Just(ComponentSelection::Empty),
    ]
    .boxed()
}

/// A `CubeRegionKey` dimension selection (its id is `SingleNCNameIDType`, the `NCName` pattern).
fn cube_region_key() -> impl Strategy<Value = CubeRegionKey> {
    (
        (ncname_lexeme(), key_value_selection()),
        (
            proptest::option::of(any::<bool>()),
            proptest::option::of(any::<bool>()),
            proptest::option::of(sdmx_time_period()),
            proptest::option::of(sdmx_time_period()),
        ),
    )
        .prop_map(|((id, selection), (include, remove_prefix, valid_from, valid_to))| {
            CubeRegionKey::new(id, selection, include, remove_prefix, valid_from, valid_to)
                .expect("the id is a generated NCName lexeme")
        })
        .boxed()
}

/// A `ComponentValueSet` component selection (its id is `NestedNCNameIDType`, dotted; validity is
/// prohibited here by omission).
fn component_value_set() -> impl Strategy<Value = ComponentValueSet> {
    (
        nested_ncname_lexeme(),
        component_selection(),
        proptest::option::of(any::<bool>()),
        proptest::option::of(any::<bool>()),
    )
        .prop_map(|(id, selection, include, remove_prefix)| {
            ComponentValueSet::new(id, selection, include, remove_prefix)
                .expect("the id is a generated NestedNCName lexeme")
        })
        .boxed()
}

/// A `CubeRegion` of dimension and component selections.
pub(crate) fn cube_region() -> impl Strategy<Value = CubeRegion> {
    (
        proptest::collection::vec(cube_region_key(), 0..=2),
        proptest::collection::vec(component_value_set(), 0..=2),
        proptest::option::of(any::<bool>()),
        annotations(),
    )
        .prop_map(|(key_values, components, include, annotations)| CubeRegion {
            key_values,
            components,
            include,
            annotations,
        })
        .boxed()
}

/// A `CubeRegions` list within the schema's bound of two.
fn cube_regions() -> impl Strategy<Value = CubeRegions> {
    proptest::collection::vec(cube_region(), 0..=2)
        .prop_map(|regions| CubeRegions::new(regions).expect("at most two regions"))
        .boxed()
}

/// A `DataComponentSelection` across all three arms.
fn data_component_selection() -> impl Strategy<Value = DataComponentSelection> {
    let values = proptest::collection::vec(data_component_value(), 1..=2).prop_map(|values| {
        DataComponentValues::new(values).expect("component values are non-empty")
    });
    prop_oneof![
        values.prop_map(DataComponentSelection::Values),
        time_range().prop_map(DataComponentSelection::TimeRange),
        Just(DataComponentSelection::Empty),
    ]
    .boxed()
}

/// A `DataComponentValueSet` key-set component selection (its id is `NestedNCNameIDType`, dotted).
fn data_component_value_set() -> impl Strategy<Value = DataComponentValueSet> {
    (
        nested_ncname_lexeme(),
        data_component_selection(),
        proptest::option::of(any::<bool>()),
        proptest::option::of(any::<bool>()),
    )
        .prop_map(|(id, selection, include, remove_prefix)| {
            DataComponentValueSet::new(id, selection, include, remove_prefix)
                .expect("the id is a generated NestedNCName lexeme")
        })
        .boxed()
}

/// A `DataKeyValue`: bare non-empty values with the schema-fixed include flag (its id is
/// `SingleNCNameIDType`, the `NCName` pattern).
fn data_key_value() -> impl Strategy<Value = DataKeyValue> {
    let values = proptest::collection::vec(any::<String>(), 1..=2)
        .prop_map(|values| SimpleKeyValues::new(values).expect("key values are non-empty"));
    (ncname_lexeme(), values, fixed_include(), proptest::option::of(any::<bool>()))
        .prop_map(|(id, values, include, remove_prefix)| {
            DataKeyValue::new(id, values, include, remove_prefix)
                .expect("the id is a generated NCName lexeme")
        })
        .boxed()
}

/// A `DataKey` with its validity window and annotations.
fn data_key() -> impl Strategy<Value = DataKey> {
    (
        (
            proptest::collection::vec(data_key_value(), 0..=2),
            proptest::collection::vec(data_component_value_set(), 0..=2),
        ),
        (
            fixed_include(),
            annotations(),
            proptest::option::of(sdmx_time_period()),
            proptest::option::of(sdmx_time_period()),
        ),
    )
        .prop_map(|((key_values, components), (include, annotations, valid_from, valid_to))| {
            DataKey { key_values, components, include, annotations, valid_from, valid_to }
        })
        .boxed()
}

/// A `DataKeySet` of at least one key.
fn data_key_set() -> impl Strategy<Value = DataKeySet> {
    (
        proptest::collection::vec(data_key(), 1..=2)
            .prop_map(|keys| DataKeys::new(keys).expect("data keys are non-empty")),
        any::<bool>(),
    )
        .prop_map(|(keys, is_included)| DataKeySet { keys, is_included })
        .boxed()
}

/// A `QueryableDataSource` (3.0-only attachment trailer).
fn queryable_data_source() -> impl Strategy<Value = QueryableDataSource> {
    (
        any::<String>(),
        proptest::option::of(any::<String>()),
        proptest::option::of(any::<String>()),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(data_url, wsdl_url, wadl_url, is_rest, is_ws)| QueryableDataSource {
            data_url,
            wsdl_url,
            wadl_url,
            is_rest_data_source: is_rest,
            is_web_service_data_source: is_ws,
        })
        .boxed()
}

/// A `DataConstraintAttachment` across all five arms.
fn data_constraint_attachment() -> impl Strategy<Value = DataConstraintAttachment> {
    let queryable = || proptest::collection::vec(queryable_data_source(), 0..=1);
    prop_oneof![
        data_provider_reference().prop_map(DataConstraintAttachment::DataProvider),
        proptest::collection::vec(any::<String>(), 1..=2).prop_map(|urls| {
            DataConstraintAttachment::SimpleDataSource(
                SimpleDataSources::new(urls).expect("urls are non-empty"),
            )
        }),
        (proptest::collection::vec(dsd_reference(), 1..=2), queryable()).prop_map(
            |(refs, queryable)| DataConstraintAttachment::DataStructure {
                refs: DataStructureRefs::new(refs).expect("refs are non-empty"),
                queryable,
            }
        ),
        (proptest::collection::vec(dataflow_reference(), 1..=2), queryable()).prop_map(
            |(refs, queryable)| DataConstraintAttachment::Dataflow {
                refs: DataflowRefs::new(refs).expect("refs are non-empty"),
                queryable,
            }
        ),
        (proptest::collection::vec(provision_agreement_reference(), 1..=2), queryable()).prop_map(
            |(refs, queryable)| DataConstraintAttachment::ProvisionAgreement {
                refs: ProvisionAgreementRefs::new(refs).expect("refs are non-empty"),
                queryable,
            }
        ),
    ]
    .boxed()
}

/// An `AvailabilityConstraintAttachment`: a single target of the data subset.
fn availability_constraint_attachment() -> impl Strategy<Value = AvailabilityConstraintAttachment> {
    prop_oneof![
        dsd_reference().prop_map(AvailabilityConstraintAttachment::DataStructure),
        dataflow_reference().prop_map(AvailabilityConstraintAttachment::Dataflow),
        provision_agreement_reference()
            .prop_map(AvailabilityConstraintAttachment::ProvisionAgreement),
    ]
    .boxed()
}

/// A `ReleaseCalendar` (3.0-only; the duration strings are held verbatim).
fn release_calendar() -> impl Strategy<Value = ReleaseCalendar> {
    (any::<String>(), any::<String>(), any::<String>())
        .prop_map(|(periodicity, offset, tolerance)| ReleaseCalendar {
            periodicity,
            offset,
            tolerance,
        })
        .boxed()
}

/// A full `DataConstraint`.
pub(crate) fn data_constraint() -> impl Strategy<Value = DataConstraint> {
    (
        (
            maintainable_metadata(),
            proptest::option::of(prop_oneof![
                Just(ConstraintRole::Allowed),
                Just(ConstraintRole::Actual)
            ]),
        ),
        (
            proptest::option::of(data_constraint_attachment()),
            proptest::option::of(release_calendar()),
            proptest::collection::vec(data_key_set(), 0..=2),
            cube_regions(),
        ),
    )
        .prop_map(|((metadata, role), (attachment, release_calendar, key_sets, regions))| {
            DataConstraint { metadata, role, attachment, release_calendar, key_sets, regions }
        })
        .boxed()
}

/// A full `AvailabilityConstraint`.
pub(crate) fn availability_constraint() -> impl Strategy<Value = AvailabilityConstraint> {
    (
        availability_constraint_attachment(),
        cube_region(),
        annotations(),
        proptest::option::of(any::<i32>()),
        proptest::option::of(any::<i32>()),
    )
        .prop_map(|(attachment, region, annotations, series_count, obs_count)| {
            AvailabilityConstraint { attachment, region, annotations, series_count, obs_count }
        })
        .boxed()
}

/// The unified `ConstraintModel` across both kinds.
pub(crate) fn constraint_model() -> impl Strategy<Value = ConstraintModel> {
    prop_oneof![
        data_constraint().prop_map(ConstraintModel::Data),
        availability_constraint().prop_map(ConstraintModel::Availability),
    ]
    .boxed()
}

// ---------------------------------------------------------------------------
// Rejection families: deliberately off-grammar input. Each strategy guarantees every
// emitted value lies outside the target grammar, so the owning property asserts rejection
// unconditionally. Only the tractable complements live here; the precise boundary stays
// with the deterministic example tests.
// ---------------------------------------------------------------------------

/// An off-grammar `xs:decimal` lexeme: digitless shapes, an embedded letter or space, a
/// second point, or an exponent.
pub(crate) fn invalid_decimal_lexeme() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just(String::from(".")),
        Just(String::from("+")),
        Just(String::from("-")),
        r"[0-9]{1,5}[A-Za-z][0-9]{0,4}",
        r"[0-9]{1,4}\.[0-9]{0,3}\.[0-9]{0,3}",
        r"[0-9]{1,4} [0-9]{1,4}",
        r"[0-9]{1,4}[eE][+-]?[0-9]{1,3}",
        r" [0-9]{1,4}",
        r"[0-9]{1,4} ",
    ]
    .boxed()
}

/// An off-grammar `xs:integer` lexeme: every invalid decimal shape plus any lexeme with a
/// point (valid as a decimal, invalid as an integer).
pub(crate) fn invalid_integer_lexeme() -> impl Strategy<Value = String> {
    prop_oneof![invalid_decimal_lexeme(), r"[+-]?[0-9]{0,4}\.[0-9]{1,4}", r"[+-]?[0-9]{1,4}\.",]
        .boxed()
}

/// An off-grammar `IDType` identifier: empty, or a valid base with one out-of-class
/// character injected.
pub(crate) fn invalid_id_lexeme() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        (id_type_lexeme(), r"[ .#/%()]", id_type_lexeme())
            .prop_map(|(head, bad, tail)| format!("{head}{bad}{tail}")),
    ]
    .boxed()
}

/// An off-grammar `NCNameIDType` identifier: empty, a bad leading character, or an
/// out-of-class character injected after a valid head.
pub(crate) fn invalid_ncname_lexeme() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        r"[0-9_-][A-Za-z0-9_-]{0,5}",
        (ncname_lexeme(), r"[@$. ]", r"[A-Za-z0-9_-]{0,5}")
            .prop_map(|(head, bad, tail)| format!("{head}{bad}{tail}")),
    ]
    .boxed()
}

/// An off-grammar `NestedNCNameIDType` identifier: empty, a leading, trailing, or doubled
/// dot, or a bad leading character on one segment.
pub(crate) fn invalid_nested_ncname_lexeme() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        ncname_lexeme().prop_map(|segment| format!(".{segment}")),
        ncname_lexeme().prop_map(|segment| format!("{segment}.")),
        (ncname_lexeme(), ncname_lexeme()).prop_map(|(a, b)| format!("{a}..{b}")),
        (r"[0-9_-][A-Za-z0-9_-]{0,4}", ncname_lexeme()).prop_map(|(bad, ok)| format!("{bad}.{ok}")),
    ]
    .boxed()
}
