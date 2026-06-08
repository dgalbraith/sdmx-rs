#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/prep-release.sh
#
# prep-release is the highest-blast-radius recipe in the repo: it rewrites EVERY
# crate's Cargo.toml. These tests drive it against real fixture manifests (so the
# sed rewrites are exercised for real) while stubbing the side-effecting tail —
# `cargo update` (CARGO) and the signed `git` commit (GIT) — so no registry
# resolve or GPG signing is needed. Contract asserted:
#   - missing version arg                -> exit 1, no manifests touched,
#   - package `^version` bumped in every manifest,
#   - exact inter-crate pins rewritten, `=` exact-pin marker PRESERVED,
#   - `rust-version` left untouched,
#   - cargo update + signed commit invoked with the captured commit message.
#
# Run with: bats tests/bats/prep-release.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$BATS_TEST_DIRNAME/../../scripts/prep-release.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/common.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" scripts/lib/

    # Real fixture manifests for all five crates, mirroring the live shape:
    # package version, an exact inter-crate pin where applicable, and a
    # rust-version line that must NOT move.
    write_manifest sdmx-types  ""
    write_manifest sdmx-parsers 'sdmx-types = { version = "=0.0.0", path = "../sdmx-types" }'
    write_manifest sdmx-writers 'sdmx-types = { version = "=0.0.0", path = "../sdmx-types" }'
    write_manifest sdmx-client  'sdmx-types = { version = "=0.0.0", path = "../sdmx-types" }
sdmx-parsers = { version = "=0.0.0", path = "../sdmx-parsers" }'
    write_manifest sdmx-rs 'sdmx-types = { version = "=0.0.0", path = "../sdmx-types" }
sdmx-parsers = { version = "=0.0.0", path = "../sdmx-parsers", optional = true }'

    mkdir -p bin
    CARGO_LOG="$TMPDIR/cargo-calls.log"
    GIT_LOG="$TMPDIR/git-calls.log"
    export CARGO_LOG GIT_LOG
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# Write a fixture Cargo.toml for <crate> with optional <deps> block.
write_manifest() {
    local crate="$1" deps="$2"
    mkdir -p "crates/$crate"
    {
        echo '[package]'
        echo "name = \"$crate\""
        echo 'version = "0.0.0"'
        echo 'rust-version = "1.91.0"'
        echo ''
        echo '[dependencies]'
        if [ -n "$deps" ]; then echo "$deps"; fi
    } > "crates/$crate/Cargo.toml"
}

make_stubs() {
    cat > bin/cargo <<'EOF'
#!/bin/sh
echo "$*" >> "$CARGO_LOG"
exit "${STUB_CARGO_EXIT:-0}"
EOF
    chmod +x bin/cargo
    export CARGO="$TMPDIR/bin/cargo"

    # git stub records argv; `git add` glob expansion happens in the script's
    # shell before git is called, so the stub just logs and succeeds.
    cat > bin/git <<'EOF'
#!/bin/sh
echo "$*" >> "$GIT_LOG"
exit 0
EOF
    chmod +x bin/git
    export GIT="$TMPDIR/bin/git"
}

@test "prep-release: missing version fails and touches no manifest" {
    make_stubs
    run_isolated ./scripts/prep-release.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"no version given"* ]]
    # No manifest was rewritten.
    grep -q 'version = "0.0.0"' crates/sdmx-types/Cargo.toml
    # Neither side-effecting step ran.
    run ! test -s "$CARGO_LOG"
    run ! test -s "$GIT_LOG"
}

@test "prep-release: bumps the package version in every manifest" {
    make_stubs
    run_isolated ./scripts/prep-release.sh 0.2.0
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        grep -q '^version = "0.2.0"$' "crates/$crate/Cargo.toml"
    done
}

@test "prep-release: rewrites exact inter-crate pins and keeps the = marker" {
    make_stubs
    run_isolated ./scripts/prep-release.sh 0.2.0
    [ "$status" -eq 0 ]
    # The exact pin moves to the new version WITH its leading `=` preserved.
    grep -q '^sdmx-types = { version = "=0.2.0", path = "../sdmx-types" }$' crates/sdmx-parsers/Cargo.toml
    grep -q '^sdmx-parsers = { version = "=0.2.0", path = "../sdmx-parsers" }$' crates/sdmx-client/Cargo.toml
    # No stale "=0.0.0" pin remains anywhere.
    run ! grep -rq '"=0.0.0"' crates/
}

@test "prep-release: leaves rust-version untouched" {
    make_stubs
    run_isolated ./scripts/prep-release.sh 0.2.0
    [ "$status" -eq 0 ]
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        grep -q '^rust-version = "1.91.0"$' "crates/$crate/Cargo.toml"
    done
}

@test "prep-release: accepts a pre-release version" {
    make_stubs
    run_isolated ./scripts/prep-release.sh 0.2.0-alpha.1
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    grep -q '^version = "0.2.0-alpha.1"$' crates/sdmx-types/Cargo.toml
    grep -q '^sdmx-types = { version = "=0.2.0-alpha.1"' crates/sdmx-parsers/Cargo.toml
}

@test "prep-release: regenerates the lockfile and makes the signed batch commit" {
    make_stubs
    run_isolated ./scripts/prep-release.sh 0.2.0
    [ "$status" -eq 0 ]
    # cargo update --workspace ran.
    grep -q -- "update --workspace" "$CARGO_LOG"
    # git staged the manifests + lockfile and committed with the batch message.
    grep -q -- "add " "$GIT_LOG"
    grep -q -- "commit --gpg-sign -m chore(release): prepare release batch 0.2.0" "$GIT_LOG"
}

@test "prep-release: guard aborts on a pin the rewrite cannot reach (no commit)" {
    make_stubs
    # Inject an exact pin whose crate name contains an uppercase letter, so the
    # rewrite's `sdmx-[a-z]+` class never matches it: it stays "=0.0.0" while the
    # rest of the tree moves to 0.2.0. The post-loop guard must catch this stale
    # pin and abort BEFORE the signed commit, rather than exiting 0 silently.
    printf '%s\n' 'sdmx-Types = { version = "=0.0.0", path = "../sdmx-types" }' \
        >> crates/sdmx-parsers/Cargo.toml
    run_isolated ./scripts/prep-release.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"not rewritten to 0.2.0"* ]]
    # The side-effecting tail never ran: no lockfile regen, no signed commit.
    run ! test -s "$CARGO_LOG"
    run ! test -s "$GIT_LOG"
}
