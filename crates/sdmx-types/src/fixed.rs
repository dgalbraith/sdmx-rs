//! The [`FixedTrue`] within-field wrapper.
//!
//! Some SDMX attributes are declared `use="optional" fixed="true"`: the only schema-valid stated
//! value is `true`, but the attribute may also be omitted. [`FixedTrue`] stores that statedness
//! faithfully (`None` when omitted, `Some(true)` when stated) while making a stated `false`, a
//! mechanical schema mismatch an XSD validator would reject, unconstructible.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

The same §7 category as the lexical newtypes: a within-field invariant, so the wire path routes
through the validated constructor via a custom `Deserialize`, and its containers stay derived
pub-field carriers.

Decisions: D-0039, D-0052.
"#
)]

use crate::error::{Error, to_de_error};

/// The statedness of a `fixed="true"` attribute: `None` (omitted) or `Some(true)` (stated).
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// Wraps an `optional`, `fixed="true"` boolean attribute, preserving whether it was stated while
/// rejecting the schema-invalid stated `false`.
///
/// ## Guarantees
///
/// A constructed value never holds `Some(false)`, so [`effective`](Self::effective) is always
/// `true`.
///
/// # Examples
///
/// ```
/// use sdmx_types::FixedTrue;
///
/// let omitted = FixedTrue::new(None)?;
/// assert_eq!(omitted.stated(), None);
/// assert!(omitted.effective());
///
/// // A stated `false` contradicts `fixed="true"` and is rejected.
/// assert!(FixedTrue::new(Some(false)).is_err());
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct FixedTrue(Option<bool>);

impl FixedTrue {
    /// Wraps the stated value of a `fixed="true"` attribute.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FixedAttributeMismatch`] if `stated` is `Some(false)`: a stated value
    /// that contradicts the schema-fixed `true`.
    pub fn new(stated: Option<bool>) -> Result<Self, Error> {
        if stated == Some(false) {
            return Err(Error::FixedAttributeMismatch("include".into(), "false".into()));
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
    pub const fn effective(&self) -> bool {
        true
    }
}

impl<'de> serde::Deserialize<'de> for FixedTrue {
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
        assert_eq!(FixedTrue::new(None).unwrap().stated(), None);
        assert_eq!(FixedTrue::new(Some(true)).unwrap().stated(), Some(true));
    }

    #[test]
    fn rejects_stated_false() {
        assert_eq!(
            FixedTrue::new(Some(false)),
            Err(Error::FixedAttributeMismatch("include".into(), "false".into()))
        );
    }

    #[test]
    fn effective_is_always_true() {
        assert!(FixedTrue::new(None).unwrap().effective());
        assert!(FixedTrue::new(Some(true)).unwrap().effective());
    }

    #[test]
    fn deserialize_routes_through_new() {
        assert_eq!(serde_json::from_str::<FixedTrue>("null").unwrap().stated(), None);
        assert_eq!(serde_json::from_str::<FixedTrue>("true").unwrap().stated(), Some(true));
        // A stated `false` contradicts `fixed="true"` and is rejected on the wire path too.
        assert!(serde_json::from_str::<FixedTrue>("false").is_err());
    }

    #[test]
    fn serialize_preserves_statedness() {
        assert_eq!(serde_json::to_string(&FixedTrue::new(None).unwrap()).unwrap(), "null");
        assert_eq!(serde_json::to_string(&FixedTrue::new(Some(true)).unwrap()).unwrap(), "true");
    }
}
