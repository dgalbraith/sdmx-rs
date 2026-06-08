//! Async HTTP client for SDMX REST endpoints with payload parsing and state
//! hydration.
//!
//! This crate provides the high-level orchestrator managing connectivity to
//! remote SDMX REST endpoints, coordinating with
//! [`sdmx-parsers`](../sdmx_parsers/index.html) for payload deserialization and
//! [`sdmx-types`](../sdmx_types/index.html) for domain representation.
//!
//! # API Design & Builder Pattern
//!
//! Requests to SDMX endpoints are built using type-safe builder APIs employing
//! the Typestate pattern to prevent invalid API queries at compile time:
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
//! The `SdmxClient` utilizes thread-safe connection pooling via an underlying
//! HTTP client design (`reqwest`). All public client endpoints are `Send` and
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
//! The client negotiates response formats dynamically according to target
//! capabilities and user preferences. It automatically manages `Accept` headers
//! to request high-performance representations:
//!
//! - **Metadata Queries:** Requests prefer SDMX-JSON or SDMX-ML XML
//!   representation.
//! - **Data Queries:** Prefers modern SDMX-CSV or SDMX-JSON payloads to
//!   minimize deserialization overhead.
//!
//! # Design Constraints
//!
//! - No unsafe code.
//! - No uncontrolled global mutable state.

/// Blocking synchronous API for SDMX client operations.
///
/// Provides zero-setup synchronous scripting via `blocking::SdmxClient`.
/// Construction is panic-free regardless of whether a Tokio runtime is active
/// on the calling thread.
///
/// # Construction
///
/// `SdmxClientBuilder::build_sync()` adapts to the caller's runtime context:
///
/// - **No Tokio runtime active:** A private `current_thread` Tokio runtime is
///   created and owned by the client. This enables CLI tools and scripts to use
///   the blocking API without manual runtime initialization.
/// - **Tokio runtime active:** The ambient runtime handle is captured. The
///   client delegates blocking calls through the active runtime using
///   `BlockingStrategy` (see below).
///
/// # Blocking Strategy
///
/// When the client uses an ambient runtime (not owned), the blocking strategy
/// determines how to bridge sync→async without blocking the scheduler. See
/// `BlockingStrategy` and [Design 0005](docs/design/0005-synchronous-and-blocking-api-execution-bridge.md) for
/// details.
///
/// - **`SpawnBlocking`** (default if created within `spawn_blocking`): Safe
///   everywhere but may yield to other tasks.
/// - **`Auto`** (default): Tries `block_in_place` (multi-threaded runtimes
///   only); falls back if single-threaded.
/// - **`BlockInPlace`**: Forces `block_in_place`. Requires multi-threaded
///   Tokio; returns error if single-threaded.
///
/// When the client owns its runtime, the strategy is ignored and
/// `rt.block_on()` is always used.
///
/// # Example
///
/// ```rust,ignore
/// // Works with or without an ambient Tokio runtime:
/// let client = SdmxClientBuilder::new("https://registry.sdmx.org/apis/public")
///     .build_sync()?;
///
/// let dataflow = client.structure()
///     .agency("ECB")
///     .resource_type("dataflow")
///     .send()?;
/// ```
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
