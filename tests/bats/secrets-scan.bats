#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/ci/secrets-scan.sh
#
# Drives the secret-scan gate WITHOUT a real gitleaks run: a stub `gitleaks`
# (wired in via the GITLEAKS env var) stands in for the binary, letting us assert
# the script's contract deterministically:
#   - CI mode (GITHUB_ACTIONS=true) invokes `gitleaks detect` (full history),
#   - local mode (default) invokes `gitleaks protect` (working tree),
#   - a clean scan exits 0 and emits the success line,
#   - gitleaks' non-zero exit (a leak) is propagated and framed as a failure.
#
# Run with: bats tests/bats/secrets-scan.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    # Mirror the real scripts/ci + scripts/lib layout so the script's
    # `. "$(dirname "$0")/../lib/log.sh"` source resolves inside the fixture.
    mkdir -p ci lib
    cp "$BATS_TEST_DIRNAME/../../scripts/ci/secrets-scan.sh" ci/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    mkdir -p bin
    LOG="$BATS_TEST_TMPDIR/gitleaks-calls.log"
    export LOG
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# Write a stub `gitleaks` to bin/gitleaks. It logs each invocation's arguments to
# $LOG (one line per call) and exits with STUB_EXIT (default 0), so a test can
# simulate a leak (non-zero) without a real scan.
make_gitleaks_stub() {
    cat > bin/gitleaks <<'EOF'
#!/bin/sh
echo "$*" >> "$LOG"
exit "${STUB_EXIT:-0}"
EOF
    chmod +x bin/gitleaks
    export GITLEAKS="$BATS_TEST_TMPDIR/bin/gitleaks"
}

@test "secrets-scan: local mode runs 'gitleaks protect' on the working tree" {
    make_gitleaks_stub
    # run_isolated unsets GITHUB_ACTIONS, so this is the default (local) branch.
    run_isolated ./ci/secrets-scan.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"no secrets found"* ]]
    grep -q -- "protect" "$LOG"
    # `run !` (Bats >= 1.5): a bare `! grep` does NOT fail the test (SC2314), so
    # assert absence through the run harness. Placed AFTER the $output check
    # because `run` overwrites $output.
    run ! grep -q -- "detect" "$LOG"
}

@test "secrets-scan: CI mode runs 'gitleaks detect' over full history" {
    make_gitleaks_stub
    # --ci re-asserts the CI environment after isolation, so the script takes its
    # GITHUB_ACTIONS branch (full-history detect) through the same harness as
    # every other test — no bespoke `run sh` bypass.
    run_isolated --ci ./ci/secrets-scan.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    grep -q -- "detect --source . --redact --verbose" "$LOG"
    # `run !` (Bats >= 1.5): bare `! grep` would not fail the test (SC2314).
    run ! grep -q -- "protect" "$LOG"
}

@test "secrets-scan: clean scan exits 0 and frames success" {
    make_gitleaks_stub
    run_isolated ./ci/secrets-scan.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"Scanning for committed secrets"* ]]
    [[ "$output" == *"no secrets found"* ]]
}

@test "secrets-scan: a leak (gitleaks non-zero) fails the gate and propagates the code" {
    export STUB_EXIT=1
    make_gitleaks_stub
    run_isolated ./ci/secrets-scan.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"gitleaks reported findings"* ]]
}
