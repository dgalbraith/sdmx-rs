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
`ObservationalTimePeriodType` superset and stays a raw `String` (the lexical-typing alignment is the
scheduled Phase-2 URN-contract work). All the leaves are invariant-free pub-field carriers with
derived `Deserialize` (ADR-0021); the newtypes and `SdmxTimePeriod` carry their own validating
paths.

Decisions: D-0026, D-0027, D-0031, D-0038, D-0040, D-0052, D-0064.
"#
)]

use alloc::{string::String, vec::Vec};

use chrono::{DateTime, FixedOffset};

use crate::{
    annotation::{Annotation, Link},
    artefact::{IdentifiableArtefact, MaintainableArtefact, NameableArtefact, VersionableArtefact},
    codelist::Cascade,
    error::{Error, to_de_error},
    fixed::FixedInclude,
    lexical::{SdmxTimePeriod, SdmxVersion},
    localised::LocalisedString,
    metadata::MaintainableMetadata,
    reference::{
        DataProviderReference, DataflowReference, DsdReference, ProvisionAgreementReference,
    },
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
///     CubeKeyValue { value: "A".to_string(), cascade: None, valid_from: None, valid_to: None };
/// let values = CubeKeyValues::new(vec![value])?;
/// assert_eq!(values.as_slice().len(), 1);
///
/// // An empty selection is mechanically schema-invalid.
/// assert!(CubeKeyValues::new(vec![]).is_err());
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
/// `<EndPeriod>` elements of a [`TimeRange`]. The `period` content admits time-range lexemes (a
/// start-and-duration form) as well as standard periods, so it is held verbatim as a string rather
/// than a validated time period. The `inclusive` flag defaults to `true` when absent.
///
/// # Examples
///
/// ```
/// use sdmx_types::TimePeriodRange;
///
/// let stated = TimePeriodRange { period: "2024".to_string(), inclusive: Some(false) };
/// assert!(!stated.effective_is_inclusive());
///
/// // Absent flag resolves to the schema default of true.
/// let absent = TimePeriodRange { period: "2024".to_string(), inclusive: None };
/// assert!(absent.effective_is_inclusive());
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Invariant-free pub-field carrier with derived `Deserialize`. `period` is the
`ObservationalTimePeriodType` union (`StandardTimePeriodType ∪ TimeRangeType`, report-5 V-7), a
strict superset of `SdmxTimePeriod`, so it stays a raw `String`: adopting `SdmxTimePeriod` here would
reject schema-valid time-range lexemes. The lexical-typing alignment is the scheduled Phase-2
reference-types/URN-contract work (ROADMAP scope item 4). `isInclusive` has schema `default="true"`,
so statedness is stored (D-0052) and `effective_is_inclusive()` is the resolved view.

Decisions: D-0031, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TimePeriodRange {
    /// The period content, held verbatim (it may be a standard period or a time-range lexeme).
    pub period: String,
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
/// let start = TimePeriodRange { period: "2020".to_string(), inclusive: None };
/// let end = TimePeriodRange { period: "2024".to_string(), inclusive: Some(false) };
/// let range =
///     TimeRange { kind: TimeRangeKind::Between { start, end }, valid_from: None, valid_to: None };
/// assert!(matches!(range.kind, TimeRangeKind::Between { .. }));
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
value-less `<Component>` versus a chosen `Value+` arm), so the old `Values(vec![])`-duplicates-`Empty`
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
/// Names a dimension by its `id` and the values it selects ([`KeyValueSelection`]). The `include`
/// flag (whether the named values are included in or excluded from the region) defaults to `true`
/// when absent. A dimension selection may carry its own validity window.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A node type named after its spec complexType. `id` is carried on the struct (D-0051; formerly the
map key) as a structural-only reference (`SingleNCNameIDType`, D-0020), not re-validated. The
`MemberSelectionType` attributes are inherited (D-0038): `include` (schema `default="true"`,
statedness stored, D-0052), `removePrefix` (no schema default, so `Option<bool>` distinguishes
absent from stated, D-0031), and the validity window (`StandardTimePeriodType`, so `SdmxTimePeriod`,
D-0027), which `CubeRegionKeyType` may carry. No cross-field invariant (every field self-enforcing),
so pub fields and derived `Deserialize` (ADR-0021); the non-empty values newtype and `SdmxTimePeriod`
carry their own validating paths.

Decisions: D-0020, D-0038, D-0051, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CubeRegionKey {
    /// The id of the dimension being selected (a structural reference, not re-validated).
    pub id: String,
    /// The values this dimension admits.
    pub selection: KeyValueSelection,
    /// Whether the named values are included or excluded; `None` ⟺ absent (schema default `true`).
    pub include: Option<bool>,
    /// Whether codes drop the codelist-extension prefix; `None` ⟺ absent (no schema default).
    pub remove_prefix: Option<bool>,
    /// The start of the selection's validity window, if stated.
    pub valid_from: Option<SdmxTimePeriod>,
    /// The end of the selection's validity window, if stated.
    pub valid_to: Option<SdmxTimePeriod>,
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
/// `id` may be a nested identifier. Unlike a dimension selection, a component selection carries no
/// validity window: the schema prohibits one here, so the field is simply absent.
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A node type named after its spec complexType. `id` is carried on the struct (D-0051) as a
structural-only reference (`NestedNCNameIDType`, dotted, e.g. `CONTACT.ADDRESS.STREET`; D-0020). It
carries the same `include`/`removePrefix` node attributes as [`CubeRegionKey`] but no
`validFrom`/`validTo`: `ComponentValueSetType` prohibits them (both editions), so the illegal state
is unrepresentable by field omission. Pub-field carrier, derived `Deserialize`.

Decisions: D-0020, D-0038, D-0051, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ComponentValueSet {
    /// The id of the component being selected (a structural reference, possibly nested).
    pub id: String,
    /// The values this component admits.
    pub selection: ComponentSelection,
    /// Whether the named values are included or excluded; `None` ⟺ absent (schema default `true`).
    pub include: Option<bool>,
    /// Whether codes drop the codelist-extension prefix; `None` ⟺ absent (no schema default).
    pub remove_prefix: Option<bool>,
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
///     CubeKeyValue { value: "A".to_string(), cascade: None, valid_from: None, valid_to: None };
/// let key = CubeRegionKey {
///     id: "FREQ".to_string(),
///     selection: KeyValueSelection::Values(CubeKeyValues::new(vec![value])?),
///     include: None,
///     remove_prefix: None,
///     valid_from: None,
///     valid_to: None,
/// };
/// let region = CubeRegion {
///     key_values: vec![key],
///     components: vec![],
///     include: None,
///     annotations: vec![],
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
/// (possibly nested) and the values it selects ([`DataComponentSelection`]), and carries the same
/// `include`/`remove_prefix` node attributes but no validity window (prohibited here).
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

A node type named after its spec complexType. `id` is carried on the struct (D-0051) as a
structural-only reference (`NestedNCNameIDType`, D-0020). Same node attributes as the cube
[`ComponentValueSet`]: `include` (schema `default="true"`, statedness stored, D-0052) and
`removePrefix` (no default, `Option<bool>`, D-0031). Validity is prohibited, so there are no such
fields. Pub-field carrier, derived `Deserialize`.

Decisions: D-0020, D-0038, D-0039, D-0051, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataComponentValueSet {
    /// The id of the component being selected (a structural reference, possibly nested).
    pub id: String,
    /// The values this component admits.
    pub selection: DataComponentSelection,
    /// Whether the named values are included or excluded; `None` ⟺ absent (schema default `true`).
    pub include: Option<bool>,
    /// Whether codes drop the codelist-extension prefix; `None` ⟺ absent (no schema default).
    pub remove_prefix: Option<bool>,
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
/// Names a dimension by its `id` and the values it takes ([`SimpleKeyValues`]). Its `include` flag is
/// schema-fixed to `true`, so it is the [`FixedInclude`] wrapper. A data-key selection carries no
/// validity window (prohibited here).
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`DataKeyValueType` diverges across editions: 3.0 allows exactly one `Value` (a `<choice>`); 3.1
allows unbounded (a `<sequence maxOccurs="unbounded">`, for keys like `FREQ = A or M or Q`). The
superset carries the 3.1 shape as the non-empty [`SimpleKeyValues`], which covers 3.0's exactly-one;
what a 3.0 writer does with more than one value is Phase-2 adapter policy (the same provenance class
as a 3.1-only attribute, D-0039/D-0037), not a Phase-1 rejection. `id` is structural-only
(`SingleNCNameIDType`, D-0020/D-0051); `include` is `fixed="true"`, so the `FixedInclude` wrapper
stores statedness and rejects a stated `false` (D-0052). Validity is prohibited, so there are no such
fields. Pub-field carrier: the rejection rides the `FixedInclude` and `SimpleKeyValues` custom impls
(§7 within-field rule).

Decisions: D-0039, D-0020, D-0051, D-0052.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataKeyValue {
    /// The id of the dimension being selected (a structural reference, not re-validated).
    pub id: String,
    /// The values this dimension takes.
    pub values: SimpleKeyValues,
    /// The schema-fixed `include` flag (always effectively `true`).
    pub include: FixedInclude,
    /// Whether codes drop the codelist-extension prefix; `None` ⟺ absent (no schema default).
    pub remove_prefix: Option<bool>,
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
/// let key_value = DataKeyValue {
///     id: "FREQ".to_string(),
///     values: SimpleKeyValues::new(vec!["A".to_string()])?,
///     include: FixedInclude::new(None)?,
///     remove_prefix: None,
/// };
/// let key = DataKey {
///     key_values: vec![key_value],
///     components: vec![],
///     include: FixedInclude::new(None)?,
///     annotations: vec![],
///     valid_from: None,
///     valid_to: None,
/// };
/// let key_set = DataKeySet { keys: DataKeys::new(vec![key])?, is_included: true };
/// assert_eq!(key_set.keys.as_slice().len(), 1);
///
/// // A key set must hold at least one key.
/// assert!(DataKeys::new(vec![]).is_err());
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
///     data_url: "https://example.com/sdmx".to_string(),
///     wsdl_url: None,
///     wadl_url: None,
///     is_rest_datasource: true,
///     is_web_service_datasource: false,
/// };
/// assert!(source.is_rest_datasource);
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
    pub is_rest_datasource: bool,
    /// Whether the source is reachable over web-service protocols.
    pub is_web_service_datasource: bool,
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
/// Always holds at least one [`DsdReference`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct DataStructureRefs(Vec<DsdReference>);

impl DataStructureRefs {
    /// Builds a data-structure reference list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyDataStructureRefs`] if `refs` is empty (a chosen attachment arm
    /// requires at least one reference).
    pub fn new(refs: Vec<DsdReference>) -> Result<Self, Error> {
        if refs.is_empty() {
            return Err(Error::EmptyDataStructureRefs);
        }
        Ok(Self(refs))
    }

    /// The references, in order (always at least one).
    #[must_use]
    pub fn as_slice(&self) -> &[DsdReference] {
        &self.0
    }
}

impl TryFrom<Vec<DsdReference>> for DataStructureRefs {
    type Error = Error;

    fn try_from(refs: Vec<DsdReference>) -> Result<Self, Error> {
        Self::new(refs)
    }
}

impl<'de> serde::Deserialize<'de> for DataStructureRefs {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(Vec::<DsdReference>::deserialize(deserializer)?).map_err(to_de_error)
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
    DataStructure(DsdReference),
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
    fn valid_from(&self) -> Option<&DateTime<FixedOffset>> {
        self.metadata.valid_from()
    }
    fn valid_to(&self) -> Option<&DateTime<FixedOffset>> {
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
///     CubeRegions, DataConstraint, DsdReference, IdentifiableMetadata, LocalisedString,
///     MaintainableMetadata, NameableMetadata, VersionableMetadata,
/// };
///
/// // A data constraint is a maintainable artefact, so it carries maintainable metadata.
/// let names = LocalisedString::new(vec![(Some("en".to_string()), "Allowed".to_string())])?;
/// let identifiable = IdentifiableMetadata::new("CR_EXR".to_string(), None, None, vec![], vec![])?;
/// let versionable = VersionableMetadata::new(
///     NameableMetadata::new(identifiable, names, None),
///     None,
///     None,
///     None,
/// );
/// let metadata =
///     MaintainableMetadata::new(versionable, "SDMX".to_string(), None, None, None, None)?;
/// let data = ConstraintModel::Data(DataConstraint {
///     metadata,
///     role: None,
///     attachment: None,
///     release_calendar: None,
///     key_sets: vec![],
///     regions: CubeRegions::new(vec![])?,
/// });
/// assert!(matches!(data, ConstraintModel::Data(_)));
///
/// // An availability constraint is not maintainable: it carries no metadata.
/// let availability = ConstraintModel::Availability(AvailabilityConstraint {
///     attachment: AvailabilityConstraintAttachment::DataStructure(DsdReference {
///         agency: "SDMX".to_string(),
///         id: "ECB_EXR1".to_string(),
///         version: "1.0.0".to_string(),
///     }),
///     region: CubeRegion {
///         key_values: vec![],
///         components: vec![],
///         include: None,
///         annotations: vec![],
///     },
///     annotations: vec![],
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
    use alloc::{format, string::ToString, vec};

    use super::*;

    fn period(p: &str) -> TimePeriodRange {
        TimePeriodRange { period: p.to_string(), inclusive: None }
    }

    #[test]
    fn cube_key_value_round_trips_with_cascade_and_validity() {
        let value = CubeKeyValue {
            value: "A".to_string(),
            cascade: Some(Cascade::IncludeChildren),
            valid_from: Some(SdmxTimePeriod::new("2020".to_string()).unwrap()),
            valid_to: Some(SdmxTimePeriod::new("2024".to_string()).unwrap()),
        };
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<CubeKeyValue>(&json).unwrap(), value);
    }

    #[test]
    fn simple_component_value_round_trips_with_lang() {
        let value = SimpleComponentValue {
            value: "EUR".to_string(),
            cascade: None,
            lang: Some("en".to_string()),
            valid_from: None,
            valid_to: None,
        };
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<SimpleComponentValue>(&json).unwrap(), value);
    }

    #[test]
    fn cube_key_values_rejects_empty() {
        assert_eq!(CubeKeyValues::new(vec![]).unwrap_err(), Error::EmptyCubeKeyValues);
        let value = CubeKeyValue {
            value: "A".to_string(),
            cascade: None,
            valid_from: None,
            valid_to: None,
        };
        assert_eq!(CubeKeyValues::new(vec![value]).unwrap().as_slice().len(), 1);
    }

    #[test]
    fn simple_component_values_rejects_empty() {
        assert_eq!(
            SimpleComponentValues::new(vec![]).unwrap_err(),
            Error::EmptySimpleComponentValues
        );
        let value = SimpleComponentValue {
            value: "EUR".to_string(),
            cascade: None,
            lang: None,
            valid_from: None,
            valid_to: None,
        };
        assert_eq!(SimpleComponentValues::new(vec![value]).unwrap().as_slice().len(), 1);
    }

    #[test]
    fn cube_key_values_deserialize_rejects_empty_on_the_wire() {
        assert!(serde_json::from_str::<CubeKeyValues>("[]").is_err());
        let value = CubeKeyValue {
            value: "A".to_string(),
            cascade: None,
            valid_from: None,
            valid_to: None,
        };
        let values = CubeKeyValues::new(vec![value]).unwrap();
        let json = serde_json::to_string(&values).unwrap();
        assert_eq!(serde_json::from_str::<CubeKeyValues>(&json).unwrap(), values);
    }

    #[test]
    fn simple_component_values_deserialize_rejects_empty_on_the_wire() {
        assert!(serde_json::from_str::<SimpleComponentValues>("[]").is_err());
    }

    #[test]
    fn time_period_range_effective_is_inclusive_applies_default() {
        assert!(period("2024").effective_is_inclusive());
        assert!(
            TimePeriodRange { period: "2024".to_string(), inclusive: Some(true) }
                .effective_is_inclusive()
        );
        assert!(
            !TimePeriodRange { period: "2024".to_string(), inclusive: Some(false) }
                .effective_is_inclusive()
        );
    }

    #[test]
    fn time_range_before_round_trips() {
        let range = TimeRange {
            kind: TimeRangeKind::Before(period("2024")),
            valid_from: None,
            valid_to: None,
        };
        let json = serde_json::to_string(&range).unwrap();
        assert_eq!(serde_json::from_str::<TimeRange>(&json).unwrap(), range);
    }

    #[test]
    fn time_range_after_round_trips() {
        let range = TimeRange {
            kind: TimeRangeKind::After(period("2020")),
            valid_from: None,
            valid_to: None,
        };
        let json = serde_json::to_string(&range).unwrap();
        assert_eq!(serde_json::from_str::<TimeRange>(&json).unwrap(), range);
    }

    #[test]
    fn time_range_between_round_trips_with_validity() {
        let range = TimeRange {
            kind: TimeRangeKind::Between { start: period("2020"), end: period("2024") },
            valid_from: Some(SdmxTimePeriod::new("2019".to_string()).unwrap()),
            valid_to: Some(SdmxTimePeriod::new("2025".to_string()).unwrap()),
        };
        let json = serde_json::to_string(&range).unwrap();
        assert_eq!(serde_json::from_str::<TimeRange>(&json).unwrap(), range);
    }

    #[test]
    fn time_range_validity_rejects_bad_period_on_the_wire() {
        let range = TimeRange {
            kind: TimeRangeKind::Before(period("2024")),
            valid_from: Some(SdmxTimePeriod::new("2020".to_string()).unwrap()),
            valid_to: None,
        };
        let json = serde_json::to_string(&range).unwrap();
        let bad = json.replace("2020", "not-a-period");
        assert!(serde_json::from_str::<TimeRange>(&bad).is_err());
    }

    fn cube_key(id: &str, value: &str) -> CubeRegionKey {
        let v = CubeKeyValue {
            value: value.to_string(),
            cascade: None,
            valid_from: None,
            valid_to: None,
        };
        CubeRegionKey {
            id: id.to_string(),
            selection: KeyValueSelection::Values(CubeKeyValues::new(vec![v]).unwrap()),
            include: None,
            remove_prefix: None,
            valid_from: None,
            valid_to: None,
        }
    }

    #[test]
    fn cube_region_key_round_trips_with_time_range_selection() {
        let key = CubeRegionKey {
            id: "TIME_PERIOD".to_string(),
            selection: KeyValueSelection::TimeRange(TimeRange {
                kind: TimeRangeKind::After(period("2020")),
                valid_from: None,
                valid_to: None,
            }),
            include: Some(false),
            remove_prefix: Some(true),
            valid_from: Some(SdmxTimePeriod::new("2019".to_string()).unwrap()),
            valid_to: None,
        };
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(serde_json::from_str::<CubeRegionKey>(&json).unwrap(), key);
    }

    #[test]
    fn component_selection_empty_is_distinct_from_values() {
        let value = SimpleComponentValue {
            value: "EUR".to_string(),
            cascade: None,
            lang: None,
            valid_from: None,
            valid_to: None,
        };
        let values = ComponentSelection::Values(SimpleComponentValues::new(vec![value]).unwrap());
        let empty = ComponentSelection::Empty;
        assert_ne!(values, empty);
        // Each round-trips to itself, so Empty is not collapsed into an empty value list.
        let empty_json = serde_json::to_string(&empty).unwrap();
        assert_eq!(serde_json::from_str::<ComponentSelection>(&empty_json).unwrap(), empty);
        let values_json = serde_json::to_string(&values).unwrap();
        assert_eq!(serde_json::from_str::<ComponentSelection>(&values_json).unwrap(), values);
    }

    #[test]
    fn component_value_set_round_trips() {
        let set = ComponentValueSet {
            id: "CONTACT.ADDRESS.STREET".to_string(),
            selection: ComponentSelection::Empty,
            include: Some(true),
            remove_prefix: None,
        };
        let json = serde_json::to_string(&set).unwrap();
        assert_eq!(serde_json::from_str::<ComponentValueSet>(&json).unwrap(), set);
    }

    #[test]
    fn cube_region_round_trips_preserving_selection_order() {
        let region = CubeRegion {
            key_values: vec![cube_key("FREQ", "A"), cube_key("REF_AREA", "EU")],
            components: vec![ComponentValueSet {
                id: "OBS_STATUS".to_string(),
                selection: ComponentSelection::Empty,
                include: None,
                remove_prefix: None,
            }],
            include: Some(true),
            annotations: vec![],
        };
        let json = serde_json::to_string(&region).unwrap();
        let restored = serde_json::from_str::<CubeRegion>(&json).unwrap();
        assert_eq!(restored, region);
        // Wire order of the dimension selections is preserved.
        assert_eq!(restored.key_values[0].id, "FREQ");
        assert_eq!(restored.key_values[1].id, "REF_AREA");
    }

    #[test]
    fn cube_region_annotations_empty_maps_absent() {
        let region = CubeRegion {
            key_values: vec![],
            components: vec![],
            include: None,
            annotations: vec![],
        };
        assert!(region.annotations.is_empty());
        let json = serde_json::to_string(&region).unwrap();
        assert_eq!(serde_json::from_str::<CubeRegion>(&json).unwrap(), region);
    }

    #[test]
    fn cube_regions_rejects_more_than_two() {
        let region = || CubeRegion {
            key_values: vec![],
            components: vec![],
            include: None,
            annotations: vec![],
        };
        assert!(CubeRegions::new(vec![]).unwrap().as_slice().is_empty());
        assert_eq!(CubeRegions::new(vec![region(), region()]).unwrap().as_slice().len(), 2);
        assert_eq!(
            CubeRegions::new(vec![region(), region(), region()]).unwrap_err(),
            Error::TooManyCubeRegions
        );
    }

    #[test]
    fn cube_regions_deserialize_rejects_more_than_two_on_the_wire() {
        let region = || CubeRegion {
            key_values: vec![],
            components: vec![],
            include: None,
            annotations: vec![],
        };
        let three = CubeRegions::new(vec![region(), region()]).unwrap();
        let mut as_vec = three.as_slice().to_vec();
        as_vec.push(region());
        let json = serde_json::to_string(&as_vec).unwrap();
        assert!(serde_json::from_str::<CubeRegions>(&json).is_err());
    }

    fn data_key_value(id: &str, value: &str) -> DataKeyValue {
        DataKeyValue {
            id: id.to_string(),
            values: SimpleKeyValues::new(vec![value.to_string()]).unwrap(),
            include: FixedInclude::new(None).unwrap(),
            remove_prefix: None,
        }
    }

    #[test]
    fn data_component_values_rejects_empty() {
        assert_eq!(DataComponentValues::new(vec![]).unwrap_err(), Error::EmptyDataComponentValues);
        let value = DataComponentValue { value: "EUR".to_string(), cascade: None, lang: None };
        assert_eq!(DataComponentValues::new(vec![value]).unwrap().as_slice().len(), 1);
    }

    #[test]
    fn simple_key_values_rejects_empty() {
        assert_eq!(SimpleKeyValues::new(vec![]).unwrap_err(), Error::EmptySimpleKeyValues);
        let ok = SimpleKeyValues::new(vec!["A".to_string()]).unwrap();
        assert_eq!(ok.as_slice().len(), 1);
        // The wire path rejects an empty list too.
        assert!(serde_json::from_str::<SimpleKeyValues>("[]").is_err());
    }

    #[test]
    fn data_keys_rejects_empty() {
        assert_eq!(DataKeys::new(vec![]).unwrap_err(), Error::EmptyDataKeys);
        let key = DataKey {
            key_values: vec![data_key_value("FREQ", "A")],
            components: vec![],
            include: FixedInclude::new(None).unwrap(),
            annotations: vec![],
            valid_from: None,
            valid_to: None,
        };
        assert_eq!(DataKeys::new(vec![key]).unwrap().as_slice().len(), 1);
    }

    #[test]
    fn data_key_value_carries_3_1_multi_value_superset() {
        // The 3.1 unbounded shape (FREQ = A or M or Q) is the carried superset; 3.0's single value
        // is the degenerate one-element case.
        let multi = DataKeyValue {
            id: "FREQ".to_string(),
            values: SimpleKeyValues::new(vec!["A".to_string(), "M".to_string(), "Q".to_string()])
                .unwrap(),
            include: FixedInclude::new(Some(true)).unwrap(),
            remove_prefix: None,
        };
        assert_eq!(multi.values.as_slice().len(), 3);
        let json = serde_json::to_string(&multi).unwrap();
        assert_eq!(serde_json::from_str::<DataKeyValue>(&json).unwrap(), multi);
    }

    #[test]
    fn data_key_value_include_rejects_stated_false_on_the_wire() {
        let value = data_key_value("FREQ", "A");
        let json = serde_json::to_string(&value).unwrap();
        // include is fixed="true"; a stated false contradicts it and is rejected.
        let bad = json.replace("\"include\":null", "\"include\":false");
        assert!(serde_json::from_str::<DataKeyValue>(&bad).is_err());
    }

    #[test]
    fn data_key_round_trips_with_validity_and_annotations() {
        let key = DataKey {
            key_values: vec![data_key_value("FREQ", "A")],
            components: vec![DataComponentValueSet {
                id: "OBS_STATUS".to_string(),
                selection: DataComponentSelection::Empty,
                include: None,
                remove_prefix: None,
            }],
            include: FixedInclude::new(Some(true)).unwrap(),
            annotations: vec![],
            valid_from: Some(SdmxTimePeriod::new("2020".to_string()).unwrap()),
            valid_to: None,
        };
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(serde_json::from_str::<DataKey>(&json).unwrap(), key);
    }

    #[test]
    fn data_component_selection_empty_is_distinct_from_values() {
        let values = DataComponentSelection::Values(
            DataComponentValues::new(vec![DataComponentValue {
                value: "EUR".to_string(),
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
                components: vec![],
                include: FixedInclude::new(None).unwrap(),
                annotations: vec![],
                valid_from: None,
                valid_to: None,
            }])
            .unwrap(),
            is_included: true,
        };
        let json = serde_json::to_string(&set).unwrap();
        assert_eq!(serde_json::from_str::<DataKeySet>(&json).unwrap(), set);
    }

    #[test]
    fn data_component_selection_time_range_arm_round_trips() {
        let set = DataComponentValueSet {
            id: "TIME_PERIOD".to_string(),
            selection: DataComponentSelection::TimeRange(TimeRange {
                kind: TimeRangeKind::Before(period("2024")),
                valid_from: None,
                valid_to: None,
            }),
            include: None,
            remove_prefix: None,
        };
        let json = serde_json::to_string(&set).unwrap();
        assert_eq!(serde_json::from_str::<DataComponentValueSet>(&json).unwrap(), set);
    }

    #[test]
    fn component_selection_time_range_arm_round_trips() {
        let set = ComponentValueSet {
            id: "TIME_PERIOD".to_string(),
            selection: ComponentSelection::TimeRange(TimeRange {
                kind: TimeRangeKind::Before(period("2024")),
                valid_from: None,
                valid_to: None,
            }),
            include: None,
            remove_prefix: None,
        };
        let json = serde_json::to_string(&set).unwrap();
        assert_eq!(serde_json::from_str::<ComponentValueSet>(&json).unwrap(), set);
    }

    #[test]
    fn data_component_value_set_values_arm_round_trips_and_rejects_empty() {
        let set = DataComponentValueSet {
            id: "CURRENCY".to_string(),
            selection: DataComponentSelection::Values(
                DataComponentValues::new(vec![DataComponentValue {
                    value: "EUR".to_string(),
                    cascade: Some(Cascade::IncludeChildren),
                    lang: Some("en".to_string()),
                }])
                .unwrap(),
            ),
            include: None,
            remove_prefix: None,
        };
        let json = serde_json::to_string(&set).unwrap();
        assert_eq!(serde_json::from_str::<DataComponentValueSet>(&json).unwrap(), set);
        // The Values deserialize path routes through new(), so an empty list is rejected on the
        // wire, not synthesised as Values([]).
        assert!(serde_json::from_str::<DataComponentValues>("[]").is_err());
    }

    #[test]
    fn queryable_data_source_round_trips_with_optional_urls() {
        let source = QueryableDataSource {
            data_url: "https://example.com/sdmx".to_string(),
            wsdl_url: Some("https://example.com/sdmx?wsdl".to_string()),
            wadl_url: None,
            is_rest_datasource: true,
            is_web_service_datasource: true,
        };
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(serde_json::from_str::<QueryableDataSource>(&json).unwrap(), source);
    }

    fn dsd_ref(id: &str) -> DsdReference {
        DsdReference {
            agency: "SDMX".to_string(),
            id: id.to_string(),
            version: "1.0.0".to_string(),
        }
    }

    fn dataflow_ref(id: &str) -> DataflowReference {
        DataflowReference {
            agency: "ECB".to_string(),
            id: id.to_string(),
            version: "1.0.0".to_string(),
        }
    }

    fn agreement_ref(id: &str) -> ProvisionAgreementReference {
        ProvisionAgreementReference {
            agency: "ECB".to_string(),
            id: id.to_string(),
            version: "1.0.0".to_string(),
        }
    }

    #[test]
    fn attachment_ref_newtypes_reject_empty_and_expose_their_slice() {
        assert_eq!(DataStructureRefs::new(vec![]).unwrap_err(), Error::EmptyDataStructureRefs);
        assert_eq!(DataflowRefs::new(vec![]).unwrap_err(), Error::EmptyDataflowRefs);
        assert_eq!(
            ProvisionAgreementRefs::new(vec![]).unwrap_err(),
            Error::EmptyProvisionAgreementRefs
        );
        assert_eq!(SimpleDataSources::new(vec![]).unwrap_err(), Error::EmptySimpleDataSources);

        assert_eq!(DataStructureRefs::new(vec![dsd_ref("ECB_EXR1")]).unwrap().as_slice().len(), 1);
        assert_eq!(DataflowRefs::new(vec![dataflow_ref("EXR")]).unwrap().as_slice().len(), 1);
        assert_eq!(
            ProvisionAgreementRefs::new(vec![agreement_ref("PA_EXR")]).unwrap().as_slice().len(),
            1
        );
        assert_eq!(
            SimpleDataSources::new(vec!["https://example.com/data".to_string()])
                .unwrap()
                .as_slice()
                .len(),
            1
        );
    }

    #[test]
    fn attachment_ref_newtypes_reject_empty_on_the_wire() {
        assert!(serde_json::from_str::<DataStructureRefs>("[]").is_err());
        assert!(serde_json::from_str::<DataflowRefs>("[]").is_err());
        assert!(serde_json::from_str::<ProvisionAgreementRefs>("[]").is_err());
        assert!(serde_json::from_str::<SimpleDataSources>("[]").is_err());
    }

    #[test]
    fn data_constraint_attachment_structural_arms_round_trip_with_queryable() {
        let queryable = vec![QueryableDataSource {
            data_url: "https://example.com/sdmx".to_string(),
            wsdl_url: None,
            wadl_url: None,
            is_rest_datasource: true,
            is_web_service_datasource: false,
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
                queryable: vec![],
            },
        ];
        for attachment in arms {
            let json = serde_json::to_string(&attachment).unwrap();
            assert_eq!(
                serde_json::from_str::<DataConstraintAttachment>(&json).unwrap(),
                attachment
            );
        }
    }

    #[test]
    fn data_constraint_attachment_3_0_only_arms_round_trip() {
        // The DataProvider single arm and the 3.0-only SimpleDataSource arm.
        let provider = DataConstraintAttachment::DataProvider(DataProviderReference {
            agency: "SDMX".to_string(),
            scheme_id: "DATA_PROVIDERS".to_string(),
            id: "ECB".to_string(),
        });
        let sources = DataConstraintAttachment::SimpleDataSource(
            SimpleDataSources::new(vec!["https://example.com/data".to_string()]).unwrap(),
        );
        for attachment in [provider, sources] {
            let json = serde_json::to_string(&attachment).unwrap();
            assert_eq!(
                serde_json::from_str::<DataConstraintAttachment>(&json).unwrap(),
                attachment
            );
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
            let json = serde_json::to_string(&attachment).unwrap();
            assert_eq!(
                serde_json::from_str::<AvailabilityConstraintAttachment>(&json).unwrap(),
                attachment
            );
        }
    }

    fn constraint_metadata(id: &str) -> MaintainableMetadata {
        use crate::metadata::{IdentifiableMetadata, NameableMetadata, VersionableMetadata};
        let names =
            LocalisedString::new(vec![(Some("en".to_string()), "A constraint".to_string())])
                .unwrap();
        let identifiable =
            IdentifiableMetadata::new(id.to_string(), None, None, vec![], vec![]).unwrap();
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable, names, None),
            None,
            None,
            None,
        );
        MaintainableMetadata::new(versionable, "SDMX".to_string(), None, None, None, None).unwrap()
    }

    fn data_key_set() -> DataKeySet {
        DataKeySet {
            keys: DataKeys::new(vec![DataKey {
                key_values: vec![data_key_value("FREQ", "A")],
                components: vec![],
                include: FixedInclude::new(None).unwrap(),
                annotations: vec![],
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
            components: vec![],
            include: None,
            annotations: vec![],
        }
    }

    #[test]
    fn data_constraint_forwards_every_artefact_accessor() {
        use crate::metadata::{IdentifiableMetadata, NameableMetadata, VersionableMetadata};
        let version = SdmxVersion::new("1.2.3".to_string()).unwrap();
        let valid_from = DateTime::parse_from_rfc3339("2024-01-01T00:00:00+00:00").unwrap();
        let annotation = Annotation {
            id: Some("a1".to_string()),
            annotation_type: None,
            annotation_title: None,
            annotation_urls: vec![],
            annotation_value: None,
            texts: None,
        };
        let link = Link {
            rel: "self".to_string(),
            url: "https://example.com/x".to_string(),
            urn: None,
            link_type: None,
        };
        let names =
            LocalisedString::new(vec![(Some("en".to_string()), "Constraint".to_string())]).unwrap();
        let descriptions =
            LocalisedString::new(vec![(Some("en".to_string()), "How much".to_string())]).unwrap();
        let identifiable = IdentifiableMetadata::new(
            "CR_EXR".to_string(),
            Some("uri".to_string()),
            Some("urn:x".to_string()),
            vec![annotation],
            vec![link],
        )
        .unwrap();
        let versionable = VersionableMetadata::new(
            NameableMetadata::new(identifiable, names, Some(descriptions)),
            Some(version),
            Some(valid_from),
            None,
        );
        let metadata = MaintainableMetadata::new(
            versionable,
            "ESTAT".to_string(),
            Some(true),
            Some(true),
            Some("https://service".to_string()),
            Some("https://structure".to_string()),
        )
        .unwrap();
        let constraint = DataConstraint {
            metadata,
            role: None,
            attachment: None,
            release_calendar: None,
            key_sets: vec![],
            regions: CubeRegions::new(vec![]).unwrap(),
        };

        // Every forwarded accessor resolves through the metadata leaf.
        assert_eq!(constraint.id(), "CR_EXR");
        assert_eq!(constraint.urn(), Some("urn:x"));
        assert_eq!(constraint.annotations().len(), 1);
        assert_eq!(constraint.links().len(), 1);
        assert_eq!(constraint.names().first(), "Constraint");
        assert_eq!(constraint.descriptions().map(LocalisedString::first), Some("How much"));
        assert_eq!(constraint.version().map(SdmxVersion::as_str), Some("1.2.3"));
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
                agency: "SDMX".to_string(),
                scheme_id: "DATA_PROVIDERS".to_string(),
                id: "ECB".to_string(),
            })),
            release_calendar: Some(ReleaseCalendar {
                periodicity: "P1M".to_string(),
                offset: "P0D".to_string(),
                tolerance: "P7D".to_string(),
            }),
            key_sets: vec![data_key_set()],
            regions: CubeRegions::new(vec![cube_region()]).unwrap(),
        };
        let json = serde_json::to_string(&constraint).unwrap();
        assert_eq!(serde_json::from_str::<DataConstraint>(&json).unwrap(), constraint);
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
        let key_set_only = with(vec![data_key_set()], vec![]);
        let region_only = with(vec![], vec![cube_region()]);
        let both = with(vec![data_key_set()], vec![cube_region(), cube_region()]);
        let neither = with(vec![], vec![]);
        for constraint in [key_set_only, region_only, both, neither] {
            let json = serde_json::to_string(&constraint).unwrap();
            assert_eq!(serde_json::from_str::<DataConstraint>(&json).unwrap(), constraint);
        }
    }

    #[test]
    fn data_constraint_deserialize_rejects_more_than_two_regions_on_the_wire() {
        let constraint = DataConstraint {
            metadata: constraint_metadata("CR_EXR"),
            role: None,
            attachment: None,
            release_calendar: None,
            key_sets: vec![],
            regions: CubeRegions::new(vec![cube_region(), cube_region()]).unwrap(),
        };
        let json = serde_json::to_string(&constraint).unwrap();
        // Splice a third region into the serialised regions array; the CubeRegions bound must reject
        // it on the wire path, so DataConstraint's derived Deserialize inherits the rejection.
        let region_json = serde_json::to_string(&cube_region()).unwrap();
        let bad =
            json.replacen(&format!("{region_json}]"), &format!("{region_json},{region_json}]"), 1);
        assert!(serde_json::from_str::<DataConstraint>(&bad).is_err());
    }

    #[test]
    fn availability_constraint_round_trips() {
        let constraint = AvailabilityConstraint {
            attachment: AvailabilityConstraintAttachment::Dataflow(dataflow_ref("EXR")),
            region: cube_region(),
            annotations: vec![],
            series_count: Some(42),
            obs_count: Some(-1),
        };
        let json = serde_json::to_string(&constraint).unwrap();
        assert_eq!(serde_json::from_str::<AvailabilityConstraint>(&json).unwrap(), constraint);
    }

    #[test]
    fn constraint_model_arms_round_trip() {
        let data = ConstraintModel::Data(DataConstraint {
            metadata: constraint_metadata("CR_EXR"),
            role: Some(ConstraintRole::Allowed),
            attachment: None,
            release_calendar: None,
            key_sets: vec![],
            regions: CubeRegions::new(vec![]).unwrap(),
        });
        let availability = ConstraintModel::Availability(AvailabilityConstraint {
            attachment: AvailabilityConstraintAttachment::Dataflow(dataflow_ref("EXR")),
            region: cube_region(),
            annotations: vec![],
            series_count: None,
            obs_count: None,
        });
        for model in [data, availability] {
            let json = serde_json::to_string(&model).unwrap();
            assert_eq!(serde_json::from_str::<ConstraintModel>(&json).unwrap(), model);
        }
    }

    #[test]
    fn cube_key_values_try_from_rejects_empty() {
        assert_eq!(CubeKeyValues::try_from(vec![]).unwrap_err(), Error::EmptyCubeKeyValues);
    }

    #[test]
    fn simple_component_values_try_from_rejects_empty() {
        assert_eq!(
            SimpleComponentValues::try_from(vec![]).unwrap_err(),
            Error::EmptySimpleComponentValues
        );
    }

    #[test]
    fn data_component_values_try_from_rejects_empty() {
        assert_eq!(
            DataComponentValues::try_from(vec![]).unwrap_err(),
            Error::EmptyDataComponentValues
        );
    }

    #[test]
    fn simple_key_values_try_from_rejects_empty() {
        assert_eq!(SimpleKeyValues::try_from(vec![]).unwrap_err(), Error::EmptySimpleKeyValues);
    }

    #[test]
    fn data_keys_try_from_rejects_empty() {
        assert_eq!(DataKeys::try_from(vec![]).unwrap_err(), Error::EmptyDataKeys);
    }

    #[test]
    fn data_structure_refs_try_from_rejects_empty() {
        assert_eq!(DataStructureRefs::try_from(vec![]).unwrap_err(), Error::EmptyDataStructureRefs);
    }

    #[test]
    fn dataflow_refs_try_from_rejects_empty() {
        assert_eq!(DataflowRefs::try_from(vec![]).unwrap_err(), Error::EmptyDataflowRefs);
    }

    #[test]
    fn provision_agreement_refs_try_from_rejects_empty() {
        assert_eq!(
            ProvisionAgreementRefs::try_from(vec![]).unwrap_err(),
            Error::EmptyProvisionAgreementRefs
        );
    }

    #[test]
    fn simple_data_sources_try_from_rejects_empty() {
        assert_eq!(SimpleDataSources::try_from(vec![]).unwrap_err(), Error::EmptySimpleDataSources);
    }

    #[test]
    fn cube_regions_try_from_rejects_more_than_two() {
        // CubeRegions permits empty; its boundary is the >2 cap, so it is exercised there.
        let region = || CubeRegion {
            key_values: vec![],
            components: vec![],
            include: None,
            annotations: vec![],
        };
        assert_eq!(
            CubeRegions::try_from(vec![region(), region(), region()]).unwrap_err(),
            Error::TooManyCubeRegions
        );
    }
}
