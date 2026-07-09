#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/ci/run-coverage.sh
#
# Validates the coverage gate's contract WITHOUT a real coverage run: a stub
# `cargo` (wired in via the CARGO env var) stands in for `cargo llvm-cov`,
# letting us drive every branch deterministically and fast. The behaviours under
# test are the gate's promises:
#   - lcov.info is generated UNCONDITIONALLY (even when the test run fails),
#   - a test-run failure is re-raised with cargo's exit code (after lcov.info),
#   - every per-crate floor is checked, with the right threshold,
#   - a crate below its floor fails the gate.
#
# Run with: bats tests/bats/run-coverage.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    # Mirror the real scripts/ci + scripts/lib layout so the script's
    # `. "$(dirname "$0")/../lib/log.sh"` source resolves inside the fixture.
    mkdir -p ci lib
    cp "$BATS_TEST_DIRNAME/../../scripts/ci/run-coverage.sh" ci/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    mkdir -p bin
    LOG="$BATS_TEST_TMPDIR/cargo-calls.log"
    export LOG
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# Write a stub `cargo` to bin/cargo. It logs each invocation's arguments to $LOG
# (one line per call) and, on `llvm-cov report --lcov`, creates the output file
# named by --output-path so the script's "report was produced" expectation holds.
#
# Behaviour is tuned by two env vars read at call time:
#   STUB_TEST_EXIT  exit code for the `llvm-cov --no-report` test run (default 0)
#   STUB_FAIL_CRATE crate name whose `--fail-under-lines` check should exit 1
#                   (simulates a below-floor crate); empty => all pass
make_cargo_stub() {
    cat > bin/cargo <<'EOF'
#!/bin/sh
echo "$*" >> "$LOG"

# The instrumented test run: `llvm-cov nextest --workspace ... --no-report`
case "$*" in
    *"llvm-cov nextest --workspace"*"--no-report"*)
        exit "${STUB_TEST_EXIT:-0}"
        ;;
esac

# The lcov report: `llvm-cov report --lcov --output-path <file>`
case "$*" in
    *"llvm-cov report --lcov"*)
        out=""
        prev=""
        for a in "$@"; do
            [ "$prev" = "--output-path" ] && out="$a"
            prev="$a"
        done
        [ -n "$out" ] && : > "$out"
        exit 0
        ;;
esac

# A per-crate floor check: `llvm-cov report --package <crate> --fail-under-lines N`
case "$*" in
    *"llvm-cov report --package"*"--fail-under-lines"*)
        prev=""
        crate=""
        for a in "$@"; do
            [ "$prev" = "--package" ] && crate="$a"
            prev="$a"
        done
        if [ -n "${STUB_FAIL_CRATE:-}" ] && [ "$crate" = "$STUB_FAIL_CRATE" ]; then
            echo "error: $crate below floor" >&2
            exit 1
        fi
        exit 0
        ;;
esac

exit 0
EOF
    chmod +x bin/cargo
    export CARGO="$BATS_TEST_TMPDIR/bin/cargo"
}

@test "run-coverage: happy path passes and generates lcov.info" {
    make_cargo_stub
    run_isolated ./ci/run-coverage.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [ -f lcov.info ]
}

@test "run-coverage: generates lcov.info even when the test run fails" {
    # STUB_TEST_EXIT is read by the stub at runtime, so it must be exported into
    # the script's environment (not just set when the stub file is written).
    export STUB_TEST_EXIT=101
    make_cargo_stub
    run_isolated ./ci/run-coverage.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    # lcov.info must exist despite the failing test run.
    [ -f lcov.info ]
    # The gate must still fail, propagating cargo's exit code (101).
    [ "$status" -eq 101 ]
    [[ "$output" == *"lcov.info was still generated"* ]]
}

@test "run-coverage: skips floor checks when the test run fails" {
    export STUB_TEST_EXIT=101
    make_cargo_stub
    run_isolated ./ci/run-coverage.sh
    echo "OUTPUT: $output" >&2
    # The report was written, but no per-crate floor check should have run,
    # since the re-raise happens before the floor loop.
    run grep -c -- "--fail-under-lines" "$LOG"
    [ "$output" -eq 0 ]
}

@test "run-coverage: unified report is suppressed by default" {
    # Default (COVERAGE_REPORT unset): gate-only, no human-readable table — this
    # is the `verify` path, where the table is pure noise. The lcov report
    # (`report --lcov`) is still written; only the bare summary `report` is gated.
    make_cargo_stub
    run_isolated ./ci/run-coverage.sh
    [ "$status" -eq 0 ]
    [ -f lcov.info ]
    ! grep -qx "llvm-cov report" "$LOG"
}

@test "run-coverage: unified report prints on the sad path when COVERAGE_REPORT=1" {
    # With the table requested, the unified summary (a bare `report`, no
    # --lcov/--package) runs even when the suite fails — it prints above the
    # "tests failed" message so collected coverage aids diagnosis of the break.
    export COVERAGE_REPORT=1
    export STUB_TEST_EXIT=101
    make_cargo_stub
    run_isolated ./ci/run-coverage.sh
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 101 ]
    grep -qx "llvm-cov report" "$LOG"
}

@test "run-coverage: checks every crate floor with the correct threshold" {
    make_cargo_stub
    run_isolated ./ci/run-coverage.sh
    [ "$status" -eq 0 ]

    # Exactly the five workspace crates, each at its documented floor.
    grep -q -- "--package sdmx-types --fail-under-lines 85"   "$LOG"
    grep -q -- "--package sdmx-writers --fail-under-lines 80" "$LOG"
    grep -q -- "--package sdmx-client --fail-under-lines 80"  "$LOG"
    grep -q -- "--package sdmx-parsers --fail-under-lines 75" "$LOG"
    grep -q -- "--package sdmx-rs --fail-under-lines 70"      "$LOG"

    # No more, no fewer than five floor checks.
    run grep -c -- "--fail-under-lines" "$LOG"
    [ "$output" -eq 5 ]
}

@test "run-coverage: fails the gate when a crate is below its floor" {
    export STUB_FAIL_CRATE="sdmx-parsers"
    make_cargo_stub
    run_isolated ./ci/run-coverage.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    # lcov.info is still present — the floor check runs after report generation.
    [ -f lcov.info ]
}

@test "run-coverage: a below-floor crate aborts before later crates are checked" {
    # sdmx-types is first in the floor list; if it fails, sdmx-rs (last) must
    # never be reached (set -e aborts on the first failing floor).
    export STUB_FAIL_CRATE="sdmx-types"
    make_cargo_stub
    run_isolated ./ci/run-coverage.sh
    [ "$status" -eq 1 ]
    grep -q -- "--package sdmx-types --fail-under-lines 85" "$LOG"
    ! grep -q -- "--package sdmx-rs --fail-under-lines 70"  "$LOG"
}
