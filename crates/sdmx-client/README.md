# sdmx-client

<!-- [![sdmx-client on crates.io](https://img.shields.io/crates/v/sdmx-client.svg)](https://crates.io/crates/sdmx-client) -->
<!-- [![docs.rs](https://img.shields.io/docsrs/sdmx-client)](https://docs.rs/sdmx-client) -->
[![MSRV: 1.91.0](https://img.shields.io/badge/MSRV-1.91.0-blue)](https://github.com/dgalbraith/sdmx-rs/blob/main/docs/project/msrv.md)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/dgalbraith/sdmx-rs#license)

The HTTP orchestrator for the `sdmx-rs` workspace.

This crate provides an asynchronous HTTP client for querying SDMX REST endpoints. It negotiates content types, delegates payload decoding to `sdmx-parsers`, and returns the pure domain models defined in `sdmx-types`.

## Design Constraints
- Built on `tokio` and `reqwest`.
- Does not contain parsing logic directly.
- Handles transport-level error propagation.

## TLS & Client Configuration
> [!NOTE]
> The `sdmx-client` crate is a Phase 3 scaffold. Full client implementation (builders, query builders, custom CA support) is planned for **Phase 3**.
> See [ROADMAP.md](../../ROADMAP.md).

### Current Version (0.1.x) Limitations
* **Supported Roots**: The host OS / native trust store is used out-of-the-box via reqwest 0.13's default `rustls-platform-verifier` (suited for public endpoints like ECB, IMF, Eurostat).
* **Custom CA Support**: Programmatic certificate injection is not yet integrated into `sdmx-client`'s own builder (planned for Phase 3); until then, use `reqwest` directly as shown below.

> **Note**: Compiling with `default-features = false` disables TLS entirely. You must either enable the `tls` feature or provide custom transport.

### Workaround for Internal/Corporate CAs
If you need to connect to an SDMX registry with a custom or internal CA today, use `reqwest` directly for transport and parse the payload manually:

```rust
// 1. Load your CA certificate
let my_ca_cert = reqwest::tls::Certificate::from_pem(
    include_bytes!("path/to/ca.pem")
)?;
// 2. Fetch the payload using reqwest with your custom CA configuration
let client = reqwest::Client::builder()
    .tls_certs_merge([my_ca_cert])
    .build()?;
let response = client.get("https://internal-registry/sdmx/dataflow").send().await?;
let body = response.text().await?;
// 3. Parse the payload separately with sdmx-parsers
let dataflow = sdmx_parsers::parse_dataflow(&body)?;
```

⚠️ **Note**: This bypasses sdmx-client's abstraction layer. You lose:
- Content-type negotiation
- Connection pooling management via sdmx-client
- Potential future middleware (retry, caching, etc.)

### Planned Client API (Phase 3)
In Phase 3, the client will expose a unified builder supporting direct certificate injection:

```rust
let client = SdmxClientBuilder::new("https://internal-registry")
    .add_ca_cert(my_ca_cert)?
    .build()?;
```

For architectural context on why rustls + bundled roots, see [ADR-0013](../../docs/adr/0013-use-rustls-over-native-tls-for-transport-layer-security.md).
