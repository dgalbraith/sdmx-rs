//! Core SDMX domain types, data structures, and validation invariants.
//!
//! This crate provides the foundational, minimal-dependency core domain
//! representations for the [SDMX](https://sdmx.org) (Statistical Data and Metadata Exchange) standard.
//! It defines the structural keys, metadata frameworks, and validation
//! invariants consumed by all other crates in the `sdmx-rs` workspace.
//!
//! # Design Constraints
//!
//! - Minimal external dependencies (restricted strictly to `serde` and
//!   `thiserror` for serialization and error modeling).
//! - No unsafe code.
//! - No binary output — this crate is a pure domain model library.

#![no_std]

extern crate alloc;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use core::hint;

    #[test]
    fn crate_compiles_in_no_std_mode() {
        // Smoke test: verify the crate exports are accessible in no_std context.
        // This test ensures basic type compilation and module structure.
        hint::black_box(());
    }

    #[test]
    fn core_types_compile() {
        // Verify that core type definitions exist and compile without errors.
        // This is a structural smoke test that catches breaking changes early.
        #[allow(unused)]
        const _: () = {
            // Placeholder: once domain types are introduced, add type-existence
            // checks: let _ = core::mem::size_of::<SomeType>();
        };
    }
}
