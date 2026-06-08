#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/run-fmt.sh
#
# Exercises the guard logic WITHOUT running a real format: a stub `cargo` (via
# the CARGO env var) stands in for `cargo fmt`, so the testable contract is the
# RUSTFMT guard, not rustfmt's behaviour:
#   - RUSTFMT unset  -> hard-fail (exit 1), `❌ Error:`, and cargo fmt MUST NOT run,
#   - RUSTFMT set    -> runs `cargo fmt -- --config-path rustfmt.toml`, exit 0.
#
# Run with: bats tests/bats/run-fmt.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$BATS_TEST_DIRNAME/../../scripts/run-fmt.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" scripts/lib/

    mkdir -p bin
    LOG="$TMPDIR/cargo-calls.log"
    export LOG
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# Stub `cargo` — logs each call, exits 0. The guard runs BEFORE cargo, so for the
# unset-RUSTFMT test the stub should never be invoked (asserted via $LOG).
make_cargo_stub() {
    cat > bin/cargo <<'EOF'
#!/bin/sh
echo "$*" >> "$LOG"
exit 0
EOF
    chmod +x bin/cargo
    export CARGO="$TMPDIR/bin/cargo"
}

@test "run-fmt: missing RUSTFMT hard-fails and does not run cargo fmt" {
    make_cargo_stub
    unset RUSTFMT
    # run_isolated does not touch RUSTFMT; unset above guarantees the guard trips.
    run_isolated ./scripts/run-fmt.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error:"* ]]
    [[ "$output" == *"RUSTFMT"* ]]
    # cargo fmt must NOT have been reached — the log file should not exist or be empty.
    run ! test -s "$LOG"
}

@test "run-fmt: with RUSTFMT set, runs cargo fmt with the project config" {
    make_cargo_stub
    export RUSTFMT="/fake/nightly/rustfmt"
    run_isolated ./scripts/run-fmt.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    grep -q -- "fmt -- --config-path rustfmt.toml" "$LOG"
    [[ "$output" == *"formatted with nightly rustfmt"* ]]
}
