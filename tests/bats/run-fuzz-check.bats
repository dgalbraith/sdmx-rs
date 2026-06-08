#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/run-fuzz-check.sh
#
# Drives the fuzz smoke-check WITHOUT a real cargo-fuzz run: a stub `cargo` (via
# CARGO) fakes the fuzz outcome and emits marker output, so we can assert the
# script's contract — especially its DEFINING behaviour, the buffer-and-flush-on-
# failure output policy:
#   - no target arg            -> exit 1, usage error,
#   - pass                     -> exit 0, success line, captured output SUPPRESSED,
#   - fail                     -> captured output FLUSHED, failure framed, exit propagated.
#
# Run with: bats tests/bats/run-fuzz-check.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$BATS_TEST_DIRNAME/../../scripts/run-fuzz-check.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" scripts/lib/

    mkdir -p bin
    LOG="$TMPDIR/cargo-calls.log"
    export LOG
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# Stub `cargo`. Logs the call, prints a recognisable marker to stdout (which the
# script captures into its temp buffer), and exits with STUB_FUZZ_EXIT (default 0).
# The marker lets a test assert whether the captured output was suppressed (pass)
# or flushed (fail).
make_cargo_stub() {
    cat > bin/cargo <<'EOF'
#!/bin/sh
echo "$*" >> "$LOG"
echo "CARGO_FUZZ_MARKER: this is captured fuzz output"
exit "${STUB_FUZZ_EXIT:-0}"
EOF
    chmod +x bin/cargo
    export CARGO="$TMPDIR/bin/cargo"
}

@test "run-fuzz-check: missing target argument fails with usage" {
    make_cargo_stub
    run_isolated ./scripts/run-fuzz-check.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"no fuzz target"* ]]
    # cargo must not have been invoked.
    run ! test -s "$LOG"
}

@test "run-fuzz-check: passing target is quiet and SUPPRESSES captured output" {
    make_cargo_stub
    run_isolated ./scripts/run-fuzz-check.sh parse_xml
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    grep -q -- "fuzz run parse_xml -- -max_total_time=10" "$LOG"
    [[ "$output" == *"compiled and survived"* ]]
    # The buffered cargo-fuzz output must NOT appear on a pass.
    [[ "$output" != *"CARGO_FUZZ_MARKER"* ]]
}

@test "run-fuzz-check: failing target FLUSHES captured output and propagates exit" {
    export STUB_FUZZ_EXIT=1
    make_cargo_stub
    run_isolated ./scripts/run-fuzz-check.sh parse_xml
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    # On failure the captured output IS shown, above the failure line.
    [[ "$output" == *"CARGO_FUZZ_MARKER"* ]]
    [[ "$output" == *"failed its 10s smoke check"* ]]
}
