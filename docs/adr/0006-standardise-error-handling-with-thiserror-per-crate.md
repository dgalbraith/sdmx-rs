# 6. Standardise Error Handling with Thiserror per Crate

Date: 2026-05-17

## Status

Accepted

---

## Context

The `sdmx-rs` library is designed as a multi-crate workspace comprising `sdmx-types` (domain validation), `sdmx-parsers` (XML/JSON deserialisation), `sdmx-client` (async HTTP orchestration), and `sdmx-rs` (facade).

Errors occurring within lower-level crates must propagate upward to the user. We must decide how to architect error representation and propagation across these modular boundaries. The system must maintain strict crate separation, prevent dependency leakage (e.g., leaking network library errors into the parsing or types crates), and align with `#![no_std]` targets where appropriate.

## Decision Drivers

* **Crate Modularity**: Maintain strict unidirectional dependency flow (`sdmx-client` -> `sdmx-parsers` -> `sdmx-types`).
* **Dependency Leakage Prevention**: Do not allow details of private dependencies (like XML tokenisers or HTTP transport clients) to leak into the public error API of upstream crates.
* **no_std Target Support**: Keep error structures in `sdmx-types` and `sdmx-parsers` compatible with `#![no_std]` execution.
* **Ergonomics & Maintainability**: Minimise manual error mapping boilerplate while providing descriptive, actionable error diagnostics.

---

## Options Considered

### Option A — Global Unified Workspace Error Enum

A single, central `Error` enum defined in `sdmx-types` or a helper crate that represents all possible error variants in the workspace.

* **Pros**:
  * Simple error conversion across the workspace; a single `Result<T, sdmx::Error>` can be used everywhere.
* **Cons**:
  * Violates modularity. The core `sdmx-types` crate would need to reference variants containing `reqwest::Error` or `quick_xml::Error`, forcing it to depend directly on those heavy transport/parsing libraries. This breaks the `#![no_std]` mandate for `sdmx-types`.
  * Leaks low-level implementation details to consumers who only want to use domain model validation.
**Verdict**: Rejected.

### Option B — Localised Crate-Level Error Enums using `thiserror`

Each crate defines its own scoped `Error` type using `thiserror` (e.g., `sdmx_types::Error`, `sdmx_parsers::Error`, `sdmx_client::Error`). Downstream crates map upstream errors using explicit conversions or wrapper variants.

* **Pros**:
  * Preserves clean boundary separation. The types crate has no knowledge of parsers or networking.
  * `thiserror` automates `Display` and `std::error::Error` (or core equivalent) derivations with declarative macros, keeping boilerplate to a minimum.
  * Supports `#![no_std]` targets natively since `thiserror` generates standard error traits that compile without `std` support.
* **Cons**:
  * Requires explicit error mapping or `#[from]` conversions at crate boundary crossings (e.g. mapping `sdmx_parsers::Error` to `sdmx_client::Error`).
**Verdict**: Accepted.

---

## Decision

**We will implement localised, crate-specific `Error` enums using `thiserror` for each library in the workspace. Lower-level errors will propagate upward across crate boundaries via explicit `#[from]` conversions or wrapper variants.**

---

## Consequences

* **Positive**: Strict compilation isolation. Changes to the parsing implementation do not affect domain modelling type compilation or error structures.
* **Positive**: Clear, targeted errors. Consumers using only `sdmx-types` are not exposed to network timeout or XML tag mismatch errors.
* **Positive**: `#![no_std]` + `alloc` compilation for `sdmx-types` and `sdmx-parsers`. With `default-features = false`, the `thiserror` `std` feature is not activated; the generated derive uses `core::error::Error` instead of `std::error::Error`. `core::error::Error` was stabilised in Rust 1.81 — below the workspace MSRV of 1.91.0 — so this is unconditionally correct. This implicit floor on a viable MSRV is the reason the workspace MSRV must not be lowered below 1.81 without revisiting this dependency.
* **Negative**: Mild implementation overhead at crate interfaces to define mapping conversions.

---

## References

* [ARCHITECTURE.md](../../ARCHITECTURE.md) — Section 1.3 (Error Handling)
* [crates/sdmx-types/src/lib.rs](../../crates/sdmx-types/src/lib.rs)
* [crates/sdmx-parsers/src/lib.rs](../../crates/sdmx-parsers/src/lib.rs)
* [crates/sdmx-client/src/lib.rs](../../crates/sdmx-client/src/lib.rs)
