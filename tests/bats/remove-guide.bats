#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/remove-guide.sh
#
# Testing approach: Integration tests for guide document removal and cleanup.
# Validates safe deletion, reference checking, and error handling.
#
# Run with: bats tests/bats/remove-guide.bats
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
# File Deletion
# ==============================================================================

@test "remove-guide: delete file with -f flag" {
    ./doc-engine.sh add guide "Test Guide" > /dev/null
    ./doc-engine.sh remove guide -f test-guide.md > /dev/null
    assert_file_not_exists "docs/guides/test-guide.md"
}

@test "remove-guide: delete by slug match" {
    ./doc-engine.sh add guide "Test Guide" > /dev/null
    ./doc-engine.sh remove guide -f test-guide > /dev/null
    assert_file_not_exists "docs/guides/test-guide.md"
}

# ==============================================================================
# Gitignore Cleanup
# ==============================================================================

@test "remove-guide: remove entry from .gitignore with -f flag" {
    ./doc-engine.sh add guide "Test Guide" > /dev/null
    ./doc-engine.sh remove guide -f test-guide.md > /dev/null
    ! grep -q "test-guide.md" .gitignore
}

@test "remove-guide: preserve .gitignore structure after removal" {
    ./doc-engine.sh add guide "Test Guide" > /dev/null
    ./doc-engine.sh remove guide -f test-guide.md > /dev/null

    grep -q "XI. Guides" .gitignore
    grep -q "XII. Source Code & Workspace Packages" .gitignore
}

# ==============================================================================
# Multiple Entries
# ==============================================================================

@test "remove-guide: handle multiple .gitignore entries" {
    ./doc-engine.sh add guide "First" > /dev/null
    ./doc-engine.sh add guide "Second" > /dev/null
    ./doc-engine.sh add guide "Third" > /dev/null

    ./doc-engine.sh remove guide -f second.md > /dev/null

    assert_guide_in_gitignore "first.md"
    assert_guide_in_gitignore "third.md"
    ! grep -q "second.md" .gitignore
}

# ==============================================================================
# User Confirmation
# ==============================================================================

@test "remove-guide: cancel removal when user says no" {
    ./doc-engine.sh add guide "Test Guide" > /dev/null
    echo "no" | ./doc-engine.sh remove guide "test-guide.md"
    assert_guide_file_exists "test-guide.md"
    assert_guide_in_gitignore "test-guide.md"
}

@test "remove-guide: show file path before confirmation" {
    ./doc-engine.sh add guide "Test Guide" > /dev/null
    run_isolated bash -c 'echo "no" | ./doc-engine.sh remove guide "test-guide.md"'
    [[ "$output" == *"test-guide.md"* ]]
}

# ==============================================================================
# Error Handling
# ==============================================================================

@test "remove-guide: reject no arguments" {
    run_isolated ./doc-engine.sh remove guide
    [ "$status" -eq 1 ]
    [[ "$output" == *"Usage:"* ]]
}

@test "remove-guide: reject missing guide" {
    run_isolated ./doc-engine.sh remove guide -f nonexistent
    [ "$status" -eq 1 ]
    [[ "$output" == *"Could not find"* ]] || [[ "$output" == *"Error"* ]]
}

@test "remove-guide: reject ambiguous match" {
    ./doc-engine.sh add guide "First Test" > /dev/null
    # Create another file with similar prefix by directly adding it
    cat > docs/guides/first-other.md <<'EOF'
# 1. Other Guide

Date: 2026-05-24

## Overview
Guide overview.

## Prerequisites
Some prerequisites.

## Step-by-Step
Steps here.
EOF
    add_guide_to_gitignore "first-other.md"

    run_isolated ./doc-engine.sh remove guide -f first
    [ "$status" -eq 1 ]
    [[ "$output" == *"Ambiguous"* ]] || [[ "$output" == *"multiple"* ]]
}

# ==============================================================================
# Output Validation
# ==============================================================================

@test "remove-guide: output indicates successful removal" {
    ./doc-engine.sh add guide "Test Guide" > /dev/null
    run_isolated ./doc-engine.sh remove guide -f test-guide.md
    [[ "$output" == *"Successfully removed"* ]]
}

# ==============================================================================
# Link Integrity
# ==============================================================================

@test "remove-guide: warns and prompts when dead links exist" {
    ./doc-engine.sh add guide "Test Guide" > /dev/null

    # Create another md file referencing test-guide.md
    cat > docs/another.md <<'EOF'
See [Guide 1](guides/test-guide.md) for context.
EOF

    # Run with confirmation 'no' and verify it aborts and shows warning
    run_isolated bash -c 'echo "no" | ./doc-engine.sh remove guide test-guide.md'
    [[ "$output" == *"Warning: Dead links/references detected"* ]]
    [[ "$output" == *"docs/another.md"* ]]
    [[ "$output" == *"Dead links detected. Do you still want to proceed"* ]]
    assert_guide_file_exists "test-guide.md"

    # Run with confirmation 'yes' and verify it deletes anyway
    run_isolated bash -c 'echo -e "yes\nyes" | ./doc-engine.sh remove guide test-guide.md'
    assert_file_not_exists "docs/guides/test-guide.md"
}

@test "remove-guide: warns but proceeds when dead links exist under force flag" {
    ./doc-engine.sh add guide "Test Guide" > /dev/null

    # Create another md file referencing test-guide.md
    cat > docs/another.md <<'EOF'
See [Guide 1](guides/test-guide.md) for context.
EOF

    # Run under -f flag and verify it deletes, printing warnings to stderr
    run_isolated ./doc-engine.sh remove guide -f test-guide.md
    [[ "$output" == *"Warning: Dead links/references"* ]]
    assert_file_not_exists "docs/guides/test-guide.md"
}
