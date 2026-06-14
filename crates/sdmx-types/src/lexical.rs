//! Validated lexical newtypes for SDMX's constrained value types.
//!
//! [`SdmxDecimal`], [`SdmxInteger`], [`SdmxVersion`], and [`SdmxTimePeriod`] wrap the SDMX
//! lexical types whose value space does not map losslessly onto a fixed Rust type. Each stores
//! the canonical lexeme verbatim, validates it at construction, and never rewrites it, so values
//! round-trip exactly.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Lexical-newtype convention: the canonical lexical form is the lossless source of truth, `new()`
validates the grammar on the single write path so every caller gets a well-formed value, and
where validation naturally classifies the value the cheap discriminant is retained
(`SdmxVersion`'s parsed components, `SdmxTimePeriod`'s kind).

`Ord`/`PartialOrd` for `SdmxVersion` are deliberately deferred, not resolved; see its design notes.

Decisions: D-0027.
"#
)]

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::error::{Error, to_de_error};

// ---------------------------------------------------------------------------
// xs:decimal
// ---------------------------------------------------------------------------

/// An SDMX `xs:decimal` value, stored losslessly as its canonical lexeme.
///
/// ## Specification
/// - **Schema**: W3C XML Schema (`xs`)
/// - **Type**: `xs:decimal`
/// - **Element**: N/A (Primitive)
/// - **Editions**: SDMX 3.0 and 3.1
///
/// An arbitrary-precision decimal, stored as its exact text so no precision is lost. The value
/// is validated when constructed and never rewritten, so it round-trips verbatim.
///
/// ## Guarantees
///
/// Round-trips losslessly through its text: `x.to_string().parse::<SdmxDecimal>() == Ok(x)`.
///
/// # Examples
///
/// ```
/// use sdmx_types::SdmxDecimal;
///
/// let value: SdmxDecimal = "-3.14".parse()?;
/// assert_eq!(value.as_str(), "-3.14");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Lexical newtype with lossless `String` storage: `raw` is the canonical form and the round-trip
source of truth (an `f64` would round; a fixed-width decimal would overflow). No useful sub-kind,
so it is a bare newtype, in contrast to `SdmxVersion`, which retains a parsed decomposition.

Decisions: D-0027.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SdmxDecimal(String);

impl SdmxDecimal {
    /// Validates `s` against the `xs:decimal` lexical grammar and stores it verbatim.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidDecimal`] if `s` is not a valid `xs:decimal` lexeme: an optional
    /// sign, decimal digits, at most one `.`, and no exponent.
    pub fn new(s: String) -> Result<Self, Error> {
        if is_xs_decimal(&s) { Ok(Self(s)) } else { Err(Error::InvalidDecimal(s)) }
    }

    /// The canonical lexeme, exactly as supplied.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// xs:integer
// ---------------------------------------------------------------------------

/// An SDMX `xs:integer` value, stored losslessly as its canonical lexeme.
///
/// ## Specification
/// - **Schema**: W3C XML Schema (`xs`)
/// - **Type**: `xs:integer`
/// - **Element**: N/A (Primitive)
/// - **Editions**: SDMX 3.0 and 3.1
///
/// An arbitrary-magnitude integer, stored as its exact text so no value is lost. It is a distinct
/// type from [`SdmxDecimal`] so a value the schema requires to be integral cannot hold a fraction:
/// `"2.5"` is unrepresentable here.
///
/// ## Guarantees
///
/// Round-trips losslessly through its text: `x.to_string().parse::<SdmxInteger>() == Ok(x)`.
///
/// # Examples
///
/// ```
/// use sdmx_types::SdmxInteger;
///
/// let value: SdmxInteger = "-7".parse()?;
/// assert_eq!(value.as_str(), "-7");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Lexical newtype with lossless `String` storage. Keeping it a distinct type from `SdmxDecimal`
makes a fractional value unrepresentable where the schema demands an integer
(make-illegal-states-unrepresentable).

Decisions: D-0027.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SdmxInteger(String);

impl SdmxInteger {
    /// Validates `s` against the `xs:integer` lexical grammar and stores it verbatim.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInteger`] if `s` is not a valid `xs:integer` lexeme: an optional
    /// sign followed by decimal digits only, with no `.` and no exponent.
    pub fn new(s: String) -> Result<Self, Error> {
        if is_xs_integer(&s) { Ok(Self(s)) } else { Err(Error::InvalidInteger(s)) }
    }

    /// The canonical lexeme, exactly as supplied.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Widens to [`SdmxDecimal`]. Every `xs:integer` lexeme is also a valid `xs:decimal` lexeme, so
/// this is total, infallible, and zero-cost: the raw string carries over verbatim.
impl From<SdmxInteger> for SdmxDecimal {
    fn from(i: SdmxInteger) -> Self {
        Self(i.0)
    }
}

/// Narrows to [`SdmxInteger`] by strict *lexical* validation, never a numeric conversion: it
/// succeeds only if the decimal's text is already a valid `xs:integer` lexeme, and never rewrites
/// it. `"42"` succeeds; `"2.5"` and `"42.0"` are rejected (the latter is integral in value but not
/// an integer lexeme, so rejecting it preserves the lossless raw form).
impl TryFrom<SdmxDecimal> for SdmxInteger {
    type Error = Error;

    fn try_from(d: SdmxDecimal) -> Result<Self, Error> {
        if is_xs_integer(&d.0) { Ok(Self(d.0)) } else { Err(Error::InvalidInteger(d.0)) }
    }
}

// ---------------------------------------------------------------------------
// VersionType
// ---------------------------------------------------------------------------

/// An SDMX `VersionType`: a semantic (`major.minor.patch[-extension]`) or legacy
/// (`major[.minor]`) version.
///
/// ## Specification
/// - **Schema**: `SDMXCommonReferences.xsd`
/// - **Type**: `VersionType`
/// - **Element**: N/A (Simple Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/VersionType.md"))]
///
/// The canonical string is stored verbatim and the parsed components are read through the
/// accessors. `patch()` returning `None` distinguishes the legacy form from the semantic form.
///
/// Equality compares the exact version string, so `"3.1"` and `"3.1.0"` are distinct. Ordering is
/// not currently provided.
///
/// ## Guarantees
///
/// Round-trips losslessly through its text: `x.to_string().parse::<SdmxVersion>() == Ok(x)`.
///
/// # Examples
///
/// ```
/// use sdmx_types::SdmxVersion;
///
/// let version: SdmxVersion = "1.0.0-rc.1".parse()?;
/// assert_eq!(version.major(), 1);
/// assert_eq!(version.extension(), Some("rc.1"));
/// assert!(!version.is_legacy());
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`raw` is the lossless source of truth; the parsed decomposition is retained alongside it.
`major`/`minor`/`patch` are `u32` because the validated grammar is digits-only (no sign), so
unsigned loses nothing.

Ordering is deliberately deferred, not resolved. SemVer §11 precedence is the intended basis, but
the legacy/semantic equivalence (for example `3.1` vs `3.1.0`) is undecided and premature to lock.
The likely shape is an explicit precedence-comparison convenience (a method or wrapper) rather than
an `Ord` impl on the type, so raw-`Eq` and SemVer ordering can coexist without an `Ord`/`Eq`
contract: distinct under equality, equal under precedence.

Decisions: D-0027.
"#
)]
#[derive(Clone, Debug)]
pub struct SdmxVersion {
    raw: String,
    major: u32,
    minor: u32,
    patch: Option<u32>,
    extension: Option<String>,
}

impl SdmxVersion {
    /// Validates `raw` against the `VersionType` union grammar and retains the parsed
    /// decomposition. On success `raw` is stored verbatim.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidVersion`] if `raw` matches neither the semantic nor the
    /// legacy form (this includes leading zeros, empty components, an extension on a legacy
    /// version, and numeric components that exceed `u32`).
    pub fn new(raw: String) -> Result<Self, Error> {
        parse_sdmx_version(raw)
    }

    /// The canonical version string, exactly as supplied.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// The major component.
    #[must_use]
    pub const fn major(&self) -> u32 {
        self.major
    }

    /// The minor component. A bare-major legacy version (for example `"1"`) reports `0`.
    #[must_use]
    pub const fn minor(&self) -> u32 {
        self.minor
    }

    /// The patch component, or `None` for the legacy form (`major[.minor]`).
    #[must_use]
    pub const fn patch(&self) -> Option<u32> {
        self.patch
    }

    /// The semantic prerelease extension (for example `"rc.1"`), or `None`.
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }

    /// `true` for the legacy form (`major[.minor]`, no patch), `false` for semantic.
    #[must_use]
    pub const fn is_legacy(&self) -> bool {
        self.patch.is_none()
    }
}

impl core::fmt::Display for SdmxVersion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.raw)
    }
}

/// Equality compares the canonical version strings, so a prerelease and its release
/// (`"1.0.0-rc"` vs `"1.0.0"`) are correctly unequal.
impl PartialEq for SdmxVersion {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl Eq for SdmxVersion {}

/// A [`Display`](core::fmt::Display) adapter for an optional [`SdmxVersion`] that renders
/// `<unversioned>` when the version is absent.
///
/// For display and logging only: the `<unversioned>` placeholder is not a valid version and must
/// not be round-tripped.
///
/// # Examples
///
/// ```
/// use sdmx_types::{SdmxVersion, VersionDisplay};
///
/// let version: SdmxVersion = "2.1.0".parse()?;
/// assert_eq!(VersionDisplay(Some(&version)).to_string(), "2.1.0");
/// assert_eq!(VersionDisplay(None).to_string(), "<unversioned>");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The `<unversioned>` sentinel lives only here. Its angle brackets are outside every SDMX identifier
and version lexical set, so it is un-roundtrippable by design: a writer that received it would fail
validation loudly rather than emit a plausible version.
"#
)]
#[derive(Clone, Copy, Debug)]
pub struct VersionDisplay<'a>(pub Option<&'a SdmxVersion>);

impl core::fmt::Display for VersionDisplay<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            Some(v) => write!(f, "{v}"),
            None => f.write_str("<unversioned>"),
        }
    }
}

// ---------------------------------------------------------------------------
// StandardTimePeriodType
// ---------------------------------------------------------------------------

/// An SDMX `StandardTimePeriodType`: a Gregorian period, an `xs:dateTime`, or a reporting period.
///
/// ## Specification
/// - **Schema**: `SDMXCommon.xsd`
/// - **Type**: `StandardTimePeriodType`
/// - **Element**: N/A (Simple Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/StandardTimePeriodType.md"))]
///
/// The canonical string is stored verbatim, and the value is classified into an
/// [`SdmxTimePeriodKind`] during validation so the kind is available without reparsing.
///
/// ## Guarantees
///
/// Round-trips losslessly through its text: `x.to_string().parse::<SdmxTimePeriod>() == Ok(x)`.
///
/// # Examples
///
/// ```
/// use sdmx_types::{SdmxTimePeriod, SdmxTimePeriodKind};
///
/// let period: SdmxTimePeriod = "2024-Q4".parse()?;
/// assert_eq!(period.kind(), SdmxTimePeriodKind::ReportingQuarter);
/// assert_eq!(period.as_str(), "2024-Q4");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Lexical newtype with lossless `String` storage; `kind` is the cheap discriminant extracted during
validation, mirroring the spec union one-to-one.

Decisions: D-0027.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SdmxTimePeriod {
    raw: String,
    kind: SdmxTimePeriodKind,
}

impl SdmxTimePeriod {
    /// Validates `raw` against `StandardTimePeriodType` and classifies it in one pass.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTimePeriod`] if `raw` is not a member of the
    /// `StandardTimePeriodType` union (a Gregorian period, `xs:dateTime`, or a reporting period).
    pub fn new(raw: String) -> Result<Self, Error> {
        match classify_time_period(&raw) {
            Some(kind) => Ok(Self { raw, kind }),
            None => Err(Error::InvalidTimePeriod(raw)),
        }
    }

    /// The canonical time-period string, exactly as supplied.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// The classified kind of this time period.
    #[must_use]
    pub const fn kind(&self) -> SdmxTimePeriodKind {
        self.kind
    }
}

/// The classified kind of an [`SdmxTimePeriod`]: which member of `StandardTimePeriodType` a value
/// belongs to.
///
/// ## Specification
/// - **Schema**: `SDMXCommon.xsd`
/// - **Type**: `StandardTimePeriodType` (`BasicTimePeriodType` and `ReportingTimePeriodType` members)
/// - **Element**: N/A (Simple Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/StandardTimePeriodType.md"))]
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/BasicTimePeriodType.md"))]
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ReportingTimePeriodType.md"))]
#[cfg_attr(design_docs, doc = "")]
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Exhaustive (no `#[non_exhaustive]`): a bounded, spec-fixed union, so an exhaustive consumer match
is correct and a future member would rightly be a breaking change.

Decisions: D-0021.
"#
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SdmxTimePeriodKind {
    /// `xs:gYear`: a Gregorian calendar year (`YYYY`).
    GregorianYear,
    /// `xs:gYearMonth`: a Gregorian calendar month (`YYYY-MM`).
    GregorianYearMonth,
    /// `xs:date`: a Gregorian calendar day (`YYYY-MM-DD`).
    GregorianDay,
    /// `xs:dateTime`: a specific instant (`YYYY-MM-DDThh:mm:ss`).
    DateTime,
    /// `ReportingYearType`: a reporting year (`YYYY-A1`).
    ReportingYear,
    /// `ReportingSemesterType`: a reporting semester (`YYYY-S1`..`YYYY-S2`).
    ReportingSemester,
    /// `ReportingTrimesterType`: a reporting trimester (`YYYY-T1`..`YYYY-T3`).
    ReportingTrimester,
    /// `ReportingQuarterType`: a reporting quarter (`YYYY-Q1`..`YYYY-Q4`).
    ReportingQuarter,
    /// `ReportingMonthType`: a reporting month (`YYYY-M01`..`YYYY-M12`).
    ReportingMonth,
    /// `ReportingWeekType`: a reporting week (`YYYY-W01`..`YYYY-W53`).
    ReportingWeek,
    /// `ReportingDayType`: a reporting day (`YYYY-D001`..`YYYY-D366`).
    ReportingDay,
}

impl SdmxTimePeriodKind {
    /// Projects the kind onto a plain calendar [`Granularity`], collapsing the
    /// calendar-system axis (for example `GregorianYear` and `ReportingYear` both map to
    /// [`Granularity::Year`]). Gives the plain-name view without losing the spec-exact kind.
    #[must_use]
    pub const fn granularity(self) -> Granularity {
        match self {
            Self::GregorianYear | Self::ReportingYear => Granularity::Year,
            Self::ReportingSemester => Granularity::Semester,
            Self::ReportingTrimester => Granularity::Trimester,
            Self::ReportingQuarter => Granularity::Quarter,
            Self::GregorianYearMonth | Self::ReportingMonth => Granularity::Month,
            Self::ReportingWeek => Granularity::Week,
            Self::GregorianDay | Self::ReportingDay => Granularity::Day,
            Self::DateTime => Granularity::Instant,
        }
    }
}

/// A calendar granularity: an [`SdmxTimePeriodKind`] with the calendar-system axis collapsed
/// (for example a Gregorian year and a reporting year both have year granularity).
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// This type does not exist in the SDMX schema; it is a convenience projection over
/// [`SdmxTimePeriodKind`].
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A Layer-2 projection (a derived view), not a stored field: it collapses the calendar-system axis
of the spec-exact kind without losing it.

Decisions: D-0031.
"#
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Granularity {
    /// A one-year period.
    Year,
    /// A six-month period.
    Semester,
    /// A four-month period.
    Trimester,
    /// A three-month period.
    Quarter,
    /// A one-month period.
    Month,
    /// A one-week period.
    Week,
    /// A one-day period.
    Day,
    /// A point in time (`xs:dateTime`).
    Instant,
}

// ---------------------------------------------------------------------------
// Standard trait impls (Display / FromStr / AsRef): uniform across the lexeme newtypes.
// `new(String)` stays the owned, no-clone entry; `from_str(&str)` is the borrowed `.parse()`
// path that clones into ownership.
// ---------------------------------------------------------------------------

impl core::fmt::Display for SdmxDecimal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

impl core::str::FromStr for SdmxDecimal {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl AsRef<str> for SdmxDecimal {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for SdmxInteger {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

impl core::str::FromStr for SdmxInteger {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl AsRef<str> for SdmxInteger {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// `SdmxVersion` already implements `Display` (above); only `FromStr` and `AsRef` are added here.
impl core::str::FromStr for SdmxVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl AsRef<str> for SdmxVersion {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

impl core::fmt::Display for SdmxTimePeriod {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.raw)
    }
}

impl core::str::FromStr for SdmxTimePeriod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl AsRef<str> for SdmxTimePeriod {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

// ---------------------------------------------------------------------------
// Serde (custom: every lexical newtype routes through its validated constructor)
// ---------------------------------------------------------------------------

impl serde::Serialize for SdmxDecimal {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for SdmxDecimal {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(to_de_error)
    }
}

impl serde::Serialize for SdmxInteger {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for SdmxInteger {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(to_de_error)
    }
}

impl serde::Serialize for SdmxVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.raw)
    }
}

impl<'de> serde::Deserialize<'de> for SdmxVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(to_de_error)
    }
}

impl serde::Serialize for SdmxTimePeriod {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.raw)
    }
}

impl<'de> serde::Deserialize<'de> for SdmxTimePeriod {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// Grammar helpers (hand-rolled, no_std, no regex)
// ---------------------------------------------------------------------------

/// `xs:decimal`: optional sign, then digits with at most one decimal point; at least one
/// digit must be present (so `"."`, `"+"`, and `""` are rejected). No exponent.
fn is_xs_decimal(s: &str) -> bool {
    let body = s.strip_prefix(['+', '-']).unwrap_or(s);
    let mut seen_dot = false;
    let mut seen_digit = false;
    for c in body.chars() {
        match c {
            '0'..='9' => seen_digit = true,
            '.' if !seen_dot => seen_dot = true,
            _ => return false,
        }
    }
    seen_digit
}

/// `xs:integer`: optional sign followed by one or more decimal digits. No `.`, no exponent.
fn is_xs_integer(s: &str) -> bool {
    let body = s.strip_prefix(['+', '-']).unwrap_or(s);
    !body.is_empty() && body.bytes().all(|b| b.is_ascii_digit())
}

/// A `SemVer` numeric component: `0` or a non-zero-leading run of digits (`0|[1-9]\d*`).
fn is_numeric_component(s: &str) -> bool {
    match s.as_bytes() {
        [b'0'] => true,
        [first, ..] if first.is_ascii_digit() && *first != b'0' => {
            s.bytes().all(|b| b.is_ascii_digit())
        }
        _ => false,
    }
}

/// A single `SemVer` prerelease identifier: either a numeric identifier (`0|[1-9][0-9]*`, no
/// leading zero) or an alphanumeric identifier over `[A-Za-z0-9-]` containing at least one
/// non-digit.
fn is_extension_identifier(s: &str) -> bool {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return false;
    }
    if s.bytes().all(|b| b.is_ascii_digit()) {
        // Numeric identifier: bare zero, or no leading zero.
        s == "0" || !s.starts_with('0')
    } else {
        true
    }
}

/// Parses and validates the `VersionType` union, populating the decomposition.
fn parse_sdmx_version(raw: String) -> Result<SdmxVersion, Error> {
    // A '-' can only introduce the semantic extension: the legacy form is digits and dots
    // only, so its presence forces the semantic interpretation.
    let (core, extension) = match raw.split_once('-') {
        Some((core, ext)) => (core, Some(ext)),
        None => (raw.as_str(), None),
    };

    let components: Vec<&str> = core.split('.').collect();
    if !components.iter().all(|c| is_numeric_component(c)) {
        return Err(Error::InvalidVersion(raw));
    }

    let parse_u32 = |c: &str| c.parse::<u32>().ok();

    let (major, minor, patch) = match (components.as_slice(), extension) {
        // Semantic: exactly three numeric components, optional extension.
        ([maj, min, pat], ext) => {
            if let Some(ext) = ext
                && (ext.is_empty() || !ext.split('.').all(is_extension_identifier))
            {
                return Err(Error::InvalidVersion(raw));
            }
            match (parse_u32(maj), parse_u32(min), parse_u32(pat)) {
                (Some(maj), Some(min), Some(pat)) => (maj, min, Some(pat)),
                _ => return Err(Error::InvalidVersion(raw)),
            }
        }
        // Legacy: one or two numeric components, no extension permitted.
        ([maj], None) => match parse_u32(maj) {
            Some(maj) => (maj, 0, None),
            None => return Err(Error::InvalidVersion(raw)),
        },
        ([maj, min], None) => match (parse_u32(maj), parse_u32(min)) {
            (Some(maj), Some(min)) => (maj, min, None),
            _ => return Err(Error::InvalidVersion(raw)),
        },
        _ => return Err(Error::InvalidVersion(raw)),
    };

    let extension = extension.map(ToString::to_string);
    Ok(SdmxVersion { raw, major, minor, patch, extension })
}

/// Validates and classifies a `StandardTimePeriodType` value, returning its kind or `None`
/// if the value is outside the union.
fn classify_time_period(s: &str) -> Option<SdmxTimePeriodKind> {
    let (core, timezone) = split_timezone(s);
    if let Some(tz) = timezone
        && !is_valid_timezone(tz)
    {
        return None;
    }

    // Reporting periods are `YYYY-<letter>...`; the letter at offset 5 disambiguates them
    // from every basic form (whose offset-5 byte, when present, is a digit or `T`).
    let bytes = core.as_bytes();
    if bytes.len() >= 6
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5].is_ascii_alphabetic()
    {
        return classify_reporting(core);
    }

    if let Some((date, time)) = core.split_once('T') {
        return (is_full_date(date) && is_time(time)).then_some(SdmxTimePeriodKind::DateTime);
    }

    classify_gregorian(core)
}

/// Splits a trailing timezone (`Z` or `±hh:mm`) off the end of a time-period string. The
/// structural shape is checked here; the numeric range is validated by [`is_valid_timezone`].
fn split_timezone(s: &str) -> (&str, Option<&str>) {
    if let Some(head) = s.strip_suffix('Z') {
        return (head, Some("Z"));
    }
    if s.len() >= 6 {
        let (head, tail) = s.split_at(s.len() - 6);
        let b = tail.as_bytes();
        if (b[0] == b'+' || b[0] == b'-')
            && b[1].is_ascii_digit()
            && b[2].is_ascii_digit()
            && b[3] == b':'
            && b[4].is_ascii_digit()
            && b[5].is_ascii_digit()
        {
            return (head, Some(tail));
        }
    }
    (s, None)
}

/// `Z`, or `±hh:mm` with the offset in the inclusive range `-14:00`..`+14:00`.
fn is_valid_timezone(tz: &str) -> bool {
    if tz == "Z" {
        return true;
    }
    let b = tz.as_bytes();
    if b.len() != 6 || (b[0] != b'+' && b[0] != b'-') || b[3] != b':' {
        return false;
    }
    match (num_in(&tz[1..3], 2, 0, 14), num_in(&tz[4..6], 2, 0, 59)) {
        (true, true) => &tz[1..3] != "14" || &tz[4..6] == "00",
        _ => false,
    }
}

/// Classifies a reporting period (timezone already stripped). Returns `None` if the period
/// designator or its numeric value is out of range.
fn classify_reporting(core: &str) -> Option<SdmxTimePeriodKind> {
    let bytes = core.as_bytes();
    // YYYY '-' <letter> <number>
    let designator = bytes[5];
    let number = &core[6..];
    if !number.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    match designator {
        b'A' if number == "1" => Some(SdmxTimePeriodKind::ReportingYear),
        b'S' if num_in(number, 1, 1, 2) => Some(SdmxTimePeriodKind::ReportingSemester),
        b'T' if num_in(number, 1, 1, 3) => Some(SdmxTimePeriodKind::ReportingTrimester),
        b'Q' if num_in(number, 1, 1, 4) => Some(SdmxTimePeriodKind::ReportingQuarter),
        b'M' if num_in(number, 2, 1, 12) => Some(SdmxTimePeriodKind::ReportingMonth),
        b'W' if num_in(number, 2, 1, 53) => Some(SdmxTimePeriodKind::ReportingWeek),
        b'D' if num_in(number, 3, 1, 366) => Some(SdmxTimePeriodKind::ReportingDay),
        _ => None,
    }
}

/// Classifies a basic Gregorian period (timezone already stripped, no `T`): `gYear`,
/// `gYearMonth`, or `date`.
fn classify_gregorian(core: &str) -> Option<SdmxTimePeriodKind> {
    // A leading '-' denotes a BC year (permitted by xs:gYear and friends).
    let body = core.strip_prefix('-').unwrap_or(core);
    let mut parts = body.split('-');
    let year = parts.next()?;
    if !is_year(year) {
        return None;
    }
    match (parts.next(), parts.next(), parts.next()) {
        (None, _, _) => Some(SdmxTimePeriodKind::GregorianYear),
        (Some(month), None, _) => {
            num_in(month, 2, 1, 12).then_some(SdmxTimePeriodKind::GregorianYearMonth)
        }
        (Some(month), Some(day), None) => (num_in(month, 2, 1, 12) && num_in(day, 2, 1, 31))
            .then_some(SdmxTimePeriodKind::GregorianDay),
        _ => None,
    }
}

/// A full `xs:date` body (`YYYY-MM-DD`, optional leading `-` for a BC year), with no timezone.
fn is_full_date(date: &str) -> bool {
    classify_gregorian(date) == Some(SdmxTimePeriodKind::GregorianDay)
}

/// An `xs:dateTime` time body: `hh:mm:ss` with an optional fractional-seconds suffix.
fn is_time(time: &str) -> bool {
    let (hms, fraction_ok) = match time.split_once('.') {
        Some((hms, fraction)) => {
            (hms, !fraction.is_empty() && fraction.bytes().all(|b| b.is_ascii_digit()))
        }
        None => (time, true),
    };
    if !fraction_ok {
        return false;
    }
    let mut parts = hms.split(':');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(h), Some(m), Some(s), None) => {
            num_in(h, 2, 0, 23) && num_in(m, 2, 0, 59) && num_in(s, 2, 0, 59)
        }
        _ => false,
    }
}

/// A four-or-more digit year (`xs:gYear` calendar component).
fn is_year(s: &str) -> bool {
    s.len() >= 4 && s.bytes().all(|b| b.is_ascii_digit())
}

/// True iff `s` is exactly `width` ASCII digits whose value lies in the inclusive range
/// `lo..=hi`.
fn num_in(s: &str, width: usize, lo: u32, hi: u32) -> bool {
    s.len() == width
        && s.bytes().all(|b| b.is_ascii_digit())
        && s.parse::<u32>().is_ok_and(|v| (lo..=hi).contains(&v))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn decimal_accepts_and_rejects() {
        for ok in ["0", "-1.5", "+42", "3.14159", ".5", "5.", "-0.0"] {
            assert!(SdmxDecimal::new(ok.into()).is_ok(), "{ok:?} should be a valid xs:decimal");
        }
        for bad in ["", ".", "+", "1.2.3", "1e5", "abc", "1,5"] {
            assert!(SdmxDecimal::new(bad.into()).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn integer_accepts_and_rejects() {
        for ok in ["0", "-7", "+1234"] {
            assert!(SdmxInteger::new(ok.into()).is_ok(), "{ok:?} should be a valid xs:integer");
        }
        for bad in ["", "1.0", "2.5", "+", "0x1f", "12a"] {
            assert!(SdmxInteger::new(bad.into()).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn integer_widens_and_narrows() {
        let i = SdmxInteger::new("42".into()).unwrap();
        let d: SdmxDecimal = i.into();
        assert_eq!(d.as_str(), "42");
        assert_eq!(SdmxInteger::try_from(d).unwrap().as_str(), "42");
        // "42.0" is integral in value but not an integer lexeme.
        let fractional = SdmxDecimal::new("42.0".into()).unwrap();
        assert!(SdmxInteger::try_from(fractional).is_err());
    }

    #[test]
    fn version_semantic_and_legacy() {
        let semantic = SdmxVersion::new("1.2.3".into()).unwrap();
        assert_eq!(
            (semantic.major(), semantic.minor(), semantic.patch(), semantic.is_legacy()),
            (1, 2, Some(3), false)
        );
        let prerelease = SdmxVersion::new("1.0.0-rc.1".into()).unwrap();
        assert_eq!(prerelease.extension(), Some("rc.1"));
        let legacy = SdmxVersion::new("1.3".into()).unwrap();
        assert_eq!((legacy.major(), legacy.minor(), legacy.patch()), (1, 3, None));
        let bare = SdmxVersion::new("1".into()).unwrap();
        assert_eq!((bare.major(), bare.minor(), bare.is_legacy()), (1, 0, true));
    }

    #[test]
    fn version_rejects_malformed() {
        for bad in [
            "",
            "banana",
            "01.0.0",
            "1.0.0.0",
            "1.0.0-",
            "1.0-rc",
            "1.",
            "v1.0.0",
            "1.0.0-rc@1",      // extension identifier with an out-of-class character
            "99999999999.0.0", // semantic component overflows u32
            "99999999999",     // legacy bare-major overflows u32
            "1.99999999999",   // legacy minor overflows u32
        ] {
            assert!(SdmxVersion::new(bad.into()).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn version_equality_is_on_raw() {
        // Trailing-zero-equivalent versions are deliberately unequal (lossless-distinct).
        assert_ne!(
            SdmxVersion::new("1.0".into()).unwrap(),
            SdmxVersion::new("1.0.0".into()).unwrap()
        );
        assert_eq!(
            SdmxVersion::new("2.1.0".into()).unwrap(),
            SdmxVersion::new("2.1.0".into()).unwrap()
        );
    }

    #[test]
    fn version_display_round_trips_raw() {
        use alloc::string::ToString;
        let v = SdmxVersion::new("3.0.0-beta.2".into()).unwrap();
        assert_eq!(v.to_string(), "3.0.0-beta.2");
        assert_eq!(VersionDisplay(Some(&v)).to_string(), "3.0.0-beta.2");
        assert_eq!(VersionDisplay(None).to_string(), "<unversioned>");
    }

    #[test]
    fn time_period_basic_kinds() {
        let cases = [
            ("2024", SdmxTimePeriodKind::GregorianYear),
            ("2024-05", SdmxTimePeriodKind::GregorianYearMonth),
            ("2024-05-01", SdmxTimePeriodKind::GregorianDay),
            ("2024-05-01T09:30:00", SdmxTimePeriodKind::DateTime),
            ("2024-05-01T09:30:00.500Z", SdmxTimePeriodKind::DateTime),
            ("2024-05-01T09:30:00+05:30", SdmxTimePeriodKind::DateTime),
            ("2024Z", SdmxTimePeriodKind::GregorianYear),
        ];
        for (input, kind) in cases {
            let parsed = SdmxTimePeriod::new(input.into()).unwrap();
            assert_eq!(parsed.kind(), kind, "{input:?}");
            assert_eq!(parsed.as_str(), input);
        }
    }

    #[test]
    fn time_period_reporting_kinds() {
        let cases = [
            ("2024-A1", SdmxTimePeriodKind::ReportingYear),
            ("2024-S2", SdmxTimePeriodKind::ReportingSemester),
            ("2024-T3", SdmxTimePeriodKind::ReportingTrimester),
            ("2024-Q4", SdmxTimePeriodKind::ReportingQuarter),
            ("2024-M12", SdmxTimePeriodKind::ReportingMonth),
            ("2024-W53", SdmxTimePeriodKind::ReportingWeek),
            ("2024-D366", SdmxTimePeriodKind::ReportingDay),
            ("2024-Q4+14:00", SdmxTimePeriodKind::ReportingQuarter),
        ];
        for (input, kind) in cases {
            assert_eq!(SdmxTimePeriod::new(input.into()).unwrap().kind(), kind, "{input:?}");
        }
    }

    #[test]
    fn time_period_rejects_out_of_range_and_malformed() {
        for bad in [
            "",
            "2024-13",
            "2024-00",
            "2024-05-32",
            "2024-A2",
            "2024-S3",
            "2024-Q5",
            "2024-M13",
            "2024-W54",
            "2024-D367",
            "2024-05-01T25:00:00",
            "24-05",
            "2024-05-01+15:00",
            "2024-05-01T00:00:00+14:30", // tz hour 14 requires minutes 00
            "2024-Q",
            "2024-QX",              // reporting period number is not numeric
            "2024-05-01-01",        // four dash-separated date parts
            "2024-05-01T09:30:00.", // empty fractional seconds
            "2024-05-01T09:30",     // time missing the seconds field
        ] {
            assert!(SdmxTimePeriod::new(bad.into()).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn granularity_projection_collapses_calendar_axis() {
        use Granularity as G;
        use SdmxTimePeriodKind as K;
        let cases = [
            (K::GregorianYear, G::Year),
            (K::ReportingYear, G::Year),
            (K::ReportingSemester, G::Semester),
            (K::ReportingTrimester, G::Trimester),
            (K::ReportingQuarter, G::Quarter),
            (K::GregorianYearMonth, G::Month),
            (K::ReportingMonth, G::Month),
            (K::ReportingWeek, G::Week),
            (K::GregorianDay, G::Day),
            (K::ReportingDay, G::Day),
            (K::DateTime, G::Instant),
        ];
        for (kind, granularity) in cases {
            assert_eq!(kind.granularity(), granularity, "{kind:?}");
        }
    }

    #[test]
    fn deserialize_routes_through_new() {
        // Each lexical newtype deserializes from a JSON string and round-trips verbatim;
        // a value its grammar rejects fails deserialization (the §7 construction contract).
        assert_eq!(serde_json::from_str::<SdmxDecimal>(r#""-3.14""#).unwrap().as_str(), "-3.14");
        assert!(serde_json::from_str::<SdmxDecimal>(r#""banana""#).is_err());

        assert_eq!(serde_json::from_str::<SdmxInteger>(r#""-7""#).unwrap().as_str(), "-7");
        assert!(serde_json::from_str::<SdmxInteger>(r#""2.5""#).is_err());

        let version = serde_json::from_str::<SdmxVersion>(r#""1.0.0-rc.1""#).unwrap();
        assert_eq!(version.extension(), Some("rc.1"));
        assert!(serde_json::from_str::<SdmxVersion>(r#""01.0.0""#).is_err());

        let period = serde_json::from_str::<SdmxTimePeriod>(r#""2024-Q4""#).unwrap();
        assert_eq!(period.kind(), SdmxTimePeriodKind::ReportingQuarter);
        assert!(serde_json::from_str::<SdmxTimePeriod>(r#""2024-Q5""#).is_err());
    }

    #[test]
    fn serialize_emits_raw_string() {
        let version = SdmxVersion::new("2.1".into()).unwrap();
        assert_eq!(serde_json::to_string(&version).unwrap(), r#""2.1""#);
        let decimal = SdmxDecimal::new("0.001".into()).unwrap();
        assert_eq!(serde_json::to_string(&decimal).unwrap(), r#""0.001""#);
        let integer = SdmxInteger::new("-7".into()).unwrap();
        assert_eq!(serde_json::to_string(&integer).unwrap(), r#""-7""#);
        let period = SdmxTimePeriod::new("2024-Q4".into()).unwrap();
        assert_eq!(serde_json::to_string(&period).unwrap(), r#""2024-Q4""#);
    }

    #[test]
    fn lexical_newtypes_round_trip_display_parse_asref() {
        use alloc::string::ToString;

        // The Display / FromStr / AsRef impls are uniform across the lexeme newtypes: every valid
        // lexeme renders verbatim, parses back to an equal value, and exposes itself as a borrowed
        // `&str`. (`FromStr` clones into the owned `new(String)` write path.)
        for raw in ["0", "-1.5", "+42", "3.14159", ".5", "5.", "-0.0"] {
            let v = SdmxDecimal::new(raw.into()).unwrap();
            assert_eq!(v.to_string(), raw);
            assert_eq!(AsRef::<str>::as_ref(&v), raw);
            assert_eq!(v.to_string().parse::<SdmxDecimal>(), Ok(v.clone()));
        }
        for raw in ["0", "-7", "+1234"] {
            let v = SdmxInteger::new(raw.into()).unwrap();
            assert_eq!(v.to_string(), raw);
            assert_eq!(AsRef::<str>::as_ref(&v), raw);
            assert_eq!(v.to_string().parse::<SdmxInteger>(), Ok(v.clone()));
        }
        for raw in ["1.2.3", "1.0.0-rc.1", "1.3", "1"] {
            let v = SdmxVersion::new(raw.into()).unwrap();
            assert_eq!(v.to_string(), raw);
            assert_eq!(AsRef::<str>::as_ref(&v), raw);
            assert_eq!(v.to_string().parse::<SdmxVersion>(), Ok(v.clone()));
        }
        for raw in ["2024", "2024-05", "2024-Q4", "2024-05-01T09:30:00", "2024-05-01T09:30:00Z"] {
            let v = SdmxTimePeriod::new(raw.into()).unwrap();
            assert_eq!(v.to_string(), raw);
            assert_eq!(AsRef::<str>::as_ref(&v), raw);
            assert_eq!(v.to_string().parse::<SdmxTimePeriod>(), Ok(v.clone()));
        }
    }
}
