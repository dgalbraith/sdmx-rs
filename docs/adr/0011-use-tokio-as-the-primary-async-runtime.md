# 11. Use Tokio as the Primary Async Runtime

Date: 2026-05-18

## Status

Accepted

---

## Context

The `sdmx-client` crate requires a robust networking foundation to handle SDMX REST endpoints efficiently. Future phases of the library (Phase 3) will require high-concurrency, non-blocking metadata harvesting, connection pooling, and the integration of middleware for resilience (such as rate-limiting and retry policies).

The Rust ecosystem offers several approaches to network I/O, including synchronous (blocking) architectures and various asynchronous runtimes (`tokio`, `async-std`, `smol`). We must choose a foundational execution model for our HTTP orchestrator that provides maximum performance and the lowest integration friction for downstream users.

## Decision Drivers

* **Ecosystem Alignment**: Minimize dual-runtime conflicts (executor panic or runtime overhead) for modern production applications.
* **Concurrency Scale**: Enable highly concurrent metadata harvesting without starving system threads.
* **Resilience Middleware**: Unblock integration with advanced pooling, retries, and rate-limiting stacks (`tower`, `reqwest-middleware`).

---

## Options Considered

### Option A — Pure Synchronous Blocking Client (`ureq` / `reqwest::blocking`)
Building a blocking REST client where every query blocks the executing thread.

* **Pros**: Simple linear code, eliminates the need for an async runtime.
* **Cons**: Scale bottleneck. Concurrent metadata harvesting requires spinning up raw OS threads, which scales poorly and starves executors if called inside existing async applications.
* **Verdict**: Rejected (will be supported only as a planned wrapper feature).

### Option B — `async-std` / `surf` Runtime
An alternative asynchronous execution model aligned with generic futures traits.

* **Pros**: Standard-library-like API aesthetics.
* **Cons**: Stagnant ecosystem alignment. Downstream Tokio services would experience dual-runtime conflicts and execution crashes.
* **Verdict**: Rejected.

### Option C — `Tokio` + `reqwest`
Tokio-based asynchronous stack utilizing the `reqwest` client engine.

* **Pros**: The de facto industry standard for Rust async networking. Provides native connection pooling, Keep-Alive socket recycling, and middleware coverage out of the box.
* **Cons**: Synchronous callers cannot consume it directly without an executor bridge.
* **Verdict**: Accepted.

---

## Decision

We will use **Tokio** as the primary asynchronous runtime for `sdmx-client`, building the core REST networking engine on `reqwest` with the memory-safe `rustls` TLS backend.

---

## Consequences

* **Positive**: Alignment with >90% of downstream production Rust applications.
* **Positive**: Unblocks automatic REST rate-limiting, retries, and correlation harvesting (Phase 3).
* **Negative**: Synchronous callers require a blocking feature-flag wrapper, which we will maintain as a standard runtime-bridging client extension.

---

## References

* [Tokio Asynchronous Runtime Ecosystem](https://tokio.rs/)
* [reqwest Async HTTP Client](https://github.com/seanmonstar/reqwest)
* [ARCHITECTURE.md](../../ARCHITECTURE.md) — Section: Async Runtime
* [ROADMAP.md](../../ROADMAP.md) — Phase 3 (Resilience & Middleware)
