#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================

# Test suite for scripts/release-merge.sh
#
# Testing approach: Integration tests for release branch merge orchestration.
# Validates merge workflow, commit message generation, and error handling.
#
# Run with: bats tests/bats/release-merge.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    # Copy scripts
    cp "$BATS_TEST_DIRNAME/../../scripts/release-merge.sh" .
    cp "$BATS_TEST_DIRNAME/../../scripts/common.sh" .
    # common.sh sources lib/log.sh transitively — mirror that layout in the fixture.
    mkdir -p lib
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    # Initialise mock git repository
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"
    # Disable signing in the sandbox: the maintainer's global config sets
    # commit.gpgsign / tag.gpgsign, which would force signed objects (and fail
    # for lack of a key). Merge-commit signing is exercised separately via
    # RELEASE_MERGE_NO_SIGN.
    git config commit.gpgsign false
    git config tag.gpgsign false
    git config tag.forceSignAnnotated false

    # Create initial commit on main with .gitignore
    touch README.md
    cat > .gitignore <<'EOF'
release-merge.sh
common.sh
lib/
EOF
    git add README.md .gitignore
    git commit -m "initial commit" -q

    # Create minimal crates directory structure with src/lib.rs files
    mkdir -p crates/sdmx-types crates/sdmx-parsers crates/sdmx-writers crates/sdmx-client crates/sdmx-rs
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        mkdir -p "crates/${crate}/src"
        touch "crates/${crate}/src/lib.rs"
        cat > "crates/${crate}/Cargo.toml" <<EOF
[package]
name = "${crate}"
version = "0.1.0"
EOF
    done
    git add crates
    git commit -m "add crates" -q

    # Create a bare repository to act as origin and push the initial commit
    ORIGIN_DIR=$(mktemp -d)
    git init --bare -q "$ORIGIN_DIR"
    git remote add origin "$ORIGIN_DIR"
    git push -u origin main -q
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "${ORIGIN_DIR:-}"
}

@test "release-merge: fails if release branch does not exist" {
    run_isolated ./release-merge.sh non-existent-branch
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error: Release branch"* ]]
}

@test "release-merge: fails if no release tags on HEAD of release branch" {
    # Create a release branch but don't add any tags
    git checkout -b release/2026-05-27 -q
    echo "update" > dummy_change
    git add dummy_change
    git commit -m "chore: release commit sdmx-types" -q

    # Run release-merge.sh from the release branch (with no tags)
    run_isolated ./release-merge.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"No release tags found on"* ]]
}

@test "release-merge: succeeds and merges when on release branch with a tag" {
    git checkout -b release/2026-05-27 -q
    echo "update" > dummy_change
    git add dummy_change
    git commit -m "chore: release commit sdmx-types" -q

    # Add a mock tag on HEAD
    git tag "sdmx-types/v0.1.0"

    # Run release-merge.sh (mock signing so we don't need GPG keys setup)
    export RELEASE_MERGE_NO_SIGN=1
    run_isolated ./release-merge.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    # Verify we are back on main and merge commit exists
    [ "$(git branch --show-current)" = "main" ]
    git log -1 --pretty=%B | grep -q "chore(release): merge release branch 2026-05-27"
    git log -1 --pretty=%B | grep -q "Released:"
    git log -1 --pretty=%B | grep -q -- "- sdmx-types: v0.1.0"
}

@test "release-merge: classifies all crates released across per-crate commits" {
    # Reproduce the real cargo-release topology: one release commit + annotated
    # tag PER crate, stamped sequentially. Earlier crates' tags therefore do NOT
    # sit on the branch tip — only the last (sdmx-rs) does. A naive
    # `git tag --points-at "$BRANCH"` would detect only sdmx-rs and mislabel the
    # other four as Unchanged. This guards the --merged/--no-merged fix.
    git checkout -b release/2026-05-27 -q
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        echo "release ${crate}" > "release_${crate}"
        git add "release_${crate}"
        git commit -m "chore: Release ${crate} version 0.1.0" -q
        git tag -a "${crate}/v0.1.0" -m "chore: Release ${crate} version 0.1.0"
    done

    export RELEASE_MERGE_NO_SIGN=1
    run_isolated ./release-merge.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    [ "$(git branch --show-current)" = "main" ]
    git log -1 --pretty=%B > msg.txt
    # All five crates must appear under Released, and none under Unchanged.
    grep -q "Released:" msg.txt
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        grep -q -- "- ${crate}: v0.1.0" msg.txt
    done
    run ! grep -q "Unchanged:" msg.txt
    grep -q "All commits and tags are cryptographically signed." msg.txt
    # Guard against heredoc-terminator leakage: no shell artifacts in the message.
    run ! grep -qE '^EOF$|cat >>|if \[ -n' msg.txt
}

@test "release-merge: separates released from unchanged crates" {
    # Only two crates get release commits + tags this batch; the other three
    # remain unchanged and must be listed under Unchanged.
    git checkout -b release/2026-05-27 -q
    for crate in sdmx-types sdmx-client; do
        echo "release ${crate}" > "release_${crate}"
        git add "release_${crate}"
        git commit -m "chore: Release ${crate} version 0.1.0" -q
        git tag -a "${crate}/v0.1.0" -m "chore: Release ${crate} version 0.1.0"
    done

    export RELEASE_MERGE_NO_SIGN=1
    run_isolated ./release-merge.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    MSG=$(git log -1 --pretty=%B)
    RELEASED_BLOCK=$(echo "$MSG" | awk '/Released:/{f=1;next} /Unchanged:/{f=0} f')
    UNCHANGED_BLOCK=$(echo "$MSG" | awk '/Unchanged:/{f=1;next} /signed/{f=0} f')
    echo "$RELEASED_BLOCK" | grep -q -- "- sdmx-types: v0.1.0"
    echo "$RELEASED_BLOCK" | grep -q -- "- sdmx-client: v0.1.0"
    echo "$UNCHANGED_BLOCK" | grep -q -- "- sdmx-parsers: v0.1.0"
    echo "$UNCHANGED_BLOCK" | grep -q -- "- sdmx-writers: v0.1.0"
    echo "$UNCHANGED_BLOCK" | grep -q -- "- sdmx-rs: v0.1.0"
}

@test "release-merge: fails if a tracked file has unstaged modifications" {
    git checkout -b release/2026-05-27 -q
    echo "update" > dummy_change
    git add dummy_change
    git commit -m "chore: release commit sdmx-types" -q
    git tag "sdmx-types/v0.1.0"

    # Dirty a tracked file in the working tree (not staged).
    echo "# local scratch edit" >> crates/sdmx-types/Cargo.toml

    export RELEASE_MERGE_NO_SIGN=1
    run_isolated ./release-merge.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"uncommitted changes to tracked files"* ]]
    # Must abort BEFORE switching branches — guard runs before any git mutation.
    [ "$(git branch --show-current)" = "release/2026-05-27" ]
}

@test "release-merge: fails if a tracked file has staged modifications" {
    git checkout -b release/2026-05-27 -q
    echo "update" > dummy_change
    git add dummy_change
    git commit -m "chore: release commit sdmx-types" -q
    git tag "sdmx-types/v0.1.0"

    # Stage a modification to a tracked file (index dirty, working tree clean).
    echo "# staged scratch edit" >> crates/sdmx-types/Cargo.toml
    git add crates/sdmx-types/Cargo.toml

    export RELEASE_MERGE_NO_SIGN=1
    run_isolated ./release-merge.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"uncommitted changes to tracked files"* ]]
    [ "$(git branch --show-current)" = "release/2026-05-27" ]
}

@test "release-merge: succeeds with untracked files present" {
    # Untracked scratch files must NOT block the release: git refuses to
    # overwrite them on checkout, so they cannot enter the signed merge commit.
    git checkout -b release/2026-05-27 -q
    echo "update" > dummy_change
    git add dummy_change
    git commit -m "chore: release commit sdmx-types" -q
    git tag "sdmx-types/v0.1.0"

    echo "scratch notes" > my-release-notes.txt

    export RELEASE_MERGE_NO_SIGN=1
    run_isolated ./release-merge.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [ "$(git branch --show-current)" = "main" ]
}

@test "release-merge: succeeds when branch name passed as argument" {
    git checkout -b release/custom-test -q
    echo "update" > dummy_change
    git add dummy_change
    git commit -m "chore: release commit sdmx-types" -q

    # Add a mock tag on HEAD
    git tag "sdmx-types/v0.1.0"

    # Go back to main
    git checkout main -q

    # Run release-merge.sh with custom branch name argument
    export RELEASE_MERGE_NO_SIGN=1
    run_isolated ./release-merge.sh release/custom-test
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    [ "$(git branch --show-current)" = "main" ]
    git log -1 --pretty=%B | grep -q "chore(release): merge release branch custom-test"
}
