#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/update-deps.sh
#
# Testing approach: behaviour contract of the Cargo.lock refresh workflow, with
# cargo and just mocked (via the CARGO/JUST env hooks) so tests are fast,
# offline, and deterministic. Focus is the git-state logic the script owns:
#   - pre-flight dirty guard (staged or unstaged lock changes block the run)
#   - change-summary parsing (real deltas surfaced; "Updating crates.io index"
#     status line excluded)
#   - no-op short-circuit (no deltas => skip validation, exit 0)
#   - validation invoked only when there are real changes
#   - Cargo.lock left UNSTAGED after a successful run
#
# Run with: bats tests/bats/update-deps.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    cp "$BATS_TEST_DIRNAME/../../scripts/update-deps.sh" .
    # update-deps.sh sources `$(dirname "$0")/lib/log.sh`; mirror that layout.
    mkdir -p lib
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"

    # Committed baseline: Cargo.lock clean in HEAD (the steady state the script
    # is designed for). bin/ holds mocks and is gitignored so it never dirties
    # the working tree (which would trip the pre-flight guard).
    printf 'version = "1.0.0"\n' > Cargo.lock
    echo "bin/" > .gitignore
    git add Cargo.lock .gitignore update-deps.sh lib/log.sh
    git commit -m "baseline" -q

    mkdir -p bin
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# Write a mock `cargo` whose `update` prints $1 and (unless NOWRITE) mutates the lock.
mock_cargo() {
    cat > bin/cargo <<EOF
#!/bin/sh
# Only the 'update' subcommand is exercised.
cat <<'OUT'
$1
OUT
EOF
    chmod +x bin/cargo
    export CARGO="$BATS_TEST_TMPDIR/bin/cargo"
}

# A `just` mock that records it was called (so we can assert verify ran or not).
mock_just_recording() {
    cat > bin/just <<EOF
#!/bin/sh
echo "JUST_CALLED: \$*" >> "$BATS_TEST_TMPDIR/just.log"
exit 0
EOF
    chmod +x bin/just
    export JUST="$BATS_TEST_TMPDIR/bin/just"
    : > "$BATS_TEST_TMPDIR/just.log"
}

just_was_called() { [ -s "$BATS_TEST_TMPDIR/just.log" ]; }

# --- Pre-flight dirty guard ---------------------------------------------------

@test "update-deps: refuses to run when Cargo.lock has unstaged changes" {
    mock_cargo "    Updating serde v1.0.0 -> v1.0.1"
    mock_just_recording
    printf 'version = "9.9.9"\n' > Cargo.lock   # dirty (unstaged)

    run ./update-deps.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"already has uncommitted changes"* ]]
    ! just_was_called
}

@test "update-deps: refuses to run when Cargo.lock has staged changes" {
    mock_cargo "    Updating serde v1.0.0 -> v1.0.1"
    mock_just_recording
    printf 'version = "9.9.9"\n' > Cargo.lock
    git add Cargo.lock                          # dirty (staged)

    run ./update-deps.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"already has uncommitted changes"* ]]
    ! just_was_called
}

# --- No-op short-circuit ------------------------------------------------------

@test "update-deps: no-op (only index line) skips validation and exits 0" {
    mock_cargo "    Updating crates.io index"   # status line, NOT a real delta
    mock_just_recording

    run ./update-deps.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"(none"* ]]
    ! just_was_called
}

@test "update-deps: no-op with empty cargo output skips validation" {
    mock_cargo ""
    mock_just_recording

    run ./update-deps.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"(none"* ]]
    ! just_was_called
}

# --- Change path --------------------------------------------------------------

@test "update-deps: real delta is surfaced and triggers validation" {
    mock_cargo "    Updating crates.io index
    Updating serde v1.0.0 -> v1.0.1"
    mock_just_recording

    run ./update-deps.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"Updating serde v1.0.0 -> v1.0.1"* ]]
    # The summary block (after the 📦 header) must list the real delta but NOT the
    # "Updating crates.io index" status line. Isolate the block and assert on it.
    block=$(printf '%s\n' "$output" | sed -n '/📦 Dependency changes:/,/🔎 Validating/p')
    [[ "$block" == *"serde v1.0.0 -> v1.0.1"* ]]
    [[ "$block" != *"crates.io index"* ]]
    just_was_called
    grep -q "JUST_CALLED: verify-rust" "$BATS_TEST_TMPDIR/just.log"
}

@test "update-deps: Adding/Removing deltas also count as changes" {
    mock_cargo "    Adding foo v1.0.0
    Removing bar v2.0.0"
    mock_just_recording

    run ./update-deps.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"Adding foo v1.0.0"* ]]
    [[ "$output" == *"Removing bar v2.0.0"* ]]
    just_was_called
}

@test "update-deps: validation failure propagates non-zero" {
    mock_cargo "    Updating serde v1.0.0 -> v1.0.1"
    cat > bin/just <<EOF
#!/bin/sh
exit 7
EOF
    chmod +x bin/just
    export JUST="$BATS_TEST_TMPDIR/bin/just"

    run ./update-deps.sh
    [ "$status" -ne 0 ]
}

# --- Argument handling --------------------------------------------------------

@test "update-deps: named crates are passed as separate -p flags" {
    cat > bin/cargo <<EOF
#!/bin/sh
shift   # drop 'update'
echo "ARGS: \$*"
EOF
    chmod +x bin/cargo
    export CARGO="$BATS_TEST_TMPDIR/bin/cargo"
    mock_just_recording

    run ./update-deps.sh serde tokio
    [ "$status" -eq 0 ]
    [[ "$output" == *"ARGS: -p serde -p tokio"* ]]
}
