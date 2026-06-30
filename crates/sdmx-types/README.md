# sdmx-types

<!-- [![sdmx-types on crates.io](https://img.shields.io/crates/v/sdmx-types.svg)](https://crates.io/crates/sdmx-types) -->
<!-- [![docs.rs](https://img.shields.io/docsrs/sdmx-types)](https://docs.rs/sdmx-types) -->
[![MSRV: 1.92.0](https://img.shields.io/badge/MSRV-1.92.0-blue)](https://github.com/dgalbraith/sdmx-rs/blob/main/docs/project/msrv.md)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/dgalbraith/sdmx-rs#license)

The core domain model for the `sdmx-rs` workspace.

This crate provides the foundational, dependency-free domain representations for the [SDMX](https://sdmx.org) standard. It defines the structural keys, metadata frameworks, and validation invariants consumed by all other crates.

## Design Constraints
- Strict `#![no_std]` compatibility with minimal dependencies (`serde`, `thiserror`).
- No unsafe code.
- Pure domain model (no binary output, no serialisation logic).

## Usage

<!-- Usage examples will be added as the API stabilises. -->

---

This crate is part of the [sdmx-rs](https://github.com/dgalbraith/sdmx-rs) workspace.
