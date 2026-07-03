//! Shared `proptest` strategies, compiled only under `cfg(test)` and never for wasm32
//! (the property suite verifies platform-independent invariants; see `docs/dev/testing.md`).
//!
//! Every strategy here emits a *grammar-valid lexeme* (or the components one is formatted
//! from), and the property owning it constructs the value through the type's validated
//! `new()`/`from_str()` — the same single write path production code uses, so the generated
//! set is exactly the legal set. A generator that bypassed the constructor would have the
//! same defect as a `Deserialize` that did (design 0010 §7). Strategies for deliberately
//! off-grammar input land separately, with the rejection-family properties.

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

use chrono::{DateTime, FixedOffset};
use proptest::prelude::*;

use crate::{
    Code, Codelist, IdentifiableMetadata, LocalisedString, LocalisedText, MaintainableMetadata,
    NameableMetadata, SdmxVersion, VersionableMetadata,
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
fn gregorian_year() -> impl Strategy<Value = String> {
    prop_oneof![6 => r"[0-9]{4}", 1 => r"[1-9][0-9]{4}", 1 => r"-[0-9]{4}"]
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
// chrono (Q-G: the stated offset is generated independently of the instant)
// ---------------------------------------------------------------------------

/// A `DateTime<FixedOffset>` whose instant, subsecond precision, and stated offset are all
/// generated independently, with whole-hour, quarter-hour, and arbitrary-minute offsets
/// across the XSD `±14:00` window — the spread the stated-offset round-trip contract needs.
pub(crate) fn fixed_offset_datetime() -> impl Strategy<Value = DateTime<FixedOffset>> {
    let offset_minutes = prop_oneof![
        (-14_i32..=14).prop_map(|hours| hours * 60),
        (-56_i32..=56).prop_map(|quarters| quarters * 15),
        -840_i32..=840,
    ];
    // Seconds spanning roughly 1841..2159 with optional subsecond nanos.
    (
        -4_000_000_000_i64..6_000_000_000,
        prop_oneof![Just(0_u32), 0_u32..1_000_000_000],
        offset_minutes,
    )
        .prop_map(|(seconds, nanos, minutes)| {
            let offset = FixedOffset::east_opt(minutes * 60).expect("offset within bounds");
            DateTime::from_timestamp(seconds, nanos)
                .expect("timestamp within chrono range")
                .with_timezone(&offset)
        })
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
}

/// A `LocalisedString`: one to three entries (the non-empty invariant holds by construction).
pub(crate) fn localised_string() -> impl Strategy<Value = LocalisedString> {
    proptest::collection::vec(localised_text(), 1..=3)
        .prop_map(|entries| LocalisedString::new(entries).expect("entries are non-empty"))
}

/// Identifiable metadata over the given id strategy. Annotations and links stay empty at
/// this stage; their strategies join with the remaining type families.
fn identifiable_metadata_from(
    id: impl Strategy<Value = String>,
) -> impl Strategy<Value = IdentifiableMetadata> {
    (id, proptest::option::of(any::<String>()), proptest::option::of(any::<String>())).prop_map(
        |(id, uri, urn)| {
            IdentifiableMetadata::new(id, uri, urn, Vec::new(), Vec::new())
                .expect("strategy emits valid ids")
        },
    )
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
        proptest::option::of(fixed_offset_datetime()),
        proptest::option::of(fixed_offset_datetime()),
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

/// A `Code`: nameable metadata plus an optional structural parent reference.
pub(crate) fn code() -> impl Strategy<Value = Code> {
    (nameable_metadata(), proptest::option::of(id_type_lexeme()))
        .prop_map(|(metadata, parent_id)| Code { metadata, parent_id })
}

/// A `Codelist` over an `NCName` scheme id, carrying zero to three codes in wire order.
pub(crate) fn codelist() -> impl Strategy<Value = Codelist> {
    (
        maintainable_metadata_from(ncname_lexeme()),
        proptest::option::of(any::<bool>()),
        proptest::collection::vec(code(), 0..=3),
    )
        .prop_map(|(metadata, is_partial, codes)| {
            let mut list =
                Codelist::new(metadata, is_partial).expect("strategy emits NCName scheme ids");
            for code in codes {
                list.push(code);
            }
            list
        })
}
