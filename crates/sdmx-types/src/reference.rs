//! Cross-references to other SDMX artefacts.
//!
//! A reference names another artefact by its coordinates rather than embedding it. These are the
//! natural map keys of the model (deduping a set of fetched artefacts, "have I already resolved
//! this codelist?"), so they derive [`Hash`]; their fields are all `String`, so it is free.
//!
//! Only the references a Milestone 2 type consumes live here so far: [`CodelistReference`] (the
//! target of a codelist extension) and [`ValueListReference`] (admitted at the enumeration
//! positions). The remaining reference structs join with their first callers.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Invariant-free pub-field carriers with derived `Serialize`/`Deserialize`: a reference self-validates
structurally (its parse contract is the scheduled Phase-2 URN work), so there is no construction
invariant. `Hash` is scoped deliberately to the reference/identity types, the natural map keys, not
applied blanket to the composite artefacts.

One struct per spec reference type rather than a unified `MaintainableReference`: each maps 1-to-1 to
a distinct concept in the information model, and the item-in-scheme references already diverge from
the flat maintainable triple, so the correspondence is kept.

Decisions: D-0020, D-0021, D-0047, D-0048.
"#
)]

use alloc::string::String;

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
