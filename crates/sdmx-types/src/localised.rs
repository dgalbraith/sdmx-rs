//! Ordered, multilingual text for SDMX names and descriptions.
//!
//! [`LocalisedString`] holds a sequence of `(language, text)` entries in wire order. SDMX names
//! and descriptions are multilingual, and the same language may legitimately appear more than
//! once. The language tag is optional on each entry; lookups apply the SDMX `"en"` default when a
//! tag is absent, so an untagged entry answers to `"en"`.
//!
//! The single construction invariant is that the entry list is non-empty: the parent `Name` and
//! `Description` elements require at least one entry. Keys and values are otherwise stored
//! verbatim, including a blank value or an unusual language tag.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Stored as an ordered `Vec` in wire order, duplicate languages preserved (no `xs:unique` constrains
them). The language key is `Option<String>` because `TextType` declares `xml:lang` with
`default="en"`: an absent tag and a stated `"en"` are distinct documents, so statedness is stored
(Layer 1) and the `"en"` default is applied only as an effective view (Layer 2). A blank value is
schema-valid; a blank or off-pattern stated key, though mechanically invalid, is fully
representable, so its well-formedness is a catalogued Layer-2 lint, not a construction error.

Decisions: D-0016, D-0031, D-0051, D-0059.
"#
)]

use alloc::{string::String, vec::Vec};

use crate::error::{Error, to_de_error};

/// An ordered list of `(language, text)` entries representing SDMX multilingual text.
///
/// ## Specification
/// - **Schema**: N/A (Virtual Type)
/// - **Type**: Rust-specific projection
/// - **Element**: N/A
/// - **Editions**: SDMX 3.0 and 3.1
///
/// This type projects a sequence of SDMX `TextType` elements (distinguished by their `xml:lang`
/// attributes) into a unified, queryable Rust structure.
///
/// ## Guarantees
///
/// Always holds at least one entry, and preserves entry order and duplicate languages exactly as
/// supplied.
///
/// # Examples
///
/// ```
/// use sdmx_types::LocalisedString;
///
/// let names = LocalisedString::new(vec![
///     (Some("en".to_string()), "Currency".to_string()),
///     (Some("fr".to_string()), "Monnaie".to_string()),
/// ])?;
/// assert_eq!(names.get("fr"), Some("Monnaie"));
/// assert_eq!(names.first(), "Currency");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`Vec` storage (not a map) preserves wire order and duplicate languages; `get` is a first-match
view, not a keyed lookup. The non-empty invariant is the sole construction check.

Decisions: D-0051, D-0059.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct LocalisedString(Vec<(Option<String>, String)>);

impl LocalisedString {
    /// Builds a localised string from `(language, text)` entries in wire order.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyLocalisation`] if `entries` is empty. This is the sole structural
    /// invariant: the parent `Name`/`Description` requires at least one entry. Keys and values
    /// are not otherwise inspected; their well-formedness is a catalogued lint, not an error.
    pub fn new(entries: Vec<(Option<String>, String)>) -> Result<Self, Error> {
        if entries.is_empty() {
            return Err(Error::EmptyLocalisation);
        }
        Ok(Self(entries))
    }

    /// The entry's effective language: the stated tag, else the schema default `"en"`
    /// (`TextType` declares `xml:lang` with `default="en"`).
    fn effective_lang(key: Option<&str>) -> &str {
        key.unwrap_or("en")
    }

    /// The value of the first entry whose *effective* language equals `lang`, in order (so
    /// `get("en")` also matches an untagged entry). When a language repeats, the first match
    /// wins; the later entries remain reachable through [`iter`](Self::iter).
    #[must_use]
    pub fn get(&self, lang: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(key, _)| Self::effective_lang(key.as_deref()) == lang)
            .map(|(_, value)| value.as_str())
    }

    /// The first entry's value in wire order: a deterministic fallback, not a locale
    /// preference. Infallible: the non-empty invariant guarantees at least one entry.
    #[must_use]
    pub fn first(&self) -> &str {
        // The empty branch is unreachable: `new` rejects an empty entry list and there is no
        // mutating path, so a constructed `LocalisedString` always has a first entry. The crate is
        // panic-free by contract (workspace clippy `unwrap_used`/`expect_used`), so the unreachable
        // case degrades to "" rather than `expect`-panicking; that default is never observed (and a
        // genuine blank first value is `""` regardless).
        self.0.first().map_or("", |(_, value)| value.as_str())
    }

    /// Yields `(stated_language, value)` pairs in wire order: the key exactly as supplied,
    /// `None` when the `xml:lang` attribute was absent. The language is independent data, so both
    /// halves are exposed (contrast [`languages`](Self::languages), which applies the default).
    pub fn iter(&self) -> impl Iterator<Item = (Option<&str>, &str)> {
        self.0.iter().map(|(key, value)| (key.as_deref(), value.as_str()))
    }

    /// The effective language of each entry in order (the stated tag, else `"en"`). The raw
    /// stated keys remain reachable via [`iter`](Self::iter).
    pub fn languages(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(key, _)| Self::effective_lang(key.as_deref()))
    }

    /// The number of entries (always at least one).
    #[must_use]
    #[allow(clippy::len_without_is_empty)] // invariant: always ≥ 1 entry, so is_empty() is always false
    pub const fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'de> serde::Deserialize<'de> for LocalisedString {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let entries = Vec::<(Option<String>, String)>::deserialize(deserializer)?;
        Self::new(entries).map_err(to_de_error)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::vec;

    use super::*;

    fn sample() -> LocalisedString {
        LocalisedString::new(vec![
            (Some("en".into()), "Name".into()),
            (Some("fr".into()), "Nom".into()),
            (None, "Untagged".into()),
        ])
        .unwrap()
    }

    #[test]
    fn rejects_empty_entry_list() {
        assert_eq!(LocalisedString::new(vec![]), Err(Error::EmptyLocalisation));
    }

    #[test]
    fn get_matches_stated_language() {
        let ls = sample();
        assert_eq!(ls.get("en"), Some("Name"));
        assert_eq!(ls.get("fr"), Some("Nom"));
        assert_eq!(ls.get("de"), None);
    }

    #[test]
    fn tagless_entry_has_effective_en() {
        // A tag-less entry's effective language is "en"; first-match means the stated "en"
        // wins here, but a sole tag-less entry resolves to "en".
        let only_untagged = LocalisedString::new(vec![(None, "Fallback".into())]).unwrap();
        assert_eq!(only_untagged.get("en"), Some("Fallback"));
    }

    #[test]
    fn first_is_wire_first() {
        assert_eq!(sample().first(), "Name");
    }

    #[test]
    fn get_returns_first_match_for_duplicate_effective_language() {
        // Three entries that all resolve to effective "en" (the middle one untagged); first-match
        // in wire order wins, the behaviour the design's duplicate-language handling relies on.
        let ls = LocalisedString::new(vec![
            (Some("en".into()), "First".into()),
            (None, "Second".into()),
            (Some("en".into()), "Third".into()),
        ])
        .unwrap();
        assert_eq!(ls.get("en"), Some("First"));
    }

    #[test]
    fn iter_exposes_raw_keys_languages_applies_default() {
        let ls = sample();
        let raw: Vec<_> = ls.iter().collect();
        assert_eq!(raw, vec![(Some("en"), "Name"), (Some("fr"), "Nom"), (None, "Untagged")]);
        let effective: Vec<_> = ls.languages().collect();
        assert_eq!(effective, vec!["en", "fr", "en"]);
        assert_eq!(ls.len(), 3);
    }

    #[test]
    fn deserialize_round_trips_and_rejects_empty() {
        let json = serde_json::to_string(&sample()).unwrap();
        assert_eq!(serde_json::from_str::<LocalisedString>(&json).unwrap(), sample());
        // An empty entry list is mechanically schema-invalid and rejected on the wire path.
        assert!(serde_json::from_str::<LocalisedString>("[]").is_err());
    }
}
