#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/update-flake.sh
#
# Testing approach: behaviour contract of the flake.lock refresh workflow, with
# nix and just mocked (via the NIX/JUST env hooks) so tests are fast, offline,
# and deterministic. Focus is the git-state logic the script owns — especially
# the staging dance:
#   - pre-flight dirty guard
#   - no-op short-circuit (no "Updated input" => skip validation, exit 0)
#   - flake.lock STAGED while verify-infra runs (Nix sandbox needs it tracked)
#   - flake.lock left UNSTAGED afterward, on BOTH success and validation failure
#     (the unstage fires from the EXIT trap)
#   - working-tree change preserved throughout (never reverted)
#
# Run with: bats tests/bats/update-flake.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    cp "$BATS_TEST_DIRNAME/../../scripts/update-flake.sh" .
    # update-flake.sh sources `$(dirname "$0")/lib/log.sh`; mirror that layout.
    mkdir -p lib
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"

    printf 'OLD-LOCK\n' > flake.lock
    echo "bin/" > .gitignore
    git add flake.lock .gitignore update-flake.sh lib/log.sh
    git commit -m "baseline" -q

    mkdir -p bin
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# Mock `nix`: on `flake update`, print $1 and mutate flake.lock to NEW content
# (so there is a real working-tree change to stage/unstage), unless $1 is empty.
mock_nix() {
    cat > bin/nix <<EOF
#!/bin/sh
printf '%s\n' "$1"
printf 'NEW-LOCK\n' > flake.lock
EOF
    chmod +x bin/nix
    export NIX="$BATS_TEST_TMPDIR/bin/nix"
}

# Mock `nix` that does NOT change the lock (no-op case).
mock_nix_noop() {
    cat > bin/nix <<'EOF'
#!/bin/sh
echo "Warning: Git tree is dirty"
EOF
    chmod +x bin/nix
    export NIX="$BATS_TEST_TMPDIR/bin/nix"
}

teardown_file() { :; }

@test "update-flake: refuses to run when flake.lock is dirty" {
    mock_nix "• Updated input 'nixpkgs'"
    printf 'DIRTY\n' > flake.lock

    run ./update-flake.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"already has uncommitted changes"* ]]
}

@test "update-flake: no-op (no Updated input) skips validation, exits 0" {
    mock_nix_noop
    cat > bin/just <<'EOF'
#!/bin/sh
echo "JUST_CALLED" >> just.log
EOF
    chmod +x bin/just
    export JUST="$BATS_TEST_TMPDIR/bin/just"

    run ./update-flake.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"(none"* ]]
    [ ! -f "$BATS_TEST_TMPDIR/just.log" ]
}

@test "update-flake: success leaves flake.lock UNSTAGED with the new content" {
    mock_nix "• Updated input 'nixpkgs'"
    # just mock asserts the lock IS staged at validation time (sandbox requirement)
    cat > bin/just <<EOF
#!/bin/sh
git diff --cached --quiet -- flake.lock && echo "NOT-STAGED-AT-VALIDATE" || echo "STAGED-AT-VALIDATE"
exit 0
EOF
    chmod +x bin/just
    export JUST="$BATS_TEST_TMPDIR/bin/just"

    run ./update-flake.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"STAGED-AT-VALIDATE"* ]]      # staged during verify-infra
    [[ "$output" == *"Updated input 'nixpkgs'"* ]]

    # After the run: working tree has NEW content, but it is NOT staged.
    [ "$(cat flake.lock)" = "NEW-LOCK" ]
    git diff --cached --quiet -- flake.lock        # index clean (unstaged)
    ! git diff --quiet -- flake.lock               # worktree differs from HEAD
}

@test "update-flake: validation FAILURE still leaves flake.lock unstaged (trap)" {
    mock_nix "• Updated input 'nixpkgs'"
    cat > bin/just <<'EOF'
#!/bin/sh
exit 5
EOF
    chmod +x bin/just
    export JUST="$BATS_TEST_TMPDIR/bin/just"

    run ./update-flake.sh
    [ "$status" -ne 0 ]
    # Even on failure, the EXIT trap must have unstaged the lock...
    git diff --cached --quiet -- flake.lock
    # ...without reverting the working-tree change.
    [ "$(cat flake.lock)" = "NEW-LOCK" ]
}
