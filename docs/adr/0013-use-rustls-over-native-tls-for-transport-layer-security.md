# 13. Use rustls over native-tls for Transport Layer Security

Date: 2026-05-19

## Status

Accepted

---

## Context

The `sdmx-client` crate connects to external SDMX endpoints, which are served over secure HTTPS protocols. Encrypting these connections and validating remote server certificates requires a Transport Layer Security (TLS) engine.

The two primary choices in the Rust ecosystem are:
1. `native-tls`, which acts as a bridge to the host operating system's native cryptographic library (OpenSSL on Linux, SChannel on Windows, Security.framework on macOS).
2. `rustls`, a modern, pure-Rust implementation of the TLS protocol.

We must decide which TLS backend to standardise on for our workspace HTTP transport. The choice impacts memory safety, cross-compilation simplicity, and compatibility with our Nix-driven deterministic development environment.

## Decision Drivers

* **Memory Safety**: Avoid security vulnerabilities at the cryptographic boundary.
* **Build Portability**: Support cross-compilation (e.g., compiling for macOS or Windows targets on a Linux CI builder) without complex native toolchain dependencies.
* **Deterministic Sandboxing**: Ensure compatibility with hermetic Nix development sandboxes.
* **Runtime Performance**: Fast connection handshake times and low memory consumption.

---

## Options Considered

### Option A — `native-tls` (Dynamic Host Bindings)

Delegating TLS operations to host OS libraries.

* **Pros**:
  * Uses the host platform's native certificate store, automatically trusting system-installed certificate authorities.
* **Cons**:
  * Uses C-based platforms (like OpenSSL), which are prone to memory-safety vulnerabilities.
  * Requires native platform development headers during compilation. Cross-compilation requires complex environment configuration and often fails.
  * Fails to align with the Nix-driven developer environment, as it relies on host OS certificate paths and dynamic linkers.
**Verdict**: Rejected.

### Option B — `rustls` (Pure-Rust TLS Engine)

A native Rust implementation of TLS.

* **Pros**:
  * Memory Safe: Leverages Rust's compile-time safety invariants, eliminating memory corruption risks.
  * Portable: Zero external C dependencies. Cross-compilation is trivial and requires no platform-specific cryptographic headers.
  * Hermetic: Fits perfectly into Nix development shells and sandboxes, as it doesn't need to resolve host libraries.
  * Efficient: Higher connection handshake throughput and lower memory footprint compared to OpenSSL.
* **Cons**:
  * `rustls` itself does not read the host OS certificate store directly; a verifier must supply the roots. reqwest 0.13 solves this for us by defaulting to `rustls-platform-verifier` (the OS/native trust store) when the `rustls` feature is enabled, so no manual `rustls-native-certs` / `webpki-roots` wiring is required for the common case.
**Verdict**: Accepted.

---

## Decision

**We will standardise on `rustls` as the TLS provider for `sdmx-client`, configured via `reqwest` feature flags. Host-level C SSL libraries (like OpenSSL) are explicitly prohibited from the dependency tree.**

---

## Consequences

* **Positive**: Compile-time memory safety extended to the cryptographic transport boundary.
* **Positive**: Cross-compilation across all target architectures without linker errors.
* **Positive**: Deterministic, reproducible builds within the Nix shell.
* **Negative**: Root-certificate sourcing is a `rustls` concern rather than an OS-managed one.
  In practice reqwest 0.13's default `rustls-platform-verifier` reuses the host trust store, so
  the common case needs no extra wiring; bespoke trust (internal CAs) is added at runtime via the
  builder rather than through a Cargo feature.

### Root Certificate Strategy and Feature Flags

`sdmx-client` gates TLS compilation behind a single feature flag that maps onto reqwest's
`rustls` feature:

**reqwest 0.13 feature flag mapping**:

| `sdmx-client` feature | `reqwest` feature activated | Certificate source                                                             |
|-----------------------|:---------------------------:|--------------------------------------------------------------------------------|
| `tls` (default)       | `rustls`                    | Host OS / native trust store, via reqwest's default `rustls-platform-verifier` |

> **Note**: reqwest 0.13 consolidated the several `rustls-tls-*-roots` feature flags that
> existed in 0.12 into the single `rustls` feature, and changed the default verifier to
> `rustls-platform-verifier` — which reuses the **host OS trust store** rather than a
> compile-time-bundled root set. This works identically across Linux, macOS, and Windows
> without host certificate configuration. (`webpki-roots` bundling still exists but is now an
> opt-in via `ClientBuilder::tls_certs_only(...)`, not the default.)
>
> Users who need to trust an additional internal/corporate CA certificate **merge** it into the
> native roots at runtime — no separate feature flag is required:
> ```rust
> ClientBuilder::tls_certs_merge([my_ca_cert])
> ```
> (reqwest 0.13 deprecated `add_root_certificate` in favour of `tls_certs_merge` /
> `tls_certs_only`.)
>
> The flag is named `tls` (on/off), not after a backend or root source: `rustls` is the only
> permitted backend (this ADR), so backend selection is not a consumer choice and is kept out
> of the flag name.

`sdmx-client` exposes the `tls` feature flag (default enabled) so consumers can
conditionally compile TLS support. The workspace `Cargo.toml` declares `reqwest` with only
`json` and `stream` features; **no TLS feature is activated at the workspace level** —
activation is exclusively through the `tls` feature in `sdmx-client`. The `sdmx-rs`
facade propagates this flag under the same name, included in its `default` feature set.

---

## References

* [`ARCHITECTURE.md` — HTTP Client Library & TLS Backend](../../ARCHITECTURE.md#http-client-library--tls-backend)
* `crates/sdmx-client/Cargo.toml` — `[features]` declaration
* `crates/sdmx-rs/Cargo.toml` — facade TLS feature passthrough
* `flake.nix` (Nix build environment)
* [rustls Crate](https://crates.io/crates/rustls)
* [webpki-roots Crate](https://crates.io/crates/webpki-roots)
* [reqwest 0.13 TLS docs](https://docs.rs/reqwest/0.13.3/reqwest/index.html#optional-features)
