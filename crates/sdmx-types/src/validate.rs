//! Hand-rolled SDMX identifier validators.
//!
//! SDMX uses three identifier lexical tiers, and the generic / `Code` id is the *loosest*:
//! validating every id as `NCName` would wrongly reject schema-valid SDMX (a code id of `1`,
//! `EUR$`, or `@INTERNAL` is a legal `IDType` but not an `NCName`). The validators are hand-rolled
//! rather than regex-backed to stay `no_std` with no extra dependency; each mirrors the exact
//! `xs:pattern` from `SDMXCommonReferences.xsd`.
//!
//! Only the two tiers with a Milestone 1 caller live here: `validate_id`
//! ([`IdentifiableMetadata`](crate::IdentifiableMetadata)) and `validate_nested_ncname`
//! ([`MaintainableMetadata`](crate::MaintainableMetadata)). The `NCNameIDType` tier
//! (`validate_ncname`) and the fixed-value check (`validate_fixed`) join when their first callers,
//! the scheme items and wrappers, land in a later milestone.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Three identifier tiers, each hand-rolled against the exact `xs:pattern` rather than a shared regex,
to stay `no_std` with no extra dependency.

Decisions: D-0023.
"#
)]

use alloc::string::ToString;

use crate::error::Error;

/// True iff `c` is a member of the SDMX `IDType` character class
/// (`A-Z`, `a-z`, `0-9`, `_`, `@`, `$`, `-`).
const fn is_id_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '@' | '$' | '-')
}

/// True iff `s` is a single `NCNameIDType` segment: a leading ASCII letter followed by
/// ASCII letters, digits, `_`, or `-` (XSD pattern `[A-Za-z][A-Za-z0-9_\-]*`). An empty
/// segment is not an `NCName`, so this also rejects leading, trailing, and doubled dots
/// when used per dot-delimited segment.
fn is_ncname_segment(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() => {
            chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
        }
        _ => false,
    }
}

/// Validates an identifier against SDMX `IDType` (`[A-Za-z0-9_@$\-]+`), the loosest tier
/// shared by every identifiable artefact.
///
/// # Errors
///
/// Returns [`Error::InvalidIdentifier`] if `id` is empty or contains a character outside
/// the `IDType` class (the `.` of a dotted id, for instance, is not in the class).
pub fn validate_id(id: &str) -> Result<(), Error> {
    if !id.is_empty() && id.chars().all(is_id_char) {
        Ok(())
    } else {
        Err(Error::InvalidIdentifier(id.to_string()))
    }
}

/// Validates an identifier against SDMX `NestedNCNameIDType`
/// (`[A-Za-z][A-Za-z0-9_\-]*(\.[A-Za-z][A-Za-z0-9_\-]*)*`), a dot-delimited sequence of
/// one or more `NCName` segments. This is the `agencyID` tier.
///
/// # Errors
///
/// Returns [`Error::InvalidAgencyIdentifier`] if any dot-delimited segment is not a valid
/// `NCName` segment (which also rejects an empty string and leading, trailing, or doubled dots).
pub fn validate_nested_ncname(id: &str) -> Result<(), Error> {
    if id.split('.').all(is_ncname_segment) {
        Ok(())
    } else {
        Err(Error::InvalidAgencyIdentifier(id.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn id_accepts_full_idtype_class() {
        // IDType is the loosest tier: leading digits, @, $, _, - are all valid.
        for ok in ["EUR", "1", "EUR$", "@INTERNAL", "A_B-C", "a1@b$c_d"] {
            assert!(validate_id(ok).is_ok(), "expected {ok:?} to be a valid IDType");
        }
    }

    #[test]
    fn id_rejects_empty_and_out_of_class() {
        // Empty (the `+` quantifier requires ≥1 char), whitespace, and the dot
        // (reserved for NestedNCName) are all outside IDType.
        for bad in ["", "a b", "a.b", "naïve", "a/b"] {
            assert_eq!(validate_id(bad), Err(Error::InvalidIdentifier(bad.into())));
        }
    }

    #[test]
    fn nested_ncname_accepts_single_and_dotted() {
        for ok in ["SDMX", "ESTAT", "ORG.DEPT", "A.B.C", "A_1-2.B3"] {
            assert!(
                validate_nested_ncname(ok).is_ok(),
                "expected {ok:?} to be a valid NestedNCNameIDType"
            );
        }
    }

    #[test]
    fn nested_ncname_rejects_bad_segments_and_dots() {
        // Leading digit, empty, and any leading/trailing/doubled dot are rejected.
        for bad in ["1ABC", "", "A.", ".A", "A..B", "A.1B", "A B"] {
            assert_eq!(
                validate_nested_ncname(bad),
                Err(Error::InvalidAgencyIdentifier(bad.into()))
            );
        }
    }
}
