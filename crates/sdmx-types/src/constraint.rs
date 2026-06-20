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

use crate::{
    annotation::Annotation,
    codelist::Cascade,
    error::{Error, to_de_error},
    lexical::SdmxTimePeriod,
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
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

impl<'de> serde::Deserialize<'de> for CubeRegions {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let regions = Vec::<CubeRegion>::deserialize(deserializer)?;
        Self::new(regions).map_err(to_de_error)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::{string::ToString, vec};

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
}
