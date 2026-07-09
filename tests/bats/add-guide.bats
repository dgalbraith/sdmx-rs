#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/add-guide.sh
#
# Testing approach: Integration tests for guide document lifecycle management.
# Validates creation with slug-based naming, metadata generation, and content structure.
#
# Run with: bats tests/bats/add-guide.bats
# ==============================================================================
setup() {
    cd "$BATS_TEST_TMPDIR" || exit 1
    source "$BATS_TEST_DIRNAME/common.sh"
    setup_guide_test
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# ==============================================================================
# File Creation
# ==============================================================================

@test "add-guide: create guide with slug-based name" {
    run_isolated ./doc-engine.sh add guide "First Guide"
    [ "$status" -eq 0 ]
    assert_guide_file_exists "first-guide.md"
    [[ "$output" == *"Created: docs/guides/first-guide.md"* ]]
}

@test "add-guide: create multiple guides with unique slugs" {
    ./doc-engine.sh add guide "Alpha" > /dev/null
    ./doc-engine.sh add guide "Beta" > /dev/null
    ./doc-engine.sh add guide "Gamma" > /dev/null

    assert_guide_file_exists "alpha.md"
    assert_guide_file_exists "beta.md"
    assert_guide_file_exists "gamma.md"
}

# ==============================================================================
# Title Sanitisation
# ==============================================================================

@test "add-guide: sanitise title with slashes" {
    run_isolated ./doc-engine.sh add guide "Getting Started/Setup/Configuration"
    [ "$status" -eq 0 ]
    assert_guide_file_exists "getting-started-setup-configuration.md"
}

@test "add-guide: convert title to lowercase" {
    run_isolated ./doc-engine.sh add guide "WorkingWithBuilders"
    [ "$status" -eq 0 ]
    assert_guide_file_exists "workingwithbuilders.md"
}

@test "add-guide: convert spaces to dashes" {
    run_isolated ./doc-engine.sh add guide "Querying Data Structures"
    [ "$status" -eq 0 ]
    assert_guide_file_exists "querying-data-structures.md"
}

# ==============================================================================
# Gitignore Registration
# ==============================================================================

@test "add-guide: register in .gitignore" {
    ./doc-engine.sh add guide "Test Guide" > /dev/null
    assert_guide_in_gitignore "test-guide.md"
}

@test "add-guide: register multiple entries in .gitignore" {
    ./doc-engine.sh add guide "First" > /dev/null
    ./doc-engine.sh add guide "Second" > /dev/null
    ./doc-engine.sh add guide "Third" > /dev/null

    assert_guide_in_gitignore "first.md"
    assert_guide_in_gitignore "second.md"
    assert_guide_in_gitignore "third.md"
}

# ==============================================================================
# Template Content Validation
# ==============================================================================

@test "add-guide: include Last Updated date in YYYY-MM-DD format" {
    ./doc-engine.sh add guide "Test Guide" > /dev/null
    grep -q "^Last Updated:" docs/guides/test-guide.md
    grep "^Last Updated:" docs/guides/test-guide.md | grep -Eq "[0-9]{4}-[0-9]{2}-[0-9]{2}"
}

@test "add-guide: include required guide template sections" {
    ./doc-engine.sh add guide "Test Guide" > /dev/null
    grep -q "^## Overview$" docs/guides/test-guide.md
    grep -q "^## Prerequisites$" docs/guides/test-guide.md
    grep -q "^## Examples$" docs/guides/test-guide.md
    grep -q "^## Troubleshooting$" docs/guides/test-guide.md
    grep -q "^## Next Steps$" docs/guides/test-guide.md
    grep -q "^## Notes$" docs/guides/test-guide.md
}

@test "add-guide: guide title in first heading" {
    ./doc-engine.sh add guide "My Test Guide" > /dev/null
    grep -q "^# My Test Guide$" docs/guides/my-test-guide.md
}

# ==============================================================================
# Error Handling
# ==============================================================================

@test "add-guide: reject empty title" {
    run_isolated ./doc-engine.sh add guide ""
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error: Guide title cannot be empty"* ]]
}

@test "add-guide: reject no arguments" {
    run_isolated ./doc-engine.sh add guide
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error: Guide title cannot be empty"* ]]
}

@test "add-guide: create multiple guides simultaneously" {
    # Create multiple guides and ensure they all succeed
    ./doc-engine.sh add guide "First" > /dev/null
    ./doc-engine.sh add guide "Second" > /dev/null
    # Both should exist with slug-based names
    [ -f "docs/guides/first.md" ]
    [ -f "docs/guides/second.md" ]
}

# ==============================================================================
# Output Validation
# ==============================================================================

@test "add-guide: output indicates file creation" {
    run_isolated ./doc-engine.sh add guide "Test Guide"
    [[ "$output" == *"Created: docs/guides/test-guide.md"* ]]
}

@test "add-guide: output indicates gitignore registration" {
    run_isolated ./doc-engine.sh add guide "Test Guide"
    [[ "$output" == *"Semantically registered"* ]]
    [[ "$output" == *".gitignore"* ]]
}
