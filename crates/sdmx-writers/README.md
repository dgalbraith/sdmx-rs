# sdmx-writers

<!-- [![sdmx-writers on crates.io](https://img.shields.io/crates/v/sdmx-writers.svg)](https://crates.io/crates/sdmx-writers) -->
<!-- [![docs.rs](https://img.shields.io/docsrs/sdmx-writers)](https://docs.rs/sdmx-writers) -->
[![MSRV: 1.92.0](https://img.shields.io/badge/MSRV-1.92.0-blue)](https://github.com/dgalbraith/sdmx-rs/blob/main/docs/project/msrv.md)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/dgalbraith/sdmx-rs#license)

The serialisation adapter for the `sdmx-rs` workspace.

This crate is the scaffold for high-performance serialisation of SDMX domain types to multiple output formats, including XML (SDMX-ML) and JSON (SDMX-JSON).

## Design Constraints
- Depends only on `sdmx-types`.
- Targets minimal memory allocations and efficient buffer management.
- Handles large SDMX metadata structures with streaming serialisation where applicable.

## Usage

<!-- Usage examples will be added as the API stabilises. -->

---

This crate is part of the [sdmx-rs](https://github.com/dgalbraith/sdmx-rs) workspace.
