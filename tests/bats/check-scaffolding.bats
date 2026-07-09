#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/check-scaffolding.sh
#
# Testing approach: Integration tests for scaffolding dependency validation.
# Validates documentation, presence checks, and permanent vs temporary classification.
#
# Run with: bats tests/bats/check-scaffolding.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    # Create temporary directory for workspace mockup
    cd "$BATS_TEST_TMPDIR" || exit 1

    # Create crates and dependency directory structure
    mkdir -p crates/test-crate/src
    mkdir -p lib

    # Copy check-scaffolding script and logging library dependency
    cp "$BATS_TEST_DIRNAME/../../scripts/check-scaffolding.sh" .
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

}

teardown() {
    # Exit temporary directory and clean up
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# ==============================================================================
# Unused Scaffolding Dependencies
# ==============================================================================

@test "check-scaffolding: passes when ignored dependencies are genuinely unused" {
    cat > crates/test-crate/Cargo.toml <<'EOF'
[package]
name = "test-crate"

[dependencies]
serde = "1.0"

[package.metadata.cargo-machete]
ignored = [
  "serde", # Phase 1: some documentation
]
EOF

    cat > crates/test-crate/src/lib.rs <<'EOF'
// Empty file
EOF

    run_isolated ./check-scaffolding.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"scaffolding: all dependencies documented and plausible"* ]]
}

# ==============================================================================
# Active Dependency Usage Checks
# ==============================================================================

@test "check-scaffolding: fails when an ignored dependency is actively used in source" {
    cat > crates/test-crate/Cargo.toml <<'EOF'
[package]
name = "test-crate"

[dependencies]
serde = "1.0"

[package.metadata.cargo-machete]
ignored = [
  "serde", # Phase 1: some documentation
]
EOF

    cat > crates/test-crate/src/lib.rs <<'EOF'
use serde::Serialize;
EOF

    run_isolated ./check-scaffolding.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"Ignored dependency 'serde' is actively used"* ]]
}

# ==============================================================================
# Comment Stripping Validation
# ==============================================================================

@test "check-scaffolding: ignores commented out usage in source" {
    cat > crates/test-crate/Cargo.toml <<'EOF'
[package]
name = "test-crate"

[dependencies]
serde = "1.0"

[package.metadata.cargo-machete]
ignored = [
  "serde", # Phase 1: some documentation
]
EOF

    cat > crates/test-crate/src/lib.rs <<'EOF'
// use serde::Serialize;
// Some comment mentioning serde::Serialize
/* block comment serde::Serialize */
EOF

    run_isolated ./check-scaffolding.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"scaffolding: all dependencies documented and plausible"* ]]
}

# ==============================================================================
# Permanent Dependency Overrides
# ==============================================================================

@test "check-scaffolding: skips validation for PERMANENT marked dependencies" {
    cat > crates/test-crate/Cargo.toml <<'EOF'
[package]
name = "test-crate"

[package.metadata.cargo-machete]
ignored = [
  "some-dep", # PERMANENT: platform specific
]
EOF

    # Since it is marked permanent, it won't check if it's declared or used.
    run_isolated ./check-scaffolding.sh
    [ "$status" -eq 0 ]
}

# ==============================================================================
# Internal Workspace Crate Validation (Check 3)
#
# These exercise the sdmx-* branch, which the cases above never reached — the
# precise gap that let a tautological bug ship. The original check
# `grep "\"$dep\""` always matched the quoted ignore entry that $dep was
# extracted from, so it could NEVER fail: an internal crate omitted from
# [dependencies] was still "validated". The fix strips the cargo-machete block
# and matches the dependency as a bare TOML key.
# ==============================================================================

@test "check-scaffolding: internal dep declared as a bare key validates" {
    # Use a real workspace crate name so the sdmx-* branch is taken.
    mkdir -p crates/sdmx-client/src
    cat > crates/sdmx-client/Cargo.toml <<'EOF'
[package]
name = "sdmx-client"

[dependencies]
sdmx-types = { version = "=0.0.0", path = "../sdmx-types" }

[package.metadata.cargo-machete]
ignored = [
  "sdmx-types", # Phase 2: depends on core types
]
EOF
    # Remove the default external test-crate so this run is internal-only.
    rm -rf crates/test-crate

    run_isolated ./check-scaffolding.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"scaffolding: all dependencies documented and plausible"* ]]
}

@test "check-scaffolding: REGRESSION — internal dep ignored but not declared fails" {
    # sdmx-types is in ignored[] but absent from [dependencies]. The old
    # tautological grep validated this falsely; the fixed check must fail.
    mkdir -p crates/sdmx-client/src
    cat > crates/sdmx-client/Cargo.toml <<'EOF'
[package]
name = "sdmx-client"

[dependencies]
serde = "1.0"

[package.metadata.cargo-machete]
ignored = [
  "sdmx-types", # Phase 2: should be declared but ISN'T
]
EOF
    rm -rf crates/test-crate

    run_isolated ./check-scaffolding.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"sdmx-types"* ]]
    [[ "$output" == *"ignored but not declared"* ]]
}

@test "check-scaffolding: declaration matches when machete block precedes [dependencies]" {
    # Section-order edge: the block-strip sed deletes through the [dependencies]
    # header line, but the declaration on the NEXT line must still be matched.
    mkdir -p crates/sdmx-client/src
    cat > crates/sdmx-client/Cargo.toml <<'EOF'
[package]
name = "sdmx-client"

[package.metadata.cargo-machete]
ignored = [
  "sdmx-types", # Phase 2: depends on core types
]

[dependencies]
sdmx-types = { version = "=0.0.0", path = "../sdmx-types" }
EOF
    rm -rf crates/test-crate

    run_isolated ./check-scaffolding.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
}
