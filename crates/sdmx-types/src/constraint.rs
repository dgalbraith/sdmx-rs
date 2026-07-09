//! SDMX constraints: cube-region value selections and time ranges.
//!
//! A constraint narrows the data or metadata an artefact admits. Its building blocks are the
//! per-value selections that name dimension and component values, optionally cascading through a
//! code hierarchy and bounded by a validity window, and the [`TimeRange`] alternative that selects
//! a span rather than an enumerated set.
//!
//! The value leaves come in two shapes by selection position. A dimension value
//! ([`CubeKeyValue`]) carries a cascade flag and a validity window but no language tag, which the
//! schema prohibits there. A component (attribute or measure) value ([`SimpleComponentValue`])
//! additionally carries an optional language tag. Both are gathered into non-empty lists
//! ([`CubeKeyValues`], [`SimpleComponentValues`]): a chosen value list always names at least one
//! value, so an empty list is rejected at construction.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

This module holds the unified constraint model (0010 §5.8): the cube-region tree, the data-key-set
tree, the constraint-attachment enums, and the constraint maintainables. The value leaves here are
the bottom of that tree.

`CubeRegion` is modelled to the full spec structure (D-0026), not the earlier
`BTreeMap<String, BTreeSet<String>>`, so four distinctions survive: dimension (KeyValue) versus
component (Component) selections (distinct spec types and id grammars), per-value `cascadeValues`,
`TimeRange` as an alternative to a value list, and the component-referenced-with-no-values state.
D-0040 then split the per-value type per position so every prohibited attribute is unrepresentable
by field omission: the cube component value carries cascade plus lang plus validity, the cube
KeyValue value drops lang, the key-set component value drops validity, and the key-set KeyValue value
(a bare string) drops everything.

The non-empty values newtypes apply D-0038's fix (split per kind by D-0040): a chosen `Value+` arm
requires at least one value, so an empty list is mechanically schema-invalid and `new()` rejects it
(D-0031). The no-values wire state is a different shape (the component choice omitted entirely),
modelled in B2 as `ComponentSelection::Empty`. Bespoke per kind, each naming its own empty-error,
following the D-0034 rationale.

`TimeRange` is the full `TimeRangeValueType`: the choice (`kind`) plus the type-level
`validFrom`/`validTo` the earlier enum-only model dropped (D-0064 corrected D-0038's three-level
validity reading). Those attributes are `StandardTimePeriodType`, so they take `SdmxTimePeriod`
(D-0027), distinct from the endpoint content on `TimePeriodRange.period`, which is the
`ObservationalTimePeriodType` superset carried by the `ObservationalTimePeriod` union type
(D-0072). All the leaves are invariant-free pub-field carriers with
derived `Deserialize` (ADR-0021); the newtypes and `SdmxTimePeriod` carry their own validating
paths.

Decisions: D-0026, D-0027, D-0031, D-0038, D-0040, D-0052, D-0064.
"#
)]

use alloc::{string::String, vec::Vec};

use crate::{
    annotation::{Annotation, Link},
    artefact::{IdentifiableArtefact, MaintainableArtefact, NameableArtefact, VersionableArtefact},
    codelist::Cascade,
    error::{Error, to_de_error},
    fixed::FixedInclude,
    lexical::{ObservationalTimePeriod, SdmxDateTime, SdmxTimePeriod, SdmxVersion},
    localised::LocalisedString,
    metadata::MaintainableMetadata,
    reference::{
        DataProviderReference, DataStructureReference, DataflowReference,
        ProvisionAgreementReference,
    },
    validate::{validate_ncname, validate_nested_ncname},
};

// ---------------------------------------------------------------------------
// Cube-region value leaves
// ---------------------------------------------------------------------------

/// A single dimension value in a cube-region selection.
///
/// ## Specification
/// - **Type**: `CubeKeyValueType`
/// - **Element**: `<Value>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/CubeKeyValueType.md"))]
///
/// A dimension value carries an optional `cascade` flag (whether the selection reaches child codes
/// in a simple hierarchy) and an optional validity window. It admits no language tag: a dimension
/// value is a code reference, not localised text, so the schema prohibits `xml:lang` here.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Invariant-free pub-field carrier with derived `Deserialize`: the value is held verbatim, `cascade`
is the validated `Cascade` enum, and the validity pair self-validates through `SdmxTimePeriod`. The
spec types `CubeKeyValueType` as a restriction of `SimpleComponentValueType` prohibiting `xml:lang`,
so the `lang` field present on `SimpleComponentValue` is simply absent here (D-0040), making the
illegal state unrepresentable by omission. The validity pair is `StandardTimePeriodType`, so it maps
to `SdmxTimePeriod` (D-0027); these are selection-level, distinct from `TimePeriodRange.period`.

Decisions: D-0040, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CubeKeyValue {
    /// The dimension value content, held verbatim.
    pub value: String,
    /// How the selection cascades through a code hierarchy; `None` ⟺ absent.
    pub cascade: Option<Cascade>,
    /// The start of the value's validity window, if stated.
    pub valid_from: Option<SdmxTimePeriod>,
    /// The end of the value's validity window, if stated.
    pub valid_to: Option<SdmxTimePeriod>,
}

/// A single component (attribute or measure) value in a cube-region selection.
///
/// ## Specification
/// - **Type**: `SimpleComponentValueType`
/// - **Element**: `<Value>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/SimpleComponentValueType.md"))]
///
/// A component value carries an optional `cascade` flag, an optional single language tag (a
/// component value may be localised text), and an optional validity window. The language tag is a
/// loose single string, not a multi-language collection.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Invariant-free pub-field carrier with derived `Deserialize`. `lang` is the loose single-tag
`Option<String>` shape (D-0011 `AnnotationUrl` precedent), not a `LocalisedString`: the wire carries
one optional `xml:lang` per value. The validity pair maps to `SdmxTimePeriod` (D-0027). The spec
reuses this type for metadata-attribute values, which 0010 leaves out of scope (D-0034); the name is
already correct should that boundary ever move.

Decisions: D-0040, D-0011, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SimpleComponentValue {
    /// The component value content, held verbatim.
    pub value: String,
    /// How the selection cascades through a code hierarchy; `None` ⟺ absent.
    pub cascade: Option<Cascade>,
    /// The single language tag for this value, if stated.
    pub lang: Option<String>,
    /// The start of the value's validity window, if stated.
    pub valid_from: Option<SdmxTimePeriod>,
    /// The end of the value's validity window, if stated.
    pub valid_to: Option<SdmxTimePeriod>,
}

/// A non-empty list of [`CubeKeyValue`]s.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps the `Value+` of a dimension selection. The schema requires at least one value when the
/// value arm is chosen, so the constructor rejects an empty list.
///
/// ## Guarantees
///
/// Always holds at least one [`CubeKeyValue`].
///
/// # Examples
///
/// ```
/// use sdmx_types::{CubeKeyValue, CubeKeyValues};
///
/// let value =
///     CubeKeyValue { value: String::from("A"), cascade: None, valid_from: None, valid_to: None };
/// let values = CubeKeyValues::new(vec![value])?;
/// assert_eq!(values.as_slice().len(), 1);
///
/// // An empty selection is mechanically schema-invalid.
/// assert!(CubeKeyValues::new(Vec::new()).is_err());
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct CubeKeyValues(Vec<CubeKeyValue>);

impl CubeKeyValues {
    /// Builds a dimension value list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyCubeKeyValues`] if `values` is empty (a chosen value arm requires
    /// `Value+`).
    pub fn new(values: Vec<CubeKeyValue>) -> Result<Self, Error> {
        if values.is_empty() {
            return Err(Error::EmptyCubeKeyValues);
        }
        Ok(Self(values))
    }

    /// The dimension values, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[CubeKeyValue] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<CubeKeyValue> {
        self.0
    }
}

impl From<CubeKeyValues> for Vec<CubeKeyValue> {
    fn from(value: CubeKeyValues) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<CubeKeyValue>> for CubeKeyValues {
    type Error = Error;

    fn try_from(values: Vec<CubeKeyValue>) -> Result<Self, Error> {
        Self::new(values)
    }
}

impl<'de> serde::Deserialize<'de> for CubeKeyValues {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = Vec::<CubeKeyValue>::deserialize(deserializer)?;
        Self::new(values).map_err(to_de_error)
    }
}

/// A non-empty list of [`SimpleComponentValue`]s.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps the `Value+` of a component selection. The schema requires at least one value when the
/// value arm is chosen, so the constructor rejects an empty list.
///
/// ## Guarantees
///
/// Always holds at least one [`SimpleComponentValue`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct SimpleComponentValues(Vec<SimpleComponentValue>);

impl SimpleComponentValues {
    /// Builds a component value list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptySimpleComponentValues`] if `values` is empty (a chosen value arm
    /// requires `Value+`).
    pub fn new(values: Vec<SimpleComponentValue>) -> Result<Self, Error> {
        if values.is_empty() {
            return Err(Error::EmptySimpleComponentValues);
        }
        Ok(Self(values))
    }

    /// The component values, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[SimpleComponentValue] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<SimpleComponentValue> {
        self.0
    }
}

impl From<SimpleComponentValues> for Vec<SimpleComponentValue> {
    fn from(value: SimpleComponentValues) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<SimpleComponentValue>> for SimpleComponentValues {
    type Error = Error;

    fn try_from(values: Vec<SimpleComponentValue>) -> Result<Self, Error> {
        Self::new(values)
    }
}

impl<'de> serde::Deserialize<'de> for SimpleComponentValues {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = Vec::<SimpleComponentValue>::deserialize(deserializer)?;
        Self::new(values).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// Time ranges
// ---------------------------------------------------------------------------

/// One endpoint of a time-range selection: a period and whether it falls inside the range.
///
/// ## Specification
/// - **Type**: `TimePeriodRangeType`
/// - **Element**: N/A (Base Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/TimePeriodRangeType.md"))]
///
/// The endpoint is the type of the `<BeforePeriod>`, `<AfterPeriod>`, `<StartPeriod>`, and
/// `<EndPeriod>` elements of a [`TimeRange`]. The `period` content is the
/// `ObservationalTimePeriodType` union, so it admits time-range lexemes (a start-and-duration
/// form) as well as standard periods. The `inclusive` flag defaults to `true` when absent.
///
/// # Examples
///
/// ```
/// use sdmx_types::TimePeriodRange;
///
/// let stated = TimePeriodRange { period: "2024".parse()?, inclusive: Some(false) };
/// assert!(!stated.effective_is_inclusive());
///
/// // Absent flag resolves to the schema default of true.
/// let absent = TimePeriodRange { period: "2024".parse()?, inclusive: None };
/// assert!(absent.effective_is_inclusive());
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Invariant-free pub-field carrier with derived `Deserialize`. `period` is the
`ObservationalTimePeriodType` union (`StandardTimePeriodType ∪ TimeRangeType`, report-5 V-7), a
strict superset of `SdmxTimePeriod`, carried by the `ObservationalTimePeriod` union type
(D-0072): a Standard-only newtype here would reject schema-valid time-range lexemes.
`isInclusive` has schema `default="true"`, so statedness is stored (D-0052) and
`effective_is_inclusive()` is the resolved view.

Decisions: D-0031, D-0052, D-0072.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TimePeriodRange {
    /// The period content: a standard period or a time-range lexeme.
    pub period: ObservationalTimePeriod,
    /// Whether the period falls inside the range; `None` ⟺ absent (schema default `true`).
    pub inclusive: Option<bool>,
}

impl TimePeriodRange {
    /// Effective: whether the period falls inside the range, with the schema default `true`
    /// applied when the flag was absent.
    #[must_use]
    pub fn effective_is_inclusive(&self) -> bool {
        self.inclusive.unwrap_or(true)
    }
}

/// The kind of a [`TimeRange`] selection: before a period, after a period, or between two.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Models the choice within a time range. The variants mirror the spec choice elements
/// (`BeforePeriod`, `AfterPeriod`, and the `StartPeriod`/`EndPeriod` pair) with the redundant
/// `Period` suffix dropped.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The content choice of `TimeRangeValueType`, lifted into its own enum so the type-level
`validFrom`/`validTo` attributes sit beside it on [`TimeRange`] rather than being lost. The
`Between` field names `start`/`end` mirror the spec's `StartPeriod`/`EndPeriod` with the suffix
dropped, as `Before`/`After` already do. Derived `Deserialize`: it composes the already-derived
`TimePeriodRange`, so there is no between-field invariant (§7).

Decisions: D-0026, D-0064.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TimeRangeKind {
    /// The range covers everything before this period.
    Before(TimePeriodRange),
    /// The range covers everything after this period.
    After(TimePeriodRange),
    /// The range covers everything between these two periods.
    Between {
        /// The start of the range.
        start: TimePeriodRange,
        /// The end of the range.
        end: TimePeriodRange,
    },
}

/// A time-range selection: a span of time, optionally bounded by its own validity window.
///
/// ## Specification
/// - **Type**: `TimeRangeValueType`
/// - **Element**: `<TimeRange>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/TimeRangeValueType.md"))]
///
/// A time range expresses a value selection as a span rather than an enumerated list: before a
/// period, after a period, or between two ([`TimeRangeKind`]). It carries its own optional validity
/// window. On a component selection, where per-value validity is prohibited, this pair is the only
/// validity a time-range selection can carry, so it is held distinctly from the span itself.
///
/// # Examples
///
/// ```
/// use sdmx_types::{TimePeriodRange, TimeRange, TimeRangeKind};
///
/// let start = TimePeriodRange { period: "2020".parse()?, inclusive: None };
/// let end = TimePeriodRange { period: "2024".parse()?, inclusive: Some(false) };
/// let range =
///     TimeRange { kind: TimeRangeKind::Between { start, end }, valid_from: None, valid_to: None };
/// assert!(matches!(range.kind, TimeRangeKind::Between { .. }));
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The full `TimeRangeValueType`: the content choice (`kind`) plus the type-level `validFrom`/`validTo`
attributes the earlier enum-only `TimeRange` dropped, a lossless-superset violation D-0064 corrected.
Those attributes are `StandardTimePeriodType`, so they map to `SdmxTimePeriod` (D-0027), distinct
from the endpoint content on `TimePeriodRange.period`. They are optional with no schema default, so
plain statedness (`None` ⟺ absent), no effective view. Invariant-free pub-field carrier with derived
`Deserialize`: it composes `TimePeriodRange` and the self-validating `SdmxTimePeriod`, so there is no
between-field invariant and the validity pair reuses `Error::InvalidTimePeriod`.

Decisions: D-0064, D-0027, D-0031, D-0026, D-0038, D-0040.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TimeRange {
    /// The span the range covers.
    pub kind: TimeRangeKind,
    /// The start of the range's own validity window, if stated.
    pub valid_from: Option<SdmxTimePeriod>,
    /// The end of the range's own validity window, if stated.
    pub valid_to: Option<SdmxTimePeriod>,
}

// ---------------------------------------------------------------------------
// Cube-region selection nodes
// ---------------------------------------------------------------------------

/// The value choice of a dimension selection: an enumerated value list or a time range.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Models the content of a [`CubeRegionKey`]. A dimension selection always names something, so the
/// choice is mandatory: there is no empty state. The value list is non-empty by construction.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The content model of `CubeRegionKeyType`; the node type (this choice plus the attributes) is
[`CubeRegionKey`]. The choice is mandatory (`Value+` or `TimeRange`), so there is no `Empty` variant:
a dimension-empty selection is unrepresentable. It composes the already-valid `CubeKeyValues`
newtype, so it keeps derived `Deserialize` (§7 cross-field rule).

Decisions: D-0038, D-0040.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum KeyValueSelection {
    /// An enumerated list of dimension values (always at least one).
    Values(CubeKeyValues),
    /// A time-range selection.
    TimeRange(TimeRange),
}

/// The value choice of a component selection: an enumerated value list, a time range, or no values.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Models the content of a [`ComponentValueSet`]. Unlike a dimension selection, a component may be
/// referenced with no values at all, a distinct state from "all values": [`Empty`](Self::Empty)
/// captures it. Combined with the node's `include` flag, an empty selection that is included means
/// "present regardless of value", and one that is excluded means "absent".
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The content model of `ComponentValueSetType`; the node type is [`ComponentValueSet`]. The choice is
optional, so `Empty` is a real, distinct no-values state, not "all values". With `Values` non-empty
by construction (D-0038), `Empty` is the sole no-values state, mirroring the wire exactly (a
value-less `<Component>` versus a chosen `Value+` arm), so the old `Values(Vec::new())`-duplicates-`Empty`
ambiguity is unrepresentable. Derived `Deserialize` (it composes already-valid pieces, §7).

Decisions: D-0026, D-0038.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ComponentSelection {
    /// An enumerated list of component values (always at least one).
    Values(SimpleComponentValues),
    /// A time-range selection.
    TimeRange(TimeRange),
    /// The component is referenced with no values, distinct from naming every value.
    Empty,
}

/// A dimension selection within a cube region: a dimension id and the values it admits.
///
/// ## Specification
/// - **Type**: `CubeRegionKeyType`
/// - **Element**: `<KeyValue>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/CubeRegionKeyType.md"))]
///
/// Names a dimension by its `id` and the values it selects ([`KeyValueSelection`]). The `id` is
/// validated against `SingleNCNameIDType` (the `NCNameIDType` pattern) at construction (D-0077);
/// whether it names a declared dimension stays a higher-layer concern (D-0020). The `include` flag (whether the
/// named values are included in or excluded from the region) defaults to `true` when absent. A
/// dimension selection may carry its own validity window.
///
/// ## Guarantees
///
/// Always holds a `SingleNCNameIDType`-valid id.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A node type named after its spec complexType. `id` is carried on the struct (D-0051; formerly the
map key) and validated against `SingleNCNameIDType` (the `NCNameIDType` pattern, both editions) at
construction (D-0077); whether it resolves to a declared dimension stays a higher-layer concern
(D-0020). The `MemberSelectionType` attributes are inherited (D-0038): `include` (schema
`default="true"`, statedness stored, D-0052), `removePrefix` (no schema default, so `Option<bool>`
distinguishes absent from stated, D-0031), and the validity window (`StandardTimePeriodType`, so
`SdmxTimePeriod`, D-0027), which `CubeRegionKeyType` may carry. The id invariant lives on the struct,
so it crosses the §7 carrier→invariant-bearing line: private fields, fallible `new()`, accessors, and
a `Deserialize` that routes through `new()`; the non-empty values newtype and `SdmxTimePeriod` carry
their own validating paths.

Decisions: D-0020, D-0038, D-0051, D-0052, D-0077.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct CubeRegionKey {
    id: String,
    selection: KeyValueSelection,
    include: Option<bool>,
    remove_prefix: Option<bool>,
    valid_from: Option<SdmxTimePeriod>,
    valid_to: Option<SdmxTimePeriod>,
}

impl CubeRegionKey {
    /// Builds a cube-region dimension selection, validating the `id` against SDMX
    /// `SingleNCNameIDType` (the `NCNameIDType` pattern).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNcNameIdentifier`] if `id` is not a valid `NCNameIDType` lexeme
    /// (which also rejects the empty string).
    pub fn new(
        id: String,
        selection: KeyValueSelection,
        include: Option<bool>,
        remove_prefix: Option<bool>,
        valid_from: Option<SdmxTimePeriod>,
        valid_to: Option<SdmxTimePeriod>,
    ) -> Result<Self, Error> {
        validate_ncname(&id)?;
        Ok(Self { id, selection, include, remove_prefix, valid_from, valid_to })
    }

    /// The id of the dimension being selected.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The values this dimension admits.
    #[must_use]
    pub const fn selection(&self) -> &KeyValueSelection {
        &self.selection
    }

    /// Stated: whether the named values are included or excluded, as the wire carried it.
    /// `None` ⟺ absent.
    #[must_use]
    pub const fn include(&self) -> Option<bool> {
        self.include
    }

    /// Stated: whether codes drop the codelist-extension prefix. `None` ⟺ absent (no schema default).
    #[must_use]
    pub const fn remove_prefix(&self) -> Option<bool> {
        self.remove_prefix
    }

    /// The start of the selection's validity window, if stated.
    #[must_use]
    pub const fn valid_from(&self) -> Option<&SdmxTimePeriod> {
        self.valid_from.as_ref()
    }

    /// The end of the selection's validity window, if stated.
    #[must_use]
    pub const fn valid_to(&self) -> Option<&SdmxTimePeriod> {
        self.valid_to.as_ref()
    }

    /// Effective: whether the named values are included, with the schema default `true` applied.
    #[must_use]
    pub fn effective_is_included(&self) -> bool {
        self.include.unwrap_or(true)
    }
}

impl<'de> serde::Deserialize<'de> for CubeRegionKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            id: String,
            selection: KeyValueSelection,
            include: Option<bool>,
            remove_prefix: Option<bool>,
            valid_from: Option<SdmxTimePeriod>,
            valid_to: Option<SdmxTimePeriod>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(
            raw.id,
            raw.selection,
            raw.include,
            raw.remove_prefix,
            raw.valid_from,
            raw.valid_to,
        )
        .map_err(to_de_error)
    }
}

/// A component selection within a cube region: a component id and the values it admits.
///
/// ## Specification
/// - **Type**: `ComponentValueSetType`
/// - **Element**: `<Component>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ComponentValueSetType.md"))]
///
/// Names an attribute or measure by its `id` and the values it selects ([`ComponentSelection`]). The
/// `id` may be a nested identifier (a dotted metadata-attribute path such as `CONTACT.ADDRESS.STREET`
/// is one lexeme), validated against `NestedNCNameIDType` at construction (D-0077). Unlike a dimension
/// selection, a component selection carries no validity window: the schema prohibits one here, so the
/// field is simply absent.
///
/// ## Guarantees
///
/// Always holds a `NestedNCNameIDType`-valid id.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A node type named after its spec complexType. `id` is carried on the struct (D-0051) and validated
against `NestedNCNameIDType` (dotted, e.g. `CONTACT.ADDRESS.STREET`, both editions) at construction
(D-0077); whether it resolves to a declared component stays a higher-layer concern (D-0020). It
carries the same `include`/`removePrefix` node attributes as [`CubeRegionKey`] but no
`validFrom`/`validTo`: `ComponentValueSetType` prohibits them (both editions), so the illegal state
is unrepresentable by field omission. The id invariant lives on the struct, so it crosses the §7
carrier→invariant-bearing line: private fields, fallible `new()`, accessors, and a `Deserialize`
that routes through `new()`.

Decisions: D-0020, D-0038, D-0051, D-0052, D-0077.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ComponentValueSet {
    id: String,
    selection: ComponentSelection,
    include: Option<bool>,
    remove_prefix: Option<bool>,
}

impl ComponentValueSet {
    /// Builds a cube-region component selection, validating the `id` against SDMX
    /// `NestedNCNameIDType` (a dotted sequence of `NCName` segments).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNestedNcNameIdentifier`] if `id` is not a valid `NestedNCNameIDType`
    /// lexeme (which also rejects the empty string and leading, trailing, or doubled dots).
    pub fn new(
        id: String,
        selection: ComponentSelection,
        include: Option<bool>,
        remove_prefix: Option<bool>,
    ) -> Result<Self, Error> {
        validate_nested_ncname(&id)?;
        Ok(Self { id, selection, include, remove_prefix })
    }

    /// The id of the component being selected (possibly nested).
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The values this component admits.
    #[must_use]
    pub const fn selection(&self) -> &ComponentSelection {
        &self.selection
    }

    /// Stated: whether the named values are included or excluded, as the wire carried it.
    /// `None` ⟺ absent.
    #[must_use]
    pub const fn include(&self) -> Option<bool> {
        self.include
    }

    /// Stated: whether codes drop the codelist-extension prefix. `None` ⟺ absent (no schema default).
    #[must_use]
    pub const fn remove_prefix(&self) -> Option<bool> {
        self.remove_prefix
    }

    /// Effective: whether the named values are included, with the schema default `true` applied.
    #[must_use]
    pub fn effective_is_included(&self) -> bool {
        self.include.unwrap_or(true)
    }
}

impl<'de> serde::Deserialize<'de> for ComponentValueSet {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            id: String,
            selection: ComponentSelection,
            include: Option<bool>,
            remove_prefix: Option<bool>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.id, raw.selection, raw.include, raw.remove_prefix).map_err(to_de_error)
    }
}

/// A region of a data cube: the dimension and component selections that bound it.
///
/// ## Specification
/// - **Type**: `CubeRegionType`
/// - **Element**: `<CubeRegion>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/CubeRegionType.md"))]
///
/// A cube region gathers the dimension selections ([`CubeRegionKey`]) and component selections
/// ([`ComponentValueSet`]) that define it, each kept in wire order. A dimension absent from the
/// selections is unconstrained ("all values in scope"). The region-level `include` flag defaults to
/// `true`. A region may carry annotations.
///
/// # Examples
///
/// ```
/// use sdmx_types::{CubeKeyValue, CubeKeyValues, CubeRegion, CubeRegionKey, KeyValueSelection};
///
/// let value =
///     CubeKeyValue { value: String::from("A"), cascade: None, valid_from: None, valid_to: None };
/// let key = CubeRegionKey::new(
///     String::from("FREQ"),
///     KeyValueSelection::Values(CubeKeyValues::new(vec![value])?),
///     None,
///     None,
///     None,
///     None,
/// )?;
/// let region = CubeRegion {
///     key_values: vec![key],
///     components: Vec::new(),
///     include: None,
///     annotations: Vec::new(),
/// };
/// assert_eq!(region.key_values.len(), 1);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Pub-field carrier with derived `Deserialize`: the two selection collections are a wire sequence
(`KeyValue*` then `Component*`, nothing interleaves), so two `Vec`s map field-by-field, every field
self-enforcing (D-0051), and no custom impl is needed. `include` exists at both levels (D-0038):
region-level here, per-selection on the node structs. Region-level `validFrom`/`validTo` are
prohibited on `CubeRegionType`, so there are no such fields. `RegionType` extends
`common:AnnotableType` (both editions), so a region is annotable; `CubeRegion` is non-identifiable,
so it carries the annotations directly. The `Vec<Annotation>` maps the wire's two states exactly,
empty ⟺ absent (D-0033, D-0031). The prose-only "each key component only once" rule is a catalogued
lint (D-0051): duplicate selection ids are schema-valid wire, held verbatim.

Decisions: D-0026, D-0033, D-0051, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CubeRegion {
    /// The dimension selections, in wire order.
    pub key_values: Vec<CubeRegionKey>,
    /// The component (attribute or measure) selections, in wire order.
    pub components: Vec<ComponentValueSet>,
    /// Whether the region is included or excluded; `None` ⟺ absent (schema default `true`).
    pub include: Option<bool>,
    /// The region's annotations; empty ⟺ absent.
    pub annotations: Vec<Annotation>,
}

impl CubeRegion {
    /// Effective: whether the region is included, with the schema default `true` applied.
    #[must_use]
    pub fn effective_is_included(&self) -> bool {
        self.include.unwrap_or(true)
    }
}

/// A bounded list of [`CubeRegion`]s, holding at most two.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps the cube regions of a data constraint. The schema caps the count at two, so the constructor
/// rejects more. It does not reject an empty list: a data constraint may carry no regions at all
/// (expressed instead through key sets).
///
/// ## Guarantees
///
/// Always holds at most two [`CubeRegion`]s.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct CubeRegions(Vec<CubeRegion>);

impl CubeRegions {
    /// Builds a cube-region list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooManyCubeRegions`] if `regions` holds more than two (the schema caps the
    /// count at `maxOccurs="2"`).
    pub fn new(regions: Vec<CubeRegion>) -> Result<Self, Error> {
        if regions.len() > 2 {
            return Err(Error::TooManyCubeRegions);
        }
        Ok(Self(regions))
    }

    /// The cube regions, in order (at most two).
    #[must_use]
    pub fn as_slice(&self) -> &[CubeRegion] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<CubeRegion> {
        self.0
    }
}

impl From<CubeRegions> for Vec<CubeRegion> {
    fn from(value: CubeRegions) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<CubeRegion>> for CubeRegions {
    type Error = Error;

    fn try_from(regions: Vec<CubeRegion>) -> Result<Self, Error> {
        Self::new(regions)
    }
}

impl<'de> serde::Deserialize<'de> for CubeRegions {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let regions = Vec::<CubeRegion>::deserialize(deserializer)?;
        Self::new(regions).map_err(to_de_error)
    }
}

// ---------------------------------------------------------------------------
// Data key sets
// ---------------------------------------------------------------------------

/// A single component (attribute or measure) value in a data key set.
///
/// ## Specification
/// - **Type**: `DataComponentValueType`
/// - **Element**: `<Value>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataComponentValueType.md"))]
///
/// The key-set counterpart of [`SimpleComponentValue`]. It carries an optional `cascade` flag and an
/// optional single language tag, but no validity window: the schema prohibits one for a key-set
/// value, so the field is simply absent.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Invariant-free pub-field carrier with derived `Deserialize`. The spec types `DataComponentValueType`
as a restriction of `SimpleComponentValueType` prohibiting `validFrom`/`validTo`, so the validity
pair present on `SimpleComponentValue` is simply absent here (D-0039), making the illegal state
unrepresentable by omission. `lang` is the loose single-tag `Option<String>` (D-0011 precedent).

Decisions: D-0039, D-0040, D-0011, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataComponentValue {
    /// The component value content, held verbatim.
    pub value: String,
    /// How the selection cascades through a code hierarchy; `None` ⟺ absent.
    pub cascade: Option<Cascade>,
    /// The single language tag for this value, if stated.
    pub lang: Option<String>,
}

/// A non-empty list of [`DataComponentValue`]s.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps the `Value+` of a key-set component selection. The schema requires at least one value when
/// the value arm is chosen, so the constructor rejects an empty list.
///
/// ## Guarantees
///
/// Always holds at least one [`DataComponentValue`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct DataComponentValues(Vec<DataComponentValue>);

impl DataComponentValues {
    /// Builds a key-set component value list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyDataComponentValues`] if `values` is empty (a chosen value arm requires
    /// `Value+`).
    pub fn new(values: Vec<DataComponentValue>) -> Result<Self, Error> {
        if values.is_empty() {
            return Err(Error::EmptyDataComponentValues);
        }
        Ok(Self(values))
    }

    /// The component values, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[DataComponentValue] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<DataComponentValue> {
        self.0
    }
}

impl From<DataComponentValues> for Vec<DataComponentValue> {
    fn from(value: DataComponentValues) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<DataComponentValue>> for DataComponentValues {
    type Error = Error;

    fn try_from(values: Vec<DataComponentValue>) -> Result<Self, Error> {
        Self::new(values)
    }
}

impl<'de> serde::Deserialize<'de> for DataComponentValues {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = Vec::<DataComponentValue>::deserialize(deserializer)?;
        Self::new(values).map_err(to_de_error)
    }
}

/// The value choice of a key-set component selection: a value list, a time range, or no values.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// The key-set counterpart of [`ComponentSelection`]. The choice is optional, so
/// [`Empty`](Self::Empty) is the distinct no-values state, with the same `include`-interaction
/// reading: an empty selection that is included means "present regardless of value", and one that is
/// excluded means "absent".
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The content model of `DataComponentValueSetType`, mirroring the cube side's `ComponentSelection`. The
choice is optional, so `Empty` is the no-values state; with `Values` non-empty by construction
(D-0038), `Empty` is the sole no-values state. Derived `Deserialize` (composes already-valid pieces,
§7).

Decisions: D-0026, D-0038, D-0039.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DataComponentSelection {
    /// An enumerated list of component values (always at least one).
    Values(DataComponentValues),
    /// A time-range selection.
    TimeRange(TimeRange),
    /// The component is referenced with no values, distinct from naming every value.
    Empty,
}

/// A component selection within a data key: a component id and the values it admits.
///
/// ## Specification
/// - **Type**: `DataComponentValueSetType`
/// - **Element**: `<Component>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataComponentValueSetType.md"))]
///
/// The key-set counterpart of [`ComponentValueSet`]. It names an attribute or measure by its `id`
/// (possibly nested, validated against `NestedNCNameIDType` at construction, D-0077) and the values
/// it selects ([`DataComponentSelection`]), and carries the same `include`/`remove_prefix` node
/// attributes but no validity window (prohibited here).
///
/// ## Guarantees
///
/// Always holds a `NestedNCNameIDType`-valid id.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A node type named after its spec complexType. `id` is carried on the struct (D-0051) and validated
against `NestedNCNameIDType` (both editions) at construction (D-0077); whether it resolves to a
declared component stays a higher-layer concern (D-0020). Same node attributes as the cube
[`ComponentValueSet`]: `include` (schema `default="true"`, statedness stored, D-0052) and
`removePrefix` (no default, `Option<bool>`, D-0031). Validity is prohibited, so there are no such
fields. The id invariant lives on the struct, so it crosses the §7 carrier→invariant-bearing line:
private fields, fallible `new()`, accessors, and a `Deserialize` that routes through `new()`.

Decisions: D-0020, D-0038, D-0039, D-0051, D-0052, D-0077.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct DataComponentValueSet {
    id: String,
    selection: DataComponentSelection,
    include: Option<bool>,
    remove_prefix: Option<bool>,
}

impl DataComponentValueSet {
    /// Builds a data-key component selection, validating the `id` against SDMX `NestedNCNameIDType`
    /// (a dotted sequence of `NCName` segments).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNestedNcNameIdentifier`] if `id` is not a valid `NestedNCNameIDType`
    /// lexeme (which also rejects the empty string and leading, trailing, or doubled dots).
    pub fn new(
        id: String,
        selection: DataComponentSelection,
        include: Option<bool>,
        remove_prefix: Option<bool>,
    ) -> Result<Self, Error> {
        validate_nested_ncname(&id)?;
        Ok(Self { id, selection, include, remove_prefix })
    }

    /// The id of the component being selected (possibly nested).
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The values this component admits.
    #[must_use]
    pub const fn selection(&self) -> &DataComponentSelection {
        &self.selection
    }

    /// Stated: whether the named values are included or excluded, as the wire carried it.
    /// `None` ⟺ absent.
    #[must_use]
    pub const fn include(&self) -> Option<bool> {
        self.include
    }

    /// Stated: whether codes drop the codelist-extension prefix. `None` ⟺ absent (no schema default).
    #[must_use]
    pub const fn remove_prefix(&self) -> Option<bool> {
        self.remove_prefix
    }

    /// Effective: whether the named values are included, with the schema default `true` applied.
    #[must_use]
    pub fn effective_is_included(&self) -> bool {
        self.include.unwrap_or(true)
    }
}

impl<'de> serde::Deserialize<'de> for DataComponentValueSet {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            id: String,
            selection: DataComponentSelection,
            include: Option<bool>,
            remove_prefix: Option<bool>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.id, raw.selection, raw.include, raw.remove_prefix).map_err(to_de_error)
    }
}

/// A non-empty list of bare dimension key values.
///
/// ## Specification
/// - **Type**: `SimpleKeyValueType`
/// - **Element**: `<Value>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/SimpleKeyValueType.md"))]
///
/// The dimension values of a [`DataKeyValue`]. A key value is a bare string: the schema prohibits
/// every attribute (`cascade`, language, validity) on it, so there is no per-value structure to
/// carry. The schema requires at least one value, so the constructor rejects an empty list.
///
/// ## Guarantees
///
/// Always holds at least one value.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`SimpleKeyValueType` restricts `SimpleComponentValueType`, prohibiting every attribute (`cascade`,
`xml:lang`, `validFrom`/`validTo`), so its content is bare `xs:string`; the values are bare `String`s
(a wrapper would invent structure that the schema forbids). The spec citation rides on this newtype
because there is no per-value leaf struct to carry it. The non-empty bound carries the 3.1 unbounded
shape, which covers 3.0's single value (the `DataKeyValueType` divergence, D-0039); `new()` rejects
empty (D-0031).

Decisions: D-0039, D-0040, D-0031.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct SimpleKeyValues(Vec<String>);

impl SimpleKeyValues {
    /// Builds a dimension key-value list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptySimpleKeyValues`] if `values` is empty (a chosen value arm requires
    /// `Value+`).
    pub fn new(values: Vec<String>) -> Result<Self, Error> {
        if values.is_empty() {
            return Err(Error::EmptySimpleKeyValues);
        }
        Ok(Self(values))
    }

    /// The key values, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<String> {
        self.0
    }
}

impl From<SimpleKeyValues> for Vec<String> {
    fn from(value: SimpleKeyValues) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<String>> for SimpleKeyValues {
    type Error = Error;

    fn try_from(values: Vec<String>) -> Result<Self, Error> {
        Self::new(values)
    }
}

impl<'de> serde::Deserialize<'de> for SimpleKeyValues {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = Vec::<String>::deserialize(deserializer)?;
        Self::new(values).map_err(to_de_error)
    }
}

/// A dimension selection within a data key: a dimension id and its values.
///
/// ## Specification
/// - **Type**: `DataKeyValueType`
/// - **Element**: `<KeyValue>`
/// - **Editions**: SDMX 3.0 and 3.1 (Divergent)
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataKeyValueType.3.0.md"))]
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataKeyValueType.3.1.md"))]
#[cfg_attr(design_docs, doc = "")]
///
/// Names a dimension by its `id` and the values it takes ([`SimpleKeyValues`]). The `id` is validated
/// against `SingleNCNameIDType` (the `NCNameIDType` pattern) at construction (D-0077). Its `include`
/// flag is schema-fixed to `true`, so it is the [`FixedInclude`] wrapper. A data-key selection
/// carries no validity window (prohibited here).
///
/// ## Guarantees
///
/// Always holds a `SingleNCNameIDType`-valid id.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`DataKeyValueType` diverges across editions: 3.0 allows exactly one `Value` (a `<choice>`); 3.1
allows unbounded (a `<sequence maxOccurs="unbounded">`, for keys like `FREQ = A or M or Q`). The
superset carries the 3.1 shape as the non-empty [`SimpleKeyValues`], which covers 3.0's exactly-one;
what a 3.0 writer does with more than one value is Phase-2 adapter policy (the same provenance class
as a 3.1-only attribute, D-0039/D-0037), not a Phase-1 rejection. `id` is validated against
`SingleNCNameIDType` (the `NCNameIDType` pattern, both editions) at construction (D-0077/D-0051);
whether it resolves to a declared dimension stays a higher-layer concern (D-0020). `include` is
`fixed="true"`, so the `FixedInclude` wrapper stores statedness and rejects a stated `false`
(D-0052). Validity is prohibited, so there are no such fields. The id invariant lives on the struct,
so it crosses the §7 carrier→invariant-bearing line: private fields, fallible `new()`, accessors, and
a `Deserialize` that routes through `new()` (which also carries the `FixedInclude` and
`SimpleKeyValues` within-field rejections).

Decisions: D-0020, D-0039, D-0051, D-0052, D-0077.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct DataKeyValue {
    id: String,
    values: SimpleKeyValues,
    include: FixedInclude,
    remove_prefix: Option<bool>,
}

impl DataKeyValue {
    /// Builds a data-key dimension selection, validating the `id` against SDMX `SingleNCNameIDType`
    /// (the `NCNameIDType` pattern).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNcNameIdentifier`] if `id` is not a valid `NCNameIDType` lexeme
    /// (which also rejects the empty string).
    pub fn new(
        id: String,
        values: SimpleKeyValues,
        include: FixedInclude,
        remove_prefix: Option<bool>,
    ) -> Result<Self, Error> {
        validate_ncname(&id)?;
        Ok(Self { id, values, include, remove_prefix })
    }

    /// The id of the dimension being selected.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The values this dimension takes.
    #[must_use]
    pub const fn values(&self) -> &SimpleKeyValues {
        &self.values
    }

    /// The schema-fixed `include` flag (always effectively `true`).
    #[must_use]
    pub const fn include(&self) -> FixedInclude {
        self.include
    }

    /// Stated: whether codes drop the codelist-extension prefix. `None` ⟺ absent (no schema default).
    #[must_use]
    pub const fn remove_prefix(&self) -> Option<bool> {
        self.remove_prefix
    }
}

impl<'de> serde::Deserialize<'de> for DataKeyValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            id: String,
            values: SimpleKeyValues,
            include: FixedInclude,
            remove_prefix: Option<bool>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.id, raw.values, raw.include, raw.remove_prefix).map_err(to_de_error)
    }
}

/// A distinct full or partial data key: the dimension and component values that identify it.
///
/// ## Specification
/// - **Type**: `DataKeyType`
/// - **Element**: `<Key>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataKeyType.md"))]
///
/// A data key gathers the dimension selections ([`DataKeyValue`]) and component selections
/// ([`DataComponentValueSet`]) that identify it, each in wire order. A dimension absent from the key
/// is wildcarded, which is how a partial key is expressed. Its `include` flag is schema-fixed to
/// `true`. A data key may carry its own validity window and annotations.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`DataKeyType` is a restriction of `RegionType`: a data key is itself a region. Unlike
`CubeRegionType`, which prohibits them, it inherits `RegionType`'s optional `validFrom`/`validTo` (the
base type does not prohibit them on the key side), so those fields are present here. `include` is
`use="optional" fixed="true"`, so the `FixedInclude` wrapper (D-0052/D-0039). The two selection
collections are a wire sequence (`KeyValue*` then `Component*`), so two `Vec`s map field-by-field
(D-0051). Annotations sit on a non-identifiable annotable type, the bare-field case (D-0033), empty
⟺ absent (D-0031). Pub-field carrier: the rejection rides `FixedInclude`'s custom impl.

Decisions: D-0039, D-0033, D-0051, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataKey {
    /// The dimension selections, in wire order.
    pub key_values: Vec<DataKeyValue>,
    /// The component (attribute or measure) selections, in wire order.
    pub components: Vec<DataComponentValueSet>,
    /// The schema-fixed `include` flag (always effectively `true`).
    pub include: FixedInclude,
    /// The key's annotations; empty ⟺ absent.
    pub annotations: Vec<Annotation>,
    /// The start of the key's validity window, if stated.
    pub valid_from: Option<SdmxTimePeriod>,
    /// The end of the key's validity window, if stated.
    pub valid_to: Option<SdmxTimePeriod>,
}

/// A non-empty list of [`DataKey`]s.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps the `Key+` of a data key set. The schema requires at least one key, so the constructor
/// rejects an empty list.
///
/// ## Guarantees
///
/// Always holds at least one [`DataKey`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct DataKeys(Vec<DataKey>);

impl DataKeys {
    /// Builds a data-key list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyDataKeys`] if `keys` is empty (the schema requires `Key+`).
    pub fn new(keys: Vec<DataKey>) -> Result<Self, Error> {
        if keys.is_empty() {
            return Err(Error::EmptyDataKeys);
        }
        Ok(Self(keys))
    }

    /// The data keys, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[DataKey] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<DataKey> {
        self.0
    }
}

impl From<DataKeys> for Vec<DataKey> {
    fn from(value: DataKeys) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<DataKey>> for DataKeys {
    type Error = Error;

    fn try_from(keys: Vec<DataKey>) -> Result<Self, Error> {
        Self::new(keys)
    }
}

impl<'de> serde::Deserialize<'de> for DataKeys {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let keys = Vec::<DataKey>::deserialize(deserializer)?;
        Self::new(keys).map_err(to_de_error)
    }
}

/// A set of data keys, marked as included in or excluded from the constraint.
///
/// ## Specification
/// - **Type**: `DataKeySetType`
/// - **Element**: `<DataKeySet>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataKeySetType.md"))]
///
/// Gathers a non-empty list of [`DataKey`]s ([`DataKeys`]) and states whether they are included in or
/// excluded from the constraint. The `is_included` flag is required on the wire: there is no absent
/// state to preserve.
///
/// # Examples
///
/// ```
/// use sdmx_types::{DataKey, DataKeySet, DataKeyValue, DataKeys, FixedInclude, SimpleKeyValues};
///
/// let key_value = DataKeyValue::new(
///     String::from("FREQ"),
///     SimpleKeyValues::new(vec![String::from("A")])?,
///     FixedInclude::new(None)?,
///     None,
/// )?;
/// let key = DataKey {
///     key_values: vec![key_value],
///     components: Vec::new(),
///     include: FixedInclude::new(None)?,
///     annotations: Vec::new(),
///     valid_from: None,
///     valid_to: None,
/// };
/// let key_set = DataKeySet { keys: DataKeys::new(vec![key])?, is_included: true };
/// assert_eq!(key_set.keys.as_slice().len(), 1);
///
/// // A key set must hold at least one key.
/// assert!(DataKeys::new(Vec::new()).is_err());
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Pub-field carrier with derived `Deserialize`: the non-empty bound rides `DataKeys`' custom impl, and
`is_included` is a mandatory `bool`, not `Option<bool>`: `isIncluded` is `use="required"` with no
schema default (both editions), so absence is mechanically schema-invalid and there is no statedness
to store (contrast the `Option<bool>` node flags, D-0031/D-0052).

Decisions: D-0039, D-0031, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataKeySet {
    /// The data keys in this set (always at least one).
    pub keys: DataKeys,
    /// Whether the keys are included in or excluded from the constraint.
    pub is_included: bool,
}

// ---------------------------------------------------------------------------
// Queryable data source
// ---------------------------------------------------------------------------

/// A queryable SDMX data source a data constraint may be attached to.
///
/// ## Specification
/// - **Type**: `QueryableDataSourceType`
/// - **Element**: `<QueryableDataSource>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/QueryableDataSourceType.md"))]
///
/// Describes a data source that accepts an SDMX query: its data URL, optional WSDL and WADL service
/// descriptions, and whether it is reachable over REST and over web-service protocols (both flags
/// are required). The URLs are held verbatim, not validated.
///
/// # Examples
///
/// ```
/// use sdmx_types::QueryableDataSource;
///
/// let source = QueryableDataSource {
///     data_url: String::from("https://example.com/sdmx"),
///     wsdl_url: None,
///     wadl_url: None,
///     is_rest_data_source: true,
///     is_web_service_data_source: false,
/// };
/// assert!(source.is_rest_data_source);
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A 3.0-only constraint-attachment member (D-0044): the 3.0 data attachment trails
`QueryableDataSource` elements after its reference sequences; 3.1 keeps the type in `SDMXCommon` for
registry use only, gone from constraint attachments. The superset carries it regardless, the same
provenance class as `role` (D-0037) and `ReleaseCalendar` (D-0042). The two discriminator flags are
required attributes, so they are plain `bool`s with no statedness to preserve. The URLs are
unvalidated `xs:anyURI` (D-0014). Invariant-free pub-field carrier, derived `Deserialize`. It lives
here rather than in `reference.rs` because it is a data *source*, not an artefact reference, and its
only consumer is the 3.0 data attachment.

Decisions: D-0044, D-0014, D-0037.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct QueryableDataSource {
    /// The URL of the data source, held verbatim.
    pub data_url: String,
    /// The optional URL of a WSDL description of the source.
    pub wsdl_url: Option<String>,
    /// The optional URL of a WADL description of the source's REST protocol.
    pub wadl_url: Option<String>,
    /// Whether the source is reachable over the REST protocol.
    pub is_rest_data_source: bool,
    /// Whether the source is reachable over web-service protocols.
    pub is_web_service_data_source: bool,
}

// ---------------------------------------------------------------------------
// Constraint attachment
// ---------------------------------------------------------------------------

/// A non-empty list of data structure definition references.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps the `DataStructure+` of a data-constraint attachment. The chosen arm requires at least one
/// reference, so the constructor rejects an empty list.
///
/// ## Guarantees
///
/// Always holds at least one [`DataStructureReference`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct DataStructureRefs(Vec<DataStructureReference>);

impl DataStructureRefs {
    /// Builds a data-structure reference list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyDataStructureRefs`] if `refs` is empty (a chosen attachment arm
    /// requires at least one reference).
    pub fn new(refs: Vec<DataStructureReference>) -> Result<Self, Error> {
        if refs.is_empty() {
            return Err(Error::EmptyDataStructureRefs);
        }
        Ok(Self(refs))
    }

    /// The references, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[DataStructureReference] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<DataStructureReference> {
        self.0
    }
}

impl From<DataStructureRefs> for Vec<DataStructureReference> {
    fn from(value: DataStructureRefs) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<DataStructureReference>> for DataStructureRefs {
    type Error = Error;

    fn try_from(refs: Vec<DataStructureReference>) -> Result<Self, Error> {
        Self::new(refs)
    }
}

impl<'de> serde::Deserialize<'de> for DataStructureRefs {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(Vec::<DataStructureReference>::deserialize(deserializer)?).map_err(to_de_error)
    }
}

/// A non-empty list of dataflow references.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps the `Dataflow+` of a data-constraint attachment. The chosen arm requires at least one
/// reference, so the constructor rejects an empty list.
///
/// ## Guarantees
///
/// Always holds at least one [`DataflowReference`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct DataflowRefs(Vec<DataflowReference>);

impl DataflowRefs {
    /// Builds a dataflow reference list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyDataflowRefs`] if `refs` is empty (a chosen attachment arm requires at
    /// least one reference).
    pub fn new(refs: Vec<DataflowReference>) -> Result<Self, Error> {
        if refs.is_empty() {
            return Err(Error::EmptyDataflowRefs);
        }
        Ok(Self(refs))
    }

    /// The references, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[DataflowReference] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<DataflowReference> {
        self.0
    }
}

impl From<DataflowRefs> for Vec<DataflowReference> {
    fn from(value: DataflowRefs) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<DataflowReference>> for DataflowRefs {
    type Error = Error;

    fn try_from(refs: Vec<DataflowReference>) -> Result<Self, Error> {
        Self::new(refs)
    }
}

impl<'de> serde::Deserialize<'de> for DataflowRefs {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(Vec::<DataflowReference>::deserialize(deserializer)?).map_err(to_de_error)
    }
}

/// A non-empty list of provision agreement references.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps the `ProvisionAgreement+` of a data-constraint attachment. The chosen arm requires at least
/// one reference, so the constructor rejects an empty list.
///
/// ## Guarantees
///
/// Always holds at least one [`ProvisionAgreementReference`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct ProvisionAgreementRefs(Vec<ProvisionAgreementReference>);

impl ProvisionAgreementRefs {
    /// Builds a provision agreement reference list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyProvisionAgreementRefs`] if `refs` is empty (a chosen attachment arm
    /// requires at least one reference).
    pub fn new(refs: Vec<ProvisionAgreementReference>) -> Result<Self, Error> {
        if refs.is_empty() {
            return Err(Error::EmptyProvisionAgreementRefs);
        }
        Ok(Self(refs))
    }

    /// The references, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[ProvisionAgreementReference] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<ProvisionAgreementReference> {
        self.0
    }
}

impl From<ProvisionAgreementRefs> for Vec<ProvisionAgreementReference> {
    fn from(value: ProvisionAgreementRefs) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<ProvisionAgreementReference>> for ProvisionAgreementRefs {
    type Error = Error;

    fn try_from(refs: Vec<ProvisionAgreementReference>) -> Result<Self, Error> {
        Self::new(refs)
    }
}

impl<'de> serde::Deserialize<'de> for ProvisionAgreementRefs {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(Vec::<ProvisionAgreementReference>::deserialize(deserializer)?)
            .map_err(to_de_error)
    }
}

/// A non-empty list of simple data source URLs.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0
///
/// Wraps the `SimpleDataSource+` of a 3.0 data-constraint attachment: URLs of SDMX-ML data or
/// metadata messages. The chosen arm requires at least one URL, so the constructor rejects an empty
/// list. The URLs are held verbatim, not validated.
///
/// ## Guarantees
///
/// Always holds at least one URL.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct SimpleDataSources(Vec<String>);

impl SimpleDataSources {
    /// Builds a simple data source URL list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptySimpleDataSources`] if `urls` is empty (a chosen attachment arm
    /// requires at least one URL).
    pub fn new(urls: Vec<String>) -> Result<Self, Error> {
        if urls.is_empty() {
            return Err(Error::EmptySimpleDataSources);
        }
        Ok(Self(urls))
    }

    /// The URLs, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<String> {
        self.0
    }
}

impl From<SimpleDataSources> for Vec<String> {
    fn from(value: SimpleDataSources) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<String>> for SimpleDataSources {
    type Error = Error;

    fn try_from(urls: Vec<String>) -> Result<Self, Error> {
        Self::new(urls)
    }
}

impl<'de> serde::Deserialize<'de> for SimpleDataSources {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(Vec::<String>::deserialize(deserializer)?).map_err(to_de_error)
    }
}

/// What a data constraint is attached to: a data provider, data sources, or one or more structural
/// artefacts.
///
/// ## Specification
/// - **Type**: `DataConstraintAttachmentType`
/// - **Element**: `<ConstraintAttachment>`
/// - **Editions**: SDMX 3.0 and 3.1 (Divergent)
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataConstraintAttachmentType.3.0.md"))]
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataConstraintAttachmentType.3.1.md"))]
#[cfg_attr(design_docs, doc = "")]
///
/// A data constraint attaches to exactly one of: a single data provider; one or more data structure
/// definitions, dataflows, or provision agreements (each arm carrying any trailing queryable data
/// sources); or, in SDMX 3.0 only, a list of simple data source URLs. The three structural arms each
/// pair their references with the queryable sources that follow them on the wire.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`DataConstraintAttachmentType` restricts the abstract `ConstraintAttachmentType` to the data side
(D-0034). Modelling it as its own enum, rather than one flat shared attachment enum, makes the
illegal cross-attachment unrepresentable (an availability constraint cannot attach to a data
provider). Exhaustive (D-0021): a bounded, spec-fixed target set.

3.0/3.1 divergence (D-0044): the 3.0 wire adds a `SimpleDataSource` arm (`xs:anyURI`, unbounded) and
trailing `QueryableDataSource` elements inside each of the three `1..*` reference sequences; 3.1 has
neither. The superset carries both, the same provenance class as `role` (D-0037). The three `1..*`
arms are struct variants because the spec nests `Ref+` then `QueryableDataSource*` in one sequence
per arm; `queryable` is empty when absent, always empty on 3.1 wire. `DataProvider` is single.
Derived `Deserialize`: it composes the already-valid non-empty newtypes (§7 cross-field rule).

Decisions: D-0034, D-0044, D-0021.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DataConstraintAttachment {
    /// The constraint is attached to a single data provider.
    DataProvider(DataProviderReference),
    /// The constraint is attached to simple data source URLs (SDMX 3.0 only).
    SimpleDataSource(SimpleDataSources),
    /// The constraint is attached to one or more data structure definitions.
    DataStructure {
        /// The data structure definitions attached to.
        refs: DataStructureRefs,
        /// Any queryable data sources trailing the references (SDMX 3.0 only; empty otherwise).
        queryable: Vec<QueryableDataSource>,
    },
    /// The constraint is attached to one or more dataflows.
    Dataflow {
        /// The dataflows attached to.
        refs: DataflowRefs,
        /// Any queryable data sources trailing the references (SDMX 3.0 only; empty otherwise).
        queryable: Vec<QueryableDataSource>,
    },
    /// The constraint is attached to one or more provision agreements.
    ProvisionAgreement {
        /// The provision agreements attached to.
        refs: ProvisionAgreementRefs,
        /// Any queryable data sources trailing the references (SDMX 3.0 only; empty otherwise).
        queryable: Vec<QueryableDataSource>,
    },
}

/// What an availability constraint is attached to: a single structural artefact.
///
/// ## Specification
/// - **Type**: `AvailabilityConstraintAttachmentType`
/// - **Element**: `<ConstraintAttachment>`
/// - **Editions**: SDMX 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/AvailabilityConstraintAttachmentType.md"))]
///
/// An availability constraint attaches to a single data structure definition, dataflow, or provision
/// agreement. Unlike a data constraint, it admits no data provider and each target is single, not a
/// list.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`AvailabilityConstraintAttachmentType` restricts the abstract `ConstraintAttachmentType` to the data
subset, every target single (`maxOccurs="1"`), so plain refs without the non-empty-vec newtypes. It
excludes `DataProvider` (the spec omits it from this restriction). A 3.1-only type (3.0 has no
availability constraint); the superset carries it regardless. Exhaustive (D-0021). Modelling it
distinctly from `DataConstraintAttachment` makes the spec's narrower target set unrepresentable
otherwise (D-0034).

Decisions: D-0034, D-0021.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AvailabilityConstraintAttachment {
    /// The constraint is attached to a single data structure definition.
    DataStructure(DataStructureReference),
    /// The constraint is attached to a single dataflow.
    Dataflow(DataflowReference),
    /// The constraint is attached to a single provision agreement.
    ProvisionAgreement(ProvisionAgreementReference),
}

// ---------------------------------------------------------------------------
// Constraint maintainables and the unified model
// ---------------------------------------------------------------------------

/// Whether a constraint states the values allowed for an artefact or the data actually present.
///
/// ## Specification
/// - **Type**: `ConstraintRoleType`
/// - **Element**: N/A (Simple Type)
/// - **Editions**: SDMX 3.0
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ConstraintRoleType.md"))]
///
/// The `role` of an SDMX 3.0 data constraint: [`Allowed`](Self::Allowed) states the values an
/// artefact may take, [`Actual`](Self::Actual) states the data actually present. SDMX 3.1 drops the
/// attribute entirely.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The 3.0 `role` attribute (`ConstraintRoleType = Allowed | Actual`), required on every 3.0 constraint
and removed entirely in 3.1 (zero occurrences). A 3.0-only superset member (D-0037), the mirror of
the 3.1-only `isPartialLanguage`. Exhaustive (D-0021): a bounded, spec-fixed set. Only data
constraints can be `Actual`; the 3.0 metadata side pins `role` to `fixed="Allowed"`, out of scope
regardless (D-0034).

Decisions: D-0037, D-0021.
"#
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ConstraintRole {
    /// The constraint states the values allowed for the attached artefact.
    Allowed,
    /// The constraint states the data actually present.
    Actual,
}

/// The release schedule a data constraint may carry.
///
/// ## Specification
/// - **Type**: `ReleaseCalendarType`
/// - **Element**: `<ReleaseCalendar>`
/// - **Editions**: SDMX 3.0
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ReleaseCalendarType.md"))]
///
/// Describes when data is released: the `periodicity` between releases, the `offset` of the first
/// release in a year, and the `tolerance` after which a release is considered late. All three are
/// held verbatim as strings (the duration format is not validated).
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A 3.0-only type (zero occurrences in any 3.1 schema; on the 3.0 `ConstraintType` base with
`minOccurs="0"`, retained by the data-side restriction), the same 3.0-only superset provenance class
as `role` (D-0042/D-0037). All three elements are required `xs:string`; the `P7D`-style duration
format is stated only in prose, not as a facet, so the fields are unvalidated (a duration-grammar
check is a lint, D-0031). Invariant-free pub-field carrier, derived `Deserialize`.

Decisions: D-0042, D-0037, D-0031.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ReleaseCalendar {
    /// The period between releases of the data set.
    pub periodicity: String,
    /// The interval between the start of the year and the first release in that year.
    pub offset: String,
    /// The period after which a release may be considered late.
    pub tolerance: String,
}

/// A constraint on the data an artefact may carry: the allowed or actual key sets and cube regions.
///
/// ## Specification
/// - **Type**: `DataConstraintType`
/// - **Element**: `<DataConstraint>`
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataConstraintType.md"))]
///
/// A maintainable artefact that narrows the data attached to one or more structural artefacts. It may
/// be expressed through data key sets, cube regions (at most two), both, or neither. In SDMX 3.0 it
/// carries a required `role` and may carry a release calendar; SDMX 3.1 has neither, so both are
/// optional here. The attachment is optional: an unattached constraint takes its attachment from
/// context.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Named after `DataConstraintType` (identical in 3.0 and 3.1; the earlier invented `ReportingConstraint`
was renamed once `role: Some(Actual)` became representable, D-0037). The artefact trait hierarchy
delegates to the `metadata` leaf, as on the other maintainables.

The `role` divergence lives in the spec base `ConstraintType`, not the leaf fragment: 3.0 wire makes
`role` required (`Allowed | Actual`), 3.1 has no such attribute, so `None` ⟺ 3.1 and `Some` ⟺ 3.0,
three states stored verbatim (D-0037/D-0031). `new()` takes no part: per-version requiredness is
Phase-2 adapter business. `attachment` is `minOccurs="0"` both editions (D-0041), so optional;
contrast `AvailabilityConstraint`, whose attachment is mandatory (the asymmetry is the spec's).
`release_calendar` is 3.0-only (D-0042). `key_sets` is `0..unbounded`, empty valid; `regions` is the
`CubeRegions` newtype (at most two, D-0036), whose custom impl enforces the bound on every path, so
`DataConstraint` derives `Deserialize` and adds no error variant. Field order mirrors the wire.

Decisions: D-0037, D-0041, D-0042, D-0036, D-0039, D-0013.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataConstraint {
    /// The constraint's maintainable metadata (id, agency, version, names, and so on).
    pub metadata: MaintainableMetadata,
    /// The constraint's role; `None` ⟺ absent (SDMX 3.1, which has no role attribute).
    pub role: Option<ConstraintRole>,
    /// What the constraint is attached to; `None` ⟺ attachment supplied by context.
    pub attachment: Option<DataConstraintAttachment>,
    /// The release calendar, if any (SDMX 3.0 only).
    pub release_calendar: Option<ReleaseCalendar>,
    /// The data key sets the constraint expresses (may be empty).
    pub key_sets: Vec<DataKeySet>,
    /// The cube regions the constraint expresses (at most two).
    pub regions: CubeRegions,
}

impl IdentifiableArtefact for DataConstraint {
    fn id(&self) -> &str {
        self.metadata.id()
    }
    fn urn(&self) -> Option<&str> {
        self.metadata.urn()
    }
    fn uri(&self) -> Option<&str> {
        self.metadata.uri()
    }
    fn annotations(&self) -> &[Annotation] {
        self.metadata.annotations()
    }
    fn links(&self) -> &[Link] {
        self.metadata.links()
    }
}

impl NameableArtefact for DataConstraint {
    fn names(&self) -> &LocalisedString {
        self.metadata.names()
    }
    fn descriptions(&self) -> Option<&LocalisedString> {
        self.metadata.descriptions()
    }
}

impl VersionableArtefact for DataConstraint {
    fn version(&self) -> Option<&SdmxVersion> {
        self.metadata.version()
    }
    fn valid_from(&self) -> Option<&SdmxDateTime> {
        self.metadata.valid_from()
    }
    fn valid_to(&self) -> Option<&SdmxDateTime> {
        self.metadata.valid_to()
    }
}

impl MaintainableArtefact for DataConstraint {
    fn agency(&self) -> &str {
        self.metadata.agency()
    }
    fn is_partial_language(&self) -> bool {
        self.metadata.is_partial_language()
    }
    fn is_external_reference(&self) -> bool {
        self.metadata.is_external_reference()
    }
    fn service_url(&self) -> Option<&str> {
        self.metadata.service_url()
    }
    fn structure_url(&self) -> Option<&str> {
        self.metadata.structure_url()
    }
}

/// A statement of the data actually available for a structural artefact.
///
/// ## Specification
/// - **Type**: `AvailabilityConstraintType`
/// - **Element**: `<AvailabilityConstraint>`
/// - **Editions**: SDMX 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/AvailabilityConstraintType.md"))]
///
/// Represents the actual data holdings returned by an availability query: what it is attached to, the
/// cube region available, and optional series and observation counts. Unlike a data constraint it is
/// not maintainable (it has no agency, version, or registry identity), and its attachment and region
/// are both mandatory.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Non-maintainable (D-0013): it carries no `MaintainableMetadata`. Non-maintainable is not
non-annotable, though: `AvailabilityConstraintType` extends `common:AnnotableType` directly, so it
carries `annotations` as a bare field (D-0033), the same empty ⟺ absent mapping as `CubeRegion`, and
in the same annotations-after-content position. Its `attachment` is `minOccurs="1"` and its `region`
single (`minOccurs="1" maxOccurs="1"`), the asymmetry with `DataConstraint` being the spec's (D-0041).
`series_count`/`obs_count` are optional `xs:int`, stored as `Option<i32>` to mirror the XSD value
space (a negative stated count is schema-valid, a coherence lint flags it, D-0043). A 3.1-only type
(3.0 has no availability constraint); the superset carries it regardless. Pub-field carrier, derived
`Deserialize`.

Decisions: D-0013, D-0033, D-0041, D-0043.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AvailabilityConstraint {
    /// What the constraint is attached to (mandatory).
    pub attachment: AvailabilityConstraintAttachment,
    /// The cube region the constraint covers (mandatory, single).
    pub region: CubeRegion,
    /// The constraint's annotations; empty ⟺ absent.
    pub annotations: Vec<Annotation>,
    /// The number of series available, if stated.
    pub series_count: Option<i32>,
    /// The number of observations available, if stated.
    pub obs_count: Option<i32>,
}

/// The unified constraint model: a data constraint or an availability constraint.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// One type spanning the two structural constraint kinds the standard defines: a [`DataConstraint`]
/// (both editions) and an [`AvailabilityConstraint`] (SDMX 3.1). A consumer matches on the kind to
/// reach the constraint it holds.
///
/// # Examples
///
/// ```
/// use sdmx_types::{
///     AvailabilityConstraint, AvailabilityConstraintAttachment, ConstraintModel, CubeRegion,
///     CubeRegions, DataConstraint, DataStructureReference, IdentifiableMetadata, LocalisedString,
///     LocalisedText, MaintainableMetadata, NameableMetadata, VersionableMetadata,
/// };
///
/// // A data constraint is a maintainable artefact, so it carries maintainable metadata.
/// let names = LocalisedString::new(vec![LocalisedText {
///     language: Some(String::from("en")),
///     text: String::from("Allowed"),
/// }])?;
/// let identifiable =
///     IdentifiableMetadata::new(String::from("CR_EXR"), None, None, Vec::new(), Vec::new())?;
/// let versionable = VersionableMetadata::new(
///     NameableMetadata::new(identifiable, names, None),
///     None,
///     None,
///     None,
/// );
/// let metadata =
///     MaintainableMetadata::new(versionable, String::from("SDMX"), None, None, None, None)?;
/// let data = ConstraintModel::Data(DataConstraint {
///     metadata,
///     role: None,
///     attachment: None,
///     release_calendar: None,
///     key_sets: Vec::new(),
///     regions: CubeRegions::new(Vec::new())?,
/// });
/// assert!(matches!(data, ConstraintModel::Data(_)));
///
/// // An availability constraint is not maintainable: it carries no metadata.
/// let availability = ConstraintModel::Availability(AvailabilityConstraint {
///     attachment: AvailabilityConstraintAttachment::DataStructure(DataStructureReference {
///         agency: String::from("SDMX"),
///         id: String::from("ECB_EXR1"),
///         version: "1.0.0".parse().unwrap(),
///     }),
///     region: CubeRegion {
///         key_values: Vec::new(),
///         components: Vec::new(),
///         include: None,
///         annotations: Vec::new(),
///     },
///     annotations: Vec::new(),
///     series_count: Some(42),
///     obs_count: None,
/// });
/// assert!(matches!(availability, ConstraintModel::Availability(_)));
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The §5 decision-#5 enum, symmetric with ADR-0008: rather than one constraint type with version
branches, the two structurally disjoint kinds are distinct types under one model. A 3.0
`role="Actual"` data constraint stays in `DataConstraint`, it never maps onto `AvailabilityConstraint`
(maintainable identity, optional wider attachment, 0..2 regions, key sets, all structurally disjoint,
D-0037). Derived `Deserialize`.

Decisions: D-0013, D-0037.
"#
)]
// The two members differ in size (a `DataConstraint` carries maintainable metadata and far more than
// an `AvailabilityConstraint`), but both are owned values of a single constraint payload; boxing the
// larger arm would add indirection the design does not model for no practical gain at this scale.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ConstraintModel {
    /// A data constraint (both editions).
    Data(DataConstraint),
    /// An availability constraint (SDMX 3.1).
    Availability(AvailabilityConstraint),
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::{string::ToString, vec};

    use super::*;
    use crate::localised::LocalisedText;

    fn period(p: &str) -> TimePeriodRange {
        TimePeriodRange { period: p.parse().unwrap(), inclusive: None }
    }

    #[test]
    fn cube_key_value_round_trips_with_cascade_and_validity() {
        let value = CubeKeyValue {
            value: String::from("A"),
            cascade: Some(Cascade::IncludeChildren),
            valid_from: Some(SdmxTimePeriod::new(String::from("2020")).unwrap()),
            valid_to: Some(SdmxTimePeriod::new(String::from("2024")).unwrap()),
        };
        crate::test_support::round_trip(&value);
    }

    #[test]
    fn simple_component_value_round_trips_with_lang() {
        let value = SimpleComponentValue {
            value: String::from("EUR"),
            cascade: None,
            lang: Some(String::from("en")),
            valid_from: None,
            valid_to: None,
        };
        crate::test_support::round_trip(&value);
    }

    #[test]
    fn cube_key_values_rejects_empty() {
        assert_eq!(CubeKeyValues::new(Vec::new()).unwrap_err(), Error::EmptyCubeKeyValues);
        let value = CubeKeyValue {
            value: String::from("A"),
            cascade: None,
            valid_from: None,
            valid_to: None,
        };
        assert_eq!(CubeKeyValues::new(vec![value]).unwrap().as_slice().len(), 1);
    }

    #[test]
    fn simple_component_values_rejects_empty() {
        assert_eq!(
            SimpleComponentValues::new(Vec::new()).unwrap_err(),
            Error::EmptySimpleComponentValues
        );
        let value = SimpleComponentValue {
            value: String::from("EUR"),
            cascade: None,
            lang: None,
            valid_from: None,
            valid_to: None,
        };
        assert_eq!(SimpleComponentValues::new(vec![value]).unwrap().as_slice().len(), 1);
    }

    #[test]
    fn cube_key_values_deserialize_rejects_empty_on_the_wire() {
        // Transparent over `Vec<CubeKeyValue>`; deserialize routes through `new()`, which rejects an
        // empty list. postcard is positional, so an empty inner vector decodes to the same bytes.
        assert!(
            postcard::from_bytes::<CubeKeyValues>(
                &postcard::to_allocvec(&Vec::<CubeKeyValue>::new()).unwrap()
            )
            .is_err()
        );
        let value = CubeKeyValue {
            value: String::from("A"),
            cascade: None,
            valid_from: None,
            valid_to: None,
        };
        let values = CubeKeyValues::new(vec![value]).unwrap();
        crate::test_support::round_trip(&values);
    }

    #[test]
    fn simple_component_values_deserialize_rejects_empty_on_the_wire() {
        // Transparent over `Vec<SimpleComponentValue>`; deserialize routes through `new()`, which
        // rejects an empty list. postcard is positional, so an empty inner vector decodes alike.
        // A valid non-empty vector decodes — guards the transparent-over-Vec shape if the newtype grows.
        assert!(
            postcard::from_bytes::<SimpleComponentValues>(
                &postcard::to_allocvec(&vec![SimpleComponentValue {
                    value: String::from("EUR"),
                    cascade: None,
                    lang: None,
                    valid_from: None,
                    valid_to: None,
                }])
                .unwrap()
            )
            .is_ok()
        );
        assert!(
            postcard::from_bytes::<SimpleComponentValues>(
                &postcard::to_allocvec(&Vec::<SimpleComponentValue>::new()).unwrap()
            )
            .is_err()
        );
    }

    #[test]
    fn time_period_range_effective_is_inclusive_applies_default() {
        assert!(period("2024").effective_is_inclusive());
        assert!(
            TimePeriodRange { period: "2024".parse().unwrap(), inclusive: Some(true) }
                .effective_is_inclusive()
        );
        assert!(
            !TimePeriodRange { period: "2024".parse().unwrap(), inclusive: Some(false) }
                .effective_is_inclusive()
        );
    }

    #[test]
    fn include_flags_effective_is_included_apply_default() {
        // Each `include`-bearing carrier defaults to `true` when the flag is absent.
        assert!(cube_key("FREQ", "A").effective_is_included());
        assert!(
            ComponentValueSet::new(
                String::from("OBS_STATUS"),
                ComponentSelection::Empty,
                None,
                None,
            )
            .unwrap()
            .effective_is_included()
        );
        assert!(
            CubeRegion {
                key_values: Vec::new(),
                components: Vec::new(),
                include: None,
                annotations: Vec::new()
            }
            .effective_is_included()
        );
        assert!(
            DataComponentValueSet::new(
                String::from("TIME_PERIOD"),
                DataComponentSelection::TimeRange(TimeRange {
                    kind: TimeRangeKind::Before(period("2024")),
                    valid_from: None,
                    valid_to: None,
                }),
                None,
                None,
            )
            .unwrap()
            .effective_is_included()
        );
        // A stated `false` is honoured (one representative carrier).
        assert!(
            !CubeRegion {
                key_values: Vec::new(),
                components: Vec::new(),
                include: Some(false),
                annotations: Vec::new(),
            }
            .effective_is_included()
        );
    }

    #[test]
    fn time_range_before_round_trips() {
        let range = TimeRange {
            kind: TimeRangeKind::Before(period("2024")),
            valid_from: None,
            valid_to: None,
        };
        crate::test_support::round_trip(&range);
    }

    #[test]
    fn time_range_after_round_trips() {
        let range = TimeRange {
            kind: TimeRangeKind::After(period("2020")),
            valid_from: None,
            valid_to: None,
        };
        crate::test_support::round_trip(&range);
    }

    #[test]
    fn time_range_between_round_trips_with_validity() {
        let range = TimeRange {
            kind: TimeRangeKind::Between { start: period("2020"), end: period("2024") },
            valid_from: Some(SdmxTimePeriod::new(String::from("2019")).unwrap()),
            valid_to: Some(SdmxTimePeriod::new(String::from("2025")).unwrap()),
        };
        crate::test_support::round_trip(&range);
    }

    #[test]
    fn time_range_validity_rejects_bad_period_on_the_wire() {
        // Bubbling demonstration, not a composite-own proof: the period-validity invariant is
        // `SdmxTimePeriod`'s (proven in lexical.rs). `TimeRange` derives Deserialize over
        // `{ kind, valid_from, valid_to }`, its `valid_from` being the self-validating
        // `SdmxTimePeriod` (which serialises transparently as its inner `String`). postcard is
        // positional, so a tuple of the field types carrying an invalid period string in
        // `valid_from` decodes at the field level but is rejected by `SdmxTimePeriod`'s deserialize,
        // which `TimeRange`'s derive propagates.
        // A valid tuple of the same field types decodes — guards this proof's shape against field-order drift.
        let ok =
            (TimeRangeKind::Before(period("2024")), Some(String::from("2024")), None::<String>);
        assert!(postcard::from_bytes::<TimeRange>(&postcard::to_allocvec(&ok).unwrap()).is_ok());
        let raw = (
            TimeRangeKind::Before(period("2024")),
            Some(String::from("not-a-period")),
            None::<String>,
        );
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<TimeRange>(&bytes).is_err());
    }

    fn cube_key(id: &str, value: &str) -> CubeRegionKey {
        let v = CubeKeyValue {
            value: value.to_string(),
            cascade: None,
            valid_from: None,
            valid_to: None,
        };
        CubeRegionKey::new(
            id.to_string(),
            KeyValueSelection::Values(CubeKeyValues::new(vec![v]).unwrap()),
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn cube_region_key_round_trips_with_time_range_selection() {
        let key = CubeRegionKey::new(
            String::from("TIME_PERIOD"),
            KeyValueSelection::TimeRange(TimeRange {
                kind: TimeRangeKind::After(period("2020")),
                valid_from: None,
                valid_to: None,
            }),
            Some(false),
            Some(true),
            Some(SdmxTimePeriod::new(String::from("2019")).unwrap()),
            None,
        )
        .unwrap();
        crate::test_support::round_trip(&key);
    }

    #[test]
    fn cube_region_key_validates_the_single_ncname_grammar() {
        let key = cube_key("FREQ", "A");
        assert_eq!(key.id(), "FREQ");
        // SingleNCNameIDType is the NCName pattern: a leading digit, @, $, an embedded dot, and the
        // empty string are all rejected.
        for bad in ["", "1BAD", "@X", "EUR$", "A.B"] {
            assert_eq!(
                CubeRegionKey::new(
                    String::from(bad),
                    KeyValueSelection::TimeRange(TimeRange {
                        kind: TimeRangeKind::After(period("2020")),
                        valid_from: None,
                        valid_to: None,
                    }),
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap_err(),
                Error::InvalidNcNameIdentifier(String::from(bad))
            );
        }
    }

    #[test]
    fn cube_region_key_deserialize_enforces_the_single_ncname_grammar() {
        // CubeRegionKey's Deserialize declares a positional `Raw` and routes through new(). postcard
        // is positional, so a tuple of the field types carrying an off-grammar id proves the wire
        // path re-runs the id check, while a valid value round-trips.
        let raw = (
            String::from("1BAD"),
            KeyValueSelection::TimeRange(TimeRange {
                kind: TimeRangeKind::After(period("2020")),
                valid_from: None,
                valid_to: None,
            }),
            None::<bool>,
            None::<bool>,
            None::<SdmxTimePeriod>,
            None::<SdmxTimePeriod>,
        );
        assert!(
            postcard::from_bytes::<CubeRegionKey>(&postcard::to_allocvec(&raw).unwrap()).is_err()
        );
        crate::test_support::round_trip(&cube_key("FREQ", "A"));
    }

    #[test]
    fn cube_region_key_accessors_expose_stated_fields() {
        let selection = KeyValueSelection::TimeRange(TimeRange {
            kind: TimeRangeKind::After(period("2020")),
            valid_from: None,
            valid_to: None,
        });
        let from = SdmxTimePeriod::new(String::from("2019")).unwrap();
        let to = SdmxTimePeriod::new(String::from("2024")).unwrap();
        let key = CubeRegionKey::new(
            String::from("FREQ"),
            selection.clone(),
            Some(false),
            Some(true),
            Some(from.clone()),
            Some(to.clone()),
        )
        .unwrap();
        assert_eq!(key.id(), "FREQ");
        assert_eq!(key.selection(), &selection);
        assert_eq!(key.include(), Some(false));
        assert_eq!(key.remove_prefix(), Some(true));
        assert_eq!(key.valid_from(), Some(&from));
        assert_eq!(key.valid_to(), Some(&to));
        assert!(!key.effective_is_included());
    }

    #[test]
    fn component_selection_empty_is_distinct_from_values() {
        let value = SimpleComponentValue {
            value: String::from("EUR"),
            cascade: None,
            lang: None,
            valid_from: None,
            valid_to: None,
        };
        let values = ComponentSelection::Values(SimpleComponentValues::new(vec![value]).unwrap());
        let empty = ComponentSelection::Empty;
        assert_ne!(values, empty);
        // Each round-trips to itself, so Empty is not collapsed into an empty value list.
        crate::test_support::round_trip(&empty);
        crate::test_support::round_trip(&values);
    }

    #[test]
    fn component_value_set_round_trips() {
        let set = ComponentValueSet::new(
            String::from("CONTACT.ADDRESS.STREET"),
            ComponentSelection::Empty,
            Some(true),
            None,
        )
        .unwrap();
        crate::test_support::round_trip(&set);
    }

    #[test]
    fn component_value_set_validates_the_nested_ncname_grammar() {
        // NestedNCNameIDType accepts a single NCName and a dotted path of NCName segments.
        for ok in ["STREET", "CONTACT.ADDRESS.STREET"] {
            assert_eq!(
                ComponentValueSet::new(String::from(ok), ComponentSelection::Empty, None, None)
                    .unwrap()
                    .id(),
                ok
            );
        }
        // A leading/trailing/doubled dot, a leading digit, and the empty string are all rejected.
        for bad in ["", "1BAD", ".A", "A.", "A..B", "A.1B", "@X"] {
            assert_eq!(
                ComponentValueSet::new(String::from(bad), ComponentSelection::Empty, None, None)
                    .unwrap_err(),
                Error::InvalidNestedNcNameIdentifier(String::from(bad))
            );
        }
    }

    #[test]
    fn component_value_set_deserialize_enforces_the_nested_ncname_grammar() {
        // ComponentValueSet's Deserialize declares a positional `Raw` and routes through new().
        // postcard is positional, so a tuple of the field types carrying an off-grammar id proves
        // the wire path re-runs the id check, while a valid dotted-path value round-trips.
        let raw = (String::from("A..B"), ComponentSelection::Empty, None::<bool>, None::<bool>);
        assert!(
            postcard::from_bytes::<ComponentValueSet>(&postcard::to_allocvec(&raw).unwrap())
                .is_err()
        );
        crate::test_support::round_trip(
            &ComponentValueSet::new(
                String::from("CONTACT.ADDRESS.STREET"),
                ComponentSelection::Empty,
                None,
                None,
            )
            .unwrap(),
        );
    }

    #[test]
    fn component_value_set_accessors_expose_stated_fields() {
        let selection = ComponentSelection::Empty;
        let set = ComponentValueSet::new(
            String::from("CONTACT.ADDRESS.STREET"),
            selection.clone(),
            Some(false),
            Some(true),
        )
        .unwrap();
        assert_eq!(set.id(), "CONTACT.ADDRESS.STREET");
        assert_eq!(set.selection(), &selection);
        assert_eq!(set.include(), Some(false));
        assert_eq!(set.remove_prefix(), Some(true));
        assert!(!set.effective_is_included());
    }

    #[test]
    fn cube_region_round_trips_preserving_selection_order() {
        let region = CubeRegion {
            key_values: vec![cube_key("FREQ", "A"), cube_key("REF_AREA", "EU")],
            components: vec![
                ComponentValueSet::new(
                    String::from("OBS_STATUS"),
                    ComponentSelection::Empty,
                    None,
                    None,
                )
                .unwrap(),
            ],
            include: Some(true),
            annotations: Vec::new(),
        };
        let bytes = postcard::to_allocvec(&region).unwrap();
        let restored: CubeRegion = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(restored, region);
        // Wire order of the dimension selections is preserved.
        assert_eq!(restored.key_values[0].id(), "FREQ");
        assert_eq!(restored.key_values[1].id(), "REF_AREA");
    }

    #[test]
    fn cube_region_annotations_empty_maps_absent() {
        let region = CubeRegion {
            key_values: Vec::new(),
            components: Vec::new(),
            include: None,
            annotations: Vec::new(),
        };
        assert!(region.annotations.is_empty());
        crate::test_support::round_trip(&region);
    }

    #[test]
    fn cube_regions_rejects_more_than_two() {
        let region = || CubeRegion {
            key_values: Vec::new(),
            components: Vec::new(),
            include: None,
            annotations: Vec::new(),
        };
        assert!(CubeRegions::new(Vec::new()).unwrap().as_slice().is_empty());
        assert_eq!(CubeRegions::new(vec![region(), region()]).unwrap().as_slice().len(), 2);
        assert_eq!(
            CubeRegions::new(vec![region(), region(), region()]).unwrap_err(),
            Error::TooManyCubeRegions
        );
    }

    #[test]
    fn cube_regions_deserialize_rejects_more_than_two_on_the_wire() {
        let region = || CubeRegion {
            key_values: Vec::new(),
            components: Vec::new(),
            include: None,
            annotations: Vec::new(),
        };
        // `CubeRegions` is transparent over `Vec<CubeRegion>`; deserialize routes through `new()`,
        // which caps the count at two. postcard is positional, so a three-element inner vector
        // decodes to the same bytes and is rejected only by the `> 2` bound.
        let three = vec![region(), region(), region()];
        let bytes = postcard::to_allocvec(&three).unwrap();
        assert!(postcard::from_bytes::<CubeRegions>(&bytes).is_err());
    }

    fn data_key_value(id: &str, value: &str) -> DataKeyValue {
        DataKeyValue::new(
            id.to_string(),
            SimpleKeyValues::new(vec![value.to_string()]).unwrap(),
            FixedInclude::new(None).unwrap(),
            None,
        )
        .unwrap()
    }

    #[test]
    fn data_component_values_rejects_empty() {
        assert_eq!(
            DataComponentValues::new(Vec::new()).unwrap_err(),
            Error::EmptyDataComponentValues
        );
        let value = DataComponentValue { value: String::from("EUR"), cascade: None, lang: None };
        assert_eq!(DataComponentValues::new(vec![value]).unwrap().as_slice().len(), 1);
    }

    #[test]
    fn simple_key_values_rejects_empty() {
        assert_eq!(SimpleKeyValues::new(Vec::new()).unwrap_err(), Error::EmptySimpleKeyValues);
        let ok = SimpleKeyValues::new(vec![String::from("A")]).unwrap();
        assert_eq!(ok.as_slice().len(), 1);
        // The wire path rejects an empty list too: transparent over `Vec<String>`, routed through
        // `new()`, so an empty inner vector's positional bytes are rejected.
        // A valid non-empty vector decodes — guards the transparent-over-Vec shape if the newtype grows.
        assert!(
            postcard::from_bytes::<SimpleKeyValues>(
                &postcard::to_allocvec(&vec![String::from("A")]).unwrap()
            )
            .is_ok()
        );
        assert!(
            postcard::from_bytes::<SimpleKeyValues>(
                &postcard::to_allocvec(&Vec::<String>::new()).unwrap()
            )
            .is_err()
        );
    }

    #[test]
    fn data_keys_rejects_empty() {
        assert_eq!(DataKeys::new(Vec::new()).unwrap_err(), Error::EmptyDataKeys);
        let key = DataKey {
            key_values: vec![data_key_value("FREQ", "A")],
            components: Vec::new(),
            include: FixedInclude::new(None).unwrap(),
            annotations: Vec::new(),
            valid_from: None,
            valid_to: None,
        };
        assert_eq!(DataKeys::new(vec![key]).unwrap().as_slice().len(), 1);
    }

    #[test]
    fn data_key_value_carries_3_1_multi_value_superset() {
        // The 3.1 unbounded shape (FREQ = A or M or Q) is the carried superset; 3.0's single value
        // is the degenerate one-element case.
        let multi = DataKeyValue::new(
            String::from("FREQ"),
            SimpleKeyValues::new(vec![String::from("A"), String::from("M"), String::from("Q")])
                .unwrap(),
            FixedInclude::new(Some(true)).unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(multi.values().as_slice().len(), 3);
        crate::test_support::round_trip(&multi);
    }

    #[test]
    fn data_key_value_validates_the_single_ncname_grammar() {
        let key = data_key_value("FREQ", "A");
        assert_eq!(key.id(), "FREQ");
        // SingleNCNameIDType is the NCName pattern: a leading digit, @, $, an embedded dot, and the
        // empty string are all rejected.
        for bad in ["", "1BAD", "@X", "EUR$", "A.B"] {
            assert_eq!(
                DataKeyValue::new(
                    String::from(bad),
                    SimpleKeyValues::new(vec![String::from("A")]).unwrap(),
                    FixedInclude::new(None).unwrap(),
                    None,
                )
                .unwrap_err(),
                Error::InvalidNcNameIdentifier(String::from(bad))
            );
        }
    }

    #[test]
    fn data_key_value_deserialize_enforces_the_single_ncname_grammar() {
        // DataKeyValue's Deserialize declares a positional `Raw` and routes through new(). postcard
        // is positional, so a tuple of the field types carrying an off-grammar id proves the wire
        // path re-runs the id check, while a valid value round-trips.
        let raw = (String::from("1BAD"), vec![String::from("A")], None::<bool>, None::<bool>);
        assert!(
            postcard::from_bytes::<DataKeyValue>(&postcard::to_allocvec(&raw).unwrap()).is_err()
        );
        crate::test_support::round_trip(&data_key_value("FREQ", "A"));
    }

    #[test]
    fn data_key_value_accessors_expose_stated_fields() {
        let values = SimpleKeyValues::new(vec![String::from("A")]).unwrap();
        let include = FixedInclude::new(Some(true)).unwrap();
        let key =
            DataKeyValue::new(String::from("FREQ"), values.clone(), include, Some(true)).unwrap();
        assert_eq!(key.id(), "FREQ");
        assert_eq!(key.values(), &values);
        assert_eq!(key.include(), include);
        assert_eq!(key.remove_prefix(), Some(true));
    }

    #[test]
    fn data_key_value_include_rejects_stated_false_on_the_wire() {
        // Bubbling demonstration, not a composite-own proof: the fixed-include invariant is
        // `FixedInclude`'s (proven in fixed.rs). `DataKeyValue`'s Deserialize routes through new()
        // over a positional `Raw { id, values, include, remove_prefix }`, its `include` being the
        // `FixedInclude` wrapper (transparent over `Option<bool>`) that rejects a stated `false`, and
        // `values` the non-empty `SimpleKeyValues` (transparent over `Vec<String>`). postcard is
        // positional, so a tuple of those field types with a valid id, a non-empty value list (so
        // `SimpleKeyValues` accepts) and `include = Some(false)` decodes at the field level but is
        // rejected only by `FixedInclude`, which the custom Deserialize propagates.
        // A valid tuple of the same field types decodes — guards this proof's shape against field-order drift.
        let ok = (String::from("FREQ"), vec![String::from("A")], None::<bool>, None::<bool>);
        assert!(postcard::from_bytes::<DataKeyValue>(&postcard::to_allocvec(&ok).unwrap()).is_ok());
        let raw = (String::from("FREQ"), vec![String::from("A")], Some(false), None::<bool>);
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<DataKeyValue>(&bytes).is_err());
    }

    #[test]
    fn data_key_round_trips_with_validity_and_annotations() {
        let key = DataKey {
            key_values: vec![data_key_value("FREQ", "A")],
            components: vec![
                DataComponentValueSet::new(
                    String::from("OBS_STATUS"),
                    DataComponentSelection::Empty,
                    None,
                    None,
                )
                .unwrap(),
            ],
            include: FixedInclude::new(Some(true)).unwrap(),
            annotations: Vec::new(),
            valid_from: Some(SdmxTimePeriod::new(String::from("2020")).unwrap()),
            valid_to: None,
        };
        crate::test_support::round_trip(&key);
    }

    #[test]
    fn data_component_selection_empty_is_distinct_from_values() {
        let values = DataComponentSelection::Values(
            DataComponentValues::new(vec![DataComponentValue {
                value: String::from("EUR"),
                cascade: None,
                lang: None,
            }])
            .unwrap(),
        );
        assert_ne!(values, DataComponentSelection::Empty);
    }

    #[test]
    fn data_key_set_round_trips_and_requires_is_included() {
        let set = DataKeySet {
            keys: DataKeys::new(vec![DataKey {
                key_values: vec![data_key_value("FREQ", "A")],
                components: Vec::new(),
                include: FixedInclude::new(None).unwrap(),
                annotations: Vec::new(),
                valid_from: None,
                valid_to: None,
            }])
            .unwrap(),
            is_included: true,
        };
        crate::test_support::round_trip(&set);
    }

    #[test]
    fn data_component_selection_time_range_arm_round_trips() {
        let set = DataComponentValueSet::new(
            String::from("TIME_PERIOD"),
            DataComponentSelection::TimeRange(TimeRange {
                kind: TimeRangeKind::Before(period("2024")),
                valid_from: None,
                valid_to: None,
            }),
            None,
            None,
        )
        .unwrap();
        crate::test_support::round_trip(&set);
    }

    #[test]
    fn component_selection_time_range_arm_round_trips() {
        let set = ComponentValueSet::new(
            String::from("TIME_PERIOD"),
            ComponentSelection::TimeRange(TimeRange {
                kind: TimeRangeKind::Before(period("2024")),
                valid_from: None,
                valid_to: None,
            }),
            None,
            None,
        )
        .unwrap();
        crate::test_support::round_trip(&set);
    }

    #[test]
    fn data_component_value_set_validates_the_nested_ncname_grammar() {
        // NestedNCNameIDType accepts a single NCName and a dotted path of NCName segments.
        for ok in ["STREET", "CONTACT.ADDRESS.STREET"] {
            assert_eq!(
                DataComponentValueSet::new(
                    String::from(ok),
                    DataComponentSelection::Empty,
                    None,
                    None,
                )
                .unwrap()
                .id(),
                ok
            );
        }
        // A leading/trailing/doubled dot, a leading digit, and the empty string are all rejected.
        for bad in ["", "1BAD", ".A", "A.", "A..B", "A.1B", "@X"] {
            assert_eq!(
                DataComponentValueSet::new(
                    String::from(bad),
                    DataComponentSelection::Empty,
                    None,
                    None,
                )
                .unwrap_err(),
                Error::InvalidNestedNcNameIdentifier(String::from(bad))
            );
        }
    }

    #[test]
    fn data_component_value_set_deserialize_enforces_the_nested_ncname_grammar() {
        // DataComponentValueSet's Deserialize declares a positional `Raw` and routes through new().
        // postcard is positional, so a tuple of the field types carrying an off-grammar id proves
        // the wire path re-runs the id check, while a valid dotted-path value round-trips.
        let raw = (String::from("A..B"), DataComponentSelection::Empty, None::<bool>, None::<bool>);
        assert!(
            postcard::from_bytes::<DataComponentValueSet>(&postcard::to_allocvec(&raw).unwrap())
                .is_err()
        );
        crate::test_support::round_trip(
            &DataComponentValueSet::new(
                String::from("CONTACT.ADDRESS.STREET"),
                DataComponentSelection::Empty,
                None,
                None,
            )
            .unwrap(),
        );
    }

    #[test]
    fn data_component_value_set_accessors_expose_stated_fields() {
        let selection = DataComponentSelection::Empty;
        let set = DataComponentValueSet::new(
            String::from("CONTACT.ADDRESS.STREET"),
            selection.clone(),
            Some(false),
            Some(true),
        )
        .unwrap();
        assert_eq!(set.id(), "CONTACT.ADDRESS.STREET");
        assert_eq!(set.selection(), &selection);
        assert_eq!(set.include(), Some(false));
        assert_eq!(set.remove_prefix(), Some(true));
        assert!(!set.effective_is_included());
    }

    #[test]
    fn data_component_value_set_values_arm_round_trips_and_rejects_empty() {
        let set = DataComponentValueSet::new(
            String::from("CURRENCY"),
            DataComponentSelection::Values(
                DataComponentValues::new(vec![DataComponentValue {
                    value: String::from("EUR"),
                    cascade: Some(Cascade::IncludeChildren),
                    lang: Some(String::from("en")),
                }])
                .unwrap(),
            ),
            None,
            None,
        )
        .unwrap();
        crate::test_support::round_trip(&set);
        // The Values deserialize path routes through new(), so an empty list is rejected on the
        // wire, not synthesised as Values([]). Transparent over `Vec<DataComponentValue>`, so an
        // empty inner vector's positional bytes are rejected.
        // A valid non-empty vector decodes — guards the transparent-over-Vec shape if the newtype grows.
        assert!(
            postcard::from_bytes::<DataComponentValues>(
                &postcard::to_allocvec(&vec![DataComponentValue {
                    value: String::from("EUR"),
                    cascade: None,
                    lang: None,
                }])
                .unwrap()
            )
            .is_ok()
        );
        assert!(
            postcard::from_bytes::<DataComponentValues>(
                &postcard::to_allocvec(&Vec::<DataComponentValue>::new()).unwrap()
            )
            .is_err()
        );
    }

    #[test]
    fn queryable_data_source_round_trips_with_optional_urls() {
        let source = QueryableDataSource {
            data_url: String::from("https://example.com/sdmx"),
            wsdl_url: Some(String::from("https://example.com/sdmx?wsdl")),
            wadl_url: None,
            is_rest_data_source: true,
            is_web_service_data_source: true,
        };
        crate::test_support::round_trip(&source);
    }

    fn dsd_ref(id: &str) -> DataStructureReference {
        DataStructureReference {
            agency: String::from("SDMX"),
            id: id.to_string(),
            version: "1.0.0".parse().unwrap(),
        }
    }

    fn dataflow_ref(id: &str) -> DataflowReference {
        DataflowReference {
            agency: String::from("ECB"),
            id: id.to_string(),
            version: "1.0.0".parse().unwrap(),
        }
    }

    fn agreement_ref(id: &str) -> ProvisionAgreementReference {
        ProvisionAgreementReference {
            agency: String::from("ECB"),
            id: id.to_string(),
            version: "1.0.0".parse().unwrap(),
        }
    }

    #[test]
    fn attachment_ref_newtypes_reject_empty_and_expose_their_slice() {
        assert_eq!(DataStructureRefs::new(Vec::new()).unwrap_err(), Error::EmptyDataStructureRefs);
        assert_eq!(DataflowRefs::new(Vec::new()).unwrap_err(), Error::EmptyDataflowRefs);
        assert_eq!(
            ProvisionAgreementRefs::new(Vec::new()).unwrap_err(),
            Error::EmptyProvisionAgreementRefs
        );
        assert_eq!(SimpleDataSources::new(Vec::new()).unwrap_err(), Error::EmptySimpleDataSources);

        assert_eq!(DataStructureRefs::new(vec![dsd_ref("ECB_EXR1")]).unwrap().as_slice().len(), 1);
        assert_eq!(DataflowRefs::new(vec![dataflow_ref("EXR")]).unwrap().as_slice().len(), 1);
        assert_eq!(
            ProvisionAgreementRefs::new(vec![agreement_ref("PA_EXR")]).unwrap().as_slice().len(),
            1
        );
        assert_eq!(
            SimpleDataSources::new(vec![String::from("https://example.com/data")])
                .unwrap()
                .as_slice()
                .len(),
            1
        );
    }

    #[test]
    fn attachment_ref_newtypes_reject_empty_on_the_wire() {
        // Each is transparent over a `Vec<Elem>` and routes deserialize through `new()`, which
        // rejects an empty list. postcard is positional, so an empty inner vector decodes alike.
        // A valid non-empty vector decodes — guards the transparent-over-Vec shape if the newtype grows.
        assert!(
            postcard::from_bytes::<DataStructureRefs>(
                &postcard::to_allocvec(&vec![dsd_ref("ECB_EXR1")]).unwrap()
            )
            .is_ok()
        );
        assert!(
            postcard::from_bytes::<DataStructureRefs>(
                &postcard::to_allocvec(&Vec::<DataStructureReference>::new()).unwrap()
            )
            .is_err()
        );
        // A valid non-empty vector decodes — guards the transparent-over-Vec shape if the newtype grows.
        assert!(
            postcard::from_bytes::<DataflowRefs>(
                &postcard::to_allocvec(&vec![dataflow_ref("EXR")]).unwrap()
            )
            .is_ok()
        );
        assert!(
            postcard::from_bytes::<DataflowRefs>(
                &postcard::to_allocvec(&Vec::<DataflowReference>::new()).unwrap()
            )
            .is_err()
        );
        // A valid non-empty vector decodes — guards the transparent-over-Vec shape if the newtype grows.
        assert!(
            postcard::from_bytes::<ProvisionAgreementRefs>(
                &postcard::to_allocvec(&vec![agreement_ref("PA_EXR")]).unwrap()
            )
            .is_ok()
        );
        assert!(
            postcard::from_bytes::<ProvisionAgreementRefs>(
                &postcard::to_allocvec(&Vec::<ProvisionAgreementReference>::new()).unwrap()
            )
            .is_err()
        );
        // A valid non-empty vector decodes — guards the transparent-over-Vec shape if the newtype grows.
        assert!(
            postcard::from_bytes::<SimpleDataSources>(
                &postcard::to_allocvec(&vec![String::from("https://example.org/data")]).unwrap()
            )
            .is_ok()
        );
        assert!(
            postcard::from_bytes::<SimpleDataSources>(
                &postcard::to_allocvec(&Vec::<String>::new()).unwrap()
            )
            .is_err()
        );
    }

    #[test]
    fn data_constraint_attachment_structural_arms_round_trip_with_queryable() {
        let queryable = vec![QueryableDataSource {
            data_url: String::from("https://example.com/sdmx"),
            wsdl_url: None,
            wadl_url: None,
            is_rest_data_source: true,
            is_web_service_data_source: false,
        }];
        let arms = [
            DataConstraintAttachment::DataStructure {
                refs: DataStructureRefs::new(vec![dsd_ref("ECB_EXR1"), dsd_ref("ECB_EXR2")])
                    .unwrap(),
                queryable: queryable.clone(),
            },
            DataConstraintAttachment::Dataflow {
                refs: DataflowRefs::new(vec![dataflow_ref("EXR")]).unwrap(),
                queryable,
            },
            DataConstraintAttachment::ProvisionAgreement {
                refs: ProvisionAgreementRefs::new(vec![agreement_ref("PA_EXR")]).unwrap(),
                // The queryable companions are empty when absent (always so on 3.1 wire).
                queryable: Vec::new(),
            },
        ];
        for attachment in arms {
            crate::test_support::round_trip(&attachment);
        }
    }

    #[test]
    fn data_constraint_attachment_3_0_only_arms_round_trip() {
        // The DataProvider single arm and the 3.0-only SimpleDataSource arm.
        let provider = DataConstraintAttachment::DataProvider(DataProviderReference {
            agency: String::from("SDMX"),
            scheme_id: String::from("DATA_PROVIDERS"),
            version: "1.0.0".parse().unwrap(),
            id: String::from("ECB"),
        });
        let sources = DataConstraintAttachment::SimpleDataSource(
            SimpleDataSources::new(vec![String::from("https://example.com/data")]).unwrap(),
        );
        for attachment in [provider, sources] {
            crate::test_support::round_trip(&attachment);
        }
    }

    #[test]
    fn availability_constraint_attachment_arms_round_trip() {
        let arms = [
            AvailabilityConstraintAttachment::DataStructure(dsd_ref("ECB_EXR1")),
            AvailabilityConstraintAttachment::Dataflow(dataflow_ref("EXR")),
            AvailabilityConstraintAttachment::ProvisionAgreement(agreement_ref("PA_EXR")),
        ];
        for attachment in arms {
            crate::test_support::round_trip(&attachment);
        }
    }

    fn constraint_metadata(id: &str) -> MaintainableMetadata {
        use crate::metadata::{IdentifiableMetadata, NameableMetadata, VersionableMetadata};
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some(String::from("en")),
            text: String::from("A constraint"),
        }])
        .unwrap();
        let identifiable =
            IdentifiableMetadata::new(id.to_string(), None, None, Vec::new(), Vec::new()).unwrap();
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable, names, None),
            None,
            None,
            None,
        );
        MaintainableMetadata::new(versionable, String::from("SDMX"), None, None, None, None)
            .unwrap()
    }

    fn data_key_set() -> DataKeySet {
        DataKeySet {
            keys: DataKeys::new(vec![DataKey {
                key_values: vec![data_key_value("FREQ", "A")],
                components: Vec::new(),
                include: FixedInclude::new(None).unwrap(),
                annotations: Vec::new(),
                valid_from: None,
                valid_to: None,
            }])
            .unwrap(),
            is_included: true,
        }
    }

    fn cube_region() -> CubeRegion {
        CubeRegion {
            key_values: vec![cube_key("FREQ", "A")],
            components: Vec::new(),
            include: None,
            annotations: Vec::new(),
        }
    }

    #[test]
    fn data_constraint_forwards_every_artefact_accessor() {
        use crate::metadata::{IdentifiableMetadata, NameableMetadata, VersionableMetadata};
        let version = SdmxVersion::new(String::from("1.2.3")).unwrap();
        let valid_from = SdmxDateTime::new(String::from("2024-01-01T00:00:00+00:00")).unwrap();
        let annotation = Annotation {
            id: Some(String::from("a1")),
            annotation_type: None,
            annotation_title: None,
            annotation_urls: Vec::new(),
            annotation_value: None,
            texts: None,
        };
        let link = Link {
            rel: String::from("self"),
            url: String::from("https://example.com/x"),
            urn: None,
            link_type: None,
        };
        let names = LocalisedString::new(vec![LocalisedText {
            language: Some(String::from("en")),
            text: String::from("Constraint"),
        }])
        .unwrap();
        let descriptions = LocalisedString::new(vec![LocalisedText {
            language: Some(String::from("en")),
            text: String::from("How much"),
        }])
        .unwrap();
        let identifiable = IdentifiableMetadata::new(
            String::from("CR_EXR"),
            Some(String::from("uri")),
            Some(String::from("urn:x")),
            vec![annotation],
            vec![link],
        )
        .unwrap();
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable, names, Some(descriptions)),
            Some(version),
            Some(valid_from.clone()),
            None,
        );
        let metadata = MaintainableMetadata::new(
            versionable,
            String::from("ESTAT"),
            Some(true),
            Some(true),
            Some(String::from("https://service")),
            Some(String::from("https://structure")),
        )
        .unwrap();
        let constraint = DataConstraint {
            metadata,
            role: None,
            attachment: None,
            release_calendar: None,
            key_sets: Vec::new(),
            regions: CubeRegions::new(Vec::new()).unwrap(),
        };

        // Every forwarded accessor resolves through the metadata leaf.
        assert_eq!(constraint.id(), "CR_EXR");
        assert_eq!(constraint.urn(), Some("urn:x"));
        assert_eq!(constraint.uri(), Some("uri"));
        assert_eq!(constraint.annotations().len(), 1);
        assert_eq!(constraint.links().len(), 1);
        assert_eq!(constraint.names().first(), "Constraint");
        assert_eq!(constraint.descriptions().map(LocalisedString::first), Some("How much"));
        assert_eq!(
            constraint.version().map(alloc::string::ToString::to_string).as_deref(),
            Some("1.2.3")
        );
        assert_eq!(constraint.valid_from(), Some(&valid_from));
        assert_eq!(constraint.valid_to(), None);
        assert_eq!(constraint.agency(), "ESTAT");
        assert!(constraint.is_partial_language());
        assert!(constraint.is_external_reference());
        assert_eq!(constraint.service_url(), Some("https://service"));
        assert_eq!(constraint.structure_url(), Some("https://structure"));
    }

    #[test]
    fn data_constraint_round_trips_with_every_field_populated() {
        let constraint = DataConstraint {
            metadata: constraint_metadata("CR_EXR"),
            role: Some(ConstraintRole::Actual),
            attachment: Some(DataConstraintAttachment::DataProvider(DataProviderReference {
                agency: String::from("SDMX"),
                scheme_id: String::from("DATA_PROVIDERS"),
                version: "1.0.0".parse().unwrap(),
                id: String::from("ECB"),
            })),
            release_calendar: Some(ReleaseCalendar {
                periodicity: String::from("P1M"),
                offset: String::from("P0D"),
                tolerance: String::from("P7D"),
            }),
            key_sets: vec![data_key_set()],
            regions: CubeRegions::new(vec![cube_region()]).unwrap(),
        };
        crate::test_support::round_trip(&constraint);
    }

    #[test]
    fn data_constraint_is_constructible_key_set_only_region_only_both_and_neither() {
        let with = |key_sets: Vec<DataKeySet>, regions: Vec<CubeRegion>| DataConstraint {
            metadata: constraint_metadata("CR_EXR"),
            role: None,
            attachment: None,
            release_calendar: None,
            key_sets,
            regions: CubeRegions::new(regions).unwrap(),
        };
        let key_set_only = with(vec![data_key_set()], Vec::new());
        let region_only = with(Vec::new(), vec![cube_region()]);
        let both = with(vec![data_key_set()], vec![cube_region(), cube_region()]);
        let neither = with(Vec::new(), Vec::new());
        for constraint in [key_set_only, region_only, both, neither] {
            crate::test_support::round_trip(&constraint);
        }
    }

    #[test]
    fn data_constraint_deserialize_rejects_more_than_two_regions_on_the_wire() {
        // Bubbling demonstration, not a composite-own proof: the at-most-two-regions invariant is
        // `CubeRegions`'s (proven above). `DataConstraint` derives Deserialize over
        // `{ metadata, role, attachment, release_calendar, key_sets, regions }`, its `regions` being
        // the `CubeRegions` newtype (transparent over `Vec<CubeRegion>`) that caps the count at two.
        // postcard is positional, so a tuple of those field types whose final `regions` position
        // carries three regions decodes at the field level but is rejected by the `CubeRegions`
        // bound, which `DataConstraint`'s derived Deserialize propagates.
        // A valid tuple of the same field types decodes — guards this proof's shape against field-order drift.
        let ok = (
            constraint_metadata("CR_EXR"),
            None::<ConstraintRole>,
            None::<DataConstraintAttachment>,
            None::<ReleaseCalendar>,
            Vec::<DataKeySet>::new(),
            vec![cube_region(), cube_region()],
        );
        assert!(
            postcard::from_bytes::<DataConstraint>(&postcard::to_allocvec(&ok).unwrap()).is_ok()
        );
        let raw = (
            constraint_metadata("CR_EXR"),
            None::<ConstraintRole>,
            None::<DataConstraintAttachment>,
            None::<ReleaseCalendar>,
            Vec::<DataKeySet>::new(),
            vec![cube_region(), cube_region(), cube_region()],
        );
        let bytes = postcard::to_allocvec(&raw).unwrap();
        assert!(postcard::from_bytes::<DataConstraint>(&bytes).is_err());
    }

    #[test]
    fn availability_constraint_round_trips() {
        let constraint = AvailabilityConstraint {
            attachment: AvailabilityConstraintAttachment::Dataflow(dataflow_ref("EXR")),
            region: cube_region(),
            annotations: Vec::new(),
            series_count: Some(42),
            obs_count: Some(-1),
        };
        crate::test_support::round_trip(&constraint);
    }

    #[test]
    fn constraint_model_arms_round_trip() {
        let data = ConstraintModel::Data(DataConstraint {
            metadata: constraint_metadata("CR_EXR"),
            role: Some(ConstraintRole::Allowed),
            attachment: None,
            release_calendar: None,
            key_sets: Vec::new(),
            regions: CubeRegions::new(Vec::new()).unwrap(),
        });
        let availability = ConstraintModel::Availability(AvailabilityConstraint {
            attachment: AvailabilityConstraintAttachment::Dataflow(dataflow_ref("EXR")),
            region: cube_region(),
            annotations: Vec::new(),
            series_count: None,
            obs_count: None,
        });
        for model in [data, availability] {
            crate::test_support::round_trip(&model);
        }
    }

    #[test]
    fn cube_key_values_try_from_rejects_empty() {
        assert_eq!(CubeKeyValues::try_from(Vec::new()).unwrap_err(), Error::EmptyCubeKeyValues);
    }

    #[test]
    fn simple_component_values_try_from_rejects_empty() {
        assert_eq!(
            SimpleComponentValues::try_from(Vec::new()).unwrap_err(),
            Error::EmptySimpleComponentValues
        );
    }

    #[test]
    fn data_component_values_try_from_rejects_empty() {
        assert_eq!(
            DataComponentValues::try_from(Vec::new()).unwrap_err(),
            Error::EmptyDataComponentValues
        );
    }

    #[test]
    fn simple_key_values_try_from_rejects_empty() {
        assert_eq!(SimpleKeyValues::try_from(Vec::new()).unwrap_err(), Error::EmptySimpleKeyValues);
    }

    #[test]
    fn data_keys_try_from_rejects_empty() {
        assert_eq!(DataKeys::try_from(Vec::new()).unwrap_err(), Error::EmptyDataKeys);
    }

    #[test]
    fn data_structure_refs_try_from_rejects_empty() {
        assert_eq!(
            DataStructureRefs::try_from(Vec::new()).unwrap_err(),
            Error::EmptyDataStructureRefs
        );
    }

    #[test]
    fn dataflow_refs_try_from_rejects_empty() {
        assert_eq!(DataflowRefs::try_from(Vec::new()).unwrap_err(), Error::EmptyDataflowRefs);
    }

    #[test]
    fn provision_agreement_refs_try_from_rejects_empty() {
        assert_eq!(
            ProvisionAgreementRefs::try_from(Vec::new()).unwrap_err(),
            Error::EmptyProvisionAgreementRefs
        );
    }

    #[test]
    fn simple_data_sources_try_from_rejects_empty() {
        assert_eq!(
            SimpleDataSources::try_from(Vec::new()).unwrap_err(),
            Error::EmptySimpleDataSources
        );
    }

    #[test]
    fn cube_regions_try_from_rejects_more_than_two() {
        // CubeRegions permits empty; its boundary is the >2 cap, so it is exercised there.
        let region = || CubeRegion {
            key_values: Vec::new(),
            components: Vec::new(),
            include: None,
            annotations: Vec::new(),
        };
        assert_eq!(
            CubeRegions::try_from(vec![region(), region(), region()]).unwrap_err(),
            Error::TooManyCubeRegions
        );
    }

    #[test]
    fn collection_newtype_into_inner_and_from() {
        let ckv = vec![CubeKeyValue {
            value: String::from("A"),
            cascade: None,
            valid_from: None,
            valid_to: None,
        }];
        assert_eq!(CubeKeyValues::new(ckv.clone()).unwrap().into_inner(), ckv);
        assert_eq!(Vec::from(CubeKeyValues::new(ckv.clone()).unwrap()), ckv);

        let scv = vec![SimpleComponentValue {
            value: String::from("EUR"),
            cascade: None,
            lang: None,
            valid_from: None,
            valid_to: None,
        }];
        assert_eq!(SimpleComponentValues::new(scv.clone()).unwrap().into_inner(), scv);
        assert_eq!(Vec::from(SimpleComponentValues::new(scv.clone()).unwrap()), scv);

        let cr = vec![CubeRegion {
            key_values: Vec::new(),
            components: Vec::new(),
            include: None,
            annotations: Vec::new(),
        }];
        assert_eq!(CubeRegions::new(cr.clone()).unwrap().into_inner(), cr);
        assert_eq!(Vec::from(CubeRegions::new(cr.clone()).unwrap()), cr);

        let dcv =
            vec![DataComponentValue { value: String::from("EUR"), cascade: None, lang: None }];
        assert_eq!(DataComponentValues::new(dcv.clone()).unwrap().into_inner(), dcv);
        assert_eq!(Vec::from(DataComponentValues::new(dcv.clone()).unwrap()), dcv);

        let skv = vec![String::from("A")];
        assert_eq!(SimpleKeyValues::new(skv.clone()).unwrap().into_inner(), skv);
        assert_eq!(Vec::from(SimpleKeyValues::new(skv.clone()).unwrap()), skv);

        let dk = vec![DataKey {
            key_values: vec![data_key_value("FREQ", "A")],
            components: Vec::new(),
            include: FixedInclude::new(None).unwrap(),
            annotations: Vec::new(),
            valid_from: None,
            valid_to: None,
        }];
        assert_eq!(DataKeys::new(dk.clone()).unwrap().into_inner(), dk);
        assert_eq!(Vec::from(DataKeys::new(dk.clone()).unwrap()), dk);

        let dsr = vec![dsd_ref("DSD")];
        assert_eq!(DataStructureRefs::new(dsr.clone()).unwrap().into_inner(), dsr);
        assert_eq!(Vec::from(DataStructureRefs::new(dsr.clone()).unwrap()), dsr);

        let dfr = vec![dataflow_ref("DF")];
        assert_eq!(DataflowRefs::new(dfr.clone()).unwrap().into_inner(), dfr);
        assert_eq!(Vec::from(DataflowRefs::new(dfr.clone()).unwrap()), dfr);

        let par = vec![ProvisionAgreementReference {
            agency: String::from("A"),
            id: String::from("P"),
            version: "1.0".parse().unwrap(),
        }];
        assert_eq!(ProvisionAgreementRefs::new(par.clone()).unwrap().into_inner(), par);
        assert_eq!(Vec::from(ProvisionAgreementRefs::new(par.clone()).unwrap()), par);

        let sds = vec![String::from("http://example.com")];
        assert_eq!(SimpleDataSources::new(sds.clone()).unwrap().into_inner(), sds);
        assert_eq!(Vec::from(SimpleDataSources::new(sds.clone()).unwrap()), sds);
    }

    // Property tests: the internal serde round-trip over the generated constraint model,
    // composing every selection, key-set, and attachment family (see `test_strategy`);
    // wasm32 is excluded with the rest of the property suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::{availability_constraint, constraint_model, data_constraint};

        proptest! {
            #[test]
            fn data_constraint_round_trips(value in data_constraint()) {
                crate::test_support::round_trip(&value);
            }

            #[test]
            fn availability_constraint_round_trips(value in availability_constraint()) {
                crate::test_support::round_trip(&value);
            }

            #[test]
            fn constraint_model_round_trips(value in constraint_model()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
