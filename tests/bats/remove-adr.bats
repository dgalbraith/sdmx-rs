#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================

# Test suite for scripts/remove-adr.sh
#
# Testing approach: Integration tests for adr document removal and cleanup.
# Validates safe deletion, reference checking, and error handling.
#
# Run with: bats tests/bats/remove-adr.bats
# ==============================================================================
setup() {
    cd "$BATS_TEST_TMPDIR" || exit 1
    source "$BATS_TEST_DIRNAME/common.sh"
    setup_adr_test

    mkdir -p docs/adr
    cat > docs/adr/0001-test.md <<'EOF'
# 1. Test

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

    git add docs/adr/0001-test.md
    git commit -q -m "Add ADR"

    add_adr_to_gitignore "0001-test.md"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# ==============================================================================
# File Deletion
# ==============================================================================

@test "remove-adr: delete file with -f flag" {
    [ -f "docs/adr/0001-test.md" ]
    ./doc-engine.sh remove adr -f 0001-test.md > /dev/null
    assert_file_not_exists "docs/adr/0001-test.md"
}

@test "remove-adr: delete by prefix match (0001)" {
    ./doc-engine.sh remove adr -f 0001 > /dev/null
    assert_file_not_exists "docs/adr/0001-test.md"
}

# ==============================================================================
# Gitignore Cleanup
# ==============================================================================

@test "remove-adr: remove entry from .gitignore with -f flag" {
    ./doc-engine.sh remove adr -f 0001-test.md > /dev/null
    ! grep -q "0001-test.md" .gitignore
}

@test "remove-adr: preserve .gitignore structure after removal" {
    ./doc-engine.sh remove adr -f 0001-test.md > /dev/null

    grep -q "X. Architecture Decision Records" .gitignore
    grep -q "XI. Guides" .gitignore
}

# ==============================================================================
# Multiple Entries
# ==============================================================================

@test "remove-adr: handle multiple .gitignore entries" {
    # Add a second ADR
    cat > docs/adr/0002-other.md <<'EOF'
# 2. Other

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

    git add docs/adr/0002-other.md
    git commit -q -m "Add second ADR"
    add_adr_to_gitignore "0002-other.md"

    # Remove first one
    ./doc-engine.sh remove adr -f 0001-test.md > /dev/null

    run ! grep -q "0001-test.md" .gitignore
    grep -q "0002-other.md" .gitignore
}

# ==============================================================================
# User Confirmation
# ==============================================================================

@test "remove-adr: cancel removal when user says no" {
    [ -f "docs/adr/0001-test.md" ]
    echo "no" | ./doc-engine.sh remove adr "0001-test.md"
    assert_adr_file_exists "0001-test.md"
    assert_adr_in_gitignore "0001-test.md"
}

@test "remove-adr: show file path before confirmation" {
    run_isolated bash -c 'echo "no" | ./doc-engine.sh remove adr "0001-test.md"'
    [[ "$output" == *"0001-test.md"* ]]
}

# ==============================================================================
# Error Handling
# ==============================================================================

@test "remove-adr: reject no arguments" {
    run_isolated ./doc-engine.sh remove adr
    [ "$status" -eq 1 ]
    [[ "$output" == *"Usage:"* ]]
}

@test "remove-adr: reject missing ADR" {
    run_isolated ./doc-engine.sh remove adr -f 9999
    [ "$status" -eq 1 ]
    [[ "$output" == *"Could not find ADR matching"* ]]
}

@test "remove-adr: reject ambiguous match" {
    # Create another file with similar prefix
    cat > docs/adr/0001-other.md <<'EOF'
# 1. Other

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

    git add docs/adr/0001-other.md
    git commit -q -m "Add second 0001"

    run_isolated ./doc-engine.sh remove adr -f 0001
    [ "$status" -eq 1 ]
    [[ "$output" == *"Ambiguous target"* ]]
}

# ==============================================================================
# Output Validation
# ==============================================================================

@test "remove-adr: output indicates successful removal" {
    run_isolated ./doc-engine.sh remove adr -f 0001-test.md
    [[ "$output" == *"Successfully removed"* ]]
}

# ==============================================================================
# Link Integrity
# ==============================================================================

@test "remove-adr: warns and prompts when dead links exist" {
    # Create another md file referencing 0001-test.md
    cat > docs/another.md <<'EOF'
See [ADR 1](adr/0001-test.md) for context.
EOF

    # Run with confirmation 'no' and verify it aborts and shows warning
    run_isolated bash -c 'echo "no" | ./doc-engine.sh remove adr "0001-test.md"'
    [[ "$output" == *"Warning: Dead links/references detected"* ]]
    [[ "$output" == *"docs/another.md"* ]]
    [[ "$output" == *"Dead links detected. Do you still want to proceed"* ]]
    assert_adr_file_exists "0001-test.md"

    # Run with confirmation 'yes' and verify it deletes anyway
    run_isolated bash -c 'echo -e "y\ny" | ./doc-engine.sh remove adr "0001-test.md"'
    assert_file_not_exists "docs/adr/0001-test.md"
}

@test "remove-adr: warns but proceeds when dead links exist under force flag" {
    # Create another md file referencing 0001-test.md
    cat > docs/another.md <<'EOF'
See [ADR 1](adr/0001-test.md) for context.
EOF

    # Run under -f flag and verify it deletes, printing warnings to stderr
    run_isolated ./doc-engine.sh remove adr -f "0001-test.md"
    [[ "$output" == *"Warning: Dead links/references"* ]]
    assert_file_not_exists "docs/adr/0001-test.md"
}
