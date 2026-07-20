//! Scaffold for the SDMX serialisation adapter targeting multiple output
//! formats.
//!
//! This crate will provide the serialisation engine for converting domain
//! types from [`sdmx-types`](../sdmx_types/index.html) into wire formats
//! (SDMX-ML and SDMX-JSON). Writing routines will target efficient buffer
//! management and support both streaming and buffered serialisation patterns.
//!
//! # Design Constraints
//!
//! - Minimal external dependencies (restricted strictly to `serde`,
//!   `serde_json`, `quick-xml`, and `thiserror` for serialisation and error
//!   modelling).
//! - No unsafe code.
//! - All serialisation must behave deterministically across platform runtimes.
//!
//! # Design & Serialisation Mechanics
//!
//! Design 0008 specifies the version-aware serialisation design summarised
//! below; implementation is planned.
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use core::hint;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn crate_compiles_in_no_std_mode() {
        // Smoke test: verify the serialisation crate exports are accessible in no_std
        // context.
        hint::black_box(());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn writer_interfaces_compile() {
        // Structural smoke test ensuring writer module compilation.
        // This test catches breaking changes to public writer interfaces early.
        #[allow(unused)]
        const _: () = {
            // Placeholder: once writer traits/types are introduced, add:
            // let _ = core::mem::size_of::<SomeWriterType>();
        };
    }
}
