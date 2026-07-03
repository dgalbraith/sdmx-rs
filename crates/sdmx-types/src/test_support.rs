//! Shared test helpers, compiled only under `cfg(test)`.
//!
//! The domain types' serde is an internal, lossless projection, not the SDMX wire
//! format (D-0068). These helpers exercise that round-trip through a non-wire binary
//! format (`postcard`), so no wire-format library sits in the infoset crate.

// `round_trip` is `pub(crate)` so tests in sibling modules can call it; the enclosing module is
// private, which makes clippy's nursery `redundant_pub_crate` fire, but the crate-scoped visibility
// is the intent and reads more accurately than bare `pub` on a test-only helper.
#![allow(clippy::expect_used, clippy::redundant_pub_crate)]

/// Asserts `value` survives a `postcard` serialise/deserialise round-trip unchanged,
/// exercising the type's `Serialize`/`Deserialize` as mutual inverses without pinning
/// the projection's shape. Returns the restored value so callers can additionally assert
/// contracts `PartialEq` cannot see (the stated-offset tuple, for instance).
pub(crate) fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + core::fmt::Debug,
{
    let bytes = postcard::to_allocvec(value).expect("postcard serialise");
    let restored: T = postcard::from_bytes(&bytes).expect("postcard deserialise");
    assert_eq!(*value, restored, "round-trip changed the value");
    restored
}

/// Pairs an optional datetime with its stated offset for round-trip assertions.
/// `DateTime<FixedOffset>` equality compares the instant and ignores the stated offset,
/// but the offset is data — the preserved datum is the XSD `dateTime` value (instant plus
/// stated numeric offset) — so datetime-bearing round-trips assert this tuple, never bare
/// `Eq`, and a projection change that normalised the offset would fail here.
pub(crate) fn with_stated_offset(
    value: Option<&chrono::DateTime<chrono::FixedOffset>>,
) -> (Option<&chrono::DateTime<chrono::FixedOffset>>, Option<chrono::FixedOffset>) {
    (value, value.map(|datetime| *datetime.offset()))
}
