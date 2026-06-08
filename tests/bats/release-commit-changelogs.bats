#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/release-commit-changelogs.sh
#
# Drives the changelog-checkpoint commit WITHOUT a real (signed) commit: a stub
# git (via GIT) records its argv and fakes the outcome, and DATE is stubbed for a
# deterministic message. Contract asserted:
#   - stages crates/*/CHANGELOG.md then commits with the signed checkpoint message,
#   - the commit message carries the (stubbed) date byte-for-byte,
#   - a git failure (e.g. nothing staged) propagates and is framed, not bare.
#
# Run with: bats tests/bats/release-commit-changelogs.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$BATS_TEST_DIRNAME/../../scripts/release-commit-changelogs.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/common.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" scripts/lib/

    # A changelog for the glob to expand onto (the stub git doesn't read it, but
    # `git add crates/*/CHANGELOG.md` must glob onto a real path).
    mkdir -p crates/sdmx-types
    echo "# Changelog" > crates/sdmx-types/CHANGELOG.md

    mkdir -p bin
    GIT_LOG="$TMPDIR/git-calls.log"
    export GIT_LOG
    # Deterministic date so the commit message is byte-assertable.
    export DATE="echo 2026-06-04"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# Stub `git`: log argv, exit STUB_GIT_EXIT (default 0). A non-zero exit models a
# real git failure (e.g. `git commit` with nothing staged).
make_git_stub() {
    cat > bin/git <<'EOF'
#!/bin/sh
echo "$*" >> "$GIT_LOG"
exit "${STUB_GIT_EXIT:-0}"
EOF
    chmod +x bin/git
    export GIT="$TMPDIR/bin/git"
}

@test "release-commit-changelogs: stages changelogs then commits with signed checkpoint message" {
    make_git_stub
    run_isolated ./scripts/release-commit-changelogs.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    # Staged the changelog glob.
    grep -q -- "add crates/sdmx-types/CHANGELOG.md" "$GIT_LOG"
    # Committed with the signed checkpoint message carrying the stubbed date.
    grep -q -- "commit --gpg-sign -m chore(release): prepare release batch for 2026-06-04" "$GIT_LOG"
    [[ "$output" == *"committed as a signed checkpoint"* ]]
}

@test "release-commit-changelogs: a git failure propagates and is framed" {
    export STUB_GIT_EXIT=1
    make_git_stub
    run_isolated ./scripts/release-commit-changelogs.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    # git's exit code propagates (set -e); the success line must NOT appear.
    [ "$status" -ne 0 ]
    [[ "$output" != *"committed as a signed checkpoint"* ]]
}
