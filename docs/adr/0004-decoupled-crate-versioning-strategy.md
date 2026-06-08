# 4. Decoupled Crate Versioning Strategy

Date: 2026-05-16

## Status

Accepted

> **Pre-1.0 operating note**: Until the workspace reaches `1.0.0`, all crates are released in lockstep at the same version. This is a temporary bootstrap policy, not a reversal of the decision below. The decoupled steady-state described here takes effect from the `1.0.0` release onward. See [ROADMAP.md — Versioning Strategy](../../ROADMAP.md) for the phase-by-phase version table.

---

## Context

In multi-crate workspaces, developers often bind all sub-crates to the same version number (lockstep versioning). While lockstep versioning simplifies release scripts, it violates the principles of Semantic Versioning (SemVer) when crates with different rates of change are grouped together.

For example, `sdmx-types` defines core domain types and is expected to be highly stable. In contrast, `sdmx-client` interacts with external web endpoints and will change frequently to address protocol updates, error mapping, or network middleware. Bumping the version of `sdmx-types` simply because `sdmx-client` received a bugfix causes unnecessary rebuilds and downstream churn.

## Decision Drivers

* **SemVer Adherence**: Ensuring version increments accurately reflect API stability (breaking vs. non-breaking) for each individual crate.
* **Churn Minimization**: Preventing updates to stable libraries (`sdmx-types`) when only client behaviors change.
* **Dependency Stability**: Reducing compile times and downstream API breakage for users who only use the domain types or parser engine.

---

## Options Considered

### Option A — Lockstep Workspace Versioning
Force all crates in the workspace (`sdmx-types`, `sdmx-parsers`, `sdmx-client`, `sdmx-rs`) to share the same version number.

* **Pros**: Easier release process. Facade version pins can simply use the shared workspace version.
* **Cons**: Artificially inflates version numbers for stable crates. Violates SemVer (a patch release for `sdmx-client` forces a bump in `sdmx-types`, potentially breaking downstream caches).
* **Verdict**: Rejected.

### Option B — Decoupled Versioning with Facade Alignment
Allow each sub-crate to have its own independent version number. Increment versions individually based on SemVer rules for modifications within that crate. Update the facade `sdmx-rs` dependency locks to match the latest compatible versions.

* **Pros**: Strictly respects SemVer. Minimizes updates to `sdmx-types`, which is the foundation of the library.
* **Cons**: Requires release management scripts to handle publishing order and dependencies between crates.
* **Verdict**: Accepted.

---

## Decision

Adopt decoupled versioning for all member crates in the workspace. Each crate maintains its own version in its `Cargo.toml`.

When sub-crates are released, the master facade crate `sdmx-rs` is updated to pin to the new versions. If a change in `sdmx-client` does not affect `sdmx-types`, only `sdmx-client` and the facade `sdmx-rs` receive version bumps.

**Internal Pinning Transition**: At the `1.0.0` boundary, all internal workspace dependencies between member crates (e.g., `sdmx-parsers` depending on `sdmx-types`) must transition from exact version pinning (`=`) to caret version requirements (`^`). This permits Cargo to resolve shared dependencies to a single unified version. The facade `sdmx-rs` will continue to pin all member crates exactly (`=`) to guarantee user-facing compatibility.

---

## Consequences

* **Positive**: `sdmx-types` remains highly stable and version-isolated, avoiding dependency churn. Downstream code using only parsing or type modeling is protected from network-layer updates.
* **Negative**: Workspace release automation is more complex. Crates must be published individually in topological order (`sdmx-types` → `sdmx-parsers` → `sdmx-writers` → `sdmx-client` → `sdmx-rs`). Post-1.0 internal dependencies cannot use exact pinning (`=`); keeping exact pins on internal dependencies would cause a Cargo resolution deadlock when updating base crates, forcing a lockstep release cascade anyway.

* **Neutral**: The unified facade ensures that consumers do not need to manage these individual version pairings themselves.

---

## References

* [ADR-0003 — Workspace Crate Facade and Version Pinning Strategy](0003-workspace-crate-facade-and-version-pinning-strategy.md)
