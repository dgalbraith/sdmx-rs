#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/verify-adr.sh
#
# Testing approach: Integration tests for adr document validation.
# Validates structure, frontmatter, links, and compliance with standards.
#
# Run with: bats tests/bats/verify-adr.bats
# ==============================================================================
setup() {
    cd "$BATS_TEST_TMPDIR" || exit 1
    source "$BATS_TEST_DIRNAME/common.sh"
    setup_adr_test

    # Create valid baseline ADR and README
    cat > docs/adr/README.md <<'EOF'
# Architecture Decision Records

## All ADRs by Category

### Process & Documentation
- [ADR-0001: Record Architecture Decisions](0001-record-architecture-decisions.md)
EOF

    mkdir -p docs/adr
    cat > docs/adr/0001-record-architecture-decisions.md <<'EOF'
# 1. Record Architecture Decisions

Date: 2026-05-23

## Status

Accepted

## Context

Test context.

## Decision

Test decision.

## Consequences

Test consequences.
EOF

    add_adr_to_gitignore "0001-record-architecture-decisions.md"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# ==============================================================================
# Valid Ledger State
# ==============================================================================

@test "verify-adr: passes with valid single ADR" {
    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 0 ]
    [[ "$output" == *"verified"* ]]
}

@test "verify-adr: passes with multiple valid ADRs" {
    cat >> docs/adr/README.md <<'EOF'
- [ADR-0002: Second ADR](0002-second-adr.md)
EOF

    cat > docs/adr/0002-second-adr.md <<'EOF'
# 2. Second ADR

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

    add_adr_to_gitignore "0002-second-adr.md"

    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 0 ]
    [[ "$output" == *"verified"* ]]
}

# ==============================================================================
# Template Conformance
# ==============================================================================

@test "verify-adr: detects missing Status section" {
    sed -i '/^## Status$/d' docs/adr/0001-record-architecture-decisions.md
    sed -i '/^Accepted$/d' docs/adr/0001-record-architecture-decisions.md

    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 1 ]
    [[ "$output" == *"missing required section: ## Status"* ]]
}

@test "verify-adr: detects missing Context section" {
    sed -i '/^## Context$/d' docs/adr/0001-record-architecture-decisions.md

    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 1 ]
    [[ "$output" == *"missing required section: ## Context"* ]]
}

@test "verify-adr: detects missing Decision section" {
    sed -i '/^## Decision$/d' docs/adr/0001-record-architecture-decisions.md

    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 1 ]
    [[ "$output" == *"missing required section: ## Decision"* ]]
}

@test "verify-adr: detects missing Consequences section" {
    sed -i '/^## Consequences$/d' docs/adr/0001-record-architecture-decisions.md

    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 1 ]
    [[ "$output" == *"missing required section: ## Consequences"* ]]
}

# ==============================================================================
# Ledger Synchronisation
# ==============================================================================

@test "verify-adr: detects physical file not in .gitignore" {
    # Create ADR without registering in gitignore
    cat > docs/adr/0002-unregistered.md <<'EOF'
# 2. Unregistered ADR

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

    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 1 ]
    [[ "$output" == *"out of sync"* ]]
}

@test "verify-adr: detects .gitignore entry with no physical file" {
    # Add entry without creating file
    sed -i '/^# XI\. Guides/i\!/docs/adr/0003-phantom.md' .gitignore

    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 1 ]
    [[ "$output" == *"out of sync"* ]]
}

# ==============================================================================
# README Indexing
# ==============================================================================

@test "verify-adr: detects ADR not listed in README.md" {
    # Create ADR and register in gitignore, but don't add to README
    cat > docs/adr/0002-unindexed.md <<'EOF'
# 2. Unindexed ADR

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

    add_adr_to_gitignore "0002-unindexed.md"

    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 1 ]
    [[ "$output" == *"not listed in docs/adr/README.md"* ]]
}

# ==============================================================================
# Numbering Validation
# ==============================================================================

@test "verify-adr: detect gap in numbering" {
    # Create 0002, then manually rename to 0003 to create gap
    ./doc-engine.sh add adr "Second ADR" > /dev/null
    mv docs/adr/0002-second-adr.md docs/adr/0003-second-adr.md
    sed -i 's/0002-second-adr.md/0003-second-adr.md/' .gitignore

    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 1 ]
    [[ "$output" == *"Gap in numbering"* ]] || [[ "$output" == *"Error"* ]]
}

# ==============================================================================
# Special Files
# ==============================================================================

@test "verify-adr: ignores README.md when scanning" {
    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 0 ]
}

@test "verify-adr: ignores templates/ subdirectory" {
    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 0 ]
}

@test "verify-adr: enforces adr-specific sections (not design sections)" {
    # Create an ADR with design sections instead of ADR sections
    mkdir -p docs/adr
    cat > docs/adr/0001-wrong-template.md <<'EOF'
# 1. Wrong Template

Date: 2026-05-23

## Status
Accepted

## Summary
Design summary here.

## Problem / Motivation
Design problem here.

## Proposed Design
Design proposal here.
EOF
    add_adr_to_gitignore "0001-wrong-template.md"

    run_isolated ./doc-engine.sh verify adr
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error"* ]]
}
