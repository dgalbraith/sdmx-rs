# 3. Workspace Crate Facade and Version Pinning Strategy

Date: 2026-05-16

## Status

Accepted

---

## Context

The `sdmx-rs` library is structured as a multi-crate workspace, separating core types (`sdmx-types`), serialisation logic (`sdmx-parsers`), and HTTP client protocols (`sdmx-client`).

If downstream consumers import these sub-crates directly, minor version mismatches (e.g., using `sdmx-types 1.1` alongside a client compiled with `sdmx-types 1.0`) will cause compiler errors due to incompatible Rust types. To prevent "dependency hell" and simplify integration, we need a unified interface and a way to guarantee version compatibility across the workspace.

## Decision Drivers

* **User Experience**: Downstream developers should only need a single dependency to use the library.
* **Type Compatibility**: Ensuring type identities match across the client, parser, and user applications.
* **API Stability**: Controlling the visibility of lower-level internal structures.

---

## Options Considered

### Option A — Direct Sub-Crate Dependencies
Instruct users to import `sdmx-types`, `sdmx-parsers`, or `sdmx-client` directly in their `Cargo.toml` as needed.

* **Pros**: Users only build what they use (e.g. if they only need domain types, they don't compile client dependencies like `tokio` or `reqwest`).
* **Cons**: Fragile version coordination. Type mismatches between libraries will trigger difficult-to-diagnose compiler errors for end-users.
* **Verdict**: Rejected.

### Option B — Workspace Facade with Exact Version Pinning
Use the main `sdmx-rs` crate as a facade that re-exports `sdmx-types`, `sdmx-parsers`, and `sdmx-client` under feature flags. Pin workspace dependencies to exact versions (`=x.y.z`) within the facade crate.

* **Pros**: Standardises usage to a single dependency (`sdmx-rs`). Eliminates version drift between crates by forcing absolute lockstep version matches.
* **Cons**: Every change to a sub-crate requires publishing a matching facade version.
* **Verdict**: Accepted.

---

## Decision

The `sdmx-rs` crate serves as the master facade. It re-exports sub-crates conditionally based on features:
* `sdmx-types` is always exported as the core model.
* `sdmx-parsers` is optional under the `parsers` feature.
* `sdmx-client` is optional under the `client` feature.
* `sdmx-writers` is optional under the `writers` feature.

In `crates/sdmx-rs/Cargo.toml`, dependencies are pinned to exact versions using the `=` operator:
```toml
[dependencies]
sdmx-types   = { version = "=0.1.0", path = "../sdmx-types" }
sdmx-parsers = { version = "=0.1.0", path = "../sdmx-parsers", optional = true }
sdmx-writers = { version = "=0.1.0", path = "../sdmx-writers", optional = true }
sdmx-client  = { version = "=0.1.0", path = "../sdmx-client", optional = true }
```

**Workspace Member Pinning Strategy (Pre- vs. Post-1.0)**:
* **Pre-1.0 (Lockstep Phase)**: Member crates pin workspace dependencies exactly using the `=` operator to align with the lockstep release model.
* **Post-1.0 (Decoupled Phase)**: Member crates must transition to compatible caret requirements (`^x.y.z` or just `x.y.z`) for internal workspace dependencies. This prevents Cargo version resolution deadlock when a shared dependency is updated, while the facade `sdmx-rs` continues to enforce exact pinning (`=`) for downstream consumers.

---

## Consequences

* **Positive**: Downstream consumers interact with a unified interface and are completely insulated from internal crate version drift.
* **Negative**: Releases require publishing the workspace crates in topological order (`types` → `parsers` → `writers` → `client` → `rs`) with synchronised version matching.

* **Neutral**: Feature flags are centralised at the facade level, giving users control over compilation sizes.

---

## References

* [Cargo.toml](../../Cargo.toml) (workspace member declaration)
* `crates/sdmx-rs/Cargo.toml` (facade feature mapping and dependencies)
