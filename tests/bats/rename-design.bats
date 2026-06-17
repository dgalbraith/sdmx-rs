#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================

# Test suite for scripts/rename-design.sh
#
# Testing approach: Integration tests for design document renaming and refactoring.
# Validates slug changes, link updates, metadata preservation, and error handling.
#
# Run with: bats tests/bats/rename-design.bats
# ==============================================================================
setup() {
    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1
    source "$BATS_TEST_DIRNAME/common.sh"
    setup_design_test

    mkdir -p docs/design
    cat > docs/design/0001-old-title.md <<'EOF'
# 1. Old Title

Date: 2026-05-23

## Status
Proposed

## Summary
Test

## Problem / Motivation
Test

## Proposed Design
Test
EOF

    git add docs/design/0001-old-title.md
    git commit -q -m "Add design"

    add_design_to_gitignore "0001-old-title.md"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# ==============================================================================
# File Renaming
# ==============================================================================

@test "rename-design: rename file with -f flag" {
    [ -f "docs/design/0001-old-title.md" ]
    ./doc-engine.sh rename design -f 0001-old-title.md "New Title" > /dev/null
    assert_file_not_exists "docs/design/0001-old-title.md"
    assert_design_file_exists "0001-new-title.md"
}

@test "rename-design: rename by prefix match (0001)" {
    ./doc-engine.sh rename design -f 0001 "New Title" > /dev/null
    assert_design_file_exists "0001-new-title.md"
}

# ==============================================================================
# Gitignore Updates
# ==============================================================================

@test "rename-design: update .gitignore with -f flag" {
    ./doc-engine.sh rename design -f 0001-old-title.md "New Title" > /dev/null
    run ! grep -q "0001-old-title.md" .gitignore
    assert_design_in_gitignore "0001-new-title.md"
}

@test "rename-design: preserve number prefix in .gitignore" {
    ./doc-engine.sh rename design -f 0001-old-title.md "Different Name" > /dev/null
    assert_design_in_gitignore "0001-different-name.md"
}

# ==============================================================================
# Title Sanitisation
# ==============================================================================

@test "rename-design: sanitise new title" {
    ./doc-engine.sh rename design -f 0001 "New/Title With Spaces" > /dev/null
    assert_design_file_exists "0001-new-title-with-spaces.md"
}

@test "rename-design: convert title to lowercase" {
    ./doc-engine.sh rename design -f 0001 "CamelCaseTitle" > /dev/null
    assert_design_file_exists "0001-camelcasetitle.md"
}

# ==============================================================================
# Sequential Renames
# ==============================================================================

@test "rename-design: rename multiple times" {
    ./doc-engine.sh rename design -f 0001 "Second Name" > /dev/null
    assert_design_file_exists "0001-second-name.md"

    ./doc-engine.sh rename design -f 0001 "Third Name" > /dev/null
    assert_file_not_exists "docs/design/0001-second-name.md"
    assert_design_file_exists "0001-third-name.md"

    assert_design_in_gitignore "0001-third-name.md"
    ! grep -q "0001-second-name.md" .gitignore
}

# ==============================================================================
# User Confirmation
# ==============================================================================

@test "rename-design: cancel rename when user says no" {
    [ -f "docs/design/0001-old-title.md" ]
    echo "no" | ./doc-engine.sh rename design "0001-old-title" "New Name"
    assert_design_file_exists "0001-old-title.md"
    assert_design_in_gitignore "0001-old-title.md"
    assert_file_not_exists "docs/design/0001-new-name.md"
}

@test "rename-design: show old and new paths before confirmation" {
    run_isolated bash -c 'echo "no" | ./doc-engine.sh rename design "0001-old-title" "New Name"'
    [[ "$output" == *"0001-old-title.md"* ]]
    [[ "$output" == *"0001-new-name.md"* ]]
}

# ==============================================================================
# Error Handling
# ==============================================================================

@test "rename-design: reject no arguments" {
    run_isolated ./doc-engine.sh rename design
    [ "$status" -eq 1 ]
    [[ "$output" == *"Usage:"* ]]
}

@test "rename-design: reject missing second argument" {
    run_isolated ./doc-engine.sh rename design 0001
    [ "$status" -eq 1 ]
    [[ "$output" == *"Usage:"* ]]
}

@test "rename-design: reject nonexistent design" {
    run_isolated ./doc-engine.sh rename design -f 9999 "New Title"
    [ "$status" -eq 1 ]
    [[ "$output" == *"Could not find Design Document matching"* ]]
}

@test "rename-design: reject identical old and new filename" {
    run_isolated ./doc-engine.sh rename design -f 0001-old-title.md "Old Title"
    [ "$status" -eq 1 ]
    [[ "$output" == *"identical to the old one"* ]]
}

@test "rename-design: reject destination file already exists" {
    # Create a second design
    cat > docs/design/0001-new-title.md <<'EOF'
# 1. New Title

Date: 2026-05-23

## Status
Proposed

## Summary
Test

## Problem / Motivation
Test

## Proposed Design
Test
EOF

    run_isolated ./doc-engine.sh rename design -f 0001-old-title.md "New Title"
    [ "$status" -eq 1 ]
    [[ "$output" == *"already exists"* ]]
}

# ==============================================================================
# Output Validation
# ==============================================================================

@test "rename-design: output indicates successful rename" {
    run_isolated ./doc-engine.sh rename design -f 0001 "New Title"
    [[ "$output" == *"Successfully renamed"* ]]
}

# ==============================================================================
# Link Integrity
# ==============================================================================

@test "rename-design: update links in other files" {
    # Create another md file referencing 0001-old-title.md
    cat > docs/another.md <<'EOF'
See [Design 1](design/0001-old-title.md) for context.
EOF

    # Create a rs file referencing 0001-old-title.md
    mkdir -p src
    cat > src/lib.rs <<'EOF'
//! See [Design 1](https://docs.rs/sdmx-rs/latest/sdmx_rs/design/0001-old-title.md)
EOF

    ./doc-engine.sh rename design -f 0001 "New Title" > /dev/null

    # Assert references are updated
    grep -q "0001-new-title.md" docs/another.md
    run ! grep -q "0001-old-title.md" docs/another.md

    grep -q "0001-new-title.md" src/lib.rs
    ! grep -q "0001-old-title.md" src/lib.rs
}
