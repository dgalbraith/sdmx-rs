#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================

# Test suite for scripts/rename-adr.sh
#
# Testing approach: Integration tests for adr document renaming and refactoring.
# Validates slug changes, link updates, metadata preservation, and error handling.
#
# Run with: bats tests/bats/rename-adr.bats
# ==============================================================================
setup() {
    cd "$BATS_TEST_TMPDIR" || exit 1
    source "$BATS_TEST_DIRNAME/common.sh"
    setup_adr_test

    mkdir -p docs/adr
    cat > docs/adr/0001-old-title.md <<'EOF'
# 1. Old Title

Date: 2026-05-23

## Status
Accepted

## Context
Test

## Decision
Test

## Consequences
Test
EOF

    git add docs/adr/0001-old-title.md
    git commit -q -m "Add ADR"

    add_adr_to_gitignore "0001-old-title.md"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# ==============================================================================
# File Renaming
# ==============================================================================

@test "rename-adr: rename file with -f flag" {
    [ -f "docs/adr/0001-old-title.md" ]
    ./doc-engine.sh rename adr -f 0001-old-title.md "New Title" > /dev/null
    assert_file_not_exists "docs/adr/0001-old-title.md"
    assert_adr_file_exists "0001-new-title.md"
}

@test "rename-adr: rename by prefix match (0001)" {
    ./doc-engine.sh rename adr -f 0001 "New Title" > /dev/null
    assert_adr_file_exists "0001-new-title.md"
}

# ==============================================================================
# Gitignore Updates
# ==============================================================================

@test "rename-adr: update .gitignore with -f flag" {
    ./doc-engine.sh rename adr -f 0001-old-title.md "New Title" > /dev/null
    run ! grep -q "0001-old-title.md" .gitignore
    assert_adr_in_gitignore "0001-new-title.md"
}

@test "rename-adr: preserve number prefix in .gitignore" {
    ./doc-engine.sh rename adr -f 0001-old-title.md "Different Name" > /dev/null
    assert_adr_in_gitignore "0001-different-name.md"
}

# ==============================================================================
# Title Sanitisation
# ==============================================================================

@test "rename-adr: sanitise new title" {
    ./doc-engine.sh rename adr -f 0001 "New/Title With Spaces" > /dev/null
    assert_adr_file_exists "0001-new-title-with-spaces.md"
}

@test "rename-adr: convert title to lowercase" {
    ./doc-engine.sh rename adr -f 0001 "CamelCaseTitle" > /dev/null
    assert_adr_file_exists "0001-camelcasetitle.md"
}

# ==============================================================================
# Sequential Renames
# ==============================================================================

@test "rename-adr: rename multiple times" {
    ./doc-engine.sh rename adr -f 0001 "Second Name" > /dev/null
    assert_adr_file_exists "0001-second-name.md"

    ./doc-engine.sh rename adr -f 0001 "Third Name" > /dev/null
    assert_file_not_exists "docs/adr/0001-second-name.md"
    assert_adr_file_exists "0001-third-name.md"

    assert_adr_in_gitignore "0001-third-name.md"
    ! grep -q "0001-second-name.md" .gitignore
}

# ==============================================================================
# User Confirmation
# ==============================================================================

@test "rename-adr: cancel rename when user says no" {
    [ -f "docs/adr/0001-old-title.md" ]
    echo "no" | ./doc-engine.sh rename adr "0001-old-title" "New Name"
    assert_adr_file_exists "0001-old-title.md"
    assert_adr_in_gitignore "0001-old-title.md"
    assert_file_not_exists "docs/adr/0001-new-name.md"
}

@test "rename-adr: show old and new paths before confirmation" {
    run_isolated bash -c 'echo "no" | ./doc-engine.sh rename adr "0001-old-title" "New Name"'
    [[ "$output" == *"0001-old-title.md"* ]]
    [[ "$output" == *"0001-new-name.md"* ]]
}

# ==============================================================================
# Error Handling
# ==============================================================================

@test "rename-adr: reject no arguments" {
    run_isolated ./doc-engine.sh rename adr
    [ "$status" -eq 1 ]
    [[ "$output" == *"Usage:"* ]]
}

@test "rename-adr: reject missing second argument" {
    run_isolated ./doc-engine.sh rename adr 0001
    [ "$status" -eq 1 ]
    [[ "$output" == *"Usage:"* ]]
}

@test "rename-adr: reject nonexistent ADR" {
    run_isolated ./doc-engine.sh rename adr -f 9999 "New Title"
    [ "$status" -eq 1 ]
    [[ "$output" == *"Could not find ADR matching"* ]]
}

@test "rename-adr: reject identical old and new filename" {
    run_isolated ./doc-engine.sh rename adr -f 0001-old-title.md "Old Title"
    [ "$status" -eq 1 ]
    [[ "$output" == *"identical to the old one"* ]]
}

@test "rename-adr: reject destination file already exists" {
    # Create a second ADR
    cat > docs/adr/0001-new-title.md <<'EOF'
# 1. New Title

Date: 2026-05-23

## Status
Accepted

## Context
Test

## Decision
Test

## Consequences
Test
EOF

    run_isolated ./doc-engine.sh rename adr -f 0001-old-title.md "New Title"
    [ "$status" -eq 1 ]
    [[ "$output" == *"already exists"* ]]
}

# ==============================================================================
# Output Validation
# ==============================================================================

@test "rename-adr: output indicates successful rename" {
    run_isolated ./doc-engine.sh rename adr -f 0001 "New Title"
    [[ "$output" == *"Successfully renamed"* ]]
}

# ==============================================================================
# Link Integrity
# ==============================================================================

@test "rename-adr: update links in other files" {
    # Create another md file referencing 0001-old-title.md
    cat > docs/another.md <<'EOF'
See [ADR 1](adr/0001-old-title.md) for context.
EOF

    # Create a rs file referencing 0001-old-title.md
    mkdir -p src
    cat > src/lib.rs <<'EOF'
//! See [ADR 1](https://docs.rs/sdmx-rs/latest/sdmx_rs/adr/0001-old-title.md)
EOF

    ./doc-engine.sh rename adr -f 0001 "New Title" > /dev/null

    # Assert references are updated
    grep -q "0001-new-title.md" docs/another.md
    run ! grep -q "0001-old-title.md" docs/another.md

    grep -q "0001-new-title.md" src/lib.rs
    ! grep -q "0001-old-title.md" src/lib.rs
}
