# 9. Maintenance Obligation Tracking System Design

Date: 2026-05-23

## Status

Accepted

<!-- Valid statuses: Proposed, Accepted, Implemented, Superseded -->

---

## Summary

Establish a lightweight, hybrid tracking mechanism for recurring software maintenance obligations (MSRV upgrades, toolchain pinning, security audits) using a central ledger validated in CI alongside inline source file marker comments to prevent maintenance debt.

---

## Problem / Motivation

The sdmx-rs project maintains multiple periodic obligations that are critical to long-term health but easy to defer: updating pinned nightly toolchain versions, reviewing MSRV upgrade windows, running security audits, re-baselining performance benchmarks, promoting lints, and evaluating coverage thresholds.

These tasks don't fit the normal feature/bugfix cycle. Without explicit tracking, they accumulate silently as maintenance debt, often only discovered when they've become urgent or blocking.

### Decision Drivers

* **Visibility**: Maintenance work must be explicit and discoverable by maintainers
* **Enforcement**: Deadlines should be enforced in CI to prevent accumulated debt
* **Scalability**: The system must scale to many periodic obligations without becoming unwieldy
* **Developer Experience**: Updating an obligation should be low-friction; automation reduces manual error
* **Auditability**: The ledger and inline context together provide an audit trail of what was maintained and when

---

## Proposed Design

Adopt the **Hybrid Approach**: central ledger (`maintenance.toml`) + inline comments + validation (`check-maintenance.sh`) + helpers (`maintenance-bump.sh`, `maintenance-sync.sh`).

### Rationale

- Provides the visibility and enforcement of a central system
- Preserves context where the work happens via inline comments
- Validation scripts ensure consistency, reducing manual error
- Automation (helpers) makes updates low-friction
- Scales cleanly as obligations grow
- Audit trail is comprehensive: both the ledger and inline comments are version-controlled

---

## Alternatives Considered

### Option A — Central Ledger Only

Maintain a single TOML file with all obligations, check deadlines in CI, but no inline comments in source files.

**Pros**:
* Single source of truth; centralised visibility
* Simple to query and validate

**Cons**:
* When reviewing a file (e.g., `flake.nix`), readers have no context about why a pin needs periodic attention
* Maintenance context is lost; future maintainers must cross-reference the ledger
* Changes to a file don't remind reviewers of periodic obligations related to it

**Verdict**: Rejected

### Option B — Hybrid Approach (Central Ledger + Inline Comments + Validation)

Maintain a central `maintenance.toml` ledger listing all obligations. For each obligation, add a breadcrumb inline comment in the source file where the work happens. Validate both stay in sync; provide helper scripts to update both atomically.

**Pros**:
* Central ledger provides project-wide visibility and enforces deadlines in CI
* Inline comments preserve local context; readers see "this file needs periodic review" while editing
* Validation prevents drift; synchronisation helpers reduce friction on updates
* Scales cleanly; adding a new obligation requires only a ledger entry + inline comment
* Audit trail: git history shows what was updated and when, inline comments explain why
* Changes to a file preserve institutional memory about its maintenance obligations

**Cons**:
* Requires discipline to keep two sources in sync (mitigated by validation + helpers)
* Slightly more operational overhead than pure ledger approach

**Verdict**: Accepted (Chosen Design)

### Option C — Distributed Per-File Tracking

Each file that needs periodic review includes an inline comment with dates, checked independently during CI (no central ledger).

**Pros**:
* Maximum context locality; everything is where the work happens

**Cons**:
* No central visibility; hard to discover all obligations without scanning every file
* No easy way to enforce consistent cadences or warn about approaching deadlines
* Difficult to query "what's due this quarter?" without parsing many files
* Doesn't scale well as obligations grow
* No unified enforcement mechanism

**Verdict**: Rejected

---

## Drawbacks / Trade-offs

* **Positive**: Maintenance obligations are explicit and enforced. CI prevents debt accumulation. Inline comments provide context during code review. Helpers reduce friction on updates. Git history provides an audit trail. System scales to many obligations.
* **Neutral**: Requires discipline to maintain both ledger and inline comments. Two validation steps (marker exists, dates match) add operational overhead, but helpers automate most of this.
* **Negative**: Drifting ledger/comments creates validation failures; requires tooling to detect. Initial setup for each obligation requires editing two places (config + source file).

---

## Questions & Resolutions

None.

---

## References

None.
