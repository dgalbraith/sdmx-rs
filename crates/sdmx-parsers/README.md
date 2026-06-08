# sdmx-parsers

<!-- [![sdmx-parsers on crates.io](https://img.shields.io/crates/v/sdmx-parsers.svg)](https://crates.io/crates/sdmx-parsers) -->
<!-- [![docs.rs](https://img.shields.io/docsrs/sdmx-parsers)](https://docs.rs/sdmx-parsers) -->
[![MSRV: 1.91.0](https://img.shields.io/badge/MSRV-1.91.0-blue)](https://github.com/dgalbraith/sdmx-rs/blob/main/docs/project/msrv.md)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/dgalbraith/sdmx-rs#license)

The streaming serialization engine for the `sdmx-rs` workspace.

This crate provides streaming serialization and deserialization of SDMX payloads in both XML (SDMX-ML) and JSON (SDMX-JSON) formats.

## Design Constraints
- Depends only on `sdmx-types`.
- Targets minimal memory allocations and zero-copy slicing where safe.
- Handles massive SDMX structural metadata documents without full DOM materialization.

## Usage

<!-- Usage examples will be added as the API stabilizes. -->

---

This crate is part of the [sdmx-rs](https://github.com/dgalbraith/sdmx-rs) workspace.
