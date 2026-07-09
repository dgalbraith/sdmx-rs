#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/guard-main-push.sh
#
# The guard is a pre-push hook: pre-commit hands it the push destination and SHA
# via PRE_COMMIT_REMOTE_BRANCH / PRE_COMMIT_TO_REF. These tests drive it by
# setting those variables directly against a real local repo with a bare origin,
# so no pre-commit install or actual network push is needed.
#
# Run with: bats tests/bats/guard-main-push.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    # The script sources lib/log.sh relative to its own directory — mirror that
    # layout in the fixture (as release-merge.bats does).
    cp "$BATS_TEST_DIRNAME/../../scripts/guard-main-push.sh" .
    mkdir -p lib
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    # Minimal repo with one commit on main, pushed to a bare origin.
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"
    git config commit.gpgsign false
    git config tag.gpgsign false

    touch README.md
    git add README.md
    git commit -m "initial commit" -q

    ORIGIN_DIR=$(mktemp -d)
    git init --bare -q "$ORIGIN_DIR"
    git remote add origin "$ORIGIN_DIR"
    git push -u origin main -q
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "${ORIGIN_DIR:-}"
}

# Advance main by one commit that was never pushed to a staging branch.
commit_unstaged_work() {
    echo "change" > unstaged
    git add unstaged
    git commit -m "chore: unstaged work" -q
}

@test "guard-main-push: passes for a non-main destination" {
    export PRE_COMMIT_REMOTE_BRANCH="refs/heads/feature/foo"
    PRE_COMMIT_TO_REF="$(git rev-parse HEAD)"
    export PRE_COMMIT_TO_REF
    run_isolated ./guard-main-push.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [ -z "$output" ]
}

@test "guard-main-push: passes when the pushed SHA is on a staging branch" {
    # The canonical path: this exact SHA was pushed to staging-* for CI first.
    git push -q origin main:staging-foo
    export PRE_COMMIT_REMOTE_BRANCH="refs/heads/main"
    PRE_COMMIT_TO_REF="$(git rev-parse HEAD)"
    export PRE_COMMIT_TO_REF
    run_isolated ./guard-main-push.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
}

@test "guard-main-push: blocks a direct push whose SHA is on no staging branch" {
    # A staging branch exists, but at an OLDER SHA — the commit being pushed was
    # never staged. Proves the check is SHA-specific, not "any staging exists".
    git push -q origin main:staging-old
    commit_unstaged_work
    export PRE_COMMIT_REMOTE_BRANCH="refs/heads/main"
    PRE_COMMIT_TO_REF="$(git rev-parse HEAD)"
    export PRE_COMMIT_TO_REF
    run_isolated ./guard-main-push.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"not on any staging-* branch"* ]]
}

@test "guard-main-push: override allows an unstaged direct push to main" {
    commit_unstaged_work
    export PRE_COMMIT_REMOTE_BRANCH="refs/heads/main"
    PRE_COMMIT_TO_REF="$(git rev-parse HEAD)"
    export PRE_COMMIT_TO_REF
    export SDMX_ALLOW_DIRECT_MAIN=1
    run_isolated ./guard-main-push.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"SDMX_ALLOW_DIRECT_MAIN=1"* ]]
}

@test "guard-main-push: passes on a branch deletion (all-zero SHA)" {
    export PRE_COMMIT_REMOTE_BRANCH="refs/heads/main"
    export PRE_COMMIT_TO_REF="0000000000000000000000000000000000000000"
    run_isolated ./guard-main-push.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [ -z "$output" ]
}

@test "guard-main-push: fails open when the remote cannot be queried" {
    # Offline / unreachable remote: a real push would fail anyway, so the guard
    # must not block spuriously. It warns and passes.
    git remote set-url origin /nonexistent/bare.git
    export PRE_COMMIT_REMOTE_BRANCH="refs/heads/main"
    PRE_COMMIT_TO_REF="$(git rev-parse HEAD)"
    export PRE_COMMIT_TO_REF
    run_isolated ./guard-main-push.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"Could not query"* ]]
}

@test "guard-main-push: passes when the destination is unknown (no PRE_COMMIT env)" {
    unset PRE_COMMIT_REMOTE_BRANCH PRE_COMMIT_TO_REF
    run_isolated ./guard-main-push.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [ -z "$output" ]
}
