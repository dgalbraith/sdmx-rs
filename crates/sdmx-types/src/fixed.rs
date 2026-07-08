//! The [`FixedInclude`] within-field wrapper.
//!
//! The `include` attribute on `DataKey`/`DataKeyValue` is declared `use="optional" fixed="true"`,
//! the only `fixed="true"` attribute in SDMX 3.0 and 3.1: its sole schema-valid stated value is
//! `true`, but it may also be omitted. [`FixedInclude`] stores that statedness exactly (`None`
//! when omitted, `Some(true)` when stated) while making a stated `false`, a mechanical schema
//! mismatch an XSD validator would reject, unconstructible. The general fixed-value case (a
//! constructor that has the attribute name in context, such as the `AGENCIES` scheme id) uses the
//! `validate_fixed` helper instead.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The same §7 category as the lexical newtypes: a within-field invariant, so the wire path routes
through the validated constructor via a custom `Deserialize`, and its containers stay derived
pub-field carriers.

Named for its sole producer: `include` is the only `fixed="true"` attribute in SDMX 3.0/3.1, so
this is a purpose-built wrapper rather than a generic `fixed="true"` primitive, and the
`FixedAttributeMismatch` site is hard-coded to `"include"` accordingly. The general fixed-value
check is `validate_fixed`, which takes the attribute name because its callers have constructor
context (the serde field path does not). A second `fixed="true"` attribute would be a future
spec-major event, handled then.

Decisions: D-0039, D-0052.
"#
)]

use alloc::string::String;

use crate::error::{Error, to_de_error};

/// The statedness of the `fixed="true"` `include` attribute: `None` (omitted) or `Some(true)`
/// (stated).
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps the `optional`, `fixed="true"` `include` attribute on `DataKey`/`DataKeyValue`, preserving
/// whether it was stated while rejecting the schema-invalid stated `false`. It is the only
/// `fixed="true"` attribute in the standard, so the wrapper is purpose-built for it.
///
/// ## Guarantees
///
/// A constructed value never holds `Some(false)`, so [`effective_is_included`](Self::effective_is_included) is always
/// `true`.
///
/// # Examples
///
/// ```
/// use sdmx_types::FixedInclude;
///
/// let omitted = FixedInclude::new(None)?;
/// assert_eq!(omitted.stated(), None);
/// assert!(omitted.effective_is_included());
///
/// // A stated `false` contradicts `fixed="true"` and is rejected.
/// assert!(FixedInclude::new(Some(false)).is_err());
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct FixedInclude(Option<bool>);

impl FixedInclude {
    /// Wraps the stated value of the `fixed="true"` `include` attribute.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FixedAttributeMismatch`] if `stated` is `Some(false)`: a stated value
    /// that contradicts the schema-fixed `true`.
    pub fn new(stated: Option<bool>) -> Result<Self, Error> {
        if stated == Some(false) {
            return Err(Error::FixedAttributeMismatch {
                attribute: String::from("include"),
                value: String::from("false"),
            });
        }
        Ok(Self(stated))
    }

    /// The statedness exactly as the wire carried it (`None` when the attribute was omitted).
    #[must_use]
    pub const fn stated(&self) -> Option<bool> {
        self.0
    }

    /// The effective value, which is always the fixed value `true`.
    #[must_use]
    pub const fn effective_is_included(&self) -> bool {
        true
    }
}

impl<'de> serde::Deserialize<'de> for FixedInclude {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let stated = Option::<bool>::deserialize(deserializer)?;
        Self::new(stated).map_err(to_de_error)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn accepts_omitted_and_stated_true() {
        assert_eq!(FixedInclude::new(None).unwrap().stated(), None);
        assert_eq!(FixedInclude::new(Some(true)).unwrap().stated(), Some(true));
    }

    #[test]
    fn rejects_stated_false() {
        assert_eq!(
            FixedInclude::new(Some(false)),
            Err(Error::FixedAttributeMismatch {
                attribute: String::from("include"),
                value: String::from("false")
            })
        );
    }

    #[test]
    fn effective_is_always_true() {
        assert!(FixedInclude::new(None).unwrap().effective_is_included());
        assert!(FixedInclude::new(Some(true)).unwrap().effective_is_included());
    }

    #[test]
    fn deserialize_routes_through_new() {
        // FixedInclude is serde(transparent) over Option<bool>, so encoding the inner
        // value and decoding as FixedInclude routes through new().
        let omitted = postcard::to_allocvec(&None::<bool>).unwrap();
        assert_eq!(postcard::from_bytes::<FixedInclude>(&omitted).unwrap().stated(), None);
        let stated_true = postcard::to_allocvec(&Some(true)).unwrap();
        assert_eq!(
            postcard::from_bytes::<FixedInclude>(&stated_true).unwrap().stated(),
            Some(true)
        );
        // A stated `false` contradicts `fixed="true"` and is rejected on the wire path too.
        let stated_false = postcard::to_allocvec(&Some(false)).unwrap();
        assert!(postcard::from_bytes::<FixedInclude>(&stated_false).is_err());
    }

    #[test]
    fn serialize_preserves_statedness() {
        // Transparent projection: a FixedInclude serialises exactly as its inner Option<bool>.
        assert_eq!(
            postcard::to_allocvec(&FixedInclude::new(None).unwrap()).unwrap(),
            postcard::to_allocvec(&None::<bool>).unwrap()
        );
        assert_eq!(
            postcard::to_allocvec(&FixedInclude::new(Some(true)).unwrap()).unwrap(),
            postcard::to_allocvec(&Some(true)).unwrap()
        );
    }

    // Property tests: the internal serde round-trip over generated values (see
    // `test_strategy`); wasm32 is excluded with the rest of the property suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::fixed_include;

        proptest! {
            #[test]
            fn fixed_include_round_trips(value in fixed_include()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
