#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/check-decision-refs.sh
#
# Validates that every decision-register reference (D-NNNN) in crate source
# resolves to an entry in docs/decisions.md. A dangling reference — a typo or a
# not-yet-created entry — must fail the check. This is the integrity guard that
# keeps the design_docs decision provenance from rotting into broken pointers.
#
# Run with: bats tests/bats/check-decision-refs.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    # Create temporary directory for workspace mockup
    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    # Copy check script and logging library dependency into the fixture,
    # mirroring the scripts/lib layout the script expects.
    cp "$BATS_TEST_DIRNAME/../../scripts/check-decision-refs.sh" .
    mkdir -p lib
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    # A minimal decision register defining D-0011, D-0027, D-0035.
    mkdir -p docs
    cat > docs/decisions.md <<'EOF'
# Decision Register

| ID                | Topic         | Summary                              |
|-------------------|---------------|--------------------------------------|
| [D-0011](#d-0011) | Annotation    | AnnotationURL is a vec of structs    |
| [D-0027](#d-0027) | Lexical types | Validated newtypes with lossless raw |
| [D-0035](#d-0035) | Link          | LinkType modelled                    |
EOF
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# Write crate source content (read from stdin) to crates/sdmx-types/src/<name>.
write_source() {
    mkdir -p crates/sdmx-types/src
    cat > "crates/sdmx-types/src/$1"
}

# ==============================================================================
# Passing cases
# ==============================================================================

@test "check-decision-refs: passes when no crate source exists" {
    run_isolated ./check-decision-refs.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"all crate-source decision references resolve"* ]]
}

@test "check-decision-refs: passes with a resolving reference" {
    write_source lib.rs <<'EOF'
//! Decisions: D-0027.
pub struct X;
EOF

    run_isolated ./check-decision-refs.sh
    [ "$status" -eq 0 ]
    [[ "$output" == *"all crate-source decision references resolve"* ]]
}

@test "check-decision-refs: passes with multiple resolving references on one line" {
    write_source lib.rs <<'EOF'
// rationale (D-0011, D-0035)
pub struct X;
EOF

    run_isolated ./check-decision-refs.sh
    [ "$status" -eq 0 ]
}

# ==============================================================================
# Failing cases
# ==============================================================================

@test "check-decision-refs: fails on a dangling reference" {
    write_source lib.rs <<'EOF'
//! Decisions: D-9999.
pub struct X;
EOF

    run_isolated ./check-decision-refs.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"D-9999"* ]]
    [[ "$output" == *"lib.rs"* ]]
    [[ "$output" == *"must resolve"* ]]
}

@test "check-decision-refs: flags only the dangling ref among several on one line" {
    write_source lib.rs <<'EOF'
// see D-0027 and D-0048 and D-0011
pub struct X;
EOF

    run_isolated ./check-decision-refs.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"dangling decision reference D-0048"* ]]
    [[ "$output" != *"dangling decision reference D-0027"* ]]
    [[ "$output" != *"dangling decision reference D-0011"* ]]
}

@test "check-decision-refs: reports the offending line number" {
    write_source lib.rs <<'EOF'
//! line one
//! see D-9999 here
pub struct X;
EOF

    run_isolated ./check-decision-refs.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"line 2"* ]]
}

@test "check-decision-refs: fails when the register is missing" {
    rm -f docs/decisions.md
    write_source lib.rs <<'EOF'
//! Decisions: D-0027.
EOF

    run_isolated ./check-decision-refs.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"register not found"* ]]
}
