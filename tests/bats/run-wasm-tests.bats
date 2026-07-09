#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/run-wasm-tests.sh
#
# Drives the WASM test runner WITHOUT a real wasm-pack/Node run: a stub
# `wasm-pack` (via WASM_PACK) fakes each crate's libtest output, so we can
# assert the script's contract — its buffer-and-flush-on-failure output policy:
#   - all crates pass -> exit 0, aggregate "<N> passed in <T>s across 3 crates",
#                        per-crate libtest output SUPPRESSED,
#   - a crate fails   -> that crate NAMED, its output FLUSHED, exit 1, and the
#                        run stops (later crates are never invoked).
#
# Run with: bats tests/bats/run-wasm-tests.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$BATS_TEST_DIRNAME/../../scripts/run-wasm-tests.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" scripts/lib/

    mkdir -p bin
    LOG="$BATS_TEST_TMPDIR/wasm-pack-calls.log"
    export LOG
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# Stub `wasm-pack`. Logs the call, emits a canned libtest result line (3 passed
# in 0.05s by default), and fails for the crate named in STUB_FAIL_CRATE. Set
# STUB_OK_LINE to override the passing-exit output (used to fake a zero-test or
# unparseable result while keeping wasm-pack's own exit status 0).
make_wasm_pack_stub() {
    cat > bin/wasm-pack <<'EOF'
#!/bin/sh
echo "$*" >> "$LOG"
crate=""
for a in "$@"; do case "$a" in crates/*) crate=${a#crates/} ;; esac; done
if [ "$crate" = "${STUB_FAIL_CRATE:-}" ]; then
    echo "test some::thing ... FAILED"
    echo "error: test failed"
    exit 1
fi
echo "${STUB_OK_LINE:-test result: ok. 3 passed; 0 failed; 0 ignored; 0 filtered out; finished in 0.05s}"
EOF
    chmod +x bin/wasm-pack
    export WASM_PACK="$BATS_TEST_TMPDIR/bin/wasm-pack"
}

@test "run-wasm-tests: all crates pass -> aggregate summary, per-crate output suppressed" {
    make_wasm_pack_stub
    run_isolated ./scripts/run-wasm-tests.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    # 3 crates x 3 passed = 9; 3 x 0.05s = 0.15s.
    [[ "$output" == *"9 passed in 0.15s across 3 crates"* ]]
    # The per-crate libtest result lines are NOT shown on a pass.
    [[ "$output" != *"test result: ok"* ]]
    # All three crates were invoked, each with the noise-suppressing flags.
    grep -q -- "crates/sdmx-types" "$LOG"
    grep -q -- "crates/sdmx-parsers" "$LOG"
    grep -q -- "crates/sdmx-writers" "$LOG"
    grep -q -- "--log-level warn" "$LOG"
    grep -q -- "--lib" "$LOG"
}

@test "run-wasm-tests: a failing crate is NAMED, its output FLUSHED, exit 1" {
    export STUB_FAIL_CRATE=sdmx-parsers
    make_wasm_pack_stub
    run_isolated ./scripts/run-wasm-tests.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"sdmx-parsers failed under Node/V8"* ]]
    # On failure the captured output IS flushed.
    [[ "$output" == *"error: test failed"* ]]
    # The run stops at the failing crate: sdmx-writers (after parsers) never runs.
    run ! grep -q -- "crates/sdmx-writers" "$LOG"
}

@test "run-wasm-tests: a crate executing zero tests is NAMED, exit 1" {
    export STUB_OK_LINE="test result: ok. 0 passed; 0 failed; 0 ignored; 0 filtered out; finished in 0.00s"
    make_wasm_pack_stub
    run_isolated ./scripts/run-wasm-tests.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    # Fails at the first crate, named, before any aggregate summary.
    [[ "$output" == *"sdmx-types executed zero tests under Node/V8"* ]]
    [[ "$output" != *"across 3 crates"* ]]
}

@test "run-wasm-tests: unparseable wasm-pack output is NAMED, exit 1" {
    export STUB_OK_LINE="running 0 tests -- no libtest result line here"
    make_wasm_pack_stub
    run_isolated ./scripts/run-wasm-tests.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"sdmx-types emitted no recognisable libtest result line"* ]]
    [[ "$output" != *"across 3 crates"* ]]
}
