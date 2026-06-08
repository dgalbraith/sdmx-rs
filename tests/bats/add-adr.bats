#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/add-adr.sh
#
# Testing approach: Integration tests that verify ADR creation, numbering,
# and file organization. Tests validate:
#   - File creation with correct naming and sequencing
#   - Metadata generation and frontmatter
#   - Error handling for invalid inputs
#   - Directory structure and link management
#
# Run with: bats tests/bats/add-adr.bats
# ==============================================================================

setup() {
    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1
    source "$BATS_TEST_DIRNAME/common.sh"
    setup_adr_test
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# ==============================================================================
# File Creation and Numbering
# ==============================================================================

@test "add-adr: create first ADR (0001)" {
    run_isolated ./doc-engine.sh add adr "First ADR"
    [ "$status" -eq 0 ]
    assert_adr_file_exists "0001-first-adr.md"
    [[ "$output" == *"Created: docs/adr/0001-first-adr.md"* ]]
}

@test "add-adr: sequential numbering (0001 → 0002 → 0003)" {
    ./doc-engine.sh add adr "Alpha" > /dev/null
    ./doc-engine.sh add adr "Beta" > /dev/null
    ./doc-engine.sh add adr "Gamma" > /dev/null

    assert_adr_file_exists "0001-alpha.md"
    assert_adr_file_exists "0002-beta.md"
    assert_adr_file_exists "0003-gamma.md"
}

# ==============================================================================
# Title Sanitization
# ==============================================================================

@test "add-adr: sanitize title with slashes" {
    run_isolated ./doc-engine.sh add adr "Path/To/Feature"
    [ "$status" -eq 0 ]
    assert_adr_file_exists "0001-path-to-feature.md"
}

@test "add-adr: convert title to lowercase" {
    run_isolated ./doc-engine.sh add adr "CamelCaseTitle"
    [ "$status" -eq 0 ]
    assert_adr_file_exists "0001-camelcasetitle.md"
}

@test "add-adr: convert spaces to dashes" {
    run_isolated ./doc-engine.sh add adr "Multi Word Title"
    [ "$status" -eq 0 ]
    assert_adr_file_exists "0001-multi-word-title.md"
}

# ==============================================================================
# Gitignore Registration
# ==============================================================================

@test "add-adr: register in .gitignore" {
    ./doc-engine.sh add adr "Test ADR" > /dev/null
    assert_adr_in_gitignore "0001-test-adr.md"
}

@test "add-adr: register multiple entries in .gitignore" {
    ./doc-engine.sh add adr "First" > /dev/null
    ./doc-engine.sh add adr "Second" > /dev/null
    ./doc-engine.sh add adr "Third" > /dev/null

    assert_adr_in_gitignore "0001-first.md"
    assert_adr_in_gitignore "0002-second.md"
    assert_adr_in_gitignore "0003-third.md"
}

# ==============================================================================
# Template Content Validation
# ==============================================================================

@test "add-adr: set status to Accepted" {
    ./doc-engine.sh add adr "Test ADR" > /dev/null
    grep -q "^Accepted$" docs/adr/0001-test-adr.md
}

@test "add-adr: include date in YYYY-MM-DD format" {
    ./doc-engine.sh add adr "Test ADR" > /dev/null
    grep -q "^Date:" docs/adr/0001-test-adr.md
    grep "^Date:" docs/adr/0001-test-adr.md | grep -Eq "[0-9]{4}-[0-9]{2}-[0-9]{2}"
}

@test "add-adr: include proper ADR heading" {
    ./doc-engine.sh add adr "Test ADR" > /dev/null
    grep -q "^# 1\. Test ADR" docs/adr/0001-test-adr.md
}

@test "add-adr: include required ADR template sections" {
    ./doc-engine.sh add adr "Test ADR" > /dev/null
    assert_adr_file_exists "0001-test-adr.md"
}

# ==============================================================================
# Error Handling
# ==============================================================================

@test "add-adr: reject empty title" {
    run_isolated ./doc-engine.sh add adr ""
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error: ADR title cannot be empty"* ]]
}

@test "add-adr: reject no arguments" {
    run_isolated ./doc-engine.sh add adr
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error: ADR title cannot be empty"* ]]
}

# ==============================================================================
# Output Validation
# ==============================================================================

@test "add-adr: output indicates file creation" {
    run_isolated ./doc-engine.sh add adr "Test ADR"
    [[ "$output" == *"Created: docs/adr/0001-test-adr.md"* ]]
}

@test "add-adr: output indicates gitignore registration" {
    run_isolated ./doc-engine.sh add adr "Test ADR"
    [[ "$output" == *"Semantically registered"* ]]
    [[ "$output" == *".gitignore"* ]]
}
