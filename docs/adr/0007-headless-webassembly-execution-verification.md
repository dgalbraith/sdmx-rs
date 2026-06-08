# 7. Headless WebAssembly Execution Verification

Date: 2026-05-17

## Status

Accepted

---

## Context

Because our core crates (`sdmx-types` and `sdmx-parsers`) adopt a `#![no_std]` architecture to support lightweight WebAssembly (WASM) targets, we must ensure these crates compile and execute correctly inside web browsers and WASM runtimes.

While `cargo check --target wasm32-unknown-unknown` validates that the code compiles for WebAssembly, it does not execute any tests. Code that compiles successfully can still panic at runtime—for example, if a transitively included crate attempts to access threads, the system clock, or local filesystem APIs that do not exist in headless WASM environments. To guarantee reliability, we need to run our test suite within a true WebAssembly execution environment.

## Decision Drivers

* **Execution Parity**: Ensuring parser logic and serialization constraints behave identically under WASM as they do on native platforms.
* **Panic Detection**: Automatically catching unsupported platform-specific API calls (like thread-local storage or file IO) before releasing packages.
* **CI Integration**: Running these tests deterministically in local workspaces and remote CI pipelines.

---

## Options Considered

### Option A — Compile Checks Only
Limit WASM verification to syntax checks via `cargo check --target wasm32-unknown-unknown`.

* **Pros**: Fast execution. Requires no external engines (Node.js, browsers) in the developer toolchain.
* **Cons**: Fails to detect runtime panics or logic differences under WASM.
* **Verdict**: Rejected

### Option B — Headless Browser Testing via wasm-pack
Use `wasm-pack test --headless --firefox` to compile and execute the test suite inside a headless browser instance.

* **Pros**: Runs unit tests within a real JavaScript/WASM engine. Guarantees that no native-only assumptions (like thread access or file I/O) exist in the code path.
* **Cons**: Requires a web browser (e.g. Firefox) and a WebDriver intermediary (e.g. Geckodriver) in the developer environment — a large toolchain addition (~200 MB browser binary). Browser/driver version mismatches are a known source of CI flakiness. Browser API surface provides no additional safety guarantee for `sdmx-types` and `sdmx-parsers`, which contain no JS interop and no browser-specific code paths.
* **Verdict**: Rejected — appropriate if/when a dedicated `sdmx-js` JS-bindings crate is introduced; that crate would warrant its own ADR and browser-mode test strategy.

### Option C — Node.js Execution via wasm-pack
Use `wasm-pack test --node` to compile and execute the test suite inside the Node.js V8 WASM runtime.

* **Pros**: Runs unit tests within a real WASM execution engine. Detects the same class of runtime panics on unsupported platform APIs (thread-local storage, filesystem access, system clock) as browser mode. Node.js is a lightweight, universally available CI dependency with no driver layer. Cross-platform reliable on Linux, macOS, and Windows without display servers.
* **Cons**: Does not exercise browser-specific JS API surface — irrelevant for pure data-processing `no_std` crates with no JS glue.
* **Verdict**: Accepted

---

## Decision

Enforce Node.js WASM test execution for all `no_std` crates using `wasm-pack test --node`. This will be integrated into our `just verify` workflow as a `test-wasm` recipe.

Any test designed to run on WASM must be annotated with `#[wasm_bindgen_test]` and run using the Node.js target runner in `crates/sdmx-types` and `crates/sdmx-parsers`.

If a future `sdmx-js` crate introduces `wasm-bindgen` JS bindings, browser-mode testing for that crate should be addressed under a separate ADR at that time.

---

## Consequences

* **Positive**: The library guarantees runtime compatibility for serverless WASM workers and backend WASM runtimes. The Nix devShell requires only `wasm-pack` and `nodejs` — no browser binary or driver.
* **Negative**: Browser JS engine behavior (SpiderMonkey, V8 in Chromium) is not exercised. This is an acceptable trade-off given the absence of any JS interop in the tested crates.
* **Neutral**: Tests require duplicate annotations if they need to be run on both native and WebAssembly engines.

---

## References

* [ADR-0005 — Adopt No-Std with Alloc for Sdmx Types and Sdmx Parsers](0005-adopt-no-std-with-alloc-for-sdmx-types-and-sdmx-parsers.md)
* [Justfile](../../Justfile) (verify rule will call `wasm-pack test`)
