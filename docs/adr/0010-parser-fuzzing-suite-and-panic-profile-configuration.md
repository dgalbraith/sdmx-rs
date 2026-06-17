# 10. Parser Fuzzing Suite and Panic Profile Configuration

Date: 2026-05-18

## Status

Accepted

---

## Context

To ensure the resilience of `sdmx-parsers` against arbitrary, malformed, or malicious XML, JSON, and CSV inputs, we implement fuzz testing using `cargo-fuzz` (powered by libFuzzer). Fuzzing engines discover inputs that trigger panics, index-out-of-bounds, or memory leaks.

To capture and report panics, the fuzzer relies on Rust's panic unwinding mechanism to intercept the panic, capture the stack trace, and report the failing input. However, to optimise compile size and execution speed, our production release profile is configured to immediately abort on panic (`panic = "abort"`). If the fuzzer compiles under this default release profile, any panic will immediately terminate the process, preventing `cargo-fuzz` from capturing the panic context or continuing to search for other crashes.

Fuzz targets are defined for all three parser formats (XML, JSON, CSV) to ensure comprehensive coverage across `sdmx-parsers` deserialisation paths, including both structure and data message types.

## Decision Drivers

* **Fuzzer Stability**: Allowing the fuzzer engine to capture stack traces, record inputs, and continue execution without crashing the harness.
* **Production Optimisation**: Retaining `panic = "abort"` in release builds to minimise binary size and optimise runtime speed.
* **Ease of Use**: Automating the configuration so developer fuzzing runs do not require manual adjustments to the workspace profiles.

---

## Options Considered

### Option A — Use Unwind Panics Globally in Release
Change the workspace release profile to use `panic = "unwind"`.

* **Pros**: Simplifies profile setup. Fuzzing works automatically under the standard release profile.
* **Cons**: Increases binary sizes and slightly slows down performance for downstream production releases.
* **Verdict**: Rejected

### Option B — No Panic Strategy / Defer to Final Binary
Adopt a strict "no opinion" panic strategy at the workspace level. Library crates must not dictate compilation profiles. The panic strategy (`abort` vs. `unwind`) is exclusively the responsibility of the final compiling consumer (the application binary or fuzzing harness).

* **Pros**: Idiomatic Rust. Removes all friction from cargo-fuzz (which relies on the compiler's default unwind behaviour). Allows native consumers to unwind safely. It also aligns workspace behaviour with downstream reality: since Cargo ignores library profile settings, downstream binaries always had to define their own panic strategies anyway. This removes the illusion that our workspace profile was protecting consumer payload sizes.
* **Cons**: None.
* **Verdict**: Accepted.

---

## Decision

No `panic = "abort"` strategy specified at the workspace root `[profile.release]` block.

The panic strategy is left to the compiling consumer. Because the default Rust behaviour is to unwind, `cargo-fuzz` can successfully capture and report panics naturally without requiring any custom profile overrides, `.cargo/config.toml` hacks, or environment variables.

---

## Consequences

* **Positive**: Fuzzing works perfectly out-of-the-box (`cargo fuzz run`). The project adheres to strict Rust ecosystem standards by not imposing compiler profile opinions on downstream users of the library crates.
* **Neutral**: WASM Payload Configuration. Because Cargo ignores dependency `[profile]` settings, downstream consumers compiling `sdmx-rs` for WebAssembly have always been responsible for adding `panic = "abort"` to their top-level application manifests to achieve minimal payload footprints. This decision simply acknowledges that boundary. The fuzz crate remains entirely isolated from production dependencies.

---

## References

* [Cargo Fuzz Documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
* [Rust Reference: The `panic` setting](https://doc.rust-lang.org/cargo/reference/profiles.html#panic)
