# 2. Workspace-wide Safety Policy: Banning unsafe code

Date: 2026-05-19

## Status

Accepted

---

## Context

As a library designed to ingest and process potentially untrusted SDMX structural metadata and data messages from external HTTP endpoints, memory safety is a critical design constraint.

Using `unsafe` blocks in Rust bypasses the compiler's safety guarantees to perform operations like raw pointer dereferencing or unchecked slicing. If invariants are violated, this can lead to memory corruption, use-after-free vulnerabilities, or undefined behavior.

## Decision Drivers

* **Security Guarantees**: Preventing exploit vectors (like buffer overflows or memory leakage) when parsing malformed or hostile XML and JSON payloads.
* **Auditability**: Minimizing the cost of security verification by ensuring that all memory management is compiler-verified.
* **Broad Utility**: Aligning with safety requirements of high-integrity enterprise and financial institutions.

---

## Options Considered

### Option A — Allow Unsafe for Micro-Optimizations
Allow selective use of `unsafe` in performance-sensitive areas, such as zero-copy string parsing in `sdmx-parsers`.

* **Pros**: Allows unchecked pointer offsets or unchecked slice conversions, bypassing bounds checking inside parser loops for minor performance gains.
* **Cons**: Increases the risk of panics turning into segmentation faults or vulnerabilities if parsing logic contains bugs.
* **Verdict**: Rejected.

### Option B — Workspace-wide absolute forbid of unsafe code
Enforce `#![forbid(unsafe_code)]` workspace-wide, ensuring no member crate can compile if it contains `unsafe` code.

* **Pros**: Guarantees 100% compiler-enforced memory safety. Modern Rust compilers heavily optimize iterator loops and slice parsing, minimizing bounds-checking overhead.
* **Cons**: Bypasses potential micro-optimizations.
* **Verdict**: Accepted.

---

## Decision

Enforce `#![forbid(unsafe_code)]` at the workspace level. This is registered globally in the workspace root `Cargo.toml` under `[workspace.lints.rust]`.

---

## Consequences

* **Positive**: The library is guaranteed to be memory-safe, eliminating classes of vulnerabilities related to pointer arithmetic or memory leaks.
* **Negative**: String slicing or stream indexing optimizations in `sdmx-parsers` must rely on safe iterators, standard library routines, or bounds-checked slicing.
* **Neutral**: The compiler verifies all memory safety guarantees during CI/CD checks.

---

## References

* [Cargo.toml](../../Cargo.toml#L27) (`unsafe_code = "forbid"` in `[workspace.lints.rust]`)
* [ARCHITECTURE.md](../../ARCHITECTURE.md#L40) (Domain Core constraints)
