#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/run-bats.sh
#
# Testing approach: the script's job is to summarise a BATS TAP stream. We mock
# the BATS env hook with shims that emit canned TAP, so the summariser is
# exercised deterministically without a real (recursive) suite run. Covers:
#   - all-pass: success summary, exit 0
#   - failures: `not ok` lines + their `#` diagnostics passed through, exit 1
#   - skips: counted separately and reported, do not fail the run
#   - passing-test noise is suppressed (only the summary, no per-ok lines)
#   - default target defaults to tests/bats/ when no args are given
#
# Run with: bats tests/bats/run-bats.bats
# ==============================================================================

setup() {
    cd "$BATS_TEST_TMPDIR" || exit 1
    cp "$BATS_TEST_DIRNAME/../../scripts/run-bats.sh" .
    mkdir -p bin
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# Install a `bats` mock that prints $1 (a TAP stream) and exits $2 (default 0).
mock_bats() {
    _tap="$1"
    _exit="${2:-0}"
    {
        echo "#!/bin/sh"
        echo "cat <<'TAP'"
        printf '%s\n' "$_tap"
        echo "TAP"
        echo "exit $_exit"
    } > bin/bats
    chmod +x bin/bats
    export BATS="$BATS_TEST_TMPDIR/bin/bats"
}

@test "run-bats: all passing => success summary, exit 0" {
    mock_bats "1..2
ok 1 first
ok 2 second" 0

    run ./run-bats.sh tests/bats/
    [ "$status" -eq 0 ]
    [[ "$output" == *"✅ BATS tests: 2 passed"* ]]
}

@test "run-bats: passing-test lines are suppressed (only summary shown)" {
    mock_bats "1..2
ok 1 alpha
ok 2 beta" 0

    run ./run-bats.sh tests/bats/
    [ "$status" -eq 0 ]
    # The per-test 'ok' lines must NOT appear — only the digest.
    [[ "$output" != *"ok 1 alpha"* ]]
    [[ "$output" != *"ok 2 beta"* ]]
}

@test "run-bats: a failure exits 1 and reports the count" {
    mock_bats "1..2
ok 1 good
not ok 2 bad" 1

    run ./run-bats.sh tests/bats/
    [ "$status" -eq 1 ]
    [[ "$output" == *"❌ BATS tests: 1 failed, 1 passed"* ]]
}

@test "run-bats: failing test line and its # diagnostics are passed through" {
    mock_bats "1..1
not ok 1 explodes
# (in test file foo.bats, line 42)
#   \`[ 1 -eq 2 ]' failed" 1

    run ./run-bats.sh tests/bats/
    [ "$status" -eq 1 ]
    [[ "$output" == *"not ok 1 explodes"* ]]
    [[ "$output" == *"in test file foo.bats, line 42"* ]]
}

@test "run-bats: # diagnostics of a PASSING test are NOT shown" {
    mock_bats "1..1
ok 1 fine
# this trailing comment belongs to a passing context" 0

    run ./run-bats.sh tests/bats/
    [ "$status" -eq 0 ]
    [[ "$output" != *"trailing comment"* ]]
}

@test "run-bats: skips are counted and reported, run still succeeds" {
    mock_bats "1..3
ok 1 runs
ok 2 # skip not on this platform
ok 3 also runs" 0

    run ./run-bats.sh tests/bats/
    [ "$status" -eq 0 ]
    [[ "$output" == *"✅ BATS tests: 2 passed (1 skipped)"* ]]
}

@test "run-bats: failures with skips report both counts" {
    mock_bats "1..3
ok 1 ok
ok 2 # skip later
not ok 3 boom" 1

    run ./run-bats.sh tests/bats/
    [ "$status" -eq 1 ]
    [[ "$output" == *"1 failed"* ]]
    [[ "$output" == *"1 skipped"* ]]
}

@test "run-bats: defaults target to tests/bats/ when no args given" {
    # bats mock records its args so we can assert the default was applied.
    cat > bin/bats <<EOF
#!/bin/sh
echo "BATS_ARGS: \$*" >&2
echo "1..1"
echo "ok 1 noop"
EOF
    chmod +x bin/bats
    export BATS="$BATS_TEST_TMPDIR/bin/bats"

    run ./run-bats.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"BATS_ARGS: tests/bats/"* ]]
}
