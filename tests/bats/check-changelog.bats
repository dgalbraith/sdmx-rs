#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/check-changelog.sh
#
# Testing approach: Integration tests for changelog validation.
# Validates synchronisation with git history and repository state.
#
# Run with: bats tests/bats/check-changelog.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    # Copy scripts and configs. common.sh sources lib/log.sh transitively, so the
    # fixture must mirror that layout (lib/log.sh alongside the flattened scripts).
    cp "$BATS_TEST_DIRNAME/../../scripts/check-changelog.sh" .
    cp "$BATS_TEST_DIRNAME/../../scripts/common.sh" .
    mkdir -p lib
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/
    cp "$BATS_TEST_DIRNAME/../../cliff.toml" .

    # Initialise mock git repository
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"

    # Create initial commit on main with .gitignore
    touch README.md
    cat > .gitignore <<'EOF'
bin/
check-changelog.sh
common.sh
lib/
cliff.toml
EOF
    git add README.md .gitignore
    git commit -m "initial commit" -q

    # Create crates structure
    mkdir -p crates/sdmx-types crates/sdmx-parsers crates/sdmx-writers crates/sdmx-client crates/sdmx-rs

    # Create bin directory for mock git-cliff
    mkdir -p bin
    export PATH="$TMPDIR/bin:$PATH"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

create_mock_git_cliff_success() {
    cat > bin/git-cliff <<'EOF'
#!/bin/sh
OUTPUT_FILE=""
while [ $# -gt 0 ]; do
    case "$1" in
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done

if [ -n "$OUTPUT_FILE" ]; then
    echo "mock changelog content" > "$OUTPUT_FILE"
fi
exit 0
EOF
    chmod +x bin/git-cliff
}

# A mock git-cliff that APPENDS its full argv (one invocation per line) to
# $TMPDIR/cliff-argv.log before producing the success output. Used to assert that
# check-changelog.sh passes the correct per-crate scoping flags — the success mock
# above discards flags, so it cannot catch a dropped/incorrect --tag-pattern.
create_mock_git_cliff_recording() {
    cat > bin/git-cliff <<EOF
#!/bin/sh
echo "\$*" >> "$TMPDIR/cliff-argv.log"
OUTPUT_FILE=""
while [ \$# -gt 0 ]; do
    case "\$1" in
        --output) OUTPUT_FILE="\$2"; shift 2 ;;
        *) shift ;;
    esac
done
[ -n "\$OUTPUT_FILE" ] && echo "mock changelog content" > "\$OUTPUT_FILE"
exit 0
EOF
    chmod +x bin/git-cliff
}

@test "check-changelog: fails if changelog has uncommitted changes" {
    # Create and commit a baseline CHANGELOG.md, then dirty it — the per-file
    # guard catches edits to the changelog itself, not unrelated working-tree changes.
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        echo "mock changelog content" > "crates/${crate}/CHANGELOG.md"
    done
    git add .
    git commit -m "add changelogs" -q
    echo "dirty edit" >> "crates/sdmx-types/CHANGELOG.md"

    create_mock_git_cliff_success

    run_isolated ./check-changelog.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"has uncommitted changes"* ]]
}

@test "check-changelog: fails if changelog file does not exist" {
    # Git working tree is already clean due to setup config and .gitignore
    run_isolated ./check-changelog.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"does not exist"* ]]
}

@test "check-changelog: passes when all changelogs are in sync" {
    # Create mock CHANGELOG.md for all crates matching the mock git-cliff output
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        echo "mock changelog content" > "crates/${crate}/CHANGELOG.md"
    done

    git add .
    git commit -m "add changelogs" -q

    create_mock_git_cliff_success

    run_isolated ./check-changelog.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"changelog: all crate changelogs synchronised"* ]]
}

@test "check-changelog: fails if a changelog is out of sync" {
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        echo "mock changelog content" > "crates/${crate}/CHANGELOG.md"
    done
    # Make one out of sync
    echo "out of sync content" > "crates/sdmx-types/CHANGELOG.md"

    git add .
    git commit -m "add out of sync changelogs" -q

    create_mock_git_cliff_success

    run_isolated ./check-changelog.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"is out of sync with history"* ]]
}

# ------------------------------------------------------------------------------
# Per-crate scoping regression tests (decoupled versioning).
#
# Under decoupled versioning, each crate releases on its own cadence. git-cliff
# must scope BOTH commits (--include-path) AND release boundaries (--tag-pattern)
# to a single crate. If tag boundaries are not crate-scoped, a foreign crate's tag
# (e.g. sdmx-types/v0.3.0) sitting on a commit that also touches another crate
# emits a PHANTOM `## [0.3.0]` section in that other crate's changelog — a version
# it never released. check-changelog.sh must mirror `just changelog-generate`'s
# scoping exactly, or it computes a different "expected" changelog and false-fails.
# ------------------------------------------------------------------------------

@test "check-changelog: passes a crate-scoped --tag-pattern to git-cliff for every crate" {
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        echo "mock changelog content" > "crates/${crate}/CHANGELOG.md"
    done
    git add .
    git commit -m "add changelogs" -q

    create_mock_git_cliff_recording

    run_isolated ./check-changelog.sh
    echo "STATUS: $status" >&2
    echo "ARGV LOG:" >&2; cat "$TMPDIR/cliff-argv.log" >&2
    [ "$status" -eq 0 ]

    # Each crate's invocation must carry a --tag-pattern that is (a) the canonical
    # semver.org regex — identified by its leading `(0|[1-9]\d*)\.` major-version
    # group — and (b) anchored to THIS crate's own tag prefix (`^<crate>/v`), paired
    # with the matching --include-path. We assert the anchored prefix rather than the
    # full ~150-char regex: pinning every byte here would be unreadable and brittle,
    # and the prefix already proves both "canonical semver" and "scoped to this crate".
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        grep -F -- "--tag-pattern ^${crate}/v(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\." \
            "$TMPDIR/cliff-argv.log"
        grep -F -- "--include-path crates/${crate}/**" "$TMPDIR/cliff-argv.log"
    done

    # Guard against the pre-fix bug: NO invocation may use the global all-crate
    # alternation as its tag boundary set.
    run grep -F -- "--tag-pattern sdmx-(types|parsers" "$TMPDIR/cliff-argv.log"
    [ "$status" -ne 0 ]

    # Guard against regression to the OLD loose pattern (`/v[0-9].*`): the canonical
    # regex must have replaced it everywhere.
    run grep -F -- "/v[0-9].*" "$TMPDIR/cliff-argv.log"
    [ "$status" -ne 0 ]
}

@test "check-changelog: real git-cliff does not emit a phantom version section for an unreleased crate" {
    command -v git-cliff >/dev/null 2>&1 || skip "git-cliff not installed"

    # Reset to a clean repo dedicated to this scenario (the shared setup's commit
    # graph is not relevant here, and we need real tags).
    cd "$TMPDIR" || exit 1
    rm -rf real-cliff && mkdir real-cliff && cd real-cliff || exit 1
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"
    cp "$BATS_TEST_DIRNAME/../../cliff.toml" .
    mkdir -p crates/sdmx-types crates/sdmx-parsers

    echo b1 > crates/sdmx-types/lib.rs
    git add -A && git commit -qm "feat(types): initial"
    git tag sdmx-types/v0.2.0 -m "release sdmx-types 0.2.0"

    # THE bug trigger: one commit touches BOTH crates, then is tagged for sdmx-types
    # ONLY. sdmx-parsers has never been released.
    echo p1 > crates/sdmx-parsers/lib.rs
    echo b2 > crates/sdmx-types/lib.rs
    git add -A && git commit -qm "feat: touch both types and parsers"
    git tag sdmx-types/v0.3.0 -m "release sdmx-types 0.3.0"

    # Generate the sdmx-parsers changelog exactly as check-changelog.sh /
    # changelog-generate do (crate-scoped --tag-pattern + --include-path).
    run git-cliff --config cliff.toml \
        --tag-pattern "sdmx-parsers/v[0-9].*" \
        --include-path "crates/sdmx-parsers/**"
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    # The parser change must land under [Unreleased] — sdmx-parsers has no tag yet.
    [[ "$output" == *"## [Unreleased]"* ]]
    # And must NOT borrow sdmx-types' version as a phantom parsers release.
    [[ "$output" != *"## [0.3.0]"* ]]
}

@test "check-changelog: real git-cliff handles pre-release (semver) tags correctly" {
    command -v git-cliff >/dev/null 2>&1 || skip "git-cliff not installed"

    # Pre-1.0 stabilisation commonly cuts pre-release tags (v1.0.0-alpha.1, -rc.1,
    # etc.). The canonical-semver tag_pattern must match them, git-cliff must order
    # them semver-correctly, and the body template's `replace(from="v", to="")` must
    # strip only the leading `v` (not mangle an internal one). This pins all three.
    cd "$TMPDIR" || exit 1
    rm -rf prerelease && mkdir prerelease && cd prerelease || exit 1
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"
    cp "$BATS_TEST_DIRNAME/../../cliff.toml" .
    mkdir -p crates/sdmx-rs

    echo a > crates/sdmx-rs/a.rs
    git add -A && git commit -qm "feat(rs): feature a"
    git tag sdmx-rs/v1.0.0-alpha.1 -m "release sdmx-rs 1.0.0-alpha.1"
    echo b > crates/sdmx-rs/b.rs
    git add -A && git commit -qm "feat(rs): feature b"
    git tag sdmx-rs/v1.0.0-alpha.2 -m "release sdmx-rs 1.0.0-alpha.2"
    echo c > crates/sdmx-rs/c.rs
    git add -A && git commit -qm "fix(rs): fix c"
    git tag sdmx-rs/v1.0.0 -m "release sdmx-rs 1.0.0"

    # Facade crate uses cliff.toml's default tag_pattern (sdmx-rs/v[0-9].*); no override.
    run git-cliff --config cliff.toml --include-path "crates/sdmx-rs/**"
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    # Headers render with the prerelease suffix intact, leading `v` stripped.
    [[ "$output" == *"## [1.0.0-alpha.1]"* ]]
    [[ "$output" == *"## [1.0.0-alpha.2]"* ]]
    [[ "$output" == *"## [1.0.0]"* ]]
    # A stray leading `v` would mean replace() failed; assert none survives in a header.
    [[ "$output" != *"## [v1.0.0"* ]]

    # Semver ordering: newest first (1.0.0 before alpha.2 before alpha.1). Compare the
    # byte offsets of each header in the output.
    final=$(echo "$output" | grep -n "## \[1.0.0\]" | head -1 | cut -d: -f1)
    a2=$(echo "$output" | grep -n "## \[1.0.0-alpha.2\]" | head -1 | cut -d: -f1)
    a1=$(echo "$output" | grep -n "## \[1.0.0-alpha.1\]" | head -1 | cut -d: -f1)
    [ "$final" -lt "$a2" ]
    [ "$a2" -lt "$a1" ]

    # Each commit is attributed to its own release range (no duplication into the final).
    [[ "$output" == *"feature a"* ]]
    [[ "$output" == *"feature b"* ]]
    [[ "$output" == *"fix c"* ]]
}

@test "check-changelog: real git-cliff excludes a non-semver tag as a release boundary" {
    command -v git-cliff >/dev/null 2>&1 || skip "git-cliff not installed"

    # The canonical-semver tag_pattern is STRICT: it asserts the same semver contract
    # the rest of the pipeline enforces, so a malformed tag (e.g. a hand-made
    # `sdmx-rs/v1-experimental`) must NOT act as a release boundary — it is excluded
    # rather than rendered under a bogus `## [1-experimental]` header. The commit it
    # points at falls through to [Unreleased]. (cargo-release cannot emit such a tag;
    # this guards the strict-gatekeeping intent against a hand-pushed bad tag.)
    cd "$TMPDIR" || exit 1
    rm -rf nonsemver && mkdir nonsemver && cd nonsemver || exit 1
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"
    cp "$BATS_TEST_DIRNAME/../../cliff.toml" .
    mkdir -p crates/sdmx-rs

    echo a > crates/sdmx-rs/a.rs
    git add -A && git commit -qm "feat(rs): released feature"
    git tag sdmx-rs/v0.1.0 -m "release sdmx-rs 0.1.0"
    echo b > crates/sdmx-rs/b.rs
    git add -A && git commit -qm "feat(rs): post-release feature"
    git tag sdmx-rs/v1-experimental -m "malformed non-semver tag"

    run git-cliff --config cliff.toml --include-path "crates/sdmx-rs/**"
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    # The valid semver tag is a boundary; the malformed one is not.
    [[ "$output" == *"## [0.1.0]"* ]]
    [[ "$output" != *"experimental"* ]]
    # The commit under the malformed tag is not lost — it sits in [Unreleased].
    [[ "$output" == *"## [Unreleased]"* ]]
    [[ "$output" == *"post-release feature"* ]]
}
