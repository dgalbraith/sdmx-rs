# sdmx-client

<!-- [![sdmx-client on crates.io](https://img.shields.io/crates/v/sdmx-client.svg)](https://crates.io/crates/sdmx-client) -->
<!-- [![docs.rs](https://img.shields.io/docsrs/sdmx-client)](https://docs.rs/sdmx-client) -->
[![MSRV: 1.92.0](https://img.shields.io/badge/MSRV-1.92.0-blue)](https://github.com/dgalbraith/sdmx-rs/blob/main/docs/project/msrv.md)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/dgalbraith/sdmx-rs#license)

The future HTTP orchestrator for the `sdmx-rs` workspace.

This crate is the scaffold for an asynchronous HTTP client for querying SDMX REST endpoints. As designed, it will negotiate content types, delegate payload decoding to `sdmx-parsers`, and return the pure domain models defined in `sdmx-types`. No client functionality is implemented at this version; the implementation arrives with the client milestone on the [roadmap](https://github.com/dgalbraith/sdmx-rs/blob/main/ROADMAP.md).

## Planned Design

- Asynchronous transport with connection pooling and transport-level error propagation.
- Content-type negotiation across the SDMX wire formats, with payload decoding delegated to `sdmx-parsers`; this crate will contain no parsing logic of its own.
- TLS through `rustls` only, using the host OS trust store, with programmatic certificate injection through the client builder.
