//! Validated lexical newtypes for SDMX's constrained value types.
//!
//! [`SdmxDecimal`], [`SdmxInteger`], [`SdmxVersion`], and [`SdmxTimePeriod`] wrap the SDMX
//! lexical types whose value space does not map losslessly onto a fixed Rust type. Each
//! validates its grammar at construction and round-trips its text exactly:
//! [`SdmxDecimal`], [`SdmxInteger`], and [`SdmxTimePeriod`] store the canonical lexeme verbatim
//! and never rewrite it, while [`SdmxVersion`]'s canonical grammar lets it hold only the parsed
//! decomposition and reconstruct the lexeme on display. [`VersionRef`] extends the family with
//! the version *reference* grammar (`WildcardVersionType`: `+` and `*` wildcards), raw-free on
//! the same grounds.
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

Decisions: D-0027, D-0070.
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

Decisions: D-0027, D-0060, D-0070.
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
    /// version, and numeric components that exceed `u32`).
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
            (1, Some(2), Some(3), false)
        );
        let prerelease = SdmxVersion::new("1.0.0-rc.1".into()).unwrap();
        assert_eq!(prerelease.extension(), Some("rc.1"));
        let legacy = SdmxVersion::new("1.3".into()).unwrap();
        assert_eq!((legacy.major(), legacy.minor(), legacy.patch()), (1, Some(3), None));
        let bare = SdmxVersion::new("1".into()).unwrap();
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
            SdmxVersion::new("1.0".into()).unwrap(),
            SdmxVersion::new("1.0.0".into()).unwrap()
        );
        assert_ne!(SdmxVersion::new("1".into()).unwrap(), SdmxVersion::new("1.0".into()).unwrap());
        assert_eq!(
            SdmxVersion::new("2.1.0".into()).unwrap(),
            SdmxVersion::new("2.1.0".into()).unwrap()
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

        let a = SdmxVersion::new("2.1.0".into()).unwrap();
        let b = SdmxVersion::new("2.1.0".into()).unwrap();
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
        let v = SdmxVersion::new("3.0.0-beta.2".into()).unwrap();
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
        assert_eq!(VersionRef::new("*".into()).unwrap(), VersionRef::Any);
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
        let decimal = SdmxDecimal::new("-3.14".into()).unwrap();
        assert_eq!(decimal.as_str(), "-3.14");
        crate::test_support::round_trip(&decimal);
        let bad = String::from("banana");
        assert!(
            postcard::from_bytes::<SdmxDecimal>(&postcard::to_allocvec(&bad).unwrap()).is_err()
        );

        let integer = SdmxInteger::new("-7".into()).unwrap();
        assert_eq!(integer.as_str(), "-7");
        crate::test_support::round_trip(&integer);
        let bad = String::from("2.5");
        assert!(
            postcard::from_bytes::<SdmxInteger>(&postcard::to_allocvec(&bad).unwrap()).is_err()
        );

        let version = SdmxVersion::new("1.0.0-rc.1".into()).unwrap();
        assert_eq!(version.extension(), Some("rc.1"));
        crate::test_support::round_trip(&version);
        let bad = String::from("01.0.0");
        assert!(
            postcard::from_bytes::<SdmxVersion>(&postcard::to_allocvec(&bad).unwrap()).is_err()
        );

        let reference = VersionRef::new("2+.3.1".into()).unwrap();
        assert!(matches!(reference, VersionRef::Latest { at: WildcardPosition::Major, .. }));
        crate::test_support::round_trip(&reference);
        crate::test_support::round_trip(&VersionRef::Any);
        let bad = String::from("1.*");
        assert!(postcard::from_bytes::<VersionRef>(&postcard::to_allocvec(&bad).unwrap()).is_err());

        let period = SdmxTimePeriod::new("2024-Q4".into()).unwrap();
        assert_eq!(period.kind(), SdmxTimePeriodKind::ReportingQuarter);
        crate::test_support::round_trip(&period);
        let bad = String::from("2024-Q5");
        assert!(
            postcard::from_bytes::<SdmxTimePeriod>(&postcard::to_allocvec(&bad).unwrap()).is_err()
        );
    }

    #[test]
    fn serialize_round_trips_the_raw_lexeme() {
        // The projection shape is deliberately not pinned (D-0068); each newtype serialises through
        // its raw lexeme and reconstructs an equal value.
        crate::test_support::round_trip(&SdmxVersion::new("2.1".into()).unwrap());
        crate::test_support::round_trip(&SdmxDecimal::new("0.001".into()).unwrap());
        crate::test_support::round_trip(&SdmxInteger::new("-7".into()).unwrap());
        crate::test_support::round_trip(&SdmxTimePeriod::new("2024-Q4".into()).unwrap());
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
    }

    #[test]
    fn lexical_newtype_into_inner_and_from() {
        let d = SdmxDecimal::new("1.5".to_string()).unwrap();
        assert_eq!(d.clone().into_inner(), "1.5");
        assert_eq!(String::from(d), "1.5");
        let i = SdmxInteger::new("42".to_string()).unwrap();
        assert_eq!(i.clone().into_inner(), "42");
        assert_eq!(String::from(i), "42");
    }
}
