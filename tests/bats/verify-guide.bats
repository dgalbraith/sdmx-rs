#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/verify-guide.sh
#
# Testing approach: Integration tests for guide document validation.
# Validates structure, frontmatter, links, and compliance with standards.
#
# Run with: bats tests/bats/verify-guide.bats
# ==============================================================================
setup() {
    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1
    source "$BATS_TEST_DIRNAME/common.sh"
    setup_guide_test
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# ==============================================================================
# Validation Success Cases
# ==============================================================================

@test "verify-guide: pass with no guides" {
    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 0 ]
    [[ "$output" == *"No guides found"* ]]
}

@test "verify-guide: pass with single guide" {
    ./doc-engine.sh add guide "First Guide" > /dev/null
    # Add guide to README.md index
    echo "- [First Guide](first-guide.md)" >> docs/guides/README.md
    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 0 ]
    [[ "$output" == *"first-guide.md"* ]]
    [[ "$output" == *"verified"* ]]
}

@test "verify-guide: pass with multiple guides" {
    ./doc-engine.sh add guide "First" > /dev/null
    ./doc-engine.sh add guide "Second" > /dev/null
    ./doc-engine.sh add guide "Third" > /dev/null
    # Add guides to README.md index
    cat >> docs/guides/README.md << 'EOF'
- [First](first.md)
- [Second](second.md)
- [Third](third.md)
EOF
    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 0 ]
    [[ "$output" == *"first.md"* ]]
    [[ "$output" == *"second.md"* ]]
    [[ "$output" == *"third.md"* ]]
}

# ==============================================================================
# Template Validation
# ==============================================================================

@test "verify-guide: detects missing Overview section" {
    # Create a guide without Overview
    mkdir -p docs/guides
    cat > docs/guides/incomplete.md << 'EOF'
# 1. Incomplete Guide

Date: 2026-05-24

## Prerequisites
Some prerequisites.

## Step-by-Step
Steps here.
EOF
    sed -i '/^# X/a\!/docs/guides/incomplete.md' .gitignore

    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error"* ]]
}

@test "verify-guide: detects missing Prerequisites section" {
    # Create a guide without Prerequisites
    mkdir -p docs/guides
    cat > docs/guides/incomplete.md << 'EOF'
# 1. Incomplete Guide

Date: 2026-05-24

## Overview
Guide overview.

## Step-by-Step
Steps here.
EOF
    sed -i '/^# X/a\!/docs/guides/incomplete.md' .gitignore

    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error"* ]]
}

@test "verify-guide: detects missing additional content section" {
    # Create a guide without additional content sections
    mkdir -p docs/guides
    cat > docs/guides/incomplete.md << 'EOF'
# 1. Incomplete Guide

Date: 2026-05-24

## Overview
Guide overview.

## Prerequisites
Some prerequisites.
EOF
    sed -i '/^# X/a\!/docs/guides/incomplete.md' .gitignore

    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error"* ]]
}

@test "verify-guide: enforces guide-specific sections (not ADR sections)" {
    # Create a guide with ADR sections instead of guide sections
    mkdir -p docs/guides
    cat > docs/guides/wrong-template.md << 'EOF'
# 1. Wrong Template

Date: 2026-05-24

## Status
Accepted

## Context
ADR context here.

## Decision
ADR decision here.

## Consequences
ADR consequences here.
EOF
    sed -i '/^# IX/a\!/docs/guides/wrong-template.md' .gitignore

    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error"* ]]
}

# ==============================================================================
# Guide Slug Validation
# ==============================================================================

@test "verify-guide: handle multiple guides with different slugs" {
    ./doc-engine.sh add guide "First" > /dev/null
    ./doc-engine.sh add guide "Second Guide" > /dev/null
    ./doc-engine.sh add guide "Third" > /dev/null
    # Add guides to README.md index
    echo "- [First](first.md)" >> docs/guides/README.md
    echo "- [Second Guide](second-guide.md)" >> docs/guides/README.md
    echo "- [Third](third.md)" >> docs/guides/README.md

    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 0 ]
}

# ==============================================================================
# Gitignore Registration Validation
# ==============================================================================

@test "verify-guide: detect unregistered guide in filesystem" {
    ./doc-engine.sh add guide "Registered Guide" > /dev/null
    # Create an unregistered guide manually
    mkdir -p docs/guides
    cat > docs/guides/unregistered.md << 'EOF'
# 1. Unregistered
Last Updated: 2026-05-24
## Overview
This guide is not in .gitignore.
EOF
    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 1 ]
    [[ "$output" == *"not registered"* ]] || [[ "$output" == *"Error"* ]]
}

@test "verify-guide: detect orphaned .gitignore entry" {
    # Create a real guide
    ./doc-engine.sh add guide "Real Guide" > /dev/null
    # Add guide to README.md index
    echo "- [Real Guide](real-guide.md)" >> docs/guides/README.md
    # Manually add an orphaned entry by inserting before the XI header
    sed -i '/^# XI\. Source Code/i\!/docs/guides/orphaned.md' .gitignore

    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 1 ]
    [[ "$output" == *"out of sync"* ]]
}

# ==============================================================================
# README Indexing
# ==============================================================================

@test "verify-guide: detects guide not listed in README.md" {
    # Create guide and register in gitignore, but don't add to README
    mkdir -p docs/guides
    cat > docs/guides/unindexed.md << 'EOF'
# 1. Unindexed Guide

Date: 2026-05-24

## Overview
Guide overview.

## Prerequisites
Some prerequisites.

## Step-by-Step
Steps here.
EOF
    sed -i '/^# X/a\!/docs/guides/unindexed.md' .gitignore

    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 1 ]
    [[ "$output" == *"not listed in docs/guides/README.md"* ]]
}

@test "verify-guide: ignores README.md when scanning" {
    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 0 ]
}

@test "verify-guide: ignores templates/ subdirectory" {
    # Create a file in templates/ directory (not a numbered guide)
    mkdir -p docs/guides/templates
    cat > docs/guides/templates/custom-template.md << 'EOF'
# Custom Template

This is in templates/ and should be ignored.
EOF

    run_isolated ./doc-engine.sh verify guide
    [ "$status" -eq 0 ]
}

# ==============================================================================
# Output Validation
# ==============================================================================

@test "verify-guide: output shows registered guides" {
    ./doc-engine.sh add guide "First" > /dev/null
    ./doc-engine.sh add guide "Second" > /dev/null
    echo "- [First](first.md)" >> docs/guides/README.md
    echo "- [Second](second.md)" >> docs/guides/README.md
    run_isolated ./doc-engine.sh verify guide
    [[ "$output" == *"registered"* ]] || [[ "$output" == *"✅"* ]]
}

@test "verify-guide: show count of guides found" {
    ./doc-engine.sh add guide "First" > /dev/null
    ./doc-engine.sh add guide "Second" > /dev/null
    echo "- [First](first.md)" >> docs/guides/README.md
    echo "- [Second](second.md)" >> docs/guides/README.md
    run_isolated ./doc-engine.sh verify guide
    [[ "$output" == *"2"* ]] && [[ "$output" == *"guide"* ]]
}
