#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/check-release-notes.sh
#
# The mandatory pre-tag gate for the facade's curated release notes. Drives the
# script against real fixture files in a temp dir. Contract:
#   - missing version arg          -> exit 1,
#   - missing curated file         -> exit 1 (names the path),
#   - empty/whitespace file        -> exit 1,
#   - fully curated file           -> exit 0,
#   - a required section absent     -> exit 1 (names the section),  [hardening]
#   - surviving template guidance   -> exit 1 (scaffold not curated), [hardening]
#   - pre-release version           -> resolves the matching file.
#
# Run with: bats tests/bats/check-release-notes.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$BATS_TEST_DIRNAME/../../scripts/check-release-notes.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" scripts/lib/

    mkdir -p crates/sdmx-rs/release-notes

    # Minimal Cargo.toml so the MSRV literal check has an authoritative source.
    # Tests that need a different rust-version overwrite this file directly.
    cat > crates/sdmx-rs/Cargo.toml <<EOF
[package]
name = "sdmx-rs"
version = "0.1.0"
rust-version = "1.91.0"
EOF
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# Write a FULLY CURATED notes file for <version> to stdout's target: all required
# sections present with negative-state content, no template guidance sentinels.
write_curated() {
    local version="$1"
    cat > "crates/sdmx-rs/release-notes/${version}.md" <<EOF
Curated summary prose for ${version}.

## Breaking Changes & Migration
This release contains no breaking changes.

## Bug Fixes
No bug fixes in this release.

## New Features & Enhancements
No new features in this release.

## Deprecations
No deprecations in this release.

## Minimum Supported Rust Version (MSRV)
* **Current MSRV**: \`1.91.0\`

## Feature Flags
No changes to Cargo feature flags.

## Security
No security advisories addressed in this release.

## Dependency Updates
No notable dependency updates in this release.

## Verifying Release Provenance
Every artifact is published with SLSA build provenance and dual-format SBOMs. See SECURITY.md.
EOF
}

@test "check-release-notes: missing version arg fails" {
    run_isolated ./scripts/check-release-notes.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"no version given"* ]]
}

@test "check-release-notes: fully curated file passes" {
    write_curated 0.2.0
    run_isolated ./scripts/check-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"complete, and curated for 0.2.0"* ]]
}

@test "check-release-notes: missing curated file fails and names the path" {
    run_isolated ./scripts/check-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"missing"* ]]
    [[ "$output" == *"crates/sdmx-rs/release-notes/0.2.0.md"* ]]
}

@test "check-release-notes: empty/whitespace-only file does NOT satisfy the gate" {
    printf '   \n\n\t\n' > crates/sdmx-rs/release-notes/0.2.0.md
    run_isolated ./scripts/check-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"empty"* ]]
}

# --- Hardening: required sections ---------------------------------------------
@test "check-release-notes: a missing required section fails and names it" {
    write_curated 0.2.0
    grep -v '^## Security$' crates/sdmx-rs/release-notes/0.2.0.md > tmp && mv tmp crates/sdmx-rs/release-notes/0.2.0.md
    run_isolated ./scripts/check-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"missing required section: ## Security"* ]]
}

# --- Hardening: sentinel rejection (the scaffold-not-curated case) ------------
@test "check-release-notes: surviving per-section GUIDANCE sentinel fails" {
    write_curated 0.2.0
    # Re-introduce a template guidance comment into one section.
    printf '\n<!-- GUIDANCE: fill me -->\n' >> crates/sdmx-rs/release-notes/0.2.0.md
    run_isolated ./scripts/check-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"unedited template guidance remains"* ]]
}

@test "check-release-notes: surviving TEMPLATE GUIDANCE header block fails" {
    write_curated 0.2.0
    # Prepend the header sentinel as if the maintainer never deleted it.
    { printf '<!-- TEMPLATE GUIDANCE — delete this -->\n'; cat crates/sdmx-rs/release-notes/0.2.0.md; } > tmp \
        && mv tmp crates/sdmx-rs/release-notes/0.2.0.md
    run_isolated ./scripts/check-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"unedited template guidance remains"* ]]
}

@test "check-release-notes: a maintainer's own HTML comment is allowed" {
    write_curated 0.2.0
    # An arbitrary comment that is NOT a template sentinel must not trip the gate.
    printf '\n<!-- TODO: link the migration guide once published -->\n' \
        >> crates/sdmx-rs/release-notes/0.2.0.md
    run_isolated ./scripts/check-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
}

@test "check-release-notes: resolves a fully curated pre-release version (1.0.0-rc.1)" {
    write_curated 1.0.0-rc.1
    run_isolated ./scripts/check-release-notes.sh 1.0.0-rc.1
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"1.0.0-rc.1"* ]]
}

# --- Hardening: MSRV literal check --------------------------------------------
@test "check-release-notes: MSRV literal matching Cargo.toml passes" {
    # write_curated produces '* **Current MSRV**: `1.91.0`' and the fixture
    # Cargo.toml declares rust-version = "1.91.0" — they must agree.
    write_curated 0.2.0
    run_isolated ./scripts/check-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
}

@test "check-release-notes: MSRV literal mismatching Cargo.toml fails" {
    write_curated 0.2.0
    # Bump the crate floor without updating the curated notes.
    sed 's/rust-version = "1.91.0"/rust-version = "1.92.0"/' \
        crates/sdmx-rs/Cargo.toml > tmp && mv tmp crates/sdmx-rs/Cargo.toml
    run_isolated ./scripts/check-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"MSRV literal"*"does not match"* ]]
    [[ "$output" == *"1.92.0"* ]]
}

@test "check-release-notes: MSRV line absent from curated notes fails" {
    write_curated 0.2.0
    # Remove the literal line while keeping the section heading — the section
    # check passes but the MSRV literal check must fire.
    grep -v '^\* \*\*Current MSRV\*\*' crates/sdmx-rs/release-notes/0.2.0.md \
        > tmp && mv tmp crates/sdmx-rs/release-notes/0.2.0.md
    run_isolated ./scripts/check-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"MSRV literal"*"does not match"* ]]
}

@test "check-release-notes: MSRV line with wrong format (no backticks) fails" {
    write_curated 0.2.0
    # Replace the correctly formatted line with one missing the backtick delimiters.
    sed 's/\* \*\*Current MSRV\*\*: `1\.91\.0`/* **Current MSRV**: 1.91.0/' \
        crates/sdmx-rs/release-notes/0.2.0.md > tmp \
        && mv tmp crates/sdmx-rs/release-notes/0.2.0.md
    run_isolated ./scripts/check-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"MSRV literal"*"does not match"* ]]
}
