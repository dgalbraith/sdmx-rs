#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================

# Test suite for scripts/remove-design.sh
#
# Testing approach: Integration tests for design document removal and cleanup.
# Validates safe deletion, reference checking, and error handling.
#
# Run with: bats tests/bats/remove-design.bats
# ==============================================================================
setup() {
    cd "$BATS_TEST_TMPDIR" || exit 1
    source "$BATS_TEST_DIRNAME/common.sh"
    setup_design_test

    mkdir -p docs/design
    cat > docs/design/0001-test.md <<'EOF'
# 1. Test Design

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

    git add docs/design/0001-test.md
    git commit -q -m "Add design"

    add_design_to_gitignore "0001-test.md"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# ==============================================================================
# File Deletion
# ==============================================================================

@test "remove-design: delete file with -f flag" {
    [ -f "docs/design/0001-test.md" ]
    ./doc-engine.sh remove design -f 0001-test.md > /dev/null
    assert_file_not_exists "docs/design/0001-test.md"
}

@test "remove-design: delete by prefix match (0001)" {
    ./doc-engine.sh remove design -f 0001 > /dev/null
    assert_file_not_exists "docs/design/0001-test.md"
}

# ==============================================================================
# Gitignore Cleanup
# ==============================================================================

@test "remove-design: remove entry from .gitignore with -f flag" {
    ./doc-engine.sh remove design -f 0001-test.md > /dev/null
    ! grep -q "0001-test.md" .gitignore
}

@test "remove-design: preserve .gitignore structure after removal" {
    ./doc-engine.sh remove design -f 0001-test.md > /dev/null

    grep -q "IX. Design Documentation" .gitignore
    grep -q "X. Architecture Decision Records" .gitignore
}

# ==============================================================================
# Multiple Entries
# ==============================================================================

@test "remove-design: handle multiple .gitignore entries" {
    # Add a second design
    cat > docs/design/0002-other.md <<'EOF'
# 2. Other

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

    git add docs/design/0002-other.md
    git commit -q -m "Add second design"
    add_design_to_gitignore "0002-other.md"

    # Remove first one
    ./doc-engine.sh remove design -f 0001-test.md > /dev/null

    run ! grep -q "0001-test.md" .gitignore
    grep -q "0002-other.md" .gitignore
}

# ==============================================================================
# User Confirmation
# ==============================================================================

@test "remove-design: cancel removal when user says no" {
    [ -f "docs/design/0001-test.md" ]
    echo "no" | ./doc-engine.sh remove design "0001-test.md"
    assert_design_file_exists "0001-test.md"
    assert_design_in_gitignore "0001-test.md"
}

@test "remove-design: show file path before confirmation" {
    run_isolated bash -c 'echo "no" | ./doc-engine.sh remove design "0001-test.md"'
    [[ "$output" == *"0001-test.md"* ]]
}

# ==============================================================================
# Error Handling
# ==============================================================================

@test "remove-design: reject no arguments" {
    run_isolated ./doc-engine.sh remove design
    [ "$status" -eq 1 ]
    [[ "$output" == *"Usage:"* ]]
}

@test "remove-design: reject missing design" {
    run_isolated ./doc-engine.sh remove design -f 9999
    [ "$status" -eq 1 ]
    [[ "$output" == *"Could not find Design Document matching"* ]]
}

@test "remove-design: reject ambiguous match" {
    # Create another file with similar prefix
    cat > docs/design/0001-other.md <<'EOF'
# 1. Other

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

    git add docs/design/0001-other.md
    git commit -q -m "Add second 0001"

    run_isolated ./doc-engine.sh remove design -f 0001
    [ "$status" -eq 1 ]
    [[ "$output" == *"Ambiguous target"* ]]
}

# ==============================================================================
# Output Validation
# ==============================================================================

@test "remove-design: output indicates successful removal" {
    run_isolated ./doc-engine.sh remove design -f 0001-test.md
    [[ "$output" == *"Successfully removed"* ]]
}

# ==============================================================================
# Link Integrity
# ==============================================================================

@test "remove-design: warns and prompts when dead links exist" {
    # Create another md file referencing 0001-test.md
    cat > docs/another.md <<'EOF'
See [Design 1](design/0001-test.md) for context.
EOF

    # Run with confirmation 'no' and verify it aborts and shows warning
    run_isolated bash -c 'echo "no" | ./doc-engine.sh remove design "0001-test.md"'
    [[ "$output" == *"Warning: Dead links/references detected"* ]]
    [[ "$output" == *"docs/another.md"* ]]
    [[ "$output" == *"Dead links detected. Do you still want to proceed"* ]]
    assert_design_file_exists "0001-test.md"

    # Run with confirmation 'yes' and verify it deletes anyway
    run_isolated bash -c 'echo -e "y\ny" | ./doc-engine.sh remove design "0001-test.md"'
    assert_file_not_exists "docs/design/0001-test.md"
}

@test "remove-design: warns but proceeds when dead links exist under force flag" {
    # Create another md file referencing 0001-test.md
    cat > docs/another.md <<'EOF'
See [Design 1](design/0001-test.md) for context.
EOF

    # Run under -f flag and verify it deletes, printing warnings to stderr
    run_isolated ./doc-engine.sh remove design -f "0001-test.md"
    [[ "$output" == *"Warning: Dead links/references"* ]]
    assert_file_not_exists "docs/design/0001-test.md"
}
