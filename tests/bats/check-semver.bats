#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/check-semver.sh
#
# The gate decides APPLICABILITY from local crate versions (cargo metadata), not
# from a crates.io probe. The probe is consulted only post-1.0, to confirm a
# baseline, and a probe FAILURE there is fatal (fail closed). A stub `cargo` (via
# the CARGO env var) fakes metadata versions, the search probe, and the
# semver-checks outcome, so every branch is deterministic and offline:
#
#   - pre-1.0 (max major 0)          -> warn-skip, exit 0, network NOT touched,
#   - 1.0+ + baseline + clean        -> runs the check, success, exit 0,
#   - 1.0+ + baseline + violation    -> propagates the non-zero exit,
#   - 1.0.0-rc (major 1) + baseline  -> MANDATORY: runs the check (rc == 1.0),
#   - 1.0+ + probe FAILURE           -> FATAL (the silent-bypass regression),
#   - 1.0+ + no baseline, no opt-in  -> fatal,
#   - 1.0+ + no baseline + opt-in    -> explicit warn-skip, exit 0,
#   - unreadable metadata            -> fatal (refuses to guess the phase).
#
# Run with: bats tests/bats/check-semver.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    # Mirror scripts/ + scripts/lib so the script's `. .../lib/log.sh` resolves.
    mkdir -p scripts/lib
    cp "$BATS_TEST_DIRNAME/../../scripts/check-semver.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" scripts/lib/

    mkdir -p bin
    LOG="$BATS_TEST_TMPDIR/cargo-calls.log"
    export LOG
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# Stub `cargo`. Logs each call to $LOG. Behaviour tuned by env vars read at call
# time:
#   STUB_VERSION      crate version reported by `cargo metadata` for every sdmx-*
#                     crate (default "0.1.0" → pre-1.0).
#   STUB_META_FAIL    if "1", `cargo metadata` exits non-zero (unreadable).
#   STUB_PUBLISHED    if "1", `cargo search` emits the probe crate (baseline
#                     exists); otherwise prints nothing (no baseline).
#   STUB_SEARCH_FAIL  if "1", `cargo search` exits non-zero (probe failure).
#   STUB_SEMVER_EXIT  exit code for `cargo semver-checks check-release` (def 0).
make_cargo_stub() {
    cat > bin/cargo <<'EOF'
#!/bin/sh
echo "$*" >> "$LOG"

case "$*" in
    "metadata "*)
        if [ "${STUB_META_FAIL:-}" = "1" ]; then
            echo "error: could not read metadata" >&2
            exit 1
        fi
        ver="${STUB_VERSION:-0.1.0}"
        # Minimal cargo-metadata JSON: the sdmx-* crates the script filters on.
        printf '{"packages":['
        printf '{"name":"sdmx-types","version":"%s"},' "$ver"
        printf '{"name":"sdmx-parsers","version":"%s"},' "$ver"
        printf '{"name":"sdmx-rs","version":"%s"}' "$ver"
        printf ']}\n'
        exit 0
        ;;
    "search "*)
        if [ "${STUB_SEARCH_FAIL:-}" = "1" ]; then
            echo "error: connection failed" >&2
            exit 1
        fi
        if [ "${STUB_PUBLISHED:-}" = "1" ]; then
            echo 'sdmx-types = "1.0.0"    # SDMX core types'
        fi
        exit 0
        ;;
    "semver-checks "*)
        exit "${STUB_SEMVER_EXIT:-0}"
        ;;
esac
exit 0
EOF
    chmod +x bin/cargo
    export CARGO="$BATS_TEST_TMPDIR/bin/cargo"
}

@test "check-semver: pre-1.0 warn-skips and never touches the network" {
    export STUB_VERSION="0.1.0"
    make_cargo_stub
    run_isolated ./scripts/check-semver.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"SKIPPED"* ]]
    [[ "$output" == *"pre-1.0"* ]]
    # No probe and no real check pre-1.0.
    run ! grep -q -- "search " "$LOG"
    run ! grep -q -- "semver-checks" "$LOG"
}

@test "check-semver: 1.0+ with baseline + clean runs the check and passes" {
    export STUB_VERSION="1.0.0"
    export STUB_PUBLISHED=1
    make_cargo_stub
    run_isolated ./scripts/check-semver.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    grep -q -- "semver-checks check-release" "$LOG"
    [[ "$output" == *"no semver violations"* ]]
}

@test "check-semver: 1.0+ with baseline + violation propagates the failure" {
    export STUB_VERSION="1.2.0"
    export STUB_PUBLISHED=1
    export STUB_SEMVER_EXIT=1
    make_cargo_stub
    run_isolated ./scripts/check-semver.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"reported a violation"* ]]
}

@test "check-semver: 1.0.0-rc is treated as 1.0 (mandatory), runs the check" {
    # The rc cycle IS the 1.0 cycle: major 1 → mandatory, not a pre-1.0 skip.
    export STUB_VERSION="1.0.0-rc.1"
    export STUB_PUBLISHED=1
    make_cargo_stub
    run_isolated ./scripts/check-semver.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    grep -q -- "semver-checks check-release" "$LOG"
    [[ "$output" != *"SKIPPED"* ]]
}

@test "check-semver: REGRESSION — 1.0+ probe FAILURE is fatal, not a silent skip" {
    export STUB_VERSION="1.0.0"
    export STUB_SEARCH_FAIL=1
    make_cargo_stub
    run_isolated ./scripts/check-semver.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"could not query crates.io"* ]]
    [[ "$output" != *"SKIPPED"* ]]
    # Must NOT have run the check, and must NOT have passed.
    run ! grep -q -- "semver-checks" "$LOG"
}

@test "check-semver: 1.0+ with no baseline and no opt-in fails closed" {
    export STUB_VERSION="1.0.0"
    # STUB_PUBLISHED unset → search succeeds but finds nothing.
    make_cargo_stub
    run_isolated ./scripts/check-semver.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"no published baseline"* ]]
}

@test "check-semver: 1.0+ no baseline + explicit opt-in warn-skips (first-1.0 bootstrap)" {
    export STUB_VERSION="1.0.0"
    export SEMVER_ALLOW_NO_BASELINE=1
    make_cargo_stub
    run_isolated ./scripts/check-semver.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"SKIPPED"* ]]
    [[ "$output" == *"SEMVER_ALLOW_NO_BASELINE"* ]]
    # Opt-in skips the check itself, never runs semver-checks.
    run ! grep -q -- "semver-checks" "$LOG"
}

@test "check-semver: opt-in does NOT suppress a probe failure (still fatal)" {
    # 'I could not check' is not 'there is nothing to check': the escape hatch
    # must not turn a network error into a skip.
    export STUB_VERSION="1.0.0"
    export STUB_SEARCH_FAIL=1
    export SEMVER_ALLOW_NO_BASELINE=1
    make_cargo_stub
    run_isolated ./scripts/check-semver.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"could not query crates.io"* ]]
}

@test "check-semver: unreadable metadata is fatal (refuses to guess the phase)" {
    export STUB_META_FAIL=1
    make_cargo_stub
    run_isolated ./scripts/check-semver.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"could not read workspace crate versions"* ]]
}
