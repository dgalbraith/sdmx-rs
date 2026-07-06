//! Ordered, multilingual text for SDMX names and descriptions.
//!
//! [`LocalisedString`] holds a sequence of [`LocalisedText`] entries in wire order. SDMX names
//! and descriptions are multilingual, and the same language may legitimately appear more than
//! once. The language tag is optional on each entry; lookups apply the SDMX `"en"` default when a
//! tag is absent, so an untagged entry answers to `"en"`.
//!
//! The single construction invariant is that the entry list is non-empty: the parent `Name` and
//! `Description` elements require at least one entry. Languages and texts are otherwise stored
//! verbatim, including a blank value or an unusual language tag.
#![cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

Stored as an ordered `Vec` of [`LocalisedText`] in wire order, duplicate languages preserved (no
`xs:unique` constrains them). Each entry's `language` is `Option<String>` because `TextType`
declares `xml:lang` with `default="en"`: an absent tag and a stated `"en"` are distinct documents,
so statedness is stored (Layer 1) and the `"en"` default is applied only as an effective view
(Layer 2). A blank value is schema-valid; a blank or off-pattern stated tag, though mechanically
invalid, is fully representable, so its well-formedness is a catalogued Layer-2 lint, not a
construction error.

Decisions: D-0016, D-0031, D-0051, D-0059, D-0066.
"#
)]

use alloc::{string::String, vec::Vec};

use crate::error::{Error, to_de_error};

/// A single language-tagged text entry: one SDMX `TextType`.
///
/// ## Specification
/// - **Schema**: `SDMXCommon.xsd`
/// - **Type**: `TextType`
/// - **Element**: N/A (Reusable Type)
/// - **Editions**: SDMX 3.0 and 3.1
///
/// `TextType` is `xs:string` content carrying an optional `xml:lang` attribute, reused by the
/// `Name`, `Description`, and `Text` elements. This type projects one such entry: [`text`] is the
/// string content and [`language`] is the `xml:lang` tag, absent when the attribute was omitted.
///
/// [`text`]: LocalisedText::text
/// [`language`]: LocalisedText::language
#[cfg_attr(
    design_docs,
    doc = r#"
## Design Notes

`language` is `Option<String>` because `TextType` declares `xml:lang` with `default="en"`: an
absent tag and a stated `"en"` are distinct documents, so statedness is stored (Layer 1) and the
`"en"` default is applied only as an effective view (Layer 2). A blank `text` is schema-valid; a
blank or off-pattern stated tag, though mechanically invalid, is held verbatim and its
well-formedness surfaces as a Layer-2 lint, not a construction error.

Decisions: D-0016, D-0031, D-0059, D-0066.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LocalisedText {
    /// The `xml:lang` tag exactly as supplied, `None` when the attribute was absent. Its effective
    /// value applies the SDMX `"en"` default (see [`LocalisedString::languages`]).
    pub language: Option<String>,
    /// The text content, stored verbatim (a blank value is schema-valid).
    pub text: String,
}

/// An ordered list of [`LocalisedText`] entries representing SDMX multilingual text.
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
/// use sdmx_types::{LocalisedString, LocalisedText};
///
/// let names = LocalisedString::new(vec![
///     LocalisedText { language: Some("en".to_string()), text: "Currency".to_string() },
///     LocalisedText { language: Some("fr".to_string()), text: "Monnaie".to_string() },
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

Decisions: D-0051, D-0059, D-0066.
"#
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct LocalisedString(Vec<LocalisedText>);

impl LocalisedString {
    /// Builds a localised string from [`LocalisedText`] entries in wire order.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyLocalisation`] if `entries` is empty. This is the sole structural
    /// invariant: the parent `Name`/`Description` requires at least one entry. Languages and
    /// texts are not otherwise inspected; their well-formedness is a catalogued lint, not an error.
    pub fn new(entries: Vec<LocalisedText>) -> Result<Self, Error> {
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
            .find(|entry| Self::effective_lang(entry.language.as_deref()) == lang)
            .map(|entry| entry.text.as_str())
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
        self.0.first().map_or("", |entry| entry.text.as_str())
    }

    /// Yields each [`LocalisedText`] entry in wire order, languages and texts exactly as supplied.
    /// The stated language is preserved (`None` when the `xml:lang` attribute was absent); contrast
    /// [`languages`](Self::languages), which applies the `"en"` default.
    pub fn iter(&self) -> impl Iterator<Item = &LocalisedText> {
        self.0.iter()
    }

    /// The entries as a slice, in wire order.
    #[must_use]
    pub fn as_slice(&self) -> &[LocalisedText] {
        &self.0
    }

    /// Consumes the newtype, returning the inner vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<LocalisedText> {
        self.0
    }

    /// The effective language of each entry in order (the stated tag, else `"en"`). The raw
    /// stated tags remain reachable via [`iter`](Self::iter).
    pub fn languages(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|entry| Self::effective_lang(entry.language.as_deref()))
    }

    /// The number of entries (always at least one).
    #[must_use]
    #[allow(clippy::len_without_is_empty)] // invariant: always ≥ 1 entry, so is_empty() is always false
    pub const fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<LocalisedString> for Vec<LocalisedText> {
    fn from(value: LocalisedString) -> Self {
        value.into_inner()
    }
}

impl TryFrom<Vec<LocalisedText>> for LocalisedString {
    type Error = Error;

    fn try_from(entries: Vec<LocalisedText>) -> Result<Self, Error> {
        Self::new(entries)
    }
}

impl<'de> serde::Deserialize<'de> for LocalisedString {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let entries = Vec::<LocalisedText>::deserialize(deserializer)?;
        Self::new(entries).map_err(to_de_error)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use alloc::{string::ToString, vec};

    use super::*;

    fn entry(language: Option<&str>, text: &str) -> LocalisedText {
        LocalisedText { language: language.map(Into::into), text: text.into() }
    }

    fn sample() -> LocalisedString {
        LocalisedString::new(vec![
            entry(Some("en"), "Name"),
            entry(Some("fr"), "Nom"),
            entry(None, "Untagged"),
        ])
        .unwrap()
    }

    #[test]
    fn rejects_empty_entry_list() {
        assert_eq!(LocalisedString::new(Vec::new()), Err(Error::EmptyLocalisation));
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
        let only_untagged = LocalisedString::new(vec![entry(None, "Fallback")]).unwrap();
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
            entry(Some("en"), "First"),
            entry(None, "Second"),
            entry(Some("en"), "Third"),
        ])
        .unwrap();
        assert_eq!(ls.get("en"), Some("First"));
    }

    #[test]
    fn iter_and_slice_expose_raw_entries_languages_applies_default() {
        let ls = sample();
        let expected =
            [entry(Some("en"), "Name"), entry(Some("fr"), "Nom"), entry(None, "Untagged")];
        // `iter` yields the entries by reference; `as_slice` hands back the same backing slice.
        let iterated: Vec<&LocalisedText> = ls.iter().collect();
        assert_eq!(iterated, expected.iter().collect::<Vec<_>>());
        assert_eq!(ls.as_slice(), &expected);
        // `languages` applies the `"en"` default to the untagged entry.
        let effective: Vec<_> = ls.languages().collect();
        assert_eq!(effective, vec!["en", "fr", "en"]);
        assert_eq!(ls.len(), 3);
    }

    #[test]
    fn deserialize_round_trips_and_rejects_empty() {
        crate::test_support::round_trip(&sample());
        // An empty entry list is mechanically schema-invalid and rejected on the wire path.
        let empty = postcard::to_allocvec(&Vec::<LocalisedText>::new()).unwrap();
        assert!(postcard::from_bytes::<LocalisedString>(&empty).is_err());
    }

    #[test]
    fn localised_string_try_from_rejects_empty() {
        assert_eq!(LocalisedString::try_from(Vec::new()).unwrap_err(), Error::EmptyLocalisation);
    }

    #[test]
    fn localised_string_into_inner_and_from() {
        let v = vec![LocalisedText { language: None, text: "T".to_string() }];
        assert_eq!(LocalisedString::new(v.clone()).unwrap().into_inner(), v);
        assert_eq!(Vec::from(LocalisedString::new(v.clone()).unwrap()), v);
    }

    // Property tests: the internal serde round-trip over generated entries (see
    // `test_strategy`); wasm32 is excluded with the rest of the property suite.
    #[cfg(not(target_arch = "wasm32"))]
    mod prop {
        use proptest::prelude::*;

        use crate::test_strategy::localised_string;

        proptest! {
            #[test]
            fn localised_string_round_trips(value in localised_string()) {
                crate::test_support::round_trip(&value);
            }
        }
    }
}
