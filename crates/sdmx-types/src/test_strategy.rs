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
#![allow(clippy::redundant_pub_crate)]

use alloc::{
    format,
    string::{String, ToString},
};

use proptest::prelude::*;

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
