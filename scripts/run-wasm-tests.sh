#!/bin/sh
# ===================================================================
# run-wasm-tests.sh
#
# Executes the curated WASM test subset of the three no_std leaf crates
# (sdmx-types, sdmx-parsers, sdmx-writers) under Node/V8 via
# `wasm-pack test --node`, and reports one framed summary line.
#
# ADR-0007 mandates that the no_std crates not only COMPILE for wasm32
# (check-wasm) but EXECUTE there, to catch a native/WASM behavioural
# divergence or a target-specific panic that a compile check cannot. Only
# the `#[wasm_bindgen_test]`-annotated subset runs (execution parity, not
# the whole suite); the property suite is wasm-excluded by design.
#
# Output:
#   success — test-wasm: <N> passed in <T>s across <M> crates (WASM/Node)
#   failure — the offending crate is named, its full output printed, exit 1
#
# `--log-level warn` + `--lib` suppress the wasm-pack/cargo progress noise
# that has no quiet flag (the wasm-bindgen-test runner rejects `--quiet`);
# each crate's output is captured and shown only on failure. <T> sums the
# libtest run times (test execution, excluding compile).
#
# POSIX sh; awk sums the float times (sh arithmetic is integer-only).
#
# Environment:
#   WASM_PACK  wasm-pack invocation to use (default: wasm-pack) — test indirection.
#
# Exit codes:
#   0 — every crate's subset executed and passed under Node/V8
#   1 — a crate failed to build, or a test failed
# ===================================================================

set -u

# Source shared loggers
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

# wasm-pack invocation to use (default: wasm-pack) — indirection for tests.
WASM_PACK="${WASM_PACK:-wasm-pack}"

log_section "Executing the WASM test subset under Node/V8..."

total=0
count=0
times=

for crate in sdmx-types sdmx-parsers sdmx-writers; do
    if ! output=$("$WASM_PACK" --log-level warn test --node "crates/${crate}" --lib 2>&1); then
        log_err "test-wasm: ${crate} failed under Node/V8"
        printf '%s\n' "$output"
        exit 1
    fi
    passed=$(printf '%s\n' "$output" | sed -n 's/^test result: ok\. \([0-9]*\) passed.*/\1/p')
    # A missing count means the libtest result line did not parse (a wasm-pack
    # output-format change), and a zero count means the subset executed nothing.
    # Either silently voids the gate — the point is that this code EXECUTES on
    # wasm32 — so both are hard failures, naming the crate like the branch above.
    if [ -z "$passed" ]; then
        log_err "test-wasm: ${crate} emitted no recognisable libtest result line (wasm-pack output format changed?)"
        printf '%s\n' "$output"
        exit 1
    fi
    if [ "$passed" -lt 1 ]; then
        log_err "test-wasm: ${crate} executed zero tests under Node/V8 (empty WASM subset)"
        printf '%s\n' "$output"
        exit 1
    fi
    secs=$(printf '%s\n' "$output" | sed -n 's/^test result:.*finished in \([0-9.]*\)s.*/\1/p')
    total=$((total + passed))
    count=$((count + 1))
    times="${times} ${secs:-0}"
done

# sh arithmetic is integer-only; awk sums the per-crate float run times.
elapsed=$(printf '%s\n' "$times" | awk '{ s = 0; for (i = 1; i <= NF; i++) s += $i; printf "%.2f", s }')

log_ok "test-wasm: ${total} passed in ${elapsed}s across ${count} crates (WASM/Node)"
