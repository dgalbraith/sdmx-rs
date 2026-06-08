#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/check-local-links.sh
#
# Testing approach: Integration tests for the absolute-file://-link ban
# across Markdown, TOML manifests, and Rust doc comments. Validates that
# committed sources cannot carry machine-specific file:///… link targets —
# the footgun lychee's offline check and `cargo doc -D warnings` both miss,
# because an absolute file:// target that exists on the checking machine is
# treated as valid.
#
# Run with: bats tests/bats/check-local-links.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    # Create temporary directory for workspace mockup
    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    # Copy check script and logging library dependency into the fixture,
    # mirroring the scripts/lib layout the script expects.
    cp "$BATS_TEST_DIRNAME/../../scripts/check-local-links.sh" .
    mkdir -p lib
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# ==============================================================================
# Clean trees pass
# ==============================================================================

@test "check-local-links: passes when no scannable files exist" {
    run_isolated ./check-local-links.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"no absolute file:// links"* ]]
}

@test "check-local-links: passes with only repo-relative Markdown links" {
    cat > README.md <<'EOF'
# Title

See [the manifest](crates/sdmx-types/Cargo.toml) and [ADR-0003](docs/adr/0003-foo.md).
EOF

    run_isolated ./check-local-links.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"no absolute file:// links"* ]]
}

@test "check-local-links: passes when file:// appears in prose, not as an absolute link" {
    # The ban targets absolute file:/// URLs (three slashes). A bare mention
    # of the scheme in documentation prose must NOT trip the check — otherwise
    # the check cannot even document itself.
    cat > docs.md <<'EOF'
# Notes

This tool bans absolute file:// links that leak local paths.
EOF

    run_isolated ./check-local-links.sh
    [ "$status" -eq 0 ]
}

@test "check-local-links: passes with https:// links (not file://)" {
    cat > Cargo.toml <<'EOF'
[package]
documentation = "https://docs.rs/sdmx-rs"
repository = "https://github.com/dgalbraith/sdmx-rs"
EOF

    run_isolated ./check-local-links.sh
    [ "$status" -eq 0 ]
}

# ==============================================================================
# Markdown — the original ROADMAP.md footgun
# ==============================================================================

@test "check-local-links: REGRESSION — rejects an absolute file:///home link in Markdown" {
    cat > ROADMAP.md <<'EOF'
# Roadmap

- depends on [sdmx-types](file:///home/davidg/Documents/Projects/sdmx-rs/crates/sdmx-types/Cargo.toml)
EOF

    run_isolated ./check-local-links.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"ROADMAP.md"* ]]
    [[ "$output" == *"file://"* ]]
    [[ "$output" == *"must be replaced with repo-relative paths"* ]]
}

@test "check-local-links: reports the offending line number" {
    printf '# Doc\n\nfirst line\n[x](file:///etc/hosts)\n' > NOTES.md

    run_isolated ./check-local-links.sh
    [ "$status" -eq 1 ]
    # log_err_file embeds the grep hit, which is prefixed with the line number.
    [[ "$output" == *"4:"* ]]
}

@test "check-local-links: detects file:// even when the target exists locally" {
    # The whole point: lychee --offline / cargo doc would pass this because
    # the path resolves on disk. The scheme ban must fail regardless.
    cat > GUIDE.md <<EOF
# Guide

[temp](file://$TMPDIR/lib/log.sh)
EOF

    run_isolated ./check-local-links.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"GUIDE.md"* ]]
}

# ==============================================================================
# TOML — published manifest fields leak the local path to crates.io
# ==============================================================================

@test "check-local-links: rejects file:// in a Cargo.toml manifest field" {
    cat > Cargo.toml <<'EOF'
[package]
name = "sdmx-types"
documentation = "file:///home/davidg/Documents/Projects/sdmx-rs/target/doc/sdmx_types"
EOF

    run_isolated ./check-local-links.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"Cargo.toml"* ]]
    [[ "$output" == *"file://"* ]]
}

# ==============================================================================
# Rust — rustdoc links in doc comments render on docs.rs
# ==============================================================================

@test "check-local-links: rejects file:// in a Rust doc comment" {
    mkdir -p src
    cat > src/lib.rs <<'EOF'
//! See [the spec](file:///home/davidg/Documents/Projects/sdmx-rs/docs/spec.md).
pub fn parse() {}
EOF

    run_isolated ./check-local-links.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"lib.rs"* ]]
    [[ "$output" == *"file://"* ]]
}

# ==============================================================================
# Exclusions
# ==============================================================================

@test "check-local-links: ignores template files" {
    mkdir -p docs/adr/templates
    cat > docs/adr/templates/template.md <<'EOF'
# Template

Example placeholder: [link](file:///path/to/example.md)
EOF

    run_isolated ./check-local-links.sh
    [ "$status" -eq 0 ]
}

@test "check-local-links: ignores target/ build artifacts" {
    mkdir -p target/doc
    cat > target/doc/generated.md <<'EOF'
# Generated

[x](file:///home/runner/work/foo.md)
EOF

    run_isolated ./check-local-links.sh
    [ "$status" -eq 0 ]
}
