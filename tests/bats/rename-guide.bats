#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================

# Test suite for scripts/rename-guide.sh
#
# Testing approach: Integration tests for guide document renaming and refactoring.
# Validates slug changes, link updates, metadata preservation, and error handling.
#
# Run with: bats tests/bats/rename-guide.bats
# ==============================================================================
setup() {
    cd "$BATS_TEST_TMPDIR" || exit 1
    source "$BATS_TEST_DIRNAME/common.sh"
    setup_guide_test

    mkdir -p docs/guides
    cat > docs/guides/old-title.md <<'EOF'
# 1. Old Title

Date: 2026-05-24

## Overview
Guide overview.

## Prerequisites
Some prerequisites.

## Step-by-Step
Steps here.
EOF

    git add docs/guides/old-title.md
    git commit -q -m "Add guide"

    add_guide_to_gitignore "old-title.md"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# ==============================================================================
# File Renaming
# ==============================================================================

@test "rename-guide: rename file with -f flag" {
    [ -f "docs/guides/old-title.md" ]
    ./doc-engine.sh rename guide -f old-title.md "New Title" > /dev/null
    assert_file_not_exists "docs/guides/old-title.md"
    assert_guide_file_exists "new-title.md"
}

@test "rename-guide: rename by slug match" {
    ./doc-engine.sh rename guide -f old-title "New Title" > /dev/null
    assert_guide_file_exists "new-title.md"
}

# ==============================================================================
# Gitignore Updates
# ==============================================================================

@test "rename-guide: update .gitignore with -f flag" {
    ./doc-engine.sh rename guide -f old-title.md "New Title" > /dev/null
    run ! grep -q "old-title.md" .gitignore
    assert_guide_in_gitignore "new-title.md"
}

@test "rename-guide: preserve slug in .gitignore" {
    ./doc-engine.sh rename guide -f old-title.md "Different Name" > /dev/null
    assert_guide_in_gitignore "different-name.md"
}

# ==============================================================================
# Title Sanitisation
# ==============================================================================

@test "rename-guide: sanitise new title" {
    ./doc-engine.sh rename guide -f old-title "New/Title With Spaces" > /dev/null
    assert_guide_file_exists "new-title-with-spaces.md"
}

@test "rename-guide: convert title to lowercase" {
    ./doc-engine.sh rename guide -f old-title "CamelCaseTitle" > /dev/null
    assert_guide_file_exists "camelcasetitle.md"
}

# ==============================================================================
# Sequential Renames
# ==============================================================================

@test "rename-guide: rename multiple times" {
    ./doc-engine.sh rename guide -f old-title "Second Name" > /dev/null
    assert_guide_file_exists "second-name.md"

    ./doc-engine.sh rename guide -f second-name "Third Name" > /dev/null
    assert_file_not_exists "docs/guides/second-name.md"
    assert_guide_file_exists "third-name.md"

    assert_guide_in_gitignore "third-name.md"
    ! grep -q "second-name.md" .gitignore
}

# ==============================================================================
# User Confirmation
# ==============================================================================

@test "rename-guide: cancel rename when user says no" {
    [ -f "docs/guides/old-title.md" ]
    echo "no" | ./doc-engine.sh rename guide "old-title" "New Name"
    assert_guide_file_exists "old-title.md"
    assert_guide_in_gitignore "old-title.md"
    assert_file_not_exists "docs/guides/new-name.md"
}

@test "rename-guide: show old and new paths before confirmation" {
    run_isolated bash -c 'echo "no" | ./doc-engine.sh rename guide "old-title" "New Name"'
    [[ "$output" == *"old-title.md"* ]]
    [[ "$output" == *"new-name.md"* ]]
}

# ==============================================================================
# Error Handling
# ==============================================================================

@test "rename-guide: reject no arguments" {
    run_isolated ./doc-engine.sh rename guide
    [ "$status" -eq 1 ]
    [[ "$output" == *"Usage:"* ]]
}

@test "rename-guide: reject missing second argument" {
    run_isolated ./doc-engine.sh rename guide 0001
    [ "$status" -eq 1 ]
    [[ "$output" == *"Usage:"* ]]
}

@test "rename-guide: reject nonexistent guide" {
    run_isolated ./doc-engine.sh rename guide -f 9999 "New Title"
    [ "$status" -eq 1 ]
    [[ "$output" == *"Could not find"* ]]
}

@test "rename-guide: reject identical old and new filename" {
    run_isolated ./doc-engine.sh rename guide -f old-title.md "Old Title"
    [ "$status" -eq 1 ]
    [[ "$output" == *"identical"* ]]
}

@test "rename-guide: reject destination file already exists" {
    # Create a second guide
    cat > docs/guides/new-title.md <<'EOF'
# 1. New Title

Date: 2026-05-24

## Overview
Guide overview.

## Prerequisites
Some prerequisites.

## Step-by-Step
Steps here.
EOF

    run_isolated ./doc-engine.sh rename guide -f old-title.md "New Title"
    [ "$status" -eq 1 ]
    [[ "$output" == *"already exists"* ]]
}

# ==============================================================================
# Output Validation
# ==============================================================================

@test "rename-guide: output indicates successful rename" {
    run_isolated ./doc-engine.sh rename guide -f old-title "New Title"
    [[ "$output" == *"Successfully renamed"* ]]
}

# ==============================================================================
# Link Integrity
# ==============================================================================

@test "rename-guide: update links in other files" {
    # Create another md file referencing old-title.md
    cat > docs/another.md <<'EOF'
See [Guide 1](guides/old-title.md) for context.
EOF

    # Create a rs file referencing old-title.md
    mkdir -p src
    cat > src/lib.rs <<'EOF'
//! See [Guide 1](https://docs.rs/sdmx-rs/latest/sdmx_rs/guides/old-title.md)
EOF

    ./doc-engine.sh rename guide -f old-title "New Title" > /dev/null

    # Assert references are updated
    grep -q "new-title.md" docs/another.md
    run ! grep -q "old-title.md" docs/another.md

    grep -q "new-title.md" src/lib.rs
    ! grep -q "old-title.md" src/lib.rs
}
