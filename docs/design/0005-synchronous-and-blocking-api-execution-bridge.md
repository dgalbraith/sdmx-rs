# 5. Synchronous and Blocking API Execution Bridge

Date: 2026-05-19

## Status

Accepted

<!-- Valid statuses: Proposed, Accepted, Implemented, Superseded -->

---

## Summary

Provide a synchronous wrapper for `SdmxClient` that delegates asynchronous HTTP requests to the underlying tokio runtime, supporting both zero-setup scripting and high-performance applications through auto-detection of ambient runtimes and configurable blocking execution strategies.

---

## Problem / Motivation

While asynchronous I/O (tokio) is our primary runtime for high-performance and concurrent network requests, many users compile Rust libraries for simple scripts, synchronous ETL pipelines, or command-line interfaces. These use cases do not benefit from async/await syntax and require a blocking API.

We provide a blocking version of the client while avoiding the duplicate maintenance overhead of writing a separate HTTP client. We must balance safety (working on any runtime) against performance (leveraging block_in_place on multi-threaded Tokio when available) without panicking or surprising users.

### Decision Drivers

* Maintainability: Sharing request formatting, URL building, and parsing logic.
* API Usability: Providing a clean, synchronous interface that does not require users to initialise or manage a Tokio runtime.
* Safety: Preventing panics by utilising correct Tokio blocking primitives based on the runtime context.
* Performance: Enabling high-performance users to opt into block_in_place when they have verified multi-threaded Tokio availability.

---

## Proposed Design

### 1. Runtime Strategy Enum

```rust
pub enum BlockingStrategy {
    /// Always use Handle::block_on. Safe and portable across all runtimes.
    SpawnBlocking,

    /// Attempt block_in_place first; fall back to Handle::block_on.
    /// Default strategy. Provides best-effort performance.
    Auto,

    /// Force block_in_place. Returns BlockingNotSupported if called
    /// on a single-threaded runtime. For verified high-performance use.
    BlockInPlace,
}
```

### 2. Builder Configuration & Client State

The `SdmxClient` stores a `RuntimeHandle` enum that encapsulates whether the client uses an ambient Tokio runtime or owns a private one. This approach enables zero-setup synchronous scripting without panic at construction time.

```rust
pub enum RuntimeHandle {
    /// An ambient Tokio runtime is active; its handle was captured at build time.
    Active(tokio::runtime::Handle),
    /// No runtime was active; a private current_thread runtime is owned by this client.
    Owned(std::sync::Arc<tokio::runtime::Runtime>),
}

pub struct SdmxClient {
    runtime_handle: RuntimeHandle,
    config: ClientConfig,
}

impl SdmxClientBuilder {
    pub fn blocking_strategy(mut self, strategy: BlockingStrategy) -> Self {
        self.blocking_strategy = strategy;
        self
    }

    pub fn build_sync(self) -> Result<blocking::SdmxClient, Error> {
        let runtime_handle = match tokio::runtime::Handle::try_current() {
            Ok(handle) => RuntimeHandle::Active(handle),
            Err(_) => {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| Error::RuntimeInitFailed(e.to_string()))?;
                RuntimeHandle::Owned(std::sync::Arc::new(rt))
            }
        };
        Ok(blocking::SdmxClient::new(async_client, runtime_handle))
    }
}
```

### 3. Blocking Execution Methods

We bridge the sync/async gap using `Handle::block_on` and `tokio::task::block_in_place`. The strategy used depends on whether the client owns the runtime or uses an ambient one.

```rust
impl SdmxClient {
    pub fn send_blocking(&self, query: Query) -> Result<Response, Error> {
        match &self.runtime_handle {
            RuntimeHandle::Owned(rt) => {
                // Client owns the runtime; always safe to block directly.
                rt.block_on(self.send_async(query))
            }
            RuntimeHandle::Active(handle) => {
                // Ambient runtime is active; strategy determines the blocking primitive.
                match self.config.blocking_strategy {
                    BlockingStrategy::SpawnBlocking => {
                        handle.block_on(self.send_async(query))
                    }
                    BlockingStrategy::Auto => {
                        if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
                            tokio::task::block_in_place(|| handle.block_on(self.send_async(query)))
                        } else {
                            // Fallback to block_on for single-threaded runtimes
                            handle.block_on(self.send_async(query))
                        }
                    }
                    BlockingStrategy::BlockInPlace => {
                        if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
                            tokio::task::block_in_place(|| handle.block_on(self.send_async(query)))
                        } else {
                            Err(Error::BlockingNotSupported {
                                strategy: BlockingStrategy::BlockInPlace,
                                detail: "block_in_place requires a multi-threaded Tokio runtime",
                            })
                        }
                    }
                }
            }
        }
    }
}
```

### 4. Error Handling

```rust
pub enum Error {
    RuntimeInitFailed(String),
    BlockingNotSupported {
        strategy: BlockingStrategy,
        detail: &'static str,
    },
}
```

### 5. Clone Semantics

Both variants of `RuntimeHandle` are `Clone`, preserving the "cheap clone" property of `SdmxClient` documented in ARCHITECTURE.md:

- **`Active(Handle)`**: Tokio's `Handle` is `Clone` with `O(1)` cost and zero allocation.
- **`Owned(Arc<Runtime>)`**: `Runtime` itself is not `Clone`. Wrapping it in `Arc` provides atomic reference counting, allowing multiple `blocking::SdmxClient` instances to share the same private runtime without contention. Cloning is `O(1)` and thread-safe.

### Runtime Safety and Panic Mitigation

* **Zero-Panic Construction**: `build_sync()` uses `Handle::try_current()` to detect an ambient runtime. If one is present it is wrapped as `RuntimeHandle::Active`; if not, a `current_thread` runtime is created and owned as `RuntimeHandle::Owned`. Construction never panics.
* **Ambient Runtime Path (`Active`)**: By utilising `tokio::runtime::Handle`, we avoid the overhead of owning a Runtime and ensure we can safely bridge into any existing Tokio environment. When present, `block_in_place` signals to the scheduler that the current thread is performing blocking work, preventing stalls.
* **Owned Runtime Path (`Owned`)**: When the client owns the runtime, `rt.block_on()` is always safe — there are no competing async tasks and no scheduler to negotiate with.
* **Constraint**: `block_in_place` requires a multi-threaded runtime. If a user selects `BlockingStrategy::BlockInPlace` in the ambient (`Active`) path and the runtime is single-threaded, they receive `Error::BlockingNotSupported` rather than a runtime panic.

---

## Alternatives Considered

### Option A — Separate Blocking Implementation
Implement a dedicated blocking client using a synchronous HTTP library (e.g., ureq). Code duplication makes this unmaintainable.

### Option B — Naive Blocking Wrapper
Spawning a full runtime *inside* a blocking call creates "nesting panics" and destroys the cheap-clone guarantee of SdmxClient. Note: Creating a runtime *at builder time* and owning it (Option C, variant described below) avoids nesting and preserves the cheap-clone guarantee via `Arc<Runtime>`.

### Option C — Runtime Strategy Selection via Builder Configuration
Configure a `BlockingStrategy` at `SdmxClientBuilder` time. The client uses a `tokio::runtime::Handle` to bridge sync calls to the async engine.

---

## Drawbacks / Trade-offs

* **Positive**:
  - Client remains runtime-agnostic and maintains "cheap clone" semantics.
  - Safe by default; high-performance users can opt-in to `BlockInPlace`.
  - Single source of truth for logic; 100% reuse of the async engine.
* **Negative**:
  - Slightly more complex error handling for the `BlockInPlace` strategy.
* **Neutral**:
  - Requires tokio dependency, which is already mandated by ADR-0011.

---

## Questions & Resolutions

None.

---

## References

* [ADR-0011 — Use Tokio as the Primary Async Runtime](../adr/0011-use-tokio-as-the-primary-async-runtime.md)
* [ADR-0014 — Fallible Client Construction and Custom Error Mapping](../adr/0014-fallible-client-construction-and-custom-error-mapping.md)
