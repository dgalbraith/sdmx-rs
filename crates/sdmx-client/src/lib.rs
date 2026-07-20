//! Scaffold for an async HTTP client for SDMX REST endpoints with payload
//! parsing and state hydration.
//!
//! This crate will provide the high-level orchestrator managing connectivity
//! to remote SDMX REST endpoints, coordinating with
//! [`sdmx-parsers`](../sdmx_parsers/index.html) for payload deserialisation and
//! [`sdmx-types`](../sdmx_types/index.html) for domain representation.
//!
//! # API Design & Builder Pattern
//!
//! Requests to SDMX endpoints will be built using type-safe builder APIs
//! employing the Typestate pattern to prevent invalid API queries at compile
//! time. The planned API shape, specified in Design 0003 and Design 0007, is:
//!
//! ```rust,ignore
//! let client = SdmxClient::new("https://registry.sdmx.org/apis/public");
//! let dataflow = client.structure()
//!     .agency("ECB")
//!     .resource_type("dataflow")
//!     .resource_id("EXR")
//!     .version("1.0")
//!     .send()
//!     .await?;
//! ```
//!
//! # Concurrency Guarantees
//!
//! ADR-0015 and Design 0006 specify the concurrency design summarised below.
//! The `SdmxClient` will use thread-safe connection pooling via the underlying
//! HTTP client (`reqwest`), and all public client endpoints will be `Send` and
//! `Sync`, facilitating safe concurrent sharing across thread boundaries:
//!
//! - **Connection Reuse:** HTTP connections are kept alive and reused across
//!   requests.
//! - **Cheap Clone:** `SdmxClient::clone()` is shallow — it copies the
//!   `reqwest::Client` handle, which internally wraps an `Arc` over the
//!   connection pool. No pool state is duplicated; all clones share the same
//!   underlying pool. `SdmxClient` itself is not wrapped in an `Arc`.
//! - **Builder Ownership:** Query builders take ownership of a cloned
//!   `SdmxClient` (i.e. `client: SdmxClient`, not `client: Arc<SdmxClient>` or
//!   `client: &SdmxClient`). This makes builders `'static` and freely sendable
//!   across thread and task boundaries without lifetime annotations on the
//!   builder type.
//! - **`Send` + `Sync`:** Both `SdmxClient` and its builders are `Send + Sync`
//!   because `reqwest::Client` is `Send + Sync`. The `Arc` is internal to
//!   `reqwest` — not part of the public `SdmxClient` interface.
//!
//! # Content-Type Negotiation
//!
//! As specified in ADR-0018, the client will negotiate response formats
//! dynamically according to target capabilities and user preferences, managing
//! `Accept` headers to request high-performance representations:
//!
//! - **Metadata Queries:** Requests prefer SDMX-JSON or SDMX-ML XML
//!   representation.
//! - **Data Queries:** Prefers modern SDMX-CSV or SDMX-JSON payloads to
//!   minimise deserialisation overhead.
//!
//! # Design Constraints
//!
//! - No unsafe code.
//! - No uncontrolled global mutable state.

/// Scaffold for the synchronous/blocking execution bridge.
///
/// This module will wrap the asynchronous client in a synchronous interface for
/// zero-setup scripting contexts. The design is documented in
/// [Design 0005](docs/design/0005-synchronous-and-blocking-api-execution-bridge.md).
pub mod blocking {
    // TODO: Implement blocking client wrapper in Phase 3.
    // This module wraps the async SdmxClient in a synchronous interface.
    // Construction via SdmxClientBuilder::build_sync() is guaranteed panic-free
    // due to lazy runtime orchestration (see [Design 0005](../../docs/design/0005-synchronous-and-blocking-api-execution-bridge.md)).
}

// TODO: Implement Query Builders module (mod query).
//
// When implementing Phase 3 builders (StructureQueryBuilder, DataQueryBuilder),
// use Cow<'static, str> for all string fields (see [Design 0006](../../docs/design/0006-builder-field-storage.md)). This ensures:
//
// 1. Builders have NO lifetime parameters (no 'a)
// 2. All builders and their returned futures are unconditionally 'static
// 3. Futures can be freely spawned onto background task pools (tokio::spawn)
// 4. No conversion step needed at execution time (.into_owned() is not needed)
//
// Callers with non-'static borrowed &str must convert to String at call site:
//   let borrowed: &str = ...;
//   client.structure().agency(borrowed.to_string()).send().await?
//
// See [Design 0006](../../docs/design/0006-builder-field-storage.md) (Builder Field Storage) and ADR-0015 (Send and IntoFuture) for
// the complete rationale and implementation patterns.
//
// Once implementation is complete, prune temporary skeleton blueprints from
// ARCHITECTURE.md.

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    #[test]
    fn client_crate_exports_are_accessible() {
        // Smoke test: verify client module is importable and structurally sound.
        // This test ensures basic compilation and re-export structure.
        std::hint::black_box(());
    }

    #[test]
    fn client_builder_interfaces_compile() {
        // Structural smoke test ensuring client API signatures compile.
        // Once query builders are implemented, this will verify Send+Sync guarantees.
        #[allow(unused)]
        const _: () = {
            // Placeholder: once builders are introduced, add:
            // let _ = std::mem::size_of::<SomeBuilder>();
        };
    }
}
