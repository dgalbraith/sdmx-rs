//! Hand-rolled SDMX identifier validators.
//!
//! SDMX uses three identifier lexical tiers, and the generic / `Code` id is the *loosest*:
//! validating every id as `NCName` would wrongly reject schema-valid SDMX (a code id of `1`,
//! `EUR$`, or `@INTERNAL` is a legal `IDType` but not an `NCName`). The validators are hand-rolled
//! rather than regex-backed to stay `no_std` with no extra dependency; each mirrors the exact
//! `xs:pattern` from `SDMXCommonReferences.xsd`.
//!
//! All three tiers now have callers: `validate_id` ([`IdentifiableMetadata`](crate::IdentifiableMetadata)),
//! `validate_ncname` (the scheme items and wrappers whose ids the spec types as `NCNameIDType`:
//! [`Concept`](crate::Concept), [`Codelist`](crate::Codelist), [`ConceptScheme`](crate::ConceptScheme)),
//! and `validate_nested_ncname` ([`MaintainableMetadata`](crate::MaintainableMetadata)). The
//! fixed-value check `validate_fixed` arrives alongside them, owned by
//! [`AgencyScheme`](crate::AgencyScheme) (the `fixed="AGENCIES"` scheme id).
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

/// Validates an identifier against SDMX `NCNameIDType` (`[A-Za-z][A-Za-z0-9_\-]*`), the middle
/// tier: a single `NCName` (no dots). Stricter than `IDType` (a leading digit, `@`, `$`, or `.`
/// are rejected here), looser than `NestedNCNameIDType` (which permits dot-delimited segments).
///
/// # Errors
///
/// Returns [`Error::InvalidNcNameIdentifier`] if `id` is not a single valid `NCName` segment
/// (which also rejects an empty string and any embedded dot).
pub fn validate_ncname(id: &str) -> Result<(), Error> {
    if is_ncname_segment(id) { Ok(()) } else { Err(Error::InvalidNcNameIdentifier(id.to_string())) }
}

/// Checks a stated value against an XSD `fixed` value. A value that is stated and differs from
/// the schema-fixed one is mechanically schema-invalid (an XSD validator would itself reject the
/// mismatch); an absent value is always accepted (the attribute may be omitted, taking the fixed
/// value by default).
///
/// `attribute` names the site for the diagnostic (the convention shared with
/// [`FixedInclude::new`](crate::FixedInclude::new), which reports `"include"`); pass the attribute name,
/// for example `"id"`.
///
/// # Errors
///
/// Returns [`Error::FixedAttributeMismatch`] if `stated` is `Some(v)` with `v != expected`.
pub fn validate_fixed(attribute: &str, stated: Option<&str>, expected: &str) -> Result<(), Error> {
    match stated {
        Some(value) if value != expected => Err(Error::FixedAttributeMismatch {
            attribute: attribute.to_string(),
            value: value.to_string(),
        }),
        _ => Ok(()),
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
    use alloc::string::String;

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
    fn ncname_accepts_single_segment_only() {
        // NCNameIDType is a single NCName: a leading letter, then letters/digits/_/-.
        for ok in ["SDMX", "ESTAT", "A_1-2", "Concept"] {
            assert!(validate_ncname(ok).is_ok(), "expected {ok:?} to be a valid NCNameIDType");
        }
        // Stricter than IDType (leading digit, @, $) and looser-tier dots are all rejected.
        for bad in ["", "1ABC", "@X", "EUR$", "A.B", "a b"] {
            assert_eq!(validate_ncname(bad), Err(Error::InvalidNcNameIdentifier(bad.into())));
        }
    }

    #[test]
    fn fixed_accepts_absent_and_matching_rejects_mismatch() {
        // Absent ⟹ takes the fixed value by default; a matching stated value is fine.
        assert!(validate_fixed("id", None, "AGENCIES").is_ok());
        assert!(validate_fixed("id", Some("AGENCIES"), "AGENCIES").is_ok());
        // A stated value differing from the fixed one is a mechanical mismatch.
        assert_eq!(
            validate_fixed("id", Some("OTHER"), "AGENCIES"),
            Err(Error::FixedAttributeMismatch {
                attribute: String::from("id"),
                value: String::from("OTHER")
            })
        );
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

    // Property tests: rejection breadth over the tractable off-grammar families (see the
    // `invalid_*` strategies in `test_strategy`); the precise boundary stays with the
    // example tests above. wasm32 is excluded with the rest of the property suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use super::super::*;
        use crate::test_strategy::{
            invalid_id_lexeme, invalid_ncname_lexeme, invalid_nested_ncname_lexeme,
        };

        proptest! {
            #[test]
            fn id_rejects_off_grammar(candidate in invalid_id_lexeme()) {
                prop_assert!(validate_id(&candidate).is_err());
            }

            #[test]
            fn ncname_rejects_off_grammar(candidate in invalid_ncname_lexeme()) {
                prop_assert!(validate_ncname(&candidate).is_err());
            }

            #[test]
            fn nested_ncname_rejects_off_grammar(candidate in invalid_nested_ncname_lexeme()) {
                prop_assert!(validate_nested_ncname(&candidate).is_err());
            }
        }
    }
}
