//! Validated lexical newtypes for SDMX's constrained value types.
//!
//! [`SdmxDecimal`], [`SdmxInteger`], [`SdmxVersion`], and [`SdmxTimePeriod`] wrap the SDMX
//! lexical types whose value space does not map losslessly onto a fixed Rust type. Each
//! validates its grammar at construction and round-trips its text exactly:
//! [`SdmxDecimal`], [`SdmxInteger`], [`SdmxTimePeriod`], and [`SdmxDuration`] store the
//! canonical lexeme verbatim and never rewrite it, while [`SdmxVersion`]'s canonical grammar
//! lets it hold only the parsed decomposition and reconstruct the lexeme on display.
//! [`VersionRef`] extends the family with the version *reference* grammar
//! (`WildcardVersionType`: `+` and `*` wildcards), raw-free on the same grounds.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Lexical-newtype convention: `new()` validates the grammar on the single write path so every
caller gets a well-formed value, and where validation naturally classifies the value the cheap
discriminant is retained (`SdmxVersion`'s parsed components, `SdmxTimePeriod`'s kind). Storage
forks on canonicity: a non-canonical grammar (several lexemes per value: `xs:decimal`,
`xs:integer`, the time-period union) keeps the lexeme as the lossless source of truth, while a
canonical grammar (`VersionType`) makes format-then-parse a bijection, so the decomposition alone
is lossless and no lexeme is stored.

`Ord`/`PartialOrd` for `SdmxVersion` are deliberately deferred, not resolved; see its design notes.

Decisions: D-0027, D-0070, D-0076.
"#
)]

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

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
source of truth (an `f64` would round; a fixed-width decimal would overflow). The grammar is not
canonical (`1.0` and `1.00` are distinct lexemes of equal value), so the lexeme is load-bearing,
in contrast to `SdmxVersion` (D-0070). No useful sub-kind, so it is a bare newtype.

Decisions: D-0027, D-0070.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

    /// Consumes the newtype, returning the inner string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

    /// Consumes the newtype, returning the inner string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
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

/// Unwraps to the inner canonical lexeme.
impl From<SdmxDecimal> for String {
    fn from(value: SdmxDecimal) -> Self {
        value.into_inner()
    }
}

/// Unwraps to the inner canonical lexeme.
impl From<SdmxInteger> for String {
    fn from(value: SdmxInteger) -> Self {
        value.into_inner()
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
/// The parsed components are read through the accessors and the canonical string is
/// reconstructed by [`Display`](core::fmt::Display): the grammar admits exactly one lexeme per
/// value, so the text is derived, not stored. `patch()` returning `None` distinguishes the
/// legacy form from the semantic form, and `minor()` returning `None` distinguishes the
/// bare-major legacy form (`"1"`) from its two-component sibling (`"1.0"`).
///
/// Equality is structural and distinguishes exactly what the lexemes distinguish, so `"3.1"` and
/// `"3.1.0"` are distinct. Ordering is not currently provided.
///
/// ## Guarantees
///
/// Round-trips losslessly through its text in both directions:
/// `x.to_string().parse::<SdmxVersion>() == Ok(x)` and
/// `s.parse::<SdmxVersion>()?.to_string() == s`.
///
/// # Examples
///
/// ```
/// use sdmx_types::SdmxVersion;
///
/// let version: SdmxVersion = "1.0.0-rc.1".parse()?;
/// assert_eq!(version.major(), 1);
/// assert_eq!(version.minor(), Some(0));
/// assert_eq!(version.extension(), Some("rc.1"));
/// assert!(!version.is_legacy());
/// assert_eq!(version.to_string(), "1.0.0-rc.1");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Raw-free by canonicity: every numeric component is `0|[1-9]\d*` (no leading zeros, no sign) and
the extension grammar admits one spelling per value, so format-then-parse is a bijection and a
stored lexeme would be a redundant second source of truth. This is the fork from
`SdmxDecimal`/`SdmxInteger`, whose grammars are not canonical (`1.0` vs `1.00`) and whose raw is
therefore load-bearing. Dropping the raw requires the decomposition to be statedness-preserving:
`minor` is `Option<u32>` so the bare-major legacy form does not collapse into `major.0`.
`major`/`minor`/`patch` are `u32` because the validated grammar is digits-only (no sign), so
unsigned loses nothing.

`PartialEq`/`Eq`/`Hash` derive structurally; by the bijection this is the same partition as
comparing lexemes. `AsRef<str>` is deliberately absent: there is no stored lexeme to borrow, and
rendering goes through `Display`.

Ordering is deliberately deferred, not resolved. SemVer §11 precedence is the intended basis, but
the legacy/semantic equivalence (for example `3.1` vs `3.1.0`) is undecided and premature to lock.
The likely shape is an explicit precedence-comparison convenience (a method or wrapper) rather
than an `Ord` impl on the type, so structural `Eq` and SemVer ordering can coexist without an
`Ord`/`Eq` contract: distinct under equality, equal under precedence.

Decisions: D-0027, D-0060, D-0070, D-0075.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SdmxVersion {
    major: u32,
    minor: Option<u32>,
    patch: Option<u32>,
    extension: Option<String>,
}

impl SdmxVersion {
    /// Validates `raw` against the `VersionType` union grammar and keeps the parsed
    /// decomposition; the lexeme is reconstructed on demand by
    /// [`Display`](core::fmt::Display).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidVersion`] if `raw` matches neither the semantic nor the
    /// legacy form (this includes leading zeros, empty components, an extension on a legacy
    /// version, and numeric components that exceed `u32`, the deliberate width bound recorded
    /// by D-0075).
    pub fn new(raw: String) -> Result<Self, Error> {
        parse_sdmx_version(raw)
    }

    /// The major component.
    #[must_use]
    pub const fn major(&self) -> u32 {
        self.major
    }

    /// The minor component, or `None` for the bare-major legacy form (for example `"1"`).
    #[must_use]
    pub const fn minor(&self) -> Option<u32> {
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

/// Reconstructs the canonical lexeme: the grammar admits exactly one spelling per value, so
/// the rendered text is the same string the value was parsed from.
impl core::fmt::Display for SdmxVersion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.major)?;
        if let Some(minor) = self.minor {
            write!(f, ".{minor}")?;
        }
        if let Some(patch) = self.patch {
            write!(f, ".{patch}")?;
        }
        if let Some(extension) = &self.extension {
            write!(f, "-{extension}")?;
        }
        Ok(())
    }
}

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
#[must_use]
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
// WildcardVersionType (version references)
// ---------------------------------------------------------------------------

/// The component of a semantic version triple that a [`VersionRef::Latest`] reference
/// wildcards with `+`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WildcardPosition {
    /// `+` on the major component: `2+.3.1`.
    Major,
    /// `+` on the minor component: `2.3+.1`.
    Minor,
    /// `+` on the patch component: `2.3.1+`.
    Patch,
}

/// An SDMX `WildcardVersionType`: the version grammar of a *reference*.
///
/// A reference admits what a declared [`SdmxVersion`] admits plus the wildcard forms:
/// `+` on exactly one component of a full semantic triple ("latest available"), or the
/// bare `*`.
///
/// ## Specification
/// - **Schema**: `SDMXCommonReferences.xsd`
/// - **Type**: `WildcardVersionType` (the union of `VersionReferenceType` and `WildcardType`; `SemanticVersionReferenceType` carries the `+` patterns)
/// - **Element**: N/A (Simple Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/WildcardVersionType.md"))]
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/VersionReferenceType.md"))]
#[cfg_attr(
    design_docs,
    doc = include_str!("../docs/xsd-fragments/SemanticVersionReferenceType.3.0.md")
)]
#[cfg_attr(
    design_docs,
    doc = include_str!("../docs/xsd-fragments/SemanticVersionReferenceType.3.1.md")
)]
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/WildcardType.md"))]
///
/// `2+.3.1` reads "the latest available version at least `2.3.1`" (per the schema
/// documentation, even if not backwards compatible). A `+` wildcard requires the full
/// `major.minor.patch` triple and cannot be combined with a prerelease extension
/// (`2.3+.1-draft` is rejected). Resolving a wildcard to a concrete version requires a
/// registry catalogue and is outside this type.
///
/// Like [`SdmxVersion`], the grammar admits one lexeme per value, so the text is
/// reconstructed by [`Display`](core::fmt::Display), not stored.
///
/// ## Guarantees
///
/// Round-trips losslessly through its text in both directions:
/// `x.to_string().parse::<VersionRef>() == Ok(x)` and
/// `s.parse::<VersionRef>()?.to_string() == s`.
///
/// # Examples
///
/// ```
/// use sdmx_types::{SdmxVersion, VersionRef, WildcardPosition};
///
/// let latest: VersionRef = "2+.3.1".parse()?;
/// assert_eq!(
///     latest,
///     VersionRef::Latest { major: 2, minor: 3, patch: 1, at: WildcardPosition::Major }
/// );
/// assert_eq!(latest.to_string(), "2+.3.1");
///
/// let exact: VersionRef = "1.0".parse()?;
/// assert_eq!(exact, VersionRef::Exact("1.0".parse::<SdmxVersion>()?));
///
/// assert_eq!("*".parse::<VersionRef>()?, VersionRef::Any);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The spec separates the declaration version grammar (`VersionType`, exact, on
`VersionableType`) from the reference grammars: `VersionReferenceType` adds the
single-`+` semantic forms and `WildcardVersionType` adds the bare `*`. `VersionRef`
models the widest reference union; `SdmxVersion` stays the declaration type. Which
reference contexts admit `*` versus only `+` is settled by the URN-contract pass, not
here.

Exactly one `+` is enforced across both editions. 3.1's three patterns each admit one
position; 3.0's third pattern is mechanically looser (it also matches `1+.2.3+`), but
the same type's documentation in both editions states that only one part may be
wildcarded, so the model follows the documented contract rather than carrying the 3.0
regex slack as a superset member. The full-triple and no-extension requirements are
mechanical in both editions' patterns, and the enum makes them unrepresentable:
`Latest` has three mandatory `u32` components and no extension slot.

Raw-free on the same canonicity grounds as `SdmxVersion` (one lexeme per value; see
the D-0070 fork). Both enums are exhaustive: the union is grammar-closed. `1.*` and
`1.0.*` are `VersionQueryType` (registry-query grammar, a different message family)
and are deliberately rejected here.

Decisions: D-0069, D-0070, D-0071.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VersionRef {
    /// A reference to exactly this version: `VersionReferenceType`'s legacy and
    /// semantic member grammars, identical to a declared [`SdmxVersion`].
    Exact(SdmxVersion),
    /// A "latest available" reference: a full semantic triple with `+` on the
    /// component named by `at`. `{ major: 2, minor: 3, patch: 1, at: Major }` renders
    /// as `2+.3.1`.
    Latest {
        /// The major component (the stated lower bound when `at` is [`WildcardPosition::Major`]).
        major: u32,
        /// The minor component.
        minor: u32,
        /// The patch component.
        patch: u32,
        /// Which component carries the `+`.
        at: WildcardPosition,
    },
    /// The bare `*` (`WildcardType`): any version of the referenced artefact.
    Any,
}

impl VersionRef {
    /// Validates `raw` against the `WildcardVersionType` union grammar; the lexeme is
    /// reconstructed on demand by [`Display`](core::fmt::Display).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidVersionReference`] if `raw` is neither a valid exact
    /// version, a full semantic triple with `+` on exactly one component (and no
    /// extension), nor the bare `*`.
    pub fn new(raw: String) -> Result<Self, Error> {
        parse_version_ref(raw)
    }
}

/// Reconstructs the canonical lexeme, as for [`SdmxVersion`].
impl core::fmt::Display for VersionRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Exact(version) => write!(f, "{version}"),
            Self::Latest { major, minor, patch, at } => {
                let wild = |p| if *at == p { "+" } else { "" };
                write!(
                    f,
                    "{major}{}.{minor}{}.{patch}{}",
                    wild(WildcardPosition::Major),
                    wild(WildcardPosition::Minor),
                    wild(WildcardPosition::Patch)
                )
            }
            Self::Any => f.write_str("*"),
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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
// TimeRangeType and ObservationalTimePeriodType
// ---------------------------------------------------------------------------

/// An SDMX `TimeRangeType`: a start date or date-time plus a duration, as `start/duration`.
///
/// ## Specification
/// - **Schema**: `SDMXCommon.xsd`
/// - **Type**: `TimeRangeType`
/// - **Element**: N/A (Simple Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/TimeRangeType.md"))]
///
/// The start is a full `xs:date` or `xs:dateTime` (an optional timezone is permitted on
/// either); the duration is an `xs:duration` with its components in order and at least one
/// present. The canonical string is stored verbatim, and the two halves are exposed through
/// [`start()`](Self::start) and [`duration()`](Self::duration).
///
/// ## Guarantees
///
/// Round-trips losslessly through its text: `x.to_string().parse::<SdmxTimeRange>() == Ok(x)`.
///
/// # Examples
///
/// ```
/// use sdmx_types::SdmxTimeRange;
///
/// let range: SdmxTimeRange = "2010-01-01/P2M".parse()?;
/// assert_eq!(range.start(), "2010-01-01");
/// assert_eq!(range.duration(), "P2M");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Lexical newtype with lossless `String` storage: date and duration lexemes are not canonical
(timezone spellings, fractional seconds), so the raw is load-bearing (the D-0070 fork). The `/`
separator occurs exactly once (neither half's grammar admits one), so the accessors are cheap
slices. The start half is validated with the shared Gregorian/date-time classifier, the same
strictness the crate applies to `xs:date`/`xs:dateTime` everywhere (the XSD chain's month-length
and leap-year patterns are not re-implemented); the duration is the chain's ordered-component
grammar. The name carries the `Sdmx` prefix because bare `TimeRange` collides with the constraint
selection type (D-0027 naming rule).

Decisions: D-0027, D-0072.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SdmxTimeRange {
    raw: String,
}

impl SdmxTimeRange {
    /// Validates `raw` against the `TimeRangeType` grammar and stores it verbatim.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTimeRange`] if `raw` is not `start/duration` with a full
    /// `xs:date` or `xs:dateTime` start and a non-empty, ordered `xs:duration`.
    pub fn new(raw: String) -> Result<Self, Error> {
        if is_time_range(&raw) { Ok(Self { raw }) } else { Err(Error::InvalidTimeRange(raw)) }
    }

    /// The canonical time-range string, exactly as supplied.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// The start half: a full `xs:date` or `xs:dateTime`, with any stated timezone.
    #[must_use]
    pub fn start(&self) -> &str {
        self.raw.split_once('/').map_or(self.raw.as_str(), |(start, _)| start)
    }

    /// The duration half: an `xs:duration` such as `P2M` or `PT30M`.
    #[must_use]
    pub fn duration(&self) -> &str {
        self.raw.split_once('/').map_or("", |(_, duration)| duration)
    }

    /// Consumes the newtype, returning the inner string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.raw
    }
}

/// Unwraps to the inner canonical lexeme.
impl From<SdmxTimeRange> for String {
    fn from(value: SdmxTimeRange) -> Self {
        value.into_inner()
    }
}

/// An SDMX `ObservationalTimePeriodType`: a standard time period or a time range.
///
/// ## Specification
/// - **Schema**: `SDMXCommon.xsd`
/// - **Type**: `ObservationalTimePeriodType` (the union of `StandardTimePeriodType` and `TimeRangeType`)
/// - **Element**: N/A (Simple Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ObservationalTimePeriodType.md"))]
///
/// The widest time-period vocabulary in SDMX: every [`SdmxTimePeriod`] lexeme plus the
/// [`SdmxTimeRange`] start-and-duration form. The two member grammars are disjoint (only a
/// time range contains `/`), so classification is unambiguous.
///
/// ## Guarantees
///
/// Round-trips losslessly through its text:
/// `x.to_string().parse::<ObservationalTimePeriod>() == Ok(x)`.
///
/// # Examples
///
/// ```
/// use sdmx_types::ObservationalTimePeriod;
///
/// let standard: ObservationalTimePeriod = "2024-Q4".parse()?;
/// assert!(matches!(standard, ObservationalTimePeriod::Standard(_)));
///
/// let range: ObservationalTimePeriod = "2010-01-01/P2M".parse()?;
/// assert!(matches!(range, ObservationalTimePeriod::Range(_)));
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A union of the member newtypes rather than a widened `SdmxTimePeriod`: `StandardTimePeriodType`
positions (`TimeRange.valid_from`/`valid_to`, the constraint validity pairs) must keep rejecting
time-range lexemes, so the union is its own type and `SdmxTimePeriod` stays exactly the standard
grammar. Exhaustive: the union is grammar-closed. Both members store their lexeme, so the union
exposes `as_str()`/`AsRef<str>` like the other lexeme-storing newtypes.

Decisions: D-0064, D-0072.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ObservationalTimePeriod {
    /// A standard time period: a Gregorian period, an `xs:dateTime`, or a reporting period.
    Standard(SdmxTimePeriod),
    /// A time range: a start date or date-time plus a duration.
    Range(SdmxTimeRange),
}

impl ObservationalTimePeriod {
    /// Validates `raw` against the `ObservationalTimePeriodType` union and classifies it
    /// into the matching member.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidObservationalTimePeriod`] if `raw` is neither a standard
    /// time period nor a time range.
    pub fn new(raw: String) -> Result<Self, Error> {
        let parsed = if raw.contains('/') {
            SdmxTimeRange::new(raw).map(Self::Range)
        } else {
            SdmxTimePeriod::new(raw).map(Self::Standard)
        };
        parsed.map_err(|e| match e {
            Error::InvalidTimeRange(raw) | Error::InvalidTimePeriod(raw) => {
                Error::InvalidObservationalTimePeriod(raw)
            }
            other => other,
        })
    }

    /// The canonical lexeme, exactly as supplied.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Standard(period) => period.as_str(),
            Self::Range(range) => range.as_str(),
        }
    }
}

// ---------------------------------------------------------------------------
// xs:dateTime
// ---------------------------------------------------------------------------

/// An SDMX `xs:dateTime` value, stored losslessly as its stated lexeme.
///
/// ## Specification
/// - **Schema**: W3C XML Schema (`xs`)
/// - **Type**: `xs:dateTime`
/// - **Element**: N/A (Primitive)
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Carries the artefact validity windows (`VersionableType.validFrom`/`validTo`), whose timezone
/// is optional and unrestricted in both editions. The stored text is the datum: it is validated
/// at construction, never rewritten, and round-trips verbatim, so a schema-valid offsetless value
/// and the `Z` and `+00:00` spellings all survive. Equality and hashing are lexeme identity. The
/// parsed date-time and stated offset are retained as a cheap derived discriminant and exposed
/// through value accessors ([`date_time`](Self::date_time), [`offset`](Self::offset)); instant
/// comparison is the explicit [`instant`](Self::instant) view, never `Eq`.
///
/// ## Guarantees
///
/// Round-trips losslessly through its text: `x.to_string().parse::<SdmxDateTime>() == Ok(x)`.
///
/// # Examples
///
/// ```
/// use sdmx_types::SdmxDateTime;
///
/// let dt: SdmxDateTime = "2024-05-01T09:30:00+05:00".parse()?;
/// assert_eq!(dt.as_str(), "2024-05-01T09:30:00+05:00");
/// assert!(dt.offset().is_some());
///
/// // An offsetless lexeme is schema-valid and round-trips verbatim.
/// let naive: SdmxDateTime = "2024-05-01T09:30:00".parse()?;
/// assert_eq!(naive.as_str(), "2024-05-01T09:30:00");
/// assert!(naive.offset().is_none());
///
/// // `Z` and `+00:00` are the same instant but distinct lexemes, so distinct values.
/// let zulu: SdmxDateTime = "2024-05-01T09:30:00Z".parse()?;
/// let plus: SdmxDateTime = "2024-05-01T09:30:00+00:00".parse()?;
/// assert_ne!(zulu, plus);
/// assert_eq!(zulu.instant(), plus.instant());
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Lexical newtype with lossless `String` storage (D-0079): under the supported schemas' XSD 1.0
value model the `dateTime` value is the timeline instant alone, so the stated offset is beyond-value
content, and the offsetless spelling is schema-valid; both survive only in the raw form, which is
therefore load-bearing (the D-0070 fork). Equality and hashing derive over the raw lexeme plus the
derived fields, but the derived fields are a deterministic function of the raw, so the relation is
exactly lexeme identity, the same shape as `SdmxTimePeriod`'s `(raw, kind)`. The parsed date-time and
stated offset are the retained cheap discriminant behind the value accessors; instant comparison is
the explicit `instant()` view, never `Eq`. The name carries the `Sdmx` prefix because bare `DateTime`
collides with `chrono` (D-0027 naming rule). Construction reuses the shared date-time grammar rather
than re-implementing it, so its edge decisions (hour-24, year zero, leading-zero years, offset bounds)
are shared with `SdmxTimePeriod` (D-0076).

Decisions: D-0027, D-0074, D-0076, D-0079.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SdmxDateTime {
    // The stated lexeme: the round-trip source of truth and the identity datum.
    raw: String,
    // Derived at construction (D-0079): the written date-time, offset-independent. `None` only for
    // the grammar-admitted forms chrono cannot represent (a calendar-invalid day such as
    // `2024-02-30`, which the shared classifier admits by design, or a year outside chrono's range).
    date_time: Option<NaiveDateTime>,
    // The stated timezone: `None` for a schema-valid offsetless lexeme, `Some` otherwise.
    offset: Option<FixedOffset>,
}

impl SdmxDateTime {
    /// Validates `raw` against the `xs:dateTime` grammar and stores it verbatim, retaining the
    /// parsed date-time and stated offset as the derived discriminant.
    ///
    /// The timezone is optional; the shared date-time grammar's edge decisions (hour-24
    /// end-of-day, year zero rejected, leading-zero years, `±14:00` offset bounds) apply.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidDateTime`] if `raw` is not a full `xs:date`, `T`, `hh:mm:ss`
    /// with an optional fractional-seconds suffix and an optional timezone.
    pub fn new(raw: String) -> Result<Self, Error> {
        if classify_time_period(&raw) != Some(SdmxTimePeriodKind::DateTime) {
            return Err(Error::InvalidDateTime(raw));
        }
        let (date_time, offset) = parse_xs_date_time(&raw);
        Ok(Self { raw, date_time, offset })
    }

    /// The stated `xs:dateTime` string, exactly as supplied.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// The written date-time, offset-independent (the local wall-clock reading).
    ///
    /// Two value-view adjustments apply here while the raw lexeme stays verbatim (D-0079): an
    /// end-of-day `24:00:00` is normalised to `00:00:00` on the following day (XSD's own
    /// mapping; `chrono` rejects hour 24), and a fractional-seconds part beyond nanosecond
    /// precision is truncated. Returns `None` for a grammar-admitted lexeme `chrono` cannot
    /// represent (a calendar-invalid day such as `2024-02-30`, or an out-of-range year), which
    /// schema-valid wire never carries. A `None` here is the lint signal for such a stored
    /// lexeme — a catalogued Layer-2 lint, not a construction error (D-0031).
    #[must_use]
    pub const fn date_time(&self) -> Option<NaiveDateTime> {
        self.date_time
    }

    /// The stated timezone offset, or `None` for a schema-valid offsetless lexeme. The offset is
    /// data: two windows that state different offsets are distinct even at the same instant.
    #[must_use]
    pub const fn offset(&self) -> Option<FixedOffset> {
        self.offset
    }

    /// The instant this window denotes, present only when an offset is stated (an offsetless
    /// lexeme fixes no point on the timeline). This is the explicit instant view: comparing two
    /// values' instants tests same-moment equality, which `Eq` deliberately does not.
    #[must_use]
    pub fn instant(&self) -> Option<DateTime<FixedOffset>> {
        let date_time = self.date_time?;
        let offset = self.offset?;
        offset.from_local_datetime(&date_time).single()
    }
}

// ---------------------------------------------------------------------------
// xs:duration
// ---------------------------------------------------------------------------

/// An SDMX `xs:duration` value, stored losslessly as its canonical lexeme.
///
/// ## Specification
/// - **Schema**: W3C XML Schema (`xs`)
/// - **Type**: `xs:duration`
/// - **Element**: N/A (Primitive)
/// - **Editions**: SDMX 3.0 and 3.1
///
/// A calendar-and-clock duration (`-P2M`, `PT30M`, `P1Y2M3DT4H5M6.5S`): an optional leading `-`,
/// `P`, then ordered date components and an optional `T` with ordered time components, at least
/// one component overall, the fraction admitted only on seconds. The value is validated when
/// constructed and never rewritten, so it round-trips verbatim. Carried by the format facets'
/// `timeInterval` ([`TextFormat`](crate::TextFormat), [`EnumerationFormat`](crate::EnumerationFormat)).
///
/// ## Guarantees
///
/// Round-trips losslessly through its text: `x.to_string().parse::<SdmxDuration>() == Ok(x)`.
///
/// # Examples
///
/// ```
/// use sdmx_types::SdmxDuration;
///
/// let duration: SdmxDuration = "P1Y2M".parse()?;
/// assert_eq!(duration.as_str(), "P1Y2M");
///
/// // xs:duration admits a leading sign, unlike the TimeRangeType duration half.
/// let negative: SdmxDuration = "-PT30M".parse()?;
/// assert_eq!(negative.as_str(), "-PT30M");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Lexical newtype with lossless `String` storage: the grammar is not canonical (`P1M` and `P01M`
are distinct lexemes of equal value; calendar components have no faithful numeric normalisation),
so the lexeme is load-bearing (the D-0070 fork). The validator reuses the ordered-component
scanner behind `SdmxTimeRange`'s duration half and adds the optional leading `-` that plain
`xs:duration` admits and the `TimeRangeType` chain does not; the two grammars stay distinct.
No useful sub-kind, so it is a bare newtype.

Decisions: D-0027, D-0076.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SdmxDuration(String);

impl SdmxDuration {
    /// Validates `s` against the `xs:duration` lexical grammar and stores it verbatim.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidDuration`] if `s` is not a valid `xs:duration` lexeme: an
    /// optional leading `-`, `P`, ordered date components, an optional `T` with ordered time
    /// components, at least one component overall, the fraction only on seconds.
    pub fn new(s: String) -> Result<Self, Error> {
        if is_xs_duration(&s) { Ok(Self(s)) } else { Err(Error::InvalidDuration(s)) }
    }

    /// The canonical lexeme, exactly as supplied.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the newtype, returning the inner string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Unwraps to the inner canonical lexeme.
impl From<SdmxDuration> for String {
    fn from(value: SdmxDuration) -> Self {
        value.into_inner()
    }
}

// ---------------------------------------------------------------------------
// Standard trait impls (Display / FromStr / AsRef): uniform across the lexeme newtypes.
// `new(String)` stays the owned, no-clone entry; `from_str(&str)` is the borrowed `.parse()`
// path that clones into ownership.
//
// `PartialEq<str>`/`PartialEq<&str>` sit only on the lexeme-storing types — the raw-backed
// `SdmxDecimal`/`SdmxInteger`/`SdmxTimePeriod`/`SdmxTimeRange` and the
// `ObservationalTimePeriod` union of the latter two — whose stored lexeme is the datum, so
// string identity is the type's defined equality (D-0027). The raw-free grammar types
// (`SdmxVersion`, `VersionRef`) take no string comparison operator: `version == "1.0.0"`
// reads two ways (lexeme identity vs SemVer equivalence), the same contested shape that
// deferred `Ord` (D-0060).
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

/// String identity with the stored lexeme: compares the verbatim raw form, never a numeric
/// view, so `"1.0"` and `"1.00"` compare unequal (D-0027 lossless-distinct).
impl PartialEq<str> for SdmxDecimal {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

/// String identity with the stored lexeme, for the borrowed literal form.
impl PartialEq<&str> for SdmxDecimal {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
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

/// String identity with the stored lexeme: compares the verbatim raw form, never a numeric
/// view, so `"7"` and `"+7"` compare unequal (D-0027 lossless-distinct).
impl PartialEq<str> for SdmxInteger {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

/// String identity with the stored lexeme, for the borrowed literal form.
impl PartialEq<&str> for SdmxInteger {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

// `SdmxVersion` and `VersionRef` already implement `Display` (above); only `FromStr` is added
// here. `AsRef<str>` is deliberately absent on both: the lexeme is reconstructed, not stored,
// so there is nothing to borrow.
impl core::str::FromStr for SdmxVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl core::str::FromStr for VersionRef {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
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

/// String identity with the stored lexeme: compares the verbatim raw form, never a
/// normalised view, so timezone spellings and equivalent periods stay distinct (D-0027).
impl PartialEq<str> for SdmxTimePeriod {
    fn eq(&self, other: &str) -> bool {
        self.raw == other
    }
}

/// String identity with the stored lexeme, for the borrowed literal form.
impl PartialEq<&str> for SdmxTimePeriod {
    fn eq(&self, other: &&str) -> bool {
        self.raw == *other
    }
}

impl core::fmt::Display for SdmxTimeRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.raw)
    }
}

impl core::str::FromStr for SdmxTimeRange {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl AsRef<str> for SdmxTimeRange {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

/// String identity with the stored lexeme: compares the verbatim raw form, never a
/// normalised view, so equivalent date and duration spellings stay distinct (D-0027).
impl PartialEq<str> for SdmxTimeRange {
    fn eq(&self, other: &str) -> bool {
        self.raw == other
    }
}

/// String identity with the stored lexeme, for the borrowed literal form.
impl PartialEq<&str> for SdmxTimeRange {
    fn eq(&self, other: &&str) -> bool {
        self.raw == *other
    }
}

impl core::fmt::Display for ObservationalTimePeriod {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::str::FromStr for ObservationalTimePeriod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl AsRef<str> for ObservationalTimePeriod {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// String identity with the stored member lexeme: both union members store their text
/// verbatim, and the member grammars are disjoint, so string identity coincides with
/// structural equality (D-0027).
impl PartialEq<str> for ObservationalTimePeriod {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

/// String identity with the stored member lexeme, for the borrowed literal form.
impl PartialEq<&str> for ObservationalTimePeriod {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl core::fmt::Display for SdmxDateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.raw)
    }
}

impl core::str::FromStr for SdmxDateTime {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl AsRef<str> for SdmxDateTime {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

/// String identity with the stored lexeme: compares the verbatim raw form, never a normalised
/// view, so `Z` and `+00:00` and fractional-second padding stay distinct (D-0074, D-0079).
impl PartialEq<str> for SdmxDateTime {
    fn eq(&self, other: &str) -> bool {
        self.raw == other
    }
}

/// String identity with the stored lexeme, for the borrowed literal form.
impl PartialEq<&str> for SdmxDateTime {
    fn eq(&self, other: &&str) -> bool {
        self.raw == *other
    }
}

impl core::fmt::Display for SdmxDuration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

impl core::str::FromStr for SdmxDuration {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl AsRef<str> for SdmxDuration {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// String identity with the stored lexeme: compares the verbatim raw form, never a
/// normalised view, so `"P1M"` and `"P01M"` compare unequal (D-0027 lossless-distinct).
impl PartialEq<str> for SdmxDuration {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

/// String identity with the stored lexeme, for the borrowed literal form.
impl PartialEq<&str> for SdmxDuration {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
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
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for SdmxVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(to_de_error)
    }
}

impl serde::Serialize for VersionRef {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for VersionRef {
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

impl serde::Serialize for SdmxTimeRange {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.raw)
    }
}

impl<'de> serde::Deserialize<'de> for SdmxTimeRange {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(to_de_error)
    }
}

impl serde::Serialize for ObservationalTimePeriod {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for ObservationalTimePeriod {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(to_de_error)
    }
}

impl serde::Serialize for SdmxDateTime {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.raw)
    }
}

impl<'de> serde::Deserialize<'de> for SdmxDateTime {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(to_de_error)
    }
}

impl serde::Serialize for SdmxDuration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for SdmxDuration {
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
                (Some(maj), Some(min), Some(pat)) => (maj, Some(min), Some(pat)),
                _ => return Err(Error::InvalidVersion(raw)),
            }
        }
        // Legacy: one or two numeric components, no extension permitted.
        ([maj], None) => match parse_u32(maj) {
            Some(maj) => (maj, None, None),
            None => return Err(Error::InvalidVersion(raw)),
        },
        ([maj, min], None) => match (parse_u32(maj), parse_u32(min)) {
            (Some(maj), Some(min)) => (maj, Some(min), None),
            _ => return Err(Error::InvalidVersion(raw)),
        },
        _ => return Err(Error::InvalidVersion(raw)),
    };

    let extension = extension.map(ToString::to_string);
    Ok(SdmxVersion { major, minor, patch, extension })
}

/// Parses and validates the `WildcardVersionType` union.
fn parse_version_ref(raw: String) -> Result<VersionRef, Error> {
    if raw == "*" {
        return Ok(VersionRef::Any);
    }
    if !raw.contains('+') {
        // No wildcard: exactly the declaration grammar.
        return match parse_sdmx_version(raw) {
            Ok(version) => Ok(VersionRef::Exact(version)),
            Err(Error::InvalidVersion(s)) => Err(Error::InvalidVersionReference(s)),
            Err(other) => Err(other),
        };
    }

    // Wildcarded: a full semantic triple with `+` on exactly one component. The
    // extension is mutually exclusive with wildcarding, so no `-` can appear.
    if raw.contains('-') {
        return Err(Error::InvalidVersionReference(raw));
    }
    let split = |c: &str| -> Option<(u32, bool)> {
        let (digits, wild) = c.strip_suffix('+').map_or((c, false), |digits| (digits, true));
        if is_numeric_component(digits) { Some((digits.parse().ok()?, wild)) } else { None }
    };
    let components: Vec<&str> = raw.split('.').collect();
    let &[maj, min, pat] = components.as_slice() else {
        return Err(Error::InvalidVersionReference(raw));
    };
    let (Some((major, major_wild)), Some((minor, minor_wild)), Some((patch, patch_wild))) =
        (split(maj), split(min), split(pat))
    else {
        return Err(Error::InvalidVersionReference(raw));
    };
    let at = match (major_wild, minor_wild, patch_wild) {
        (true, false, false) => WildcardPosition::Major,
        (false, true, false) => WildcardPosition::Minor,
        (false, false, true) => WildcardPosition::Patch,
        // Zero or several `+`: outside the documented one-wildcard contract.
        _ => return Err(Error::InvalidVersionReference(raw)),
    };
    Ok(VersionRef::Latest { major, minor, patch, at })
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
    // from every basic form (whose offset-5 byte, when present, is a digit or `T`). The
    // reporting year is pinned to exactly four digits here (offsets 0..4, `-` at offset 4),
    // matching `BaseReportPeriodType`'s `\d{4}` and unlike the unbounded `xs:gYear` that
    // `is_year` admits (four or more): a five-digit run is not a reporting period, so it
    // falls through to the Gregorian path. The asymmetry is deliberate; do not unify it.
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
    // YYYY '-' <letter> <number>: the fixed offsets 5 (designator) and 6 (number) encode the
    // four-digit reporting year `BaseReportPeriodType` pins (`\d{4}`), the same asymmetry
    // against the unbounded `xs:gYear` that the caller's disambiguation guard enforces.
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
        // Day-of-month is range-checked `1..=31` for every month, so `2024-02-30` is
        // accepted: the XSD chain's month-length and leap-year validity is deliberately not
        // re-implemented (the `SdmxTimeRange` design note). This is not superset preservation
        // (schema-valid wire can never carry a calendar-invalid date, so admitting one round-
        // trips nothing); it is the one boundary this classifier leaves to calendar-aware code.
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
    let (hms, fraction) = match time.split_once('.') {
        Some((hms, fraction)) => {
            if fraction.is_empty() || !fraction.bytes().all(|b| b.is_ascii_digit()) {
                return false;
            }
            (hms, Some(fraction))
        }
        None => (time, None),
    };
    let mut parts = hms.split(':');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(h), Some(m), Some(s), None) => {
            // Seconds are `0..=59`: a leap-second `60` is rejected, matching `xs:dateTime`,
            // which does not admit it (a detail often assumed the other way).
            //
            // Hour 24 is the end-of-day instant: XSD 1.0 `xs:dateTime` (§3.2.7) admits `24:00:00`
            // only with zero minutes and seconds, and an all-zero fraction if one is stated; every
            // other hour-24 form is out of the value space (D-0027 lexical family; D-0069/D-0072
            // time chain). Requiring the seconds field to be exactly `00` keeps the relaxation from
            // swallowing an out-of-range second, so `24:00:60` is still rejected.
            if h == "24" {
                return m == "00"
                    && s == "00"
                    && fraction.is_none_or(|f| f.bytes().all(|b| b == b'0'));
            }
            num_in(h, 2, 0, 23) && num_in(m, 2, 0, 59) && num_in(s, 2, 0, 59)
        }
        _ => false,
    }
}

/// A four-or-more digit calendar year (`xs:gYear` component), excluding year zero and a leading
/// zero once the year exceeds four digits.
fn is_year(s: &str) -> bool {
    // XSD 1.0 (§3.2.7) constrains the year across the builtin date/time types (`xs:gYear`,
    // `xs:gYearMonth`, `xs:date`, `xs:dateTime`): year `0000` is prohibited, so an all-zero run is
    // rejected, and leading zeros are prohibited once the year exceeds four digits, so a longer run
    // may not start with `0`. This is the one choke point every Gregorian/basic year reaches,
    // including the `xs:dateTime` date and the `TimeRangeType` starts (D-0069/D-0072 time chain;
    // D-0027 lexical family). Reporting-period years never reach this: their grammar is the `\d{4}`
    // pattern (`BaseReportPeriodType`), which admits `0000`, and `classify_reporting` scans them by
    // fixed offset.
    s.len() >= 4
        && s.bytes().all(|b| b.is_ascii_digit())
        && s.bytes().any(|b| b != b'0')
        && (s.len() == 4 || !s.starts_with('0'))
}

/// True iff `s` is exactly `width` ASCII digits whose value lies in the inclusive range
/// `lo..=hi`.
fn num_in(s: &str, width: usize, lo: u32, hi: u32) -> bool {
    s.len() == width
        && s.bytes().all(|b| b.is_ascii_digit())
        && s.parse::<u32>().is_ok_and(|v| (lo..=hi).contains(&v))
}

/// Parses an already-validated `xs:dateTime` lexeme into its derived discriminant: the written
/// date-time (offset-independent) and the stated timezone. Infallible in the timezone (the grammar
/// range-checks it); the date-time is `Option` because the shared grammar admits forms `chrono`
/// cannot represent (a calendar-invalid day, an out-of-range year), which schema-valid wire never
/// carries. The raw lexeme is the source of truth; these are the cheap derived views (D-0079).
fn parse_xs_date_time(raw: &str) -> (Option<NaiveDateTime>, Option<FixedOffset>) {
    let (core, timezone) = split_timezone(raw);
    let offset = match timezone {
        None => None,
        Some("Z") => FixedOffset::east_opt(0),
        Some(numeric) => parse_numeric_offset(numeric),
    };
    let date_time = core.split_once('T').and_then(|(date, time)| build_naive_date_time(date, time));
    (date_time, offset)
}

/// Parses a `±hh:mm` timezone (already range-validated) into a [`FixedOffset`].
fn parse_numeric_offset(tz: &str) -> Option<FixedOffset> {
    let sign = match tz.as_bytes().first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let hours: i32 = tz.get(1..3)?.parse().ok()?;
    let minutes: i32 = tz.get(4..6)?.parse().ok()?;
    FixedOffset::east_opt(sign * (hours * 3600 + minutes * 60))
}

/// Builds the written [`NaiveDateTime`] from an already-validated `xs:date` and time body. Applies
/// the two value-view adjustments (D-0079): the end-of-day `24:00:00` maps to `00:00:00` on the
/// following day (XSD's own mapping, which `chrono` rejects), and a fraction beyond nanosecond
/// precision is truncated. Returns `None` for a day or year `chrono` cannot represent.
fn build_naive_date_time(date: &str, time: &str) -> Option<NaiveDateTime> {
    let (year, month, day) = parse_ymd(date)?;
    let (hour, minute, second, nanos) = parse_hms(time)?;
    let calendar_day = NaiveDate::from_ymd_opt(year, month, day)?;
    let (calendar_day, hour) =
        if hour == 24 { (calendar_day.succ_opt()?, 0) } else { (calendar_day, hour) };
    let clock = NaiveTime::from_hms_nano_opt(hour, minute, second, nanos)?;
    Some(NaiveDateTime::new(calendar_day, clock))
}

/// Splits an already-validated `xs:date` body (`[-]YYYY-MM-DD`) into numeric components. A leading
/// `-` denotes a BC year, carried as a negative year.
fn parse_ymd(date: &str) -> Option<(i32, u32, u32)> {
    let (negative, body) = date.strip_prefix('-').map_or((false, date), |rest| (true, rest));
    let mut parts = body.split('-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((if negative { -year } else { year }, month, day))
}

/// Splits an already-validated time body (`hh:mm:ss[.fraction]`) into numeric components, the
/// fraction converted to nanoseconds. Hour `24` is passed through for the caller's end-of-day
/// mapping.
fn parse_hms(time: &str) -> Option<(u32, u32, u32, u32)> {
    let (hms, fraction) = time.split_once('.').map_or((time, None), |(h, f)| (h, Some(f)));
    let mut parts = hms.split(':');
    let hour: u32 = parts.next()?.parse().ok()?;
    let minute: u32 = parts.next()?.parse().ok()?;
    let second: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((hour, minute, second, fraction.map_or(0, nanos_from_fraction)))
}

/// Converts an `xs:dateTime` fractional-seconds part (validated digits) to nanoseconds, truncating
/// beyond nanosecond precision and right-padding shorter fractions to nine digits.
fn nanos_from_fraction(fraction: &str) -> u32 {
    let mut nanos = 0_u32;
    let mut digits = fraction.bytes();
    for _ in 0..9 {
        nanos = nanos * 10 + u32::from(digits.next().map_or(0, |b| b - b'0'));
    }
    nanos
}

/// `TimeRangeType`: `start/duration` where the start is a full `xs:date` or `xs:dateTime`
/// (optional timezone) and the duration is a non-empty, ordered `xs:duration`.
fn is_time_range(s: &str) -> bool {
    let Some((start, duration)) = s.split_once('/') else {
        return false;
    };
    matches!(
        classify_time_period(start),
        Some(SdmxTimePeriodKind::GregorianDay | SdmxTimePeriodKind::DateTime)
    ) && is_time_range_duration(duration)
}

/// The `TimeRangeType` duration grammar: `P` then ordered optional `nY nM nD`, then an
/// optional `T` with ordered optional `nH nM n[.n]S`; at least one component overall and at
/// least one after any `T` (per the chain's `.+/P.+` and `T.+` requirements).
fn is_time_range_duration(d: &str) -> bool {
    let Some(body) = d.strip_prefix('P') else {
        return false;
    };
    if body.is_empty() {
        return false;
    }
    let (date, time) = match body.split_once('T') {
        Some((date, time)) => (date, Some(time)),
        None => (body, None),
    };
    if !date.is_empty() && !scan_duration_components(date, &['Y', 'M', 'D'], None) {
        return false;
    }
    time.map_or(!date.is_empty(), |time| {
        !time.is_empty() && scan_duration_components(time, &['H', 'M', 'S'], Some('S'))
    })
}

/// Plain `xs:duration`: the ordered-component grammar with the optional leading `-` the W3C
/// type admits (and the `TimeRangeType` chain's duration half does not).
fn is_xs_duration(s: &str) -> bool {
    is_time_range_duration(s.strip_prefix('-').unwrap_or(s))
}

/// A run of `<digits><unit>` components drawn from `units` in order, each unit at most once;
/// a fractional part (`.digits`) is admitted only immediately before `fraction_unit`.
fn scan_duration_components(part: &str, units: &[char], fraction_unit: Option<char>) -> bool {
    let mut rest = part;
    let mut next_units = units;
    while !rest.is_empty() {
        let digits_end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        if digits_end == 0 {
            return false;
        }
        let mut after = &rest[digits_end..];
        let has_fraction = if let Some(fraction) = after.strip_prefix('.') {
            let fraction_end =
                fraction.find(|c: char| !c.is_ascii_digit()).unwrap_or(fraction.len());
            if fraction_end == 0 {
                return false;
            }
            after = &fraction[fraction_end..];
            true
        } else {
            false
        };
        let Some(unit) = after.chars().next() else {
            return false;
        };
        let Some(position) = next_units.iter().position(|&u| u == unit) else {
            return false;
        };
        if has_fraction && Some(unit) != fraction_unit {
            return false;
        }
        next_units = &next_units[position + 1..];
        rest = &after[1..];
    }
    true
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn decimal_accepts_and_rejects() {
        for ok in ["0", "-1.5", "+42", "3.14159", ".5", "5.", "-0.0"] {
            assert!(SdmxDecimal::new(ok.into()).is_ok(), "{ok:?} should be a valid xs:decimal");
        }
        for bad in ["", ".", "+", "1.2.3", "1e5", "abc", "1,5"] {
            assert!(SdmxDecimal::new(bad.into()).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
        let i = SdmxInteger::new(String::from("42")).unwrap();
        let d: SdmxDecimal = i.into();
        assert_eq!(d.as_str(), "42");
        assert_eq!(SdmxInteger::try_from(d).unwrap().as_str(), "42");
        // "42.0" is integral in value but not an integer lexeme.
        let fractional = SdmxDecimal::new(String::from("42.0")).unwrap();
        assert!(SdmxInteger::try_from(fractional).is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn version_semantic_and_legacy() {
        let semantic = SdmxVersion::new(String::from("1.2.3")).unwrap();
        assert_eq!(
            (semantic.major(), semantic.minor(), semantic.patch(), semantic.is_legacy()),
            (1, Some(2), Some(3), false)
        );
        let prerelease = SdmxVersion::new(String::from("1.0.0-rc.1")).unwrap();
        assert_eq!(prerelease.extension(), Some("rc.1"));
        let legacy = SdmxVersion::new(String::from("1.3")).unwrap();
        assert_eq!((legacy.major(), legacy.minor(), legacy.patch()), (1, Some(3), None));
        let bare = SdmxVersion::new(String::from("1")).unwrap();
        assert_eq!((bare.major(), bare.minor(), bare.is_legacy()), (1, None, true));
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
    fn version_equality_preserves_statedness() {
        // The structural fields distinguish exactly what the lexemes distinguish, so
        // trailing-zero-equivalent versions are deliberately unequal (lossless-distinct).
        assert_ne!(
            SdmxVersion::new(String::from("1.0")).unwrap(),
            SdmxVersion::new(String::from("1.0.0")).unwrap()
        );
        assert_ne!(
            SdmxVersion::new(String::from("1")).unwrap(),
            SdmxVersion::new(String::from("1.0")).unwrap()
        );
        assert_eq!(
            SdmxVersion::new(String::from("2.1.0")).unwrap(),
            SdmxVersion::new(String::from("2.1.0")).unwrap()
        );
    }

    #[test]
    fn version_hash_agrees_with_eq() {
        // Hash derives over the same structural fields as Eq, so equal versions hash
        // identically (the Hash/Eq contract). Distinct versions need not, and are not
        // asserted here.
        fn hash_bytes(version: &SdmxVersion) -> alloc::vec::Vec<u8> {
            #[derive(Default)]
            struct ByteCollector(alloc::vec::Vec<u8>);
            impl core::hash::Hasher for ByteCollector {
                fn finish(&self) -> u64 {
                    0
                }
                fn write(&mut self, bytes: &[u8]) {
                    self.0.extend_from_slice(bytes);
                }
            }
            let mut collector = ByteCollector::default();
            core::hash::Hash::hash(version, &mut collector);
            collector.0
        }

        let a = SdmxVersion::new(String::from("2.1.0")).unwrap();
        let b = SdmxVersion::new(String::from("2.1.0")).unwrap();
        assert_eq!(a, b);
        assert_eq!(hash_bytes(&a), hash_bytes(&b), "equal versions must hash equally");
    }

    #[test]
    fn version_display_reconstructs_the_lexeme() {
        use alloc::string::ToString;
        // Every accepted lexeme renders back to itself: the canonical grammar makes
        // format-then-parse a bijection, which is what licenses the raw-free storage.
        for lexeme in
            ["1", "1.0", "1.0.0", "0.0.0", "3.0.0-beta.2", "1.0.0-rc.1", "2.5.1-alpha-1.7"]
        {
            let v = SdmxVersion::new(lexeme.into()).unwrap();
            assert_eq!(v.to_string(), lexeme);
            assert_eq!(v, v.to_string().parse().unwrap());
        }
        let v = SdmxVersion::new(String::from("3.0.0-beta.2")).unwrap();
        assert_eq!(VersionDisplay(Some(&v)).to_string(), "3.0.0-beta.2");
        assert_eq!(VersionDisplay(None).to_string(), "<unversioned>");
    }

    #[test]
    fn version_ref_parses_each_arm() {
        // Exact covers the whole declaration grammar (legacy and semantic, extension included).
        for exact in ["1", "1.0", "1.2.3", "1.0.0-rc.1"] {
            let parsed = VersionRef::new(exact.into()).unwrap();
            assert_eq!(parsed, VersionRef::Exact(exact.parse().unwrap()), "{exact}");
        }
        // Latest wildcards exactly one component of a full triple.
        for (lexeme, at) in [
            ("2+.3.1", WildcardPosition::Major),
            ("2.3+.1", WildcardPosition::Minor),
            ("2.3.1+", WildcardPosition::Patch),
        ] {
            assert_eq!(
                VersionRef::new(lexeme.into()).unwrap(),
                VersionRef::Latest { major: 2, minor: 3, patch: 1, at },
                "{lexeme}"
            );
        }
        assert_eq!(VersionRef::new(String::from("*")).unwrap(), VersionRef::Any);
    }

    #[test]
    fn version_ref_rejects_malformed() {
        for bad in [
            "",
            "**",
            "1.*",              // VersionQueryType registry-query grammar, not a reference
            "1.0.*",            // likewise
            "2.3+.1-draft",     // wildcard and extension are mutually exclusive
            "1+.2+.3",          // more than one wildcard
            "1+.2.3+",          // 3.0's regex admits this; the documented contract does not
            "1+",               // wildcard requires the full triple
            "1.2+",             // likewise
            "1.2.3.4+",         // too many components
            "+1.2.3",           // leading wildcard is not the grammar
            "01+.2.3",          // leading zero
            "99999999999+.0.0", // component overflows u32
        ] {
            assert!(VersionRef::new(bad.into()).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn version_ref_display_reconstructs_the_lexeme() {
        use alloc::string::ToString;
        for lexeme in ["1", "1.0", "1.2.3", "1.0.0-rc.1", "2+.3.1", "2.3+.1", "2.3.1+", "*"] {
            let parsed = VersionRef::new(lexeme.into()).unwrap();
            assert_eq!(parsed.to_string(), lexeme);
            assert_eq!(parsed, parsed.to_string().parse().unwrap());
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn time_range_accepts_date_and_datetime_starts() {
        for lexeme in [
            "2010-01-01/P2M",
            "2010-01-01Z/P2M",
            "2010-01-01T09:30:00/PT30M",
            "2010-01-01T09:30:00+05:30/P1Y2M3DT4H5M6.5S",
            "2024-02-29/P1D",
        ] {
            assert!(SdmxTimeRange::new(lexeme.into()).is_ok(), "{lexeme:?} should be accepted");
        }
        let range = SdmxTimeRange::new(String::from("2010-01-01/P2M")).unwrap();
        assert_eq!((range.start(), range.duration()), ("2010-01-01", "P2M"));
    }

    #[test]
    fn time_range_rejects_malformed() {
        for bad in [
            "",
            "2010-01-01", // no duration half
            "P2M",        // no start half
            "/P2M",
            "2010-01-01/",
            "2010/P2M",          // start must be a full date, not a year
            "2010-01/P2M",       // nor a year-month
            "2024-Q4/P2M",       // nor a reporting period
            "2010-01-01/2M",     // duration must open with P
            "2010-01-01/P",      // empty duration
            "2010-01-01/PT",     // empty time part
            "2010-01-01/P2M1Y",  // components out of order
            "2010-01-01/PT2H1H", // repeated unit
            "2010-01-01/P2X",    // unknown unit
            "2010-01-01/P1Y2",   // trailing digit run has no unit
            "2010-01-01/PT0.5M", // fraction only on seconds
            "2010-01-01/PT0.S",  // fraction needs digits
            "2010-01-01/P2M/P1D",
        ] {
            assert!(SdmxTimeRange::new(bad.into()).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn duration_accepts_signed_and_component_forms() {
        for ok in [
            "P2M",
            "P1Y2M3D",
            "PT30M",
            "PT6.5S",
            "P1Y2M3DT4H5M6.5S",
            "-P2M",
            "-PT0.5S",
            "P0D", // zero-valued components are grammar-valid
        ] {
            assert!(SdmxDuration::new(ok.into()).is_ok(), "{ok:?} should be a valid xs:duration");
        }
        let duration = SdmxDuration::new(String::from("-P1Y2M")).unwrap();
        assert_eq!(duration.as_str(), "-P1Y2M");
    }

    #[test]
    fn duration_rejects_malformed() {
        for bad in [
            "", "-", "P",      // no components
            "PT",     // empty time part
            "-P",     // signed but empty
            "--P1D",  // doubled sign
            "P1M2Y",  // components out of order
            "PT2H1H", // repeated unit
            "2M",     // missing P
            "P2X",    // unknown unit
            "P1Y2",   // trailing digit run has no unit
            "PT0.5M", // fraction only on seconds
            "PT0.S",  // fraction needs digits
            "P1DT",   // T requires a time component
            "P2M ",   // trailing space
            "+P2M",   // xs:duration admits only the minus sign
            "P-1D",   // sign only leads the whole lexeme
        ] {
            assert!(SdmxDuration::new(bad.into()).is_err(), "{bad:?} should be rejected");
        }
        assert!(matches!(
            SdmxDuration::new(String::from("banana")),
            Err(Error::InvalidDuration(_))
        ));
    }

    #[test]
    fn duration_round_trips_traits_and_serde() {
        use alloc::string::ToString;
        for raw in ["P2M", "-PT30M", "P1Y2M3DT4H5M6.5S"] {
            let v = SdmxDuration::new(raw.into()).unwrap();
            assert_eq!(v.to_string(), raw);
            assert_eq!(AsRef::<str>::as_ref(&v), raw);
            assert_eq!(v.to_string().parse::<SdmxDuration>(), Ok(v.clone()));
            crate::test_support::round_trip(&v);
        }
        // String identity, never a normalised view: P1M and P01M are value-equal, lexically
        // distinct (D-0027 lossless-distinct).
        let duration = SdmxDuration::new(String::from("P1M")).unwrap();
        assert!(duration == *"P1M");
        assert!(duration == "P1M");
        assert!(duration != "P01M");
        assert_ne!(duration, SdmxDuration::new(String::from("P01M")).unwrap());
        // into_inner / From unwrap the verbatim lexeme.
        assert_eq!(duration.clone().into_inner(), "P1M");
        assert_eq!(String::from(duration), "P1M");
        // A grammar-invalid lexeme is rejected on the wire path (routes through new()).
        let bad = String::from("P");
        assert!(
            postcard::from_bytes::<SdmxDuration>(&postcard::to_allocvec(&bad).unwrap()).is_err()
        );
    }

    #[test]
    fn observational_classifies_both_members() {
        use alloc::string::ToString;
        let standard = ObservationalTimePeriod::new(String::from("2024-Q4")).unwrap();
        assert!(
            matches!(&standard, ObservationalTimePeriod::Standard(p) if p.kind() == SdmxTimePeriodKind::ReportingQuarter)
        );
        let range = ObservationalTimePeriod::new(String::from("2010-01-01/P2M")).unwrap();
        assert!(matches!(&range, ObservationalTimePeriod::Range(_)));
        for value in [standard, range] {
            assert_eq!(value.to_string(), value.as_str());
            assert_eq!(value, value.as_str().parse::<ObservationalTimePeriod>().unwrap());
        }
        assert!(matches!(
            ObservationalTimePeriod::new(String::from("banana")),
            Err(Error::InvalidObservationalTimePeriod(_))
        ));
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
    fn gregorian_day_admits_calendar_invalid_dates() {
        // Day-of-month is range-checked `1..=31` for every month, so `2024-02-30` classifies
        // as a Gregorian day: calendar arithmetic is deliberately not re-implemented (the
        // `SdmxTimeRange` design note), a boundary this locks against a well-meaning refactor.
        let parsed = SdmxTimePeriod::new(String::from("2024-02-30")).unwrap();
        assert_eq!(parsed.kind(), SdmxTimePeriodKind::GregorianDay);
    }

    #[test]
    fn reporting_year_is_fixed_at_four_digits() {
        // `BaseReportPeriodType` pins the reporting year at exactly four digits (`\d{4}`),
        // unlike the unbounded `xs:gYear` that `is_year` admits, so a five-digit reporting
        // year is not a reporting period and is rejected outright.
        assert!(SdmxTimePeriod::new(String::from("20241-A1")).is_err());
        // Contrast: the four-digit form is a reporting year.
        assert_eq!(
            SdmxTimePeriod::new(String::from("2024-A1")).unwrap().kind(),
            SdmxTimePeriodKind::ReportingYear
        );
    }

    #[test]
    fn gregorian_classifiers_reject_year_zero() {
        // XSD 1.0 prohibits year `0000` across the builtin date/time types, so it is rejected at
        // every Gregorian site the grammar reaches: gYear, gYearMonth, date, dateTime, and the
        // `TimeRangeType` starts that reuse those scanners.
        for bad in ["0000", "0000-01", "0000-01-01", "0000-01-01T00:00:00"] {
            assert!(SdmxTimePeriod::new(bad.into()).is_err(), "{bad:?} should be rejected");
        }
        assert!(SdmxTimeRange::new(String::from("0000-01-01/P1D")).is_err());
        assert!(SdmxTimeRange::new(String::from("0000-01-01T00:00:00/P1D")).is_err());
    }

    #[test]
    fn gregorian_year_rejects_leading_zeros_past_four_digits() {
        // XSD 1.0 (§3.2.7) prohibits a leading zero once the year exceeds four digits, so a longer
        // run starting with `0` is rejected at every Gregorian site. Four-digit years (`0001`),
        // five-or-more-digit non-zero-leading years, and BC (`-`-prefixed) forms stay accepted.
        for ok in ["0001", "0001-06", "12024", "12024-06-30", "-2024", "-12024"] {
            assert!(SdmxTimePeriod::new(ok.into()).is_ok(), "{ok:?} should be accepted");
        }
        for bad in ["02024", "012345", "02024-01", "02024-01-01T00:00:00", "-02024"] {
            assert!(SdmxTimePeriod::new(bad.into()).is_err(), "{bad:?} should be rejected");
        }
        // The prohibition holds at a `TimeRangeType` start too (the shared scanner).
        assert!(SdmxTimeRange::new(String::from("02024-01-01/P1D")).is_err());
        // `+2024` is the same spec sentence's other prohibition (an explicit leading `+`), already
        // rejected by the digit scan today and pinned here so it stays rejected.
        assert!(SdmxTimePeriod::new(String::from("+2024")).is_err());
    }

    #[test]
    fn reporting_period_admits_year_zero() {
        // The year-zero prohibition is a value-space property of the XSD builtins only. Reporting
        // periods are pattern-restricted strings (`BaseReportPeriodType`, `\d{4}-...`), and `\d{4}`
        // admits `0000`, so `0000-Q1` is schema-valid and stays accepted: the pattern-grammar
        // boundary against the Gregorian rejection above.
        assert_eq!(
            SdmxTimePeriod::new(String::from("0000-Q1")).unwrap().kind(),
            SdmxTimePeriodKind::ReportingQuarter
        );
    }

    #[test]
    fn date_time_rejects_leap_second() {
        // `xs:dateTime` seconds are `0..=59`: a leap-second `60` is rejected.
        assert!(SdmxTimePeriod::new(String::from("2024-05-01T09:30:60")).is_err());
    }

    #[test]
    fn date_time_admits_end_of_day_hour_24() {
        // XSD 1.0 `xs:dateTime` (§3.2.7) admits `24:00:00` as the end-of-day instant, a stated
        // fraction permitted only if it is all zeros. A boundary this locks against a refactor
        // re-capping the hour at 23.
        for ok in ["2024-06-30T24:00:00", "2024-06-30T24:00:00.0", "2024-06-30T24:00:00.000"] {
            assert_eq!(
                SdmxTimePeriod::new(ok.into()).unwrap().kind(),
                SdmxTimePeriodKind::DateTime,
                "{ok:?} should be accepted"
            );
        }
        // Hour 24 rides the shared classifiers, so acceptance must also hold at a `TimeRangeType`
        // start and under a timezone suffix: both are correct only because those paths reuse
        // `is_time`, exactly the coupling a refactor could silently break.
        assert!(SdmxTimeRange::new(String::from("2024-06-30T24:00:00/P1D")).is_ok());
        assert_eq!(
            SdmxTimePeriod::new(String::from("2024-06-30T24:00:00Z")).unwrap().kind(),
            SdmxTimePeriodKind::DateTime
        );
    }

    #[test]
    fn date_time_rejects_malformed_hour_24() {
        // Hour 24 is admitted only as the exact end-of-day instant: a nonzero minute, second, or
        // fractional part falls outside the value space.
        for bad in ["2024-06-30T24:00:01", "2024-06-30T24:01:00", "2024-06-30T24:00:00.5"] {
            assert!(SdmxTimePeriod::new(bad.into()).is_err(), "{bad:?} should be rejected");
        }
        // `24:00:60`: the hour-24 acceptance path still enforces the seconds field, so an
        // out-of-range second is rejected under the relaxation rather than waved through by the
        // hour being 24. Not redundant with `24:00:01`, which pins the nonzero-but-valid second.
        assert!(SdmxTimePeriod::new(String::from("2024-06-30T24:00:60")).is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
    fn is_valid_timezone_rejects_structurally_malformed_offsets() {
        // `split_timezone` only ever hands `is_valid_timezone` a well-formed `±hh:mm`
        // (or `Z`), so its structural guard is unreachable through `SdmxTimePeriod::new`
        // and is exercised here at the unit boundary. The guard also keeps the
        // `tz[1..3]`/`tz[4..6]` range slicing panic-free on a malformed offset.
        assert!(!is_valid_timezone("+1:00")); // too short (len != 6)
        assert!(!is_valid_timezone("+01000")); // no colon at index 3
        assert!(!is_valid_timezone("010:00")); // no sign
        // The well-formed forms the guard lets through to the range check.
        assert!(is_valid_timezone("Z"));
        assert!(is_valid_timezone("+00:00"));
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
        // Each lexical newtype round-trips a valid lexeme verbatim; a value its grammar rejects
        // fails deserialisation (the §7 construction contract). Every impl declares an inner
        // `Raw = String` (`String::deserialize` then `Self::new(s)`), and postcard encodes a
        // newtype identically to that one field, so serialising a `String` carrying a
        // grammar-invalid lexeme decodes into new(), which rejects it.
        let decimal = SdmxDecimal::new(String::from("-3.14")).unwrap();
        assert_eq!(decimal.as_str(), "-3.14");
        crate::test_support::round_trip(&decimal);
        let bad = String::from("banana");
        assert!(
            postcard::from_bytes::<SdmxDecimal>(&postcard::to_allocvec(&bad).unwrap()).is_err()
        );

        let integer = SdmxInteger::new(String::from("-7")).unwrap();
        assert_eq!(integer.as_str(), "-7");
        crate::test_support::round_trip(&integer);
        let bad = String::from("2.5");
        assert!(
            postcard::from_bytes::<SdmxInteger>(&postcard::to_allocvec(&bad).unwrap()).is_err()
        );

        let version = SdmxVersion::new(String::from("1.0.0-rc.1")).unwrap();
        assert_eq!(version.extension(), Some("rc.1"));
        crate::test_support::round_trip(&version);
        let bad = String::from("01.0.0");
        assert!(
            postcard::from_bytes::<SdmxVersion>(&postcard::to_allocvec(&bad).unwrap()).is_err()
        );

        let reference = VersionRef::new(String::from("2+.3.1")).unwrap();
        assert!(matches!(reference, VersionRef::Latest { at: WildcardPosition::Major, .. }));
        crate::test_support::round_trip(&reference);
        crate::test_support::round_trip(&VersionRef::Any);
        let bad = String::from("1.*");
        assert!(postcard::from_bytes::<VersionRef>(&postcard::to_allocvec(&bad).unwrap()).is_err());

        let period = SdmxTimePeriod::new(String::from("2024-Q4")).unwrap();
        assert_eq!(period.kind(), SdmxTimePeriodKind::ReportingQuarter);
        crate::test_support::round_trip(&period);
        let bad = String::from("2024-Q5");
        assert!(
            postcard::from_bytes::<SdmxTimePeriod>(&postcard::to_allocvec(&bad).unwrap()).is_err()
        );

        crate::test_support::round_trip(
            &SdmxTimeRange::new(String::from("2010-01-01/P2M")).unwrap(),
        );
        crate::test_support::round_trip(
            &ObservationalTimePeriod::new(String::from("2010-01-01/P2M")).unwrap(),
        );
        crate::test_support::round_trip(
            &ObservationalTimePeriod::new(String::from("2024-Q4")).unwrap(),
        );
        let bad = String::from("2010-01-01/P");
        assert!(
            postcard::from_bytes::<ObservationalTimePeriod>(&postcard::to_allocvec(&bad).unwrap())
                .is_err()
        );

        crate::test_support::round_trip(
            &SdmxDateTime::new(String::from("2024-05-01T09:30:00Z")).unwrap(),
        );
        let bad = String::from("2024-13-01T00:00:00");
        assert!(
            postcard::from_bytes::<SdmxDateTime>(&postcard::to_allocvec(&bad).unwrap()).is_err()
        );
    }

    #[test]
    fn serialize_round_trips_the_raw_lexeme() {
        // The projection shape is deliberately not pinned (D-0068); each newtype serialises through
        // its raw lexeme and reconstructs an equal value.
        crate::test_support::round_trip(&SdmxVersion::new(String::from("2.1")).unwrap());
        crate::test_support::round_trip(&SdmxDecimal::new(String::from("0.001")).unwrap());
        crate::test_support::round_trip(&SdmxInteger::new(String::from("-7")).unwrap());
        crate::test_support::round_trip(&SdmxTimePeriod::new(String::from("2024-Q4")).unwrap());
    }

    // -----------------------------------------------------------------------
    // SdmxDateTime (D-0079)
    // -----------------------------------------------------------------------

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn date_time_accepts_and_rejects() {
        // The timezone is optional; the offsetless, Z, and numeric-offset spellings are all
        // schema-valid, as are fractional seconds and the hour-24 end-of-day instant.
        for ok in [
            "2024-05-01T09:30:00",
            "2024-05-01T09:30:00Z",
            "2024-05-01T09:30:00+00:00",
            "2024-05-01T09:30:00-05:30",
            "2024-05-01T09:30:00.5",
            "2024-05-01T24:00:00",
        ] {
            assert!(SdmxDateTime::new(ok.into()).is_ok(), "{ok:?} should be a valid xs:dateTime");
        }
        // Rejections: garbage, a bare date (no time), a reporting period, year zero, an
        // out-of-range offset, and a leap second.
        for bad in [
            "banana",
            "2024-05-01",
            "2024-Q4",
            "0000-01-01T00:00:00",
            "2024-05-01T00:00:00+15:00",
            "2024-05-01T09:30:60",
        ] {
            assert_eq!(
                SdmxDateTime::new(bad.into()),
                Err(Error::InvalidDateTime(String::from(bad))),
                "{bad:?} should be rejected"
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn date_time_round_trips_lexeme_byte_exact() {
        use alloc::string::ToString;
        // Every spelling the value model would collapse survives verbatim through Display and
        // the serde projection (D-0079): offsetless, Z, +00:00, and a fractional part.
        for raw in [
            "2024-05-01T09:30:00",
            "2024-05-01T09:30:00Z",
            "2024-05-01T09:30:00+00:00",
            "2024-05-01T09:30:00.500Z",
        ] {
            let dt = SdmxDateTime::new(raw.into()).unwrap();
            assert_eq!(dt.to_string(), raw);
            assert_eq!(dt.as_str(), raw);
            assert_eq!(AsRef::<str>::as_ref(&dt), raw);
            assert_eq!(dt.to_string().parse::<SdmxDateTime>(), Ok(dt.clone()));
            assert_eq!(crate::test_support::round_trip(&dt).as_str(), raw);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn date_time_identity_is_the_lexeme() {
        fn hash_bytes(dt: &SdmxDateTime) -> alloc::vec::Vec<u8> {
            #[derive(Default)]
            struct ByteCollector(alloc::vec::Vec<u8>);
            impl core::hash::Hasher for ByteCollector {
                fn finish(&self) -> u64 {
                    0
                }
                fn write(&mut self, bytes: &[u8]) {
                    self.0.extend_from_slice(bytes);
                }
            }
            let mut collector = ByteCollector::default();
            core::hash::Hash::hash(dt, &mut collector);
            collector.0
        }

        // `Z` and `+00:00` are the same instant but distinct lexemes: unequal and hash-distinct.
        let zulu = SdmxDateTime::new(String::from("2024-05-01T09:30:00Z")).unwrap();
        let plus = SdmxDateTime::new(String::from("2024-05-01T09:30:00+00:00")).unwrap();
        assert_ne!(zulu, plus);
        assert_ne!(hash_bytes(&zulu), hash_bytes(&plus));
        // The `PartialEq<str>` operator (D-0074) is lexeme identity too.
        assert_eq!(zulu, *"2024-05-01T09:30:00Z");
        assert_ne!(zulu, *"2024-05-01T09:30:00+00:00");

        // Identical lexemes are equal and hash equally.
        let again = SdmxDateTime::new(String::from("2024-05-01T09:30:00Z")).unwrap();
        assert_eq!(zulu, again);
        assert_eq!(hash_bytes(&zulu), hash_bytes(&again));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn date_time_value_views() {
        let expected = NaiveDate::from_ymd_opt(2024, 5, 1)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(9, 30, 0).unwrap());

        // Offsetless: a written date-time, no offset, and no instant (no point is fixed).
        let naive = SdmxDateTime::new(String::from("2024-05-01T09:30:00")).unwrap();
        assert_eq!(naive.date_time(), Some(expected));
        assert_eq!(naive.offset(), None);
        assert_eq!(naive.instant(), None);

        // Stated offset: the offset is data and the instant is present.
        let offset = SdmxDateTime::new(String::from("2024-05-01T09:30:00-05:30")).unwrap();
        assert_eq!(offset.offset(), FixedOffset::east_opt(-(5 * 3600 + 30 * 60)));
        assert!(offset.instant().is_some());

        // Same instant, different spelling: distinct values, equal instants (via the views).
        let zulu = SdmxDateTime::new(String::from("2024-05-01T09:30:00Z")).unwrap();
        let plus = SdmxDateTime::new(String::from("2024-05-01T09:30:00+00:00")).unwrap();
        assert_ne!(zulu, plus);
        assert_eq!(zulu.instant(), plus.instant());
        assert!(zulu.instant().is_some());

        // Hour 24 is the end-of-day instant: the derived view is next-day 00:00:00, the raw
        // lexeme is untouched.
        let midnight = SdmxDateTime::new(String::from("2024-05-01T24:00:00")).unwrap();
        assert_eq!(midnight.as_str(), "2024-05-01T24:00:00");
        assert_eq!(
            midnight.date_time(),
            Some(
                NaiveDate::from_ymd_opt(2024, 5, 2)
                    .unwrap()
                    .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            )
        );

        // Fractional seconds beyond nanosecond precision truncate in the view; the raw is kept.
        let fraction = SdmxDateTime::new(String::from("2024-05-01T09:30:00.1234567891")).unwrap();
        assert_eq!(fraction.as_str(), "2024-05-01T09:30:00.1234567891");
        assert_eq!(
            fraction.date_time(),
            Some(
                NaiveDate::from_ymd_opt(2024, 5, 1)
                    .unwrap()
                    .and_time(NaiveTime::from_hms_nano_opt(9, 30, 0, 123_456_789).unwrap())
            )
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn date_time_derived_view_absent_for_calendar_invalid_lexeme() {
        // The shared grammar admits a calendar-invalid day (it does not re-implement month
        // lengths), which chrono cannot represent, so the lexeme constructs and round-trips but
        // has no derived date-time (D-0079). Schema-valid wire never carries such a value.
        let invalid = SdmxDateTime::new(String::from("2024-02-30T00:00:00Z")).unwrap();
        assert_eq!(invalid.as_str(), "2024-02-30T00:00:00Z");
        assert_eq!(invalid.date_time(), None);
        assert!(invalid.offset().is_some());
        // No written date-time, so no instant even though an offset is stated.
        assert_eq!(invalid.instant(), None);
        crate::test_support::round_trip(&invalid);

        // A BC year (leading `-`) is schema-valid and representable: the derived view carries it.
        let bc = SdmxDateTime::new(String::from("-0044-03-15T12:00:00")).unwrap();
        assert_eq!(bc.as_str(), "-0044-03-15T12:00:00");
        assert!(bc.date_time().is_some());
    }

    #[test]
    fn lexical_newtypes_round_trip_display_parse_asref() {
        use alloc::string::ToString;

        // The Display / FromStr impls are uniform across the lexeme newtypes: every valid
        // lexeme renders verbatim and parses back to an equal value. (`FromStr` clones into the
        // owned `new(String)` write path.) The lexeme-storing types also expose the borrowed
        // `&str` via `AsRef`; `SdmxVersion` reconstructs its lexeme instead, so it has none.
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
            assert_eq!(v.to_string().parse::<SdmxVersion>(), Ok(v.clone()));
        }
        for raw in ["2024", "2024-05", "2024-Q4", "2024-05-01T09:30:00", "2024-05-01T09:30:00Z"] {
            let v = SdmxTimePeriod::new(raw.into()).unwrap();
            assert_eq!(v.to_string(), raw);
            assert_eq!(AsRef::<str>::as_ref(&v), raw);
            assert_eq!(v.to_string().parse::<SdmxTimePeriod>(), Ok(v.clone()));
        }
        for raw in ["2010-01-01/P2M", "2010-01-01T09:30:00Z/PT30M"] {
            let v = SdmxTimeRange::new(raw.into()).unwrap();
            assert_eq!(v.to_string(), raw);
            assert_eq!(AsRef::<str>::as_ref(&v), raw);
            assert_eq!(v.to_string().parse::<SdmxTimeRange>(), Ok(v.clone()));
        }
        for raw in ["2024-Q4", "2010-01-01/P2M"] {
            let v = ObservationalTimePeriod::new(raw.into()).unwrap();
            assert_eq!(v.to_string(), raw);
            assert_eq!(AsRef::<str>::as_ref(&v), raw);
            assert_eq!(v.to_string().parse::<ObservationalTimePeriod>(), Ok(v.clone()));
        }
    }

    #[test]
    fn lexical_newtype_into_inner_and_from() {
        let d = SdmxDecimal::new(String::from("1.5")).unwrap();
        assert_eq!(d.clone().into_inner(), "1.5");
        assert_eq!(String::from(d), "1.5");
        let i = SdmxInteger::new(String::from("42")).unwrap();
        assert_eq!(i.clone().into_inner(), "42");
        assert_eq!(String::from(i), "42");
        let r = SdmxTimeRange::new(String::from("2010-01-01/P2M")).unwrap();
        assert_eq!(r.clone().into_inner(), "2010-01-01/P2M");
        assert_eq!(String::from(r.clone()), "2010-01-01/P2M");
        // The unwrapped string reconstructs the same value through the constructor.
        assert_eq!(SdmxTimeRange::new(r.clone().into_inner()).unwrap(), r);
    }

    #[test]
    fn raw_backed_newtypes_compare_to_literals_by_identity() {
        // `PartialEq<str>`/`PartialEq<&str>` are string identity with the stored lexeme,
        // never a normalised or numeric view (D-0027 lossless-distinct).
        let decimal = SdmxDecimal::new(String::from("1.0")).unwrap();
        assert!(decimal == *"1.0");
        assert!(decimal == "1.0");
        assert!(decimal != "1.00"); // numerically equal, lexically distinct
        let integer = SdmxInteger::new(String::from("7")).unwrap();
        assert!(integer == *"7");
        assert!(integer == "7");
        assert!(integer != "+7");
        let period = SdmxTimePeriod::new(String::from("2024-Q4")).unwrap();
        assert!(period == *"2024-Q4");
        assert!(period == "2024-Q4");
        assert!(period != "2024-Q4Z"); // stated timezone is part of the lexeme
        let range = SdmxTimeRange::new(String::from("2010-01-01/P2M")).unwrap();
        assert!(range == *"2010-01-01/P2M");
        assert!(range == "2010-01-01/P2M");
        assert!(range != "2010-01-01/P60D"); // equivalent span, distinct lexeme
        let observational = ObservationalTimePeriod::new(String::from("2024-Q4")).unwrap();
        assert!(observational == *"2024-Q4");
        assert!(observational == "2024-Q4");
        assert!(observational != "2010-01-01/P2M"); // the other member's grammar
    }

    // Property tests: fuzzed breadth over the validated grammars, generated through the
    // constructors (see `test_strategy`). They complement the example tests above, which
    // stay the deterministic coverage backbone; wasm32 is excluded with the rest of the
    // property suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use alloc::{format, string::ToString};

        use proptest::prelude::*;

        use super::super::*;
        use crate::test_strategy::{
            invalid_decimal_lexeme, invalid_integer_lexeme, observational_time_period_lexeme,
            standard_time_period_lexeme, time_range_lexeme, version_lexeme, version_ref_lexeme,
            xs_decimal_lexeme, xs_duration_lexeme, xs_integer_lexeme,
        };

        proptest! {
            #[test]
            fn decimal_grammar_round_trips(lexeme in xs_decimal_lexeme()) {
                // Lossless raw (D-0027): stored verbatim, rendered verbatim, parsed back equal.
                let value = SdmxDecimal::new(lexeme.clone()).unwrap();
                prop_assert_eq!(value.as_str(), lexeme.as_str());
                prop_assert_eq!(&lexeme.parse::<SdmxDecimal>().unwrap(), &value);
                prop_assert_eq!(value.to_string(), lexeme);
            }

            #[test]
            fn integer_grammar_round_trips(lexeme in xs_integer_lexeme()) {
                let value = SdmxInteger::new(lexeme.clone()).unwrap();
                prop_assert_eq!(value.as_str(), lexeme.as_str());
                prop_assert_eq!(&lexeme.parse::<SdmxInteger>().unwrap(), &value);
                prop_assert_eq!(value.to_string(), lexeme);
            }

            #[test]
            fn integer_widens_into_decimal_losslessly(lexeme in xs_integer_lexeme()) {
                // The subset relationship made executable (D-0027): widening is verbatim and
                // total, and narrowing an integer-lexeme decimal recovers the original.
                let integer = SdmxInteger::new(lexeme.clone()).unwrap();
                let decimal = SdmxDecimal::from(integer.clone());
                prop_assert_eq!(decimal.as_str(), lexeme.as_str());
                prop_assert_eq!(SdmxInteger::try_from(decimal).unwrap(), integer);
            }

            #[test]
            fn decimal_narrows_exactly_when_the_lexeme_is_integral(lexeme in xs_decimal_lexeme()) {
                // Narrowing is lexical, never numeric: it succeeds iff the stored lexeme is
                // already an xs:integer lexeme, and preserves it verbatim when it does.
                let decimal = SdmxDecimal::new(lexeme.clone()).unwrap();
                let narrowed = SdmxInteger::try_from(decimal);
                prop_assert_eq!(narrowed.is_ok(), is_xs_integer(&lexeme));
                if let Ok(integer) = narrowed {
                    prop_assert_eq!(integer.as_str(), lexeme.as_str());
                }
            }

            #[test]
            fn version_grammar_round_trips(lexeme in version_lexeme()) {
                // The canonical grammar's bijection (D-0070): the reconstructed lexeme is the
                // parsed one, and format-then-parse is the identity on values.
                let version = SdmxVersion::new(lexeme.clone()).unwrap();
                prop_assert_eq!(version.to_string(), lexeme);
                prop_assert_eq!(version.to_string().parse::<SdmxVersion>().unwrap(), version);
            }

            #[test]
            fn version_ref_grammar_round_trips(lexeme in version_ref_lexeme()) {
                let reference = VersionRef::new(lexeme.clone()).unwrap();
                prop_assert_eq!(reference.to_string(), lexeme);
                prop_assert_eq!(reference.to_string().parse::<VersionRef>().unwrap(), reference);
            }

            #[test]
            fn time_period_grammar_round_trips(lexeme in standard_time_period_lexeme()) {
                let period = SdmxTimePeriod::new(lexeme.clone()).unwrap();
                prop_assert_eq!(period.as_str(), lexeme.as_str());
                prop_assert_eq!(&lexeme.parse::<SdmxTimePeriod>().unwrap(), &period);
                prop_assert_eq!(period.to_string(), lexeme);
            }

            #[test]
            fn duration_grammar_round_trips(lexeme in xs_duration_lexeme()) {
                // Lossless raw (D-0027): stored verbatim, rendered verbatim, parsed back equal.
                let duration = SdmxDuration::new(lexeme.clone()).unwrap();
                prop_assert_eq!(duration.as_str(), lexeme.as_str());
                prop_assert_eq!(&lexeme.parse::<SdmxDuration>().unwrap(), &duration);
                prop_assert_eq!(duration.to_string(), lexeme);
            }

            #[test]
            fn duration_eq_str_is_lexeme_identity(
                a in xs_duration_lexeme(),
                b in xs_duration_lexeme(),
            ) {
                let value = SdmxDuration::new(a.clone()).unwrap();
                prop_assert!(value == a[..]);
                prop_assert!(value == a.as_str());
                prop_assert_eq!(value == b[..], a == b);
            }

            #[test]
            fn time_range_grammar_round_trips(lexeme in time_range_lexeme()) {
                let range = SdmxTimeRange::new(lexeme.clone()).unwrap();
                prop_assert_eq!(range.as_str(), lexeme.as_str());
                prop_assert_eq!(&lexeme.parse::<SdmxTimeRange>().unwrap(), &range);
                // The accessor halves reassemble the verbatim lexeme.
                prop_assert_eq!(format!("{}/{}", range.start(), range.duration()), lexeme);
            }

            #[test]
            fn decimal_eq_str_is_lexeme_identity(
                a in xs_decimal_lexeme(),
                b in xs_decimal_lexeme(),
            ) {
                // The operator agrees exactly with string equality on the raw lexeme: it
                // holds for the stored lexeme and never for a merely value-equal spelling.
                let value = SdmxDecimal::new(a.clone()).unwrap();
                prop_assert!(value == a[..]);
                prop_assert!(value == a.as_str());
                prop_assert_eq!(value == b[..], a == b);
            }

            #[test]
            fn integer_eq_str_is_lexeme_identity(
                a in xs_integer_lexeme(),
                b in xs_integer_lexeme(),
            ) {
                let value = SdmxInteger::new(a.clone()).unwrap();
                prop_assert!(value == a[..]);
                prop_assert!(value == a.as_str());
                prop_assert_eq!(value == b[..], a == b);
            }

            #[test]
            fn time_period_eq_str_is_lexeme_identity(
                a in standard_time_period_lexeme(),
                b in standard_time_period_lexeme(),
            ) {
                let value = SdmxTimePeriod::new(a.clone()).unwrap();
                prop_assert!(value == a[..]);
                prop_assert!(value == a.as_str());
                prop_assert_eq!(value == b[..], a == b);
            }

            #[test]
            fn time_range_eq_str_is_lexeme_identity(
                a in time_range_lexeme(),
                b in time_range_lexeme(),
            ) {
                let value = SdmxTimeRange::new(a.clone()).unwrap();
                prop_assert!(value == a[..]);
                prop_assert!(value == a.as_str());
                prop_assert_eq!(value == b[..], a == b);
            }

            #[test]
            fn observational_eq_str_is_lexeme_identity(
                a in observational_time_period_lexeme(),
                b in observational_time_period_lexeme(),
            ) {
                let value = ObservationalTimePeriod::new(a.clone()).unwrap();
                prop_assert!(value == a[..]);
                prop_assert!(value == a.as_str());
                prop_assert_eq!(value == b[..], a == b);
            }

            // The value-equal-sibling adversary below applies only to the numeric newtypes:
            // `xs:decimal` and `xs:integer` admit many lexemes per value, so lexeme identity
            // and value identity genuinely diverge. The time-storing types (`SdmxTimePeriod`,
            // `SdmxTimeRange`, `ObservationalTimePeriod`) keep the lexeme AS the datum
            // (statedness-preserving, D-0027), so no value-equal-but-distinct spelling exists
            // to construct; their lexeme identity is covered by the properties above.
            #[test]
            fn decimal_eq_str_rejects_value_equal_sibling(a in xs_decimal_lexeme()) {
                // Equality is lexeme identity, never numeric value (D-0027 lossless-distinct):
                // a value-equal but distinctly-spelled sibling (a trailing zero, `.0` when the
                // lexeme is integral) must compare unequal. The transform is total over the
                // generator, so the constructor accepts the sibling.
                let sibling =
                    if a.contains('.') { format!("{a}0") } else { format!("{a}.0") };
                prop_assert_ne!(sibling.as_str(), a.as_str());
                let value = SdmxDecimal::new(a).unwrap();
                let twin = SdmxDecimal::new(sibling.clone())
                    .expect("a trailing-zero decimal is value-equal and grammar-valid");
                prop_assert!(value != sibling[..]);
                prop_assert!(value != sibling.as_str());
                prop_assert_ne!(value, twin);
            }

            #[test]
            fn integer_eq_str_rejects_value_equal_sibling(a in xs_integer_lexeme()) {
                // The same lossless-distinct claim for `xs:integer`: a leading zero (after any
                // sign) is value-equal, distinctly spelled, and grammar-valid, so it compares
                // unequal.
                let sibling = a.strip_prefix(['+', '-']).map_or_else(
                    || format!("0{a}"),
                    |rest| format!("{}0{rest}", &a[..1]),
                );
                prop_assert_ne!(sibling.as_str(), a.as_str());
                let value = SdmxInteger::new(a).unwrap();
                let twin = SdmxInteger::new(sibling.clone())
                    .expect("a leading-zero integer is value-equal and grammar-valid");
                prop_assert!(value != sibling[..]);
                prop_assert!(value != sibling.as_str());
                prop_assert_ne!(value, twin);
            }

            #[test]
            fn decimal_rejects_off_grammar(lexeme in invalid_decimal_lexeme()) {
                // Rejection breadth over the tractable complement families; the precise
                // boundary stays with the example tests.
                prop_assert!(SdmxDecimal::new(lexeme).is_err());
            }

            #[test]
            fn integer_rejects_off_grammar(lexeme in invalid_integer_lexeme()) {
                prop_assert!(SdmxInteger::new(lexeme).is_err());
            }

            #[test]
            fn observational_grammar_round_trips(lexeme in observational_time_period_lexeme()) {
                let period = ObservationalTimePeriod::new(lexeme.clone()).unwrap();
                prop_assert_eq!(period.as_str(), lexeme.as_str());
                prop_assert_eq!(&lexeme.parse::<ObservationalTimePeriod>().unwrap(), &period);
                // The member grammars are disjoint on '/', so classification is determined
                // by the lexeme.
                prop_assert_eq!(
                    matches!(period, ObservationalTimePeriod::Range(_)),
                    lexeme.contains('/')
                );
            }
        }
    }
}
