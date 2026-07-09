#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/ci/verify-tag-on-main.sh
#
# Validates the source-in-main publish gate: a release tag's commit must be
# reachable from the canonical origin/main before publishing is allowed.
#
# Run with: bats tests/bats/verify-tag-on-main.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    # Mirror the real scripts/ci + scripts/lib layout so the script's
    # `$(dirname "$0")/../lib/log.sh` source path resolves inside the fixture.
    mkdir -p ci lib
    cp "$BATS_TEST_DIRNAME/../../scripts/ci/verify-tag-on-main.sh" ci/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"
    git config commit.gpgsign false
    git config tag.gpgsign false
    git config tag.forceSignAnnotated false

    # Initial main history.
    touch README.md
    git add README.md
    git commit -m "initial commit" -q
    echo "more" > file.txt
    git add file.txt
    git commit -m "second commit" -q

    # Bare origin holding main.
    ORIGIN_DIR=$(mktemp -d)
    git init --bare -q "$ORIGIN_DIR"
    git remote add origin "$ORIGIN_DIR"
    git push -u origin main -q
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "${ORIGIN_DIR:-}"
}

@test "verify-tag-on-main: passes when tag commit is the tip of main" {
    SHA=$(git rev-parse HEAD)
    run_isolated ./ci/verify-tag-on-main.sh "$SHA"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"reachable from origin/main"* ]]
}

@test "verify-tag-on-main: passes when tag commit is an older ancestor of main" {
    # A commit deeper in main's history is still reachable -> allowed.
    SHA=$(git rev-parse HEAD~1)
    run_isolated ./ci/verify-tag-on-main.sh "$SHA"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
}

@test "verify-tag-on-main: passes for an annotated tag pointing at a main commit" {
    git tag -a "sdmx-types/v0.1.0" -m "release" HEAD
    # Pass the tag name: the script must peel the annotated tag to its commit.
    run_isolated ./ci/verify-tag-on-main.sh "sdmx-types/v0.1.0"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
}

@test "verify-tag-on-main: fails when tag commit is NOT in main" {
    # Build a commit on a side branch that was never merged into main, then push
    # nothing of it to origin. This is the core exposure: a tag could point here.
    git checkout -b release/2026-05-27 -q
    echo "unmerged release work" > release.txt
    git add release.txt
    git commit -m "chore: release commit (not on main)" -q
    ORPHAN_SHA=$(git rev-parse HEAD)

    run_isolated ./ci/verify-tag-on-main.sh "$ORPHAN_SHA"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"NOT in main"* ]]
}

@test "verify-tag-on-main: classifies against REMOTE main, not stale local main" {
    # Advance local main WITHOUT pushing, then point a tag at the un-pushed
    # commit. Because origin/main has not moved, the commit must be judged
    # off-main: the gate trusts the remote, not the local branch.
    echo "local only" > local.txt
    git add local.txt
    git commit -m "local advance, not pushed" -q
    LOCAL_SHA=$(git rev-parse HEAD)

    run_isolated ./ci/verify-tag-on-main.sh "$LOCAL_SHA"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"NOT in main"* ]]
}

@test "verify-tag-on-main: fails on an unresolvable commit" {
    run_isolated ./ci/verify-tag-on-main.sh "does-not-exist"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"does not resolve to a commit"* ]]
}
