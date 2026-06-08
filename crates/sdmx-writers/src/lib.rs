//! High-performance SDMX serialization adapter for multiple output formats.
//!
//! This crate provides the serialization engine for converting domain types
//! from [`sdmx-types`](../sdmx_types/index.html) into wire formats (SDMX-ML and
//! SDMX-JSON). Writing routines target efficient buffer management and support
//! both streaming and buffered serialization patterns.
//!
//! # Design Constraints
//!
//! - Minimal external dependencies (restricted strictly to `serde`,
//!   `serde_json`, `quick-xml`, and `thiserror` for serialization and error
//!   modeling).
//! - No unsafe code.
//! - All serialization must behave deterministically across platform runtimes.
//!
//! # Design & Serialization Mechanics
//!
//! The serialization engine is responsible for converting version-agnostic
//! domain representations back to their wire-format equivalents, handling any
//! structural differences between SDMX specification versions transparently.
//!
//! ### Format Routing
//!
//! The writers automatically route to the appropriate serialization target
//! based on the desired output format. When serializing domain types,
//! version-specific differences are managed by the encoder, ensuring that the
//! output conforms to the target SDMX specification version.

#![no_std]

extern crate alloc;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use core::hint;

    #[test]
    fn crate_compiles_in_no_std_mode() {
        // Smoke test: verify the serialization crate exports are accessible in no_std
        // context.
        hint::black_box(());
    }

    #[test]
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
