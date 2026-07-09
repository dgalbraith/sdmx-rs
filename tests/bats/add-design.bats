#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/add-design.sh
#
# Testing approach: Integration tests for design document lifecycle management.
# Validates creation, numbering, metadata generation, and error handling.
#
# Run with: bats tests/bats/add-design.bats
# ==============================================================================

setup() {
    cd "$BATS_TEST_TMPDIR" || exit 1
    source "$BATS_TEST_DIRNAME/common.sh"
    setup_design_test
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# ==============================================================================
# File Creation and Numbering
# ==============================================================================

@test "add-design: create first design (0001)" {
    run_isolated ./doc-engine.sh add design "First Design"
    [ "$status" -eq 0 ]
    assert_design_file_exists "0001-first-design.md"
    [[ "$output" == *"Created: docs/design/0001-first-design.md"* ]]
}

@test "add-design: sequential numbering (0001 → 0002 → 0003)" {
    ./doc-engine.sh add design "Alpha" > /dev/null
    ./doc-engine.sh add design "Beta" > /dev/null
    ./doc-engine.sh add design "Gamma" > /dev/null

    assert_design_file_exists "0001-alpha.md"
    assert_design_file_exists "0002-beta.md"
    assert_design_file_exists "0003-gamma.md"
}

# ==============================================================================
# Title Sanitisation
# ==============================================================================

@test "add-design: sanitise title with slashes" {
    run_isolated ./doc-engine.sh add design "Architecture/Design/Notes"
    [ "$status" -eq 0 ]
    assert_design_file_exists "0001-architecture-design-notes.md"
}

@test "add-design: convert title to lowercase" {
    run_isolated ./doc-engine.sh add design "CamelCaseDesign"
    [ "$status" -eq 0 ]
    assert_design_file_exists "0001-camelcasedesign.md"
}

@test "add-design: convert spaces to dashes" {
    run_isolated ./doc-engine.sh add design "Multi Word Design"
    [ "$status" -eq 0 ]
    assert_design_file_exists "0001-multi-word-design.md"
}

# ==============================================================================
# Gitignore Registration
# ==============================================================================

@test "add-design: register in .gitignore" {
    ./doc-engine.sh add design "Test Design" > /dev/null
    assert_design_in_gitignore "0001-test-design.md"
}

@test "add-design: register multiple entries in .gitignore" {
    ./doc-engine.sh add design "First" > /dev/null
    ./doc-engine.sh add design "Second" > /dev/null
    ./doc-engine.sh add design "Third" > /dev/null

    assert_design_in_gitignore "0001-first.md"
    assert_design_in_gitignore "0002-second.md"
    assert_design_in_gitignore "0003-third.md"
}

# ==============================================================================
# Template Content Validation
# ==============================================================================

@test "add-design: set status to Proposed" {
    ./doc-engine.sh add design "Test Design" > /dev/null
    grep -q "^Proposed$" docs/design/0001-test-design.md
}

@test "add-design: include date in YYYY-MM-DD format" {
    ./doc-engine.sh add design "Test Design" > /dev/null
    grep -q "^Date:" docs/design/0001-test-design.md
    grep "^Date:" docs/design/0001-test-design.md | grep -Eq "[0-9]{4}-[0-9]{2}-[0-9]{2}"
}

@test "add-design: include required design template sections" {
    ./doc-engine.sh add design "Test Design" > /dev/null
    grep -q "^## Summary$" docs/design/0001-test-design.md
    grep -q "^## Problem / Motivation$" docs/design/0001-test-design.md
    grep -q "^## Proposed Design$" docs/design/0001-test-design.md
}

@test "add-design: use Proposed status (not Accepted)" {
    ./doc-engine.sh add design "Test Design" > /dev/null
    grep -q "^Proposed$" docs/design/0001-test-design.md
    ! grep -q "^Accepted$" docs/design/0001-test-design.md
}

# ==============================================================================
# Error Handling
# ==============================================================================

@test "add-design: reject empty title" {
    run_isolated ./doc-engine.sh add design ""
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error: Design Document title cannot be empty"* ]]
}

@test "add-design: reject no arguments" {
    run_isolated ./doc-engine.sh add design
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error: Design Document title cannot be empty"* ]]
}

# ==============================================================================
# Output Validation
# ==============================================================================

@test "add-design: output indicates file creation" {
    run_isolated ./doc-engine.sh add design "Test Design"
    [[ "$output" == *"Created: docs/design/0001-test-design.md"* ]]
}

@test "add-design: output indicates gitignore registration" {
    run_isolated ./doc-engine.sh add design "Test Design"
    [[ "$output" == *"Semantically registered"* ]]
    [[ "$output" == *".gitignore"* ]]
}
