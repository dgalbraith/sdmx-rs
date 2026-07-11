#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/ci/parse-release-tag.sh
#
# Validates the tag-parsing gate of publish.yml's setup job: a release tag
# "<crate>/v<version>" must name a real workspace crate, agree with that
# crate's Cargo.toml version, and never name the 0.0.0 core (the categorical
# rejection: between releases every in-tree manifest reads 0.0.0, so the
# version-match assertion alone would PASS a mistaken v0.0.0 tag).
#
# Lightweight: `cargo` is stubbed via a PATH shim that records its argv and
# prints a canned `cargo metadata` JSON document, so the suite asserts the
# script's contract without a real cargo invocation. `jq` is real.
#
# Run with: bats tests/bats/parse-release-tag.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    # Mirror the real scripts/ci + scripts/lib layout so the script's
    # `$(dirname "$0")/../lib/log.sh` source path resolves inside the fixture.
    mkdir -p ci lib
    cp "$BATS_TEST_DIRNAME/../../scripts/ci/parse-release-tag.sh" ci/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    # Workspace-member fixture: the script checks crates/<name>/Cargo.toml
    # exists before consulting cargo metadata (content is irrelevant here, the
    # authoritative version comes from the stubbed metadata).
    mkdir -p crates/sdmx-types
    printf 'version = "0.2.0"\n' > crates/sdmx-types/Cargo.toml

    # PATH-shim cargo: log argv (one line per call) and print canned metadata.
    CARGO_LOG="$BATS_TEST_TMPDIR/cargo-calls.log"
    export CARGO_LOG
    METADATA_JSON="$BATS_TEST_TMPDIR/metadata.json"
    export METADATA_JSON
    printf '{"packages":[{"name":"sdmx-types","version":"0.2.0"}]}\n' > "$METADATA_JSON"
    mkdir -p bin
    cat > bin/cargo <<'EOF'
#!/bin/sh
echo "$*" >> "$CARGO_LOG"
cat "$METADATA_JSON"
EOF
    chmod +x bin/cargo
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

@test "parse-release-tag: parses a consistent tag and emits crate, version, crate_file" {
    run_isolated ./ci/parse-release-tag.sh "sdmx-types/v0.2.0"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"crate=sdmx-types"* ]]
    [[ "$output" == *"version=0.2.0"* ]]
    [[ "$output" == *"crate_file=target/package/sdmx-types-0.2.0.crate"* ]]
}

@test "parse-release-tag: fails on a malformed tag without a /v version segment" {
    run_isolated ./ci/parse-release-tag.sh "sdmx-types-0.2.0"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Malformed release tag"* ]]
}

@test "parse-release-tag: fails when the tag names a crate that is not a workspace member" {
    run_isolated ./ci/parse-release-tag.sh "sdmx-nonesuch/v0.2.0"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"does not exist"* ]]
}

@test "parse-release-tag: fails when tag version disagrees with cargo metadata" {
    run_isolated ./ci/parse-release-tag.sh "sdmx-types/v0.3.0"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Version mismatch"* ]]
}

# ---------------------------------------------------------------------------
# THE CATEGORICAL REJECTION: between releases every in-tree manifest reads
# version = "0.0.0", so a v0.0.0 tag AGREES with Cargo.toml and the
# version-match assertion alone would pass it. The rejection must fire on the
# tag's version string alone, before any manifest comparison can vouch for it.
# ---------------------------------------------------------------------------
@test "parse-release-tag: rejects a v0.0.0 tag even when the manifest agrees" {
    # Recreate the trap exactly: metadata also reads 0.0.0, so only the
    # categorical check stands between this tag and a passing parse.
    printf '{"packages":[{"name":"sdmx-types","version":"0.0.0"}]}\n' > "$METADATA_JSON"
    run_isolated ./ci/parse-release-tag.sh "sdmx-types/v0.0.0"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"0.0.0 is never published"* ]]
    # No publish-relevant output was emitted.
    run ! grep -q "crate_file=" <<< "$output"
}

@test "parse-release-tag: rejects a 0.0.0 pre-release tag (whole 0.0.0 core)" {
    run_isolated ./ci/parse-release-tag.sh "sdmx-types/v0.0.0-alpha.1"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"0.0.0 is never published"* ]]
}

@test "parse-release-tag: the 0.0.0 rejection fires before cargo metadata is consulted" {
    run_isolated ./ci/parse-release-tag.sh "sdmx-types/v0.0.0"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    # Keyed on the tag alone: the cargo stub was never invoked.
    [ ! -s "$CARGO_LOG" ]
}
