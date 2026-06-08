# sdmx-rs

<!-- [![sdmx-rs on crates.io](https://img.shields.io/crates/v/sdmx-rs.svg)](https://crates.io/crates/sdmx-rs) -->
<!-- [![docs.rs](https://img.shields.io/docsrs/sdmx-rs)](https://docs.rs/sdmx-rs) -->
[![MSRV: 1.91.0](https://img.shields.io/badge/MSRV-1.91.0-blue)](https://github.com/dgalbraith/sdmx-rs/blob/main/docs/project/msrv.md)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/dgalbraith/sdmx-rs#license)

Universal Statistical Data and Metadata Exchange (SDMX) framework for Rust.

This crate serves as the top-level facade and entry point for the `sdmx-rs` workspace, coordinating re-exports of individual sub-crates under clear feature flags.

## Workspace Architecture

```mermaid
graph TD
    Facade[sdmx-rs <br/> Facade Meta-Crate]:::facade
    Client[sdmx-client <br/> HTTP Orchestrator]:::feature
    Parsers[sdmx-parsers <br/> Serialization Engine]:::feature
    Writers[sdmx-writers <br/> Serialization Adapter]:::feature
    Types[sdmx-types <br/> Domain Core]:::core

    Facade -.->|client feature| Client
    Facade -.->|parsers feature| Parsers
    Facade -.->|writers feature| Writers
    Facade ===>|unconditional| Types

    Client --> Parsers
    Client --> Types
    Parsers --> Types
    Writers --> Types

    classDef facade fill:#1e293b,stroke:#3b82f6,stroke-width:2px,color:#f8fafc;
    classDef core fill:#1e293b,stroke:#f59e0b,stroke-width:2px,color:#f8fafc;
    classDef feature fill:#1e293b,stroke:#10b981,stroke-width:1px,color:#f8fafc;
```

## Features

*   **`types`** (Always Compiled): Pure, `#![no_std]`, dependency-free domain models, metadata schemas, and validation invariants.
*   **`parsers`** (Default Feature): Streaming XML and JSON parser engine.
*   **`writers`** (Default Feature): Serialization adapter for SDMX output generation (XML, JSON, CSV) via the `TargetVersion` API contract.
*   **`client`** (Default Feature): Tokio-based async HTTP orchestrator managing REST endpoints.

### TLS (when `client` is enabled)

| Feature | Default | TLS engine | Certificate source                                      |
|---------|:-------:|:----------:|---------------------------------------------------------|
| `tls`   | ✓       | rustls     | Host OS / native trust store (rustls-platform-verifier) |

By default this library uses the **host OS trust store** via reqwest 0.13's default
`rustls-platform-verifier`. This works identically on Linux, macOS, and Windows — no host
certificate configuration required. (`rustls` is the only TLS backend, so the flag is simply
`tls` — on or off.)

**Corporate / internal CA environments**: add your internal CA certificate at runtime — it is
**merged** into the native roots:

```rust
use reqwest::tls::Certificate;
let ca = Certificate::from_pem(include_bytes!("internal-ca.pem"))?;
let client = reqwest::Client::builder().tls_certs_merge([ca]).build()?;
```

To compile without any TLS support (advanced / custom transport scenarios):

```toml
[dependencies]
sdmx-rs = { version = "0.1", default-features = false, features = ["parsers", "client"] }
```

## Usage

Add `sdmx-rs` to your `Cargo.toml` dependencies. By default, both the parser and HTTP client layers are enabled:

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
