# sdmx-rs

<!-- [![sdmx-rs on crates.io](https://img.shields.io/crates/v/sdmx-rs.svg)](https://crates.io/crates/sdmx-rs) -->
<!-- [![docs.rs](https://img.shields.io/docsrs/sdmx-rs)](https://docs.rs/sdmx-rs) -->
[![MSRV: 1.92.0](https://img.shields.io/badge/MSRV-1.92.0-blue)](https://github.com/dgalbraith/sdmx-rs/blob/main/docs/project/msrv.md)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/dgalbraith/sdmx-rs#license)

Universal Statistical Data and Metadata Exchange (SDMX) framework for Rust.

This crate serves as the top-level facade and entry point for the `sdmx-rs` workspace, coordinating re-exports of individual sub-crates under clear feature flags.

## Workspace Architecture

```text
                          ┌───────────────┐
                          │    sdmx-rs    │  (Universal Facade API)
                          └───┬───┬───┬───┘
                              │   │   │
                  ┌───────────┘   │   └───────────┐
     [client]     │     [parsers] │     [writers] │  (Conditional
     feature      │      feature  │      feature  │   Re-exports)
                  ▼               ▼               ▼
            sdmx-client     sdmx-parsers    sdmx-writers
                  │               │               │
                  └───────────┬───┬───┬───────────┘
                              ▼   ▼   ▼
                          ┌───────────────┐
                          │  sdmx-types   │  (Core Types - Always Enabled)
                          └───────────────┘
```

## Features

*   **`types`** (Always Compiled): Pure, `#![no_std]` domain models, metadata schemas, and validation invariants.
*   **`parsers`** (Default Feature): Future home of the streaming format parsers; the re-export is wired, implementation is planned.
*   **`writers`** (Default Feature): Future home of the format writers; the re-export is wired, implementation is planned.
*   **`client`** (Default Feature): Future home of the async HTTP client; the re-export is wired, implementation is planned.

### TLS (when `client` is enabled)

The `tls` feature flag (enabled by default) is forward-declared: it takes effect when the client implementation lands. TLS will be provided by `rustls` only, using the host OS trust store. Until then the flag gates nothing.

## Usage

Add `sdmx-rs` to your `Cargo.toml` dependencies. By default, the parser, writer, and HTTP client layers are enabled, with TLS support:

```toml
[dependencies]
sdmx-rs = "0.1"
```

For pure `#![no_std]`, embedded, or WASM-minimal environments, disable default features to compile only the core domain types layer:

```toml
[dependencies]
sdmx-rs = { version = "0.1", default-features = false }
```

## License

Licensed under either of:

*   Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/dgalbraith/sdmx-rs/blob/main/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
*   MIT license ([LICENSE-MIT](https://github.com/dgalbraith/sdmx-rs/blob/main/LICENSE-MIT) or http://opensource.org/licenses/MIT)
