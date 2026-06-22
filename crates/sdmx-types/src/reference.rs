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
//! id in place of a version.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Invariant-free pub-field carriers with derived `Serialize`/`Deserialize`: a reference self-validates
structurally (its parse contract is the scheduled Phase-2 URN work), so there is no construction
invariant. `Hash` originated on these reference/identity types as the natural map keys; D-0065 later
generalised it to every float-free value type, so the references are no longer distinctive in
deriving it.

One struct per spec reference type rather than a unified `MaintainableReference`: each maps 1-to-1 to
a distinct concept in the information model, and the item-in-scheme references already diverge from
the flat maintainable triple, so the correspondence is kept. Each reference lands with its first
caller; the constraint-attachment references (dataflow, provision agreement, data provider) join
with the constraint types that consume them.

The three constraint-attachment references take the shapes D-0034 fixes: `DataflowReference` and
`ProvisionAgreementReference` are flat maintainable triples (a dataflow and a provision agreement are
both maintainable), while `DataProviderReference` takes the item-in-scheme shape (agency, scheme id,
item id), because its spec type shares `ComponentUrnReferenceType` with `ConceptReferenceType`: a
data provider is an item in a data-provider scheme, not a maintainable in its own right. The
URN-string-versus-decomposed-fields question and the `version: String` to `SdmxVersion` tightening
are the deferred Phase-2 reference-types pass (D-0002), uniform across all the reference structs.

Decisions: D-0020, D-0021, D-0034, D-0047, D-0048, D-0065.
"#
)]

use alloc::string::String;

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
///     version: "1.0.0".to_string(),
/// };
/// assert_eq!(reference.id, "ECB_EXR1");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DsdReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The referenced data structure definition's id.
    pub id: String,
    /// The referenced data structure definition's version.
    pub version: String,
}

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
/// maintainable triple carried by [`CodelistReference`] and [`DsdReference`]: the version belongs to
/// the enclosing scheme, so it is not repeated here.
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
///     id: "FREQ".to_string(),
/// };
/// assert_eq!(reference.scheme_id, "CS_FREQ");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ConceptReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The id of the concept scheme the concept belongs to (`maintainableParentID`).
    pub scheme_id: String,
    /// The referenced concept's id.
    pub id: String,
}

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
///     version: "1.0.0".to_string(),
/// };
/// assert_eq!(reference.id, "CL_FREQ");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CodelistReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The referenced codelist's id.
    pub id: String,
    /// The referenced codelist's version.
    pub version: String,
}

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
///     version: "1.0.0".to_string(),
/// };
/// assert_eq!(reference.agency, "SDMX");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ValueListReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The referenced value list's id.
    pub id: String,
    /// The referenced value list's version.
    pub version: String,
}

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
///     version: "1.0.0".to_string(),
/// };
/// assert_eq!(reference.id, "EXR");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataflowReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The referenced dataflow's id.
    pub id: String,
    /// The referenced dataflow's version.
    pub version: String,
}

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
///     version: "1.0.0".to_string(),
/// };
/// assert_eq!(reference.id, "PA_EXR");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProvisionAgreementReference {
    /// The maintenance agency id (`agencyID`).
    pub agency: String,
    /// The referenced provision agreement's id.
    pub id: String,
    /// The referenced provision agreement's version.
    pub version: String,
}

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
///     id: "ECB".to_string(),
/// };
/// assert_eq!(reference.id, "ECB");
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Item-in-scheme shape (agency, scheme id, item id), mirroring `ConceptReference` exactly, because its
spec type `DataProviderReferenceType` shares `ComponentUrnReferenceType` with `ConceptReferenceType`
(D-0034). The fixed `DATA_PROVIDERS` scheme id is stored in `scheme_id`; the scheme version is
dropped, as `ConceptReference` drops its enclosing version. The data provider itself, and its scheme,
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
    /// The referenced data provider's id.
    pub id: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn constraint_attachment_references_round_trip_through_serde() {
        let dataflow =
            DataflowReference { agency: "ECB".into(), id: "EXR".into(), version: "1.0.0".into() };
        let agreement = ProvisionAgreementReference {
            agency: "ECB".into(),
            id: "PA_EXR".into(),
            version: "1.0.0".into(),
        };
        let provider = DataProviderReference {
            agency: "SDMX".into(),
            scheme_id: "DATA_PROVIDERS".into(),
            id: "ECB".into(),
        };
        let dataflow_json = serde_json::to_string(&dataflow).unwrap();
        assert_eq!(serde_json::from_str::<DataflowReference>(&dataflow_json).unwrap(), dataflow);
        let agreement_json = serde_json::to_string(&agreement).unwrap();
        assert_eq!(
            serde_json::from_str::<ProvisionAgreementReference>(&agreement_json).unwrap(),
            agreement
        );
        let provider_json = serde_json::to_string(&provider).unwrap();
        assert_eq!(
            serde_json::from_str::<DataProviderReference>(&provider_json).unwrap(),
            provider
        );
    }
}
