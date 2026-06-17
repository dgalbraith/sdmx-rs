#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/verify-design.sh
#
# Testing approach: Integration tests for design document validation.
# Validates structure, frontmatter, links, and compliance with standards.
#
# Run with: bats tests/bats/verify-design.bats
# ==============================================================================
setup() {
    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1
    source "$BATS_TEST_DIRNAME/common.sh"
    setup_design_test

    # Create valid baseline design and README
    cat > docs/design/README.md <<'EOF'
# Design Documentation — Index

## Design Documents

- [0001: Design Documentation Process](0001-design-documentation-process.md)
EOF

    mkdir -p docs/design
    cat > docs/design/0001-design-documentation-process.md <<'EOF'
# 1. Design Documentation Process

Date: 2026-05-23

## Status

Accepted

## Summary

Test summary.

## Problem / Motivation

Test problem.

## Proposed Design

Test design.
EOF

    add_design_to_gitignore "0001-design-documentation-process.md"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# ==============================================================================
# Valid Ledger State
# ==============================================================================

@test "verify-design: passes with valid single design" {
    run_isolated ./doc-engine.sh verify design
    [ "$status" -eq 0 ]
    [[ "$output" == *"verified"* ]]
}

@test "verify-design: passes with multiple valid designs" {
    cat >> docs/design/README.md <<'EOF'
- [0002: Second Design](0002-second-design.md)
EOF

    cat > docs/design/0002-second-design.md <<'EOF'
# 2. Second Design

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

    add_design_to_gitignore "0002-second-design.md"

    run_isolated ./doc-engine.sh verify design
    [ "$status" -eq 0 ]
    [[ "$output" == *"verified"* ]]
}

# ==============================================================================
# Template Conformance
# ==============================================================================

@test "verify-design: detects missing Status section" {
    sed -i '/^## Status$/d' docs/design/0001-design-documentation-process.md
    sed -i '/^Accepted$/d' docs/design/0001-design-documentation-process.md

    run_isolated ./doc-engine.sh verify design
    [ "$status" -eq 1 ]
    [[ "$output" == *"missing required section: ## Status"* ]]
}

@test "verify-design: detects missing Summary section" {
    sed -i '/^## Summary$/d' docs/design/0001-design-documentation-process.md

    run_isolated ./doc-engine.sh verify design
    [ "$status" -eq 1 ]
    [[ "$output" == *"missing required section: ## Summary"* ]]
}

@test "verify-design: detects missing Problem / Motivation section" {
    sed -i '/^## Problem \/ Motivation$/d' docs/design/0001-design-documentation-process.md

    run_isolated ./doc-engine.sh verify design
    [ "$status" -eq 1 ]
    [[ "$output" == *"missing required section: ## Problem / Motivation"* ]]
}

@test "verify-design: detects missing Proposed Design section" {
    sed -i '/^## Proposed Design$/d' docs/design/0001-design-documentation-process.md

    run_isolated ./doc-engine.sh verify design
    [ "$status" -eq 1 ]
    [[ "$output" == *"missing required section: ## Proposed Design"* ]]
}

# ==============================================================================
# Ledger Synchronisation
# ==============================================================================

@test "verify-design: detects physical file not in .gitignore" {
    # Create design without registering in gitignore
    cat > docs/design/0002-unregistered.md <<'EOF'
# 2. Unregistered Design

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

    run_isolated ./doc-engine.sh verify design
    [ "$status" -eq 1 ]
    [[ "$output" == *"out of sync"* ]]
}

@test "verify-design: detects .gitignore entry with no physical file" {
    # Add entry without creating file
    sed -i '/^# X\. Architecture Decision Records/i\!/docs/design/0003-phantom.md' .gitignore

    run_isolated ./doc-engine.sh verify design
    [ "$status" -eq 1 ]
    [[ "$output" == *"out of sync"* ]]
}

# ==============================================================================
# README Indexing
# ==============================================================================

@test "verify-design: detects design not listed in README.md" {
    # Create design and register in gitignore, but don't add to README
    cat > docs/design/0002-unindexed.md <<'EOF'
# 2. Unindexed Design

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

    add_design_to_gitignore "0002-unindexed.md"

    run_isolated ./doc-engine.sh verify design
    [ "$status" -eq 1 ]
    [[ "$output" == *"not listed in docs/design/README.md"* ]]
}

# ==============================================================================
# Numbering Validation
# ==============================================================================

@test "verify-design: detect gap in numbering" {
    # Create 0002, then manually rename to 0003 to create gap
    ./doc-engine.sh add design "Second Design" > /dev/null
    mv docs/design/0002-second-design.md docs/design/0003-second-design.md
    sed -i 's/0002-second-design.md/0003-second-design.md/' .gitignore

    run_isolated ./doc-engine.sh verify design
    [ "$status" -eq 1 ]
    [[ "$output" == *"Gap in numbering"* ]] || [[ "$output" == *"Error"* ]]
}

# ==============================================================================
# Special Files and Template Differences
# ==============================================================================

@test "verify-design: ignores README.md when scanning" {
    run_isolated ./doc-engine.sh verify design
    [ "$status" -eq 0 ]
}

@test "verify-design: ignores templates/ subdirectory" {
    run_isolated ./doc-engine.sh verify design
    [ "$status" -eq 0 ]
}

@test "verify-design: enforces design-specific sections (not ADR sections)" {
    # Verify design requires "Proposed Design" not "Decision"
    grep -q "^## Proposed Design$" docs/design/0001-design-documentation-process.md
    ! grep -q "^## Decision$" docs/design/0001-design-documentation-process.md
}
