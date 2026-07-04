//! Cross-references to other SDMX artefacts.
//!
//! A reference names another artefact by its coordinates rather than embedding it. These are the
//! natural map keys of the model (deduping a set of fetched artefacts, "have I already resolved
//! this codelist?"), so using them as keys is a natural case; like every float-free value type in
//! the crate, they derive [`Hash`].
//!
//! The references group by coordinate shape. [`CodelistReference`], [`ValueListReference`],
//! [`DsdReference`], [`DataflowReference`], and [`ProvisionAgreementReference`] name a maintainable
//! artefact (a codelist, value list, data structure definition, dataflow, or provision agreement) by
//! its full triple of agency, id, and version. [`ConceptReference`] (a concept) and
//! [`DataProviderReference`] (a data provider) name an item within its scheme, carrying the scheme
//! id and the scheme's version.
//!
//! Every reference owns its URN contract: [`Display`](core::fmt::Display) renders the full SDMX
//! URN for the reference's class (`urn:sdmx:org.sdmx.infomodel.codelist.Codelist=SDMX:CL_FREQ(1.0.0)`),
//! and [`FromStr`](core::str::FromStr) parses exactly that class, splitting the URN into the
//! decomposed fields. Versions are typed [`VersionRef`], so the `+` wildcard reference forms are
//! carried structurally. [`Display`](core::fmt::Display) renders the fields verbatim, so the round-trip
//! through [`FromStr`](core::str::FromStr) is guaranteed only for grammar-valid fields: emitting a
//! conformant URN from hand-built fields is the writer's obligation, and a field outside the URN
//! grammar (an off-grammar agency, or a [`VersionRef::Any`] no reference class admits) renders
//! but does not parse back.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Invariant-free pub-field carriers with derived `Serialize`/`Deserialize`: identifiers are validated
at declaration, not at reference (D-0020), so the fields hold their content verbatim and there is no
construction invariant. The URN contract sits on the parse path instead: `FromStr` validates the
per-class URN grammar (prefix, class, agency, id, version, item tail) and is the contract the
Phase-2 parsers split wire references through; `Display` renders the fields verbatim, so emitting a
wire-conformant URN from hand-built fields is the writer's obligation (lint territory, like every
other carrier field). The serde impls stay field-wise derived: the internal projection is not the
wire (D-0068), so it does not collapse to the URN string.

All seven reference classes descend from the URN *reference* chain (`UrnReferenceVersionPart`), so
their version part admits the `+` wildcard forms and is typed `VersionRef` (D-0071); none admits the
bare `*`, whose `WildcardUrnType` family is consumed only by unmodelled metadata targets, so a
`VersionRef::Any` in a reference is grammar-unparseable and rejected by `FromStr`, while remaining
carrier-representable like any other unvalidated field (a catalogued lint, D-0073). The
item-in-scheme URN mandates the scheme version (`agency:scheme_id(version).item`), so
`ConceptReference` and `DataProviderReference` carry it; the item tail is held verbatim and may be
nested. `Hash` originated on these reference/identity types as the natural map keys; D-0065 later
generalised it to every float-free value type, so the references are no longer distinctive in
deriving it.

One struct per spec reference type rather than a unified `MaintainableReference`: each maps 1-to-1 to
a distinct concept in the information model, owns its class URN, and the item-in-scheme references
diverge from the flat maintainable triple, so the correspondence is kept (D-0002). Each reference
lands with its first caller; the constraint-attachment references (dataflow, provision agreement,
data provider) joined with the constraint types that consume them, taking the shapes D-0034 fixes
(`DataProviderReference` shares `ComponentUrnReferenceType` with `ConceptReferenceType`: a data
provider is an item in a data-provider scheme, not a maintainable in its own right).

Decisions: D-0002, D-0020, D-0021, D-0034, D-0047, D-0048, D-0065, D-0071, D-0073.
"#
)]

use alloc::string::{String, ToString};

use crate::{error::Error, lexical::VersionRef};

// ---------------------------------------------------------------------------
// URN grammar helpers (shared by every reference's Display/FromStr)
// ---------------------------------------------------------------------------

/// The URN prefix common to every SDMX structural reference.
const URN_PREFIX: &str = "urn:sdmx:org.sdmx.infomodel.";

/// The decomposed parts of a reference URN, borrowed from the input.
struct UrnParts<'a> {
    agency: &'a str,
    id: &'a str,
    version: VersionRef,
    item: Option<&'a str>,
}

/// An agency identifier: dot-separated `NCName` segments (sub-agencies nest).
fn is_agency(s: &str) -> bool {
    !s.is_empty()
        && s.split('.').all(|seg| {
            seg.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
                && seg.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        })
}

/// A single identifier segment from the URN id character class.
fn is_id_segment(s: &str) -> bool {
    !s.is_empty()
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'@' | b'$' | b'-'))
}

/// An item tail: one or more dot-separated id segments (nested container paths are wire-legal).
fn is_item_path(s: &str) -> bool {
    !s.is_empty() && s.split('.').all(is_id_segment)
}

/// Splits `urn` for the given `class` path (for example `codelist.Codelist`) into its parts,
/// validating each against the URN grammar. The version part is the reference grammar
/// (`VersionRef`), in which the bare `*` is not admitted by any structural reference class.
fn parse_reference_urn<'a>(urn: &'a str, class: &str) -> Option<UrnParts<'a>> {
    let rest = urn.strip_prefix(URN_PREFIX)?.strip_prefix(class)?.strip_prefix('=')?;
    let (agency, rest) = rest.split_once(':')?;
    let (id, rest) = rest.split_once('(')?;
    let (version, tail) = rest.split_once(')')?;
    let item = match tail {
        "" => None,
        tail => {
            let item = tail.strip_prefix('.')?;
            is_item_path(item).then_some(item)?;
            Some(item)
        }
    };
    if !is_agency(agency) || !is_id_segment(id) {
        return None;
    }
    match VersionRef::new(version.to_string()).ok()? {
        VersionRef::Any => None,
        version => Some(UrnParts { agency, id, version, item }),
    }
}

/// Parses a maintainable-triple URN (`agency:id(version)`, no item tail) for `class`.
fn parse_triple(urn: &str, class: &'static str) -> Result<(String, String, VersionRef), Error> {
    match parse_reference_urn(urn, class) {
        Some(UrnParts { agency, id, version, item: None }) => {
            Ok((agency.to_string(), id.to_string(), version))
        }
        _ => Err(Error::InvalidReferenceUrn { urn: urn.to_string(), class }),
    }
}

/// Parses an item-in-scheme URN (`agency:scheme_id(version).item`) for `class`.
#[allow(clippy::type_complexity)]
fn parse_item(
    urn: &str,
    class: &'static str,
) -> Result<(String, String, VersionRef, String), Error> {
    match parse_reference_urn(urn, class) {
        Some(UrnParts { agency, id, version, item: Some(item) }) => {
            Ok((agency.to_string(), id.to_string(), version, item.to_string()))
        }
        _ => Err(Error::InvalidReferenceUrn { urn: urn.to_string(), class }),
    }
}

// ---------------------------------------------------------------------------
// DsdReference
// ---------------------------------------------------------------------------

/// A reference to a [`DataStructureDefinition`](crate::DataStructureDefinition) by its maintenance
/// coordinates.
///
/// ## Specification
/// - **Type**: `DataStructureReferenceType`
/// - **Element**: N/A (Reference Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataStructureReferenceType.md"))]
///
/// Identifies a data structure definition by the flat maintainable triple (agency, id, version). A
/// DSD is a maintainable artefact, so its reference carries a version, like [`CodelistReference`].
///
/// # Examples
///
/// ```
/// use sdmx_types::DsdReference;
///
/// // All fields are public, so you can construct one directly.
/// let reference = DsdReference {
///     agency: "SDMX".to_string(),
///     id: "ECB_EXR1".to_string(),
///     version: "1.0.0".parse()?,
/// };
/// let urn = "urn:sdmx:org.sdmx.infomodel.datastructure.DataStructure=SDMX:ECB_EXR1(1.0.0)";
/// assert_eq!(reference.to_string(), urn);
/// assert_eq!(urn.parse::<DsdReference>()?, reference);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DsdReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The referenced data structure definition's id.
    pub id: String,
    /// The referenced data structure definition's version reference.
    pub version: VersionRef,
}

// ---------------------------------------------------------------------------
// ConceptReference
// ---------------------------------------------------------------------------

/// A reference to a [`Concept`](crate::Concept) by its coordinates within a concept scheme.
///
/// ## Specification
/// - **Type**: `ConceptReferenceType`
/// - **Element**: N/A (Reference Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ConceptReferenceType.md"))]
///
/// A concept is an *item in a concept scheme*, not a maintainable artefact in its own right, so its
/// reference takes the item-in-scheme shape (agency, scheme id, item id) rather than the flat
/// maintainable triple carried by [`CodelistReference`] and [`DsdReference`]: the carried version is
/// the enclosing scheme's, which the reference URN mandates.
///
/// # Examples
///
/// ```
/// use sdmx_types::ConceptReference;
///
/// // All fields are public, so you can construct one directly.
/// let reference = ConceptReference {
///     agency: "SDMX".to_string(),
///     scheme_id: "CS_FREQ".to_string(),
///     version: "1.0.0".parse()?,
///     id: "FREQ".to_string(),
/// };
/// let urn = "urn:sdmx:org.sdmx.infomodel.conceptscheme.Concept=SDMX:CS_FREQ(1.0.0).FREQ";
/// assert_eq!(reference.to_string(), urn);
/// assert_eq!(urn.parse::<ConceptReference>()?, reference);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ConceptReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The id of the concept scheme the concept belongs to (`maintainableParentID`).
    pub scheme_id: String,
    /// The enclosing concept scheme's version reference.
    pub version: VersionRef,
    /// The referenced concept's id.
    pub id: String,
}

// ---------------------------------------------------------------------------
// CodelistReference
// ---------------------------------------------------------------------------

/// A reference to a [`Codelist`](crate::Codelist) by its maintenance coordinates.
///
/// ## Specification
/// - **Type**: `CodelistReferenceType`
/// - **Element**: N/A (Reference Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/CodelistReferenceType.md"))]
///
/// Identifies a codelist by the flat maintainable triple (agency, id, version). A codelist is a
/// maintainable artefact, so its reference carries a version, unlike the item-in-scheme references.
///
/// # Examples
///
/// ```
/// use sdmx_types::CodelistReference;
///
/// // All fields are public, so you can construct one directly.
/// let reference = CodelistReference {
///     agency: "SDMX".to_string(),
///     id: "CL_FREQ".to_string(),
///     version: "1.0.0".parse()?,
/// };
/// let urn = "urn:sdmx:org.sdmx.infomodel.codelist.Codelist=SDMX:CL_FREQ(1.0.0)";
/// assert_eq!(reference.to_string(), urn);
/// assert_eq!(urn.parse::<CodelistReference>()?, reference);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CodelistReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The referenced codelist's id.
    pub id: String,
    /// The referenced codelist's version reference.
    pub version: VersionRef,
}

// ---------------------------------------------------------------------------
// ValueListReference
// ---------------------------------------------------------------------------

/// A reference to a [`ValueList`](crate::ValueList) by its maintenance coordinates.
///
/// ## Specification
/// - **Type**: `ValueListReferenceType`
/// - **Element**: N/A (Reference Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ValueListReferenceType.md"))]
///
/// Identifies a value list by the flat maintainable triple (agency, id, version). A value list is a
/// maintainable artefact, so, like [`CodelistReference`], its reference carries a version.
///
/// # Examples
///
/// ```
/// use sdmx_types::ValueListReference;
///
/// // All fields are public, so you can construct one directly.
/// let reference = ValueListReference {
///     agency: "SDMX".to_string(),
///     id: "VL_CURRENCY".to_string(),
///     version: "1.0.0".parse()?,
/// };
/// let urn = "urn:sdmx:org.sdmx.infomodel.codelist.ValueList=SDMX:VL_CURRENCY(1.0.0)";
/// assert_eq!(reference.to_string(), urn);
/// assert_eq!(urn.parse::<ValueListReference>()?, reference);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ValueListReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The referenced value list's id.
    pub id: String,
    /// The referenced value list's version reference.
    pub version: VersionRef,
}

// ---------------------------------------------------------------------------
// DataflowReference
// ---------------------------------------------------------------------------

/// A reference to a [`Dataflow`](crate::Dataflow) by its maintenance coordinates.
///
/// ## Specification
/// - **Type**: `DataflowReferenceType`
/// - **Element**: N/A (Reference Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataflowReferenceType.md"))]
///
/// Identifies a dataflow by the flat maintainable triple (agency, id, version). A dataflow is a
/// maintainable artefact, so its reference carries a version, like [`DsdReference`]. Used by a data
/// constraint to name a dataflow it is attached to.
///
/// # Examples
///
/// ```
/// use sdmx_types::DataflowReference;
///
/// // All fields are public, so you can construct one directly.
/// let reference = DataflowReference {
///     agency: "SDMX".to_string(),
///     id: "EXR".to_string(),
///     version: "1.0.0".parse()?,
/// };
/// let urn = "urn:sdmx:org.sdmx.infomodel.datastructure.Dataflow=SDMX:EXR(1.0.0)";
/// assert_eq!(reference.to_string(), urn);
/// assert_eq!(urn.parse::<DataflowReference>()?, reference);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataflowReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The referenced dataflow's id.
    pub id: String,
    /// The referenced dataflow's version reference.
    pub version: VersionRef,
}

// ---------------------------------------------------------------------------
// ProvisionAgreementReference
// ---------------------------------------------------------------------------

/// A reference to a provision agreement by its maintenance coordinates.
///
/// ## Specification
/// - **Type**: `ProvisionAgreementReferenceType`
/// - **Element**: N/A (Reference Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/ProvisionAgreementReferenceType.md"))]
///
/// Identifies a provision agreement by the flat maintainable triple (agency, id, version). A
/// provision agreement is a maintainable artefact, so its reference carries a version, like
/// [`DsdReference`]. Used by a data constraint to name a provision agreement it is attached to.
///
/// # Examples
///
/// ```
/// use sdmx_types::ProvisionAgreementReference;
///
/// // All fields are public, so you can construct one directly.
/// let reference = ProvisionAgreementReference {
///     agency: "SDMX".to_string(),
///     id: "PA_EXR".to_string(),
///     version: "1.0.0".parse()?,
/// };
/// let urn = "urn:sdmx:org.sdmx.infomodel.registry.ProvisionAgreement=SDMX:PA_EXR(1.0.0)";
/// assert_eq!(reference.to_string(), urn);
/// assert_eq!(urn.parse::<ProvisionAgreementReference>()?, reference);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProvisionAgreementReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The referenced provision agreement's id.
    pub id: String,
    /// The referenced provision agreement's version reference.
    pub version: VersionRef,
}

// ---------------------------------------------------------------------------
// DataProviderReference
// ---------------------------------------------------------------------------

/// A reference to a data provider by its coordinates within a data-provider scheme.
///
/// ## Specification
/// - **Type**: `DataProviderReferenceType`
/// - **Element**: N/A (Reference Type)
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/DataProviderReferenceType.md"))]
///
/// A data provider is an *item in a data-provider scheme*, not a maintainable artefact in its own
/// right, so its reference takes the item-in-scheme shape (agency, scheme id, item id) rather than
/// the flat maintainable triple, like [`ConceptReference`]. The scheme id is the fixed
/// `DATA_PROVIDERS`, stored here verbatim. Used by a data constraint to name the data provider it is
/// attached to.
///
/// # Examples
///
/// ```
/// use sdmx_types::DataProviderReference;
///
/// // All fields are public, so you can construct one directly.
/// let reference = DataProviderReference {
///     agency: "SDMX".to_string(),
///     scheme_id: "DATA_PROVIDERS".to_string(),
///     version: "1.0.0".parse()?,
///     id: "ECB".to_string(),
/// };
/// let urn = "urn:sdmx:org.sdmx.infomodel.base.DataProvider=SDMX:DATA_PROVIDERS(1.0.0).ECB";
/// assert_eq!(reference.to_string(), urn);
/// assert_eq!(urn.parse::<DataProviderReference>()?, reference);
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Item-in-scheme shape (agency, scheme id, item id), mirroring `ConceptReference` exactly, because its
spec type `DataProviderReferenceType` shares `ComponentUrnReferenceType` with `ConceptReferenceType`
(D-0034). The fixed `DATA_PROVIDERS` scheme id is stored in `scheme_id`; the scheme version is
carried, as the item-in-scheme URN mandates (D-0073). The data provider itself, and its scheme,
are not modelled in this crate (only the reference is), exactly as `DsdReference` predates any
cross-crate resolver.

Decisions: D-0034, D-0020, D-0021.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataProviderReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The id of the data-provider scheme (the fixed `DATA_PROVIDERS`).
    pub scheme_id: String,
    /// The enclosing data-provider scheme's version reference.
    pub version: VersionRef,
    /// The referenced data provider's id.
    pub id: String,
}

// ---------------------------------------------------------------------------
// URN contract (Display / FromStr): each reference renders and parses its own class URN.
// ---------------------------------------------------------------------------

impl core::fmt::Display for DsdReference {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{URN_PREFIX}datastructure.DataStructure={}:{}({})",
            self.agency, self.id, self.version
        )
    }
}

impl core::str::FromStr for DsdReference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (agency, id, version) = parse_triple(s, "datastructure.DataStructure")?;
        Ok(Self { agency, id, version })
    }
}

impl core::fmt::Display for CodelistReference {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{URN_PREFIX}codelist.Codelist={}:{}({})", self.agency, self.id, self.version)
    }
}

impl core::str::FromStr for CodelistReference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (agency, id, version) = parse_triple(s, "codelist.Codelist")?;
        Ok(Self { agency, id, version })
    }
}

impl core::fmt::Display for ValueListReference {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{URN_PREFIX}codelist.ValueList={}:{}({})", self.agency, self.id, self.version)
    }
}

impl core::str::FromStr for ValueListReference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (agency, id, version) = parse_triple(s, "codelist.ValueList")?;
        Ok(Self { agency, id, version })
    }
}

impl core::fmt::Display for DataflowReference {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{URN_PREFIX}datastructure.Dataflow={}:{}({})",
            self.agency, self.id, self.version
        )
    }
}

impl core::str::FromStr for DataflowReference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (agency, id, version) = parse_triple(s, "datastructure.Dataflow")?;
        Ok(Self { agency, id, version })
    }
}

impl core::fmt::Display for ProvisionAgreementReference {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{URN_PREFIX}registry.ProvisionAgreement={}:{}({})",
            self.agency, self.id, self.version
        )
    }
}

impl core::str::FromStr for ProvisionAgreementReference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (agency, id, version) = parse_triple(s, "registry.ProvisionAgreement")?;
        Ok(Self { agency, id, version })
    }
}

impl core::fmt::Display for ConceptReference {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{URN_PREFIX}conceptscheme.Concept={}:{}({}).{}",
            self.agency, self.scheme_id, self.version, self.id
        )
    }
}

impl core::str::FromStr for ConceptReference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (agency, scheme_id, version, id) = parse_item(s, "conceptscheme.Concept")?;
        Ok(Self { agency, scheme_id, version, id })
    }
}

impl core::fmt::Display for DataProviderReference {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{URN_PREFIX}base.DataProvider={}:{}({}).{}",
            self.agency, self.scheme_id, self.version, self.id
        )
    }
}

impl core::str::FromStr for DataProviderReference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (agency, scheme_id, version, id) = parse_item(s, "base.DataProvider")?;
        Ok(Self { agency, scheme_id, version, id })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn constraint_attachment_references_round_trip_through_serde() {
        let dataflow = DataflowReference {
            agency: "ECB".into(),
            id: "EXR".into(),
            version: "1.0.0".parse().unwrap(),
        };
        let agreement = ProvisionAgreementReference {
            agency: "ECB".into(),
            id: "PA_EXR".into(),
            version: "1.0.0".parse().unwrap(),
        };
        let provider = DataProviderReference {
            agency: "SDMX".into(),
            scheme_id: "DATA_PROVIDERS".into(),
            version: "1.0.0".parse().unwrap(),
            id: "ECB".into(),
        };
        crate::test_support::round_trip(&dataflow);
        crate::test_support::round_trip(&agreement);
        crate::test_support::round_trip(&provider);
    }

    #[test]
    fn urn_round_trips_every_reference_class() {
        use alloc::string::ToString;
        // Each class URN parses to its type and renders back verbatim, wildcard
        // versions and nested agencies included.
        let urns = [
            "urn:sdmx:org.sdmx.infomodel.datastructure.DataStructure=SDMX:ECB_EXR1(1.0.0)",
            "urn:sdmx:org.sdmx.infomodel.codelist.Codelist=SDMX.SUB:CL_FREQ(2+.3.1)",
            "urn:sdmx:org.sdmx.infomodel.codelist.ValueList=SDMX:VL_CURRENCY(1.0)",
            "urn:sdmx:org.sdmx.infomodel.datastructure.Dataflow=ECB:EXR(1.0.0-draft)",
            "urn:sdmx:org.sdmx.infomodel.registry.ProvisionAgreement=ECB:PA_EXR(1)",
            "urn:sdmx:org.sdmx.infomodel.conceptscheme.Concept=SDMX:CS_FREQ(2.3.1+).FREQ",
            "urn:sdmx:org.sdmx.infomodel.base.DataProvider=SDMX:DATA_PROVIDERS(1.0.0).ECB",
        ];
        assert_eq!(urns[0].parse::<DsdReference>().unwrap().to_string(), urns[0]);
        assert_eq!(urns[1].parse::<CodelistReference>().unwrap().to_string(), urns[1]);
        assert_eq!(urns[2].parse::<ValueListReference>().unwrap().to_string(), urns[2]);
        assert_eq!(urns[3].parse::<DataflowReference>().unwrap().to_string(), urns[3]);
        assert_eq!(urns[4].parse::<ProvisionAgreementReference>().unwrap().to_string(), urns[4]);
        assert_eq!(urns[5].parse::<ConceptReference>().unwrap().to_string(), urns[5]);
        assert_eq!(urns[6].parse::<DataProviderReference>().unwrap().to_string(), urns[6]);
    }

    #[test]
    fn urn_round_trip_holds_only_for_grammar_valid_fields() {
        // Display renders the fields verbatim (D-0073), so the round-trip is guaranteed only
        // when they are grammar-valid. An off-grammar agency renders but does not parse back.
        let off_grammar_agency = CodelistReference {
            agency: "SDMX AGENCY".into(), // space is outside the agency NCName grammar
            id: "CL_FREQ".into(),
            version: "1.0.0".parse().unwrap(),
        };
        assert!(off_grammar_agency.to_string().parse::<CodelistReference>().is_err());
        // `VersionRef::Any` is the design's own lint value: no reference class admits the bare
        // `*` version (D-0073), so it renders (`(*)`) but FromStr rejects it.
        let any_version = CodelistReference {
            agency: "SDMX".into(),
            id: "CL_FREQ".into(),
            version: VersionRef::Any,
        };
        assert!(any_version.to_string().parse::<CodelistReference>().is_err());
    }

    #[test]
    fn urn_parse_is_class_exact() {
        // A well-formed URN of the wrong class is rejected with the expected class named.
        let codelist = "urn:sdmx:org.sdmx.infomodel.codelist.Codelist=SDMX:CL_FREQ(1.0.0)";
        let err = codelist.parse::<DsdReference>().unwrap_err();
        assert!(matches!(
            err,
            crate::Error::InvalidReferenceUrn { class: "datastructure.DataStructure", .. }
        ));
    }

    #[test]
    fn urn_parse_splits_the_decomposed_fields() {
        let concept = "urn:sdmx:org.sdmx.infomodel.conceptscheme.Concept=SDMX:CS_FREQ(1.0.0).FREQ"
            .parse::<ConceptReference>()
            .unwrap();
        assert_eq!(
            (concept.agency.as_str(), concept.scheme_id.as_str(), concept.id.as_str()),
            ("SDMX", "CS_FREQ", "FREQ")
        );
        assert_eq!(concept.version, "1.0.0".parse().unwrap());
        // A nested item tail is wire-legal and held verbatim.
        let nested = "urn:sdmx:org.sdmx.infomodel.conceptscheme.Concept=SDMX:CS_X(1.0.0).A.B"
            .parse::<ConceptReference>()
            .unwrap();
        assert_eq!(nested.id, "A.B");
    }

    #[test]
    fn urn_parse_rejects_malformed() {
        for bad in [
            "",
            "CL_FREQ",
            "SDMX:CL_FREQ(1.0.0)", // bare, no URN prefix
            "urn:sdmx:org.sdmx.infomodel.codelist.Codelist=SDMX:CL_FREQ", // no version part
            "urn:sdmx:org.sdmx.infomodel.codelist.Codelist=SDMX:CL_FREQ()", // empty version
            "urn:sdmx:org.sdmx.infomodel.codelist.Codelist=SDMX:CL_FREQ(*)", // * not admitted here
            "urn:sdmx:org.sdmx.infomodel.codelist.Codelist=SDMX:CL_FREQ(1.*)", // query grammar
            "urn:sdmx:org.sdmx.infomodel.codelist.Codelist=:CL_FREQ(1.0.0)", // empty agency
            "urn:sdmx:org.sdmx.infomodel.codelist.Codelist=1SDMX:CL_FREQ(1.0.0)", // agency NCName
            "urn:sdmx:org.sdmx.infomodel.codelist.Codelist=SDMX:CL FREQ(1.0.0)", // id charset
            "urn:sdmx:org.sdmx.infomodel.codelist.Codelist=SDMX:CL_FREQ(1.0.0).Y", // item on a triple
        ] {
            assert!(bad.parse::<CodelistReference>().is_err(), "{bad:?} should be rejected");
        }
        // The item shape requires the item tail.
        assert!(
            "urn:sdmx:org.sdmx.infomodel.conceptscheme.Concept=SDMX:CS_FREQ(1.0.0)"
                .parse::<ConceptReference>()
                .is_err()
        );
        assert!(
            "urn:sdmx:org.sdmx.infomodel.conceptscheme.Concept=SDMX:CS_FREQ(1.0.0)."
                .parse::<ConceptReference>()
                .is_err()
        );
    }

    // Property tests: the URN contract over generated grammar-valid components. Every
    // reference class renders its URN and parses it back to an equal value (Display and
    // FromStr as mutual inverses, D-0073); the version part is generated over the full
    // reference grammar (exact and `+`-wildcarded, never the bare `*`). Complements the
    // example tests above; wasm32 is excluded with the rest of the property suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use alloc::{format, string::ToString};

        use proptest::prelude::*;

        use super::super::*;
        use crate::{
            lexical::VersionRef,
            test_strategy::{reference_version_lexeme, urn_agency, urn_id, urn_item_path},
        };

        proptest! {
            #[test]
            fn triple_reference_urns_round_trip(
                agency in urn_agency(),
                id in urn_id(),
                version in reference_version_lexeme(),
            ) {
                let version: VersionRef = version.parse().unwrap();
                let dsd = DsdReference {
                    agency: agency.clone(),
                    id: id.clone(),
                    version: version.clone(),
                };
                prop_assert_eq!(dsd.to_string().parse::<DsdReference>().unwrap(), dsd);
                let codelist = CodelistReference {
                    agency: agency.clone(),
                    id: id.clone(),
                    version: version.clone(),
                };
                prop_assert_eq!(codelist.to_string().parse::<CodelistReference>().unwrap(), codelist);
                let value_list = ValueListReference {
                    agency: agency.clone(),
                    id: id.clone(),
                    version: version.clone(),
                };
                prop_assert_eq!(
                    value_list.to_string().parse::<ValueListReference>().unwrap(),
                    value_list
                );
                let dataflow = DataflowReference {
                    agency: agency.clone(),
                    id: id.clone(),
                    version: version.clone(),
                };
                prop_assert_eq!(dataflow.to_string().parse::<DataflowReference>().unwrap(), dataflow);
                let agreement = ProvisionAgreementReference { agency, id, version };
                prop_assert_eq!(
                    agreement.to_string().parse::<ProvisionAgreementReference>().unwrap(),
                    agreement
                );
            }

            #[test]
            fn mutated_urns_are_rejected(
                agency in urn_agency(),
                id in urn_id(),
                version in reference_version_lexeme(),
                item in urn_item_path(),
            ) {
                // Each case mutates one component of an otherwise-valid URN (D-0073):
                // the wrong class token for the parsing type, the bare `*` version no
                // structural reference admits, an item tail on a maintainable triple, a
                // dropped item tail on an item-in-scheme class, and an off-grammar agency.
                let codelist =
                    format!("urn:sdmx:org.sdmx.infomodel.codelist.Codelist={agency}:{id}({version})");
                prop_assert!(codelist.parse::<DsdReference>().is_err());
                let any_version =
                    format!("urn:sdmx:org.sdmx.infomodel.codelist.Codelist={agency}:{id}(*)");
                prop_assert!(any_version.parse::<CodelistReference>().is_err());
                let item_on_triple = format!(
                    "urn:sdmx:org.sdmx.infomodel.codelist.Codelist={agency}:{id}({version}).{item}"
                );
                prop_assert!(item_on_triple.parse::<CodelistReference>().is_err());
                let dropped_item = format!(
                    "urn:sdmx:org.sdmx.infomodel.conceptscheme.Concept={agency}:{id}({version})"
                );
                prop_assert!(dropped_item.parse::<ConceptReference>().is_err());
                let bad_agency = format!(
                    "urn:sdmx:org.sdmx.infomodel.codelist.Codelist={agency} x:{id}({version})"
                );
                prop_assert!(bad_agency.parse::<CodelistReference>().is_err());
            }

            #[test]
            fn item_reference_urns_round_trip(
                agency in urn_agency(),
                scheme_id in urn_id(),
                version in reference_version_lexeme(),
                id in urn_item_path(),
            ) {
                let version: VersionRef = version.parse().unwrap();
                let concept = ConceptReference {
                    agency: agency.clone(),
                    scheme_id: scheme_id.clone(),
                    version: version.clone(),
                    id: id.clone(),
                };
                prop_assert_eq!(concept.to_string().parse::<ConceptReference>().unwrap(), concept);
                let provider = DataProviderReference { agency, scheme_id, version, id };
                prop_assert_eq!(
                    provider.to_string().parse::<DataProviderReference>().unwrap(),
                    provider
                );
            }
        }
    }
}
