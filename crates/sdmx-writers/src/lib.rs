//! Scaffold for the SDMX serialisation adapter targeting multiple output
//! formats.
//!
//! This crate will provide the serialisation engine for converting domain
//! types from [`sdmx-types`](sdmx_types) into wire formats
//! (SDMX-ML and SDMX-JSON). Writing routines will target efficient buffer
//! management and support both streaming and buffered serialisation patterns.
//!
//! # Design Constraints
//!
//! - Minimal dependencies: the workspace-internal [`sdmx-types`](sdmx_types)
//!   crate for the core domain model, with a small fixed set of serialisation
//!   and error-modelling libraries arriving with the implementation.
//! - No unsafe code.
//! - All serialisation must behave deterministically across platform runtimes.
//!
//! # Design & Serialisation Mechanics
//!
//! The version-aware serialisation design summarised below is settled;
//! implementation is planned.
//!
//! The serialisation engine is responsible for converting version-agnostic
//! domain representations back to their wire-format equivalents, handling any
//! structural differences between SDMX specification versions transparently.
//!
//! ### Format Routing
//!
//! The writers automatically route to the appropriate serialisation target
//! based on the desired output format. When serialising domain types,
//! version-specific differences are managed by the encoder, ensuring that the
//! output conforms to the target SDMX specification version.

#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests {
    use core::hint;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn crate_compiles() {
        // Smoke test: the placeholder crate compiles and its test harness links.
        hint::black_box(());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn dependencies_link() {
        // Smoke test: the declared workspace dependencies link.
        use sdmx_types as _;
    }
}
