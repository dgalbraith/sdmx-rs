# 14. Fallible Client Construction and Custom Error Mapping

Date: 2026-05-19

## Status

Accepted

---

## Context

The `sdmx-client` crate provides an asynchronous HTTP client to retrieve SDMX structural definitions and data messages. Instantiating an `SdmxClient` requires parsing a base service URL and initialising network parameters (such as connection pools, timeouts, and TLS trust roots).

If URL parsing or TLS initialisation fails, we must decide whether to return an error immediately during client construction (`SdmxClient::new`) or to defer validation to the first network request. Additionally, we need to determine the public signature of these errors to prevent leaky abstractions.

## Decision Drivers

* **Fail-Fast Semantics**: Validating client configurations (like malformed base URLs or missing TLS certs) immediately at construction time.
* **Encapsulation**: Preventing implementation details of the underlying HTTP backend (e.g. `reqwest::Error`) from leaking into public API signatures.
* **Consistent Error Pattern**: Aligning with workspace error-handling standards.

---

## Options Considered

### Option A — Infallible Construction with Deferred Validation
Make `SdmxClient::new()` infallible, returning `Self` directly. Store the raw configuration strings and defer URL parsing and TLS setup until the first network call.

* **Pros**: Simple constructor signature (`SdmxClient::new` does not return a `Result`).
* **Cons**: Postpones configuration errors to runtime requests. Leads to confusing errors where a query fails due to a bad base URL provided minutes earlier.
* **Verdict**: Rejected.

### Option B — Fallible Construction with Custom Mapped Error
Make `SdmxClient::new()` return a `Result<SdmxClient, Error>`. Validate the URL and initialise the TLS client during construction. Define a custom `Error` enum using `thiserror` that wraps underlying failures (e.g., `InvalidUrl`, `TlsInitializationFailed`).

* **Pros**: Guarantees that any successfully constructed client is configured correctly and ready to perform network calls. Prevents leaky API signatures by mapping third-party library errors (like `reqwest::Error`) to domain-specific variants.
* **Cons**: Forces callers to handle or propagate a constructor error.
* **Verdict**: Accepted.

---

## Decision

Client construction is fallible and follows a two-path design:

### Primary Path: Builder Pattern

For full configuration control, clients use the builder pattern via `SdmxClient::builder()` → `SdmxClientBuilder::build()`:

```rust
impl SdmxClient {
    pub fn builder() -> SdmxClientBuilder { ... }
}

impl SdmxClientBuilder {
    pub fn base_url(mut self, url: impl Into<Url>) -> Self { ... }
    pub fn timeout(mut self, duration: Duration) -> Self { ... }
    pub fn build(self) -> Result<SdmxClient, Error> { ... }
}
```

This is the recommended path for callers requiring custom timeouts, TLS configuration, or other advanced parameters.

### Convenience Path: Direct Constructor

For simple cases requiring only a base URL, `SdmxClient::new()` provides a fallible convenience wrapper:

```rust
impl SdmxClient {
    pub fn new(base_url: &str) -> Result<Self, Error>;
}
```

Internally, `::new()` delegates to the builder, initialising it with the provided URL and applying default values for all other configuration parameters. This maintains API consistency: both paths return `Result<Self, Error>`.

### Error Mapping

Both construction paths return errors mapped to a crate-specific `sdmx_client::Error` enum:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid base URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Failed to initialise TLS/HTTP client: {0}")]
    TlsInitialization(String),

    // Other request-time variants...
}
```

---

## Consequences

* **Positive**: Developers receive immediate feedback if a client is misconfigured. The builder pattern scales to arbitrary configuration parameters without multiplying constructor variants. The convenience `::new()` wrapper lowers friction for simple URL-only initialisation.
* **Positive**: Internal HTTP library changes (e.g., swapping `reqwest` configuration) do not break the client constructor's public API; the builder encapsulates all implementation details.
* **Positive**: Both construction paths share the same error handling and fallibility contract, eliminating inconsistency between simple and advanced usage.
* **Negative**: Callers must unpack the constructor result using `?` or matching.
* **Neutral**: Crate-local error structures are maintained using `thiserror`, keeping the interface clean and idiomatic.

---

## References

* [ADR-0006 — Standardise Error Handling with Thiserror per Crate](0006-standardise-error-handling-with-thiserror-per-crate.md)
* [ARCHITECTURE.md](../../ARCHITECTURE.md#L450) (Client construction signatures)
