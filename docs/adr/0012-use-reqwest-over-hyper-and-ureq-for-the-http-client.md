# 12. Use reqwest over hyper and ureq for the HTTP Client

Date: 2026-05-19

## Status

Accepted

---

## Context

The `sdmx-client` crate acts as the network orchestrator, querying SDMX REST API endpoints to fetch statistical metadata and data structures. SDMX endpoints are often distributed across multiple international institutions (e.g., Eurostat, IMF, BIS). Fetching from these endpoints requires transport-level capabilities: HTTP connection pooling, socket recycling (Keep-Alive), redirect handling, proxy configuration, and DNS caching.

Additionally, our Phase 3 roadmap dictates the introduction of transport resilience, including automatic retry loops, rate-limiting, and correlation telemetry. We must select a client engine that satisfies these features with minimal internal maintenance overhead.

## Decision Drivers

* **Async Ecosystem Integration**: Native alignment with the Tokio async runtime to prevent runtime conflicts.
* **Harvesting Concurrency**: Support non-blocking, parallel harvesting across multiple remote endpoints.
* **Resilience & Middleware**: Out-of-the-box support for rate-limiting, retries, and telemetry wrapper layers.
* **Boilerplate Reduction**: Avoid implementing transport-level socket management, connection pools, and payload streaming.

---

## Options Considered

### Option A — Raw `hyper` (Direct Protocol Engine)

Building the client directly on `hyper`, Rust's low-level protocol engine.

* **Pros**:
  * Complete, fine-grained control over raw HTTP frame operations.
  * Minimal dependency tree.
* **Cons**:
  * Extremely low-level. `hyper` does not provide redirect tracking, cookie jars, proxy routing, or body buffering.
  * Creating and maintaining equivalent client abstractions would require thousands of lines of complex networking code unrelated to SDMX logic.
**Verdict**: Rejected.

### Option B — Synchronous/Blocking Clients (e.g., `ureq` or `reqwest::blocking`)

Executing synchronous network calls where each request blocks the executing thread.

* **Pros**:
  * Linear code execution; eliminates the overhead of managing async executors and lifetimes.
* **Cons**:
  * Highly inefficient for concurrent harvesting. Parallel requests require spawning raw OS threads, which starves application runtimes.
  * Downstream async services (which are predominantly Tokio-based) would experience blocked worker threads, leading to starvation and poor scalability.
**Verdict**: Rejected.

### Option C — Native Async `reqwest` (Tokio Stack)

Utilising `reqwest::Client`, an async client built natively on Tokio and hyper.

* **Pros**:
  * Industry-standard library with native connection pooling, Keep-Alive, and DNS caching.
  * Integrates with `reqwest-middleware` and `tower` stacks, enabling retry/rate-limiting layers in Phase 3.
  * Aligns with the Tokio executor choice (ADR-0011), avoiding dual-runtime conflicts for downstream consumers.
* **Cons**:
  * Relatively large dependency graph.
  * Requires a runtime bridge (e.g. `block_on`) for synchronous callers.
**Verdict**: Accepted.

---

## Decision

**We will use `reqwest` as our primary async HTTP client within `sdmx-client`, utilizing its native Tokio integration, built-in connection pooler, and middleware compatibility.**

---

## Consequences

* **Positive**: Integrates with standard Rust services (which are predominantly Tokio-based).
* **Positive**: Unblocks Phase 3 resilience features (rate-limiting, retries, tracing) via standard middleware engines.
* **Positive**: Avoids maintenance of transport boilerplates.
* **Negative**: Introduces a larger dependency tree, but this matches production expectations for network-bound library engines.

---

## References

* `ARCHITECTURE.md` — Section 1.4 (HTTP Client Library)
* [ADR-0011 — Use Tokio as the Primary Async Runtime](0011-use-tokio-as-the-primary-async-runtime.md)
* [reqwest Crate](https://crates.io/crates/reqwest)
