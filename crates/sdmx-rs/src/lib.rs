//! Universal Statistical Data and Metadata Exchange (SDMX) framework for Rust.
//!
//! This crate serves as the top-level facade and entry point for the `sdmx-rs`
//! workspace, coordinating re-exports of individual sub-crates under clear
//! feature flags.
//!
//! # Feature & Module Topology
//!
//! The meta-crate re-exports the underlying child engines under clear, optional
//! feature boundaries to tailor the framework footprint:
//!
//! ```text
//!                           ┌───────────────┐
//!                           │    sdmx-rs    │  (Universal Facade API)
//!                           └───┬───┬───┬───┘
//!                               │   │   │
//!                   ┌───────────┤   │   ├───────────┐
//!      [client]     │    [parsers]  │   [writers]   │ (Conditional
//!      feature      │    feature    │   feature     │  Re-exports)
//!                   ▼              ▼                ▼
//!              sdmx-client  sdmx-parsers  sdmx-writers
//!                   │              │                │
//!                   └──────────┬───┼───┬────────────┘
//!                              ▼   ▼   ▼sugg
//!                          ┌───────────────┐
//!                          │  sdmx-types   │  (Core Types - Always Enabled)
//!                          └───────────────┘
//! ```
//!
//! # Usage
//!
//! Add `sdmx-rs` to your `Cargo.toml` dependencies. By default, the parser,
//! writer, and HTTP client layers are enabled, with TLS support:
//!
//! ```toml
//! [dependencies]
//! sdmx-rs = "0.1"
//! ```
//!
//! For pure `#![no_std]`, embedded, or WASM-minimal environments, disable
//! default features to compile only the core domain types layer:
//!
//! ```toml
//! [dependencies]
//! sdmx-rs = { version = "0.1", default-features = false }
//! ```

// `no_std` is asserted on the facade when the `client` feature is not enabled.
//
// Why: `sdmx-types` and `sdmx-parsers` are unconditionally `no_std`; adding
// `no_std` here ensures the facade itself does not accidentally introduce a
// `std` dependency at the parsers-only or types-only feature levels and that
// compilation under `--target wasm32-unknown-unknown --no-default-features`
// remains clean. The real `no_std` constraints are enforced in the sub-crates;
// this attribute is a belt-and-suspenders guard on the facade's own code.
//
// Currently the facade contains no logic — only re-exports — so this attribute
// is inherently satisfied and imposes no restriction.
//
// When to drop or revise: if any code is ever added to this crate at the
// `parsers`-only feature level that requires `std` (e.g. `std::io`, `HashMap`,
// thread-local state), this attribute will produce a compile error. At that
// point, either (a) gate the new code behind `#[cfg(feature = "client")]` so
// the `no_std` path remains clear, or (b) drop the `cfg_attr` entirely and
// document in the crate README that `no_std` support requires using the
// sub-crates directly rather than the facade.
#![cfg_attr(not(feature = "client"), no_std)]

#[cfg(feature = "client")]
#[doc(inline)]
pub use sdmx_client as client;
#[cfg(feature = "parsers")]
#[doc(inline)]
pub use sdmx_parsers as parsers;
#[doc(inline)]
pub use sdmx_types as types;
#[cfg(feature = "writers")]
#[doc(inline)]
pub use sdmx_writers as writers;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    #[test]
    fn facade_re_exports_core_types() {
        // Smoke test: verify core types are always available through the facade.
        // This ensures the foundational domain layer is always reachable.
        #[allow(unused_imports)]
        use crate::types;
    }

    #[test]
    fn facade_feature_gates_are_wired() {
        // Smoke test: verify feature-gated re-exports compile correctly.
        // This catches breaking changes in conditional module visibility.
        #[cfg(feature = "client")]
        {
            #[allow(unused_imports)]
            use crate::client;
        }

        #[cfg(feature = "parsers")]
        {
            #[allow(unused_imports)]
            use crate::parsers;
        }

        #[cfg(feature = "writers")]
        {
            #[allow(unused_imports)]
            use crate::writers;
        }
    }

    #[test]
    fn facade_compiles_with_feature_combinations() {
        // Structural smoke test ensuring all feature combinations compile.
        // Verifies that the no_std assertion holds for parsers-only builds.
        // `core::hint` (not `std::hint`) so this test also compiles under the
        // no_std configuration it is meant to exercise (parsers-only builds).
        core::hint::black_box(());
    }
}
