#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/maintenance-bump.sh
#
# Testing approach: Integration tests for automated maintenance deadline updates.
# Validates date shifting, file modification, and validation integration.
#
# Run with: bats tests/bats/maintenance-bump.bats
# ==============================================================================
setup() {
    # Load common test helpers
    source "tests/bats/common.sh"

    # Change to test directory
    cd "$BATS_TEST_TMPDIR" || exit 1

    # Inject "today" for maintenance calculations (deterministic, no PATH manipulation)
    export MAINTENANCE_TODAY="2026-08-15"
}

teardown() {
    :
}

# ==============================================================================
# Standard Bumping Logic
# ==============================================================================

@test "bumping a valid item updates maintenance.toml and source file" {
    create_test_maintenance_toml
    create_test_source_file "test.txt" "2026-07-15" "2026-08-15" "test-item"

    # Run maintenance-bump.sh
    run_isolated "$BATS_TEST_DIRNAME/../../scripts/maintenance-bump.sh" "test-item"

    [ "$status" -eq 0 ]

    # Check that maintenance.toml was updated:
    # 2026-08-15 is the mocked today date. Cadence is 30, so next review is 2026-09-14
    grep -q 'last_updated = "2026-08-15"' maintenance.toml
    grep -q 'next_review = "2026-09-14"' maintenance.toml

    # Check that inline comments in test.txt were updated:
    grep -q 'Last updated: 2026-08-15' test.txt
    grep -q 'Next review: 2026-09-14' test.txt
}

# ==============================================================================
# TOML Parsing & Key Management
# ==============================================================================

@test "consecutive bumps do not duplicate next_review key in maintenance.toml" {
    create_test_maintenance_toml
    create_test_source_file "test.txt" "2026-07-15" "2026-08-15" "test-item"

    # Run first bump
    run_isolated "$BATS_TEST_DIRNAME/../../scripts/maintenance-bump.sh" "test-item"
    [ "$status" -eq 0 ]

    # Run second bump immediately
    run_isolated "$BATS_TEST_DIRNAME/../../scripts/maintenance-bump.sh" "test-item"
    [ "$status" -eq 0 ]

    # Count the number of next_review entries in maintenance.toml
    count=$(grep -c "next_review =" maintenance.toml || true)
    [ "$count" -eq 1 ]
}

@test "bump works correctly when next_review is missing from toml block initially" {
    cat > maintenance.toml << 'EOF'
[[maintenance]]
item = "test-item-1"
file = "test1.txt"
marker = "# MAINTENANCE: test-item-1"
review_cadence = 30
warn_threshold = 45
fail_threshold = 60
last_updated = "2026-07-15"

[[maintenance]]
item = "test-item-2"
file = "test2.txt"
marker = "# MAINTENANCE: test-item-2"
review_cadence = 30
warn_threshold = 45
fail_threshold = 60
last_updated = "2026-07-15"
next_review = "2026-08-15"
EOF

    create_test_source_file "test1.txt" "2026-07-15" "2026-08-15" "test-item-1"
    create_test_source_file "test2.txt" "2026-07-15" "2026-08-15" "test-item-2"

    # Bump test-item-1 (which lacks next_review)
    run_isolated "$BATS_TEST_DIRNAME/../../scripts/maintenance-bump.sh" "test-item-1"
    [ "$status" -eq 0 ]

    # Verify that test-item-2 next_review was NOT changed or duplicated!
    # If in_block leaked, it would have changed test-item-2's next_review to 2026-09-14
    grep -q 'item = "test-item-2"' maintenance.toml
    # The next_review of test-item-2 should still be 2026-08-15
    sed -n '/item = "test-item-2"/,/^$/p' maintenance.toml | grep -q 'next_review = "2026-08-15"'
}

# ==============================================================================
# Error Handling
# ==============================================================================

@test "bumping with invalid item returns error" {
    create_test_maintenance_toml
    run_isolated "$BATS_TEST_DIRNAME/../../scripts/maintenance-bump.sh" "non-existent-item"
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Item not found" ]]
}

# ==============================================================================
# Comment Formatting
# ==============================================================================

@test "bumping updates inline comment formats of different kinds" {
    cat > maintenance.toml << 'EOF'
[[maintenance]]
item = "test-item"
file = "test.txt"
marker = "// MAINTENANCE: test-item"
review_cadence = 30
warn_threshold = 45
fail_threshold = 60
last_updated = "2026-07-15"
next_review = "2026-08-15"
EOF

    # Test case with C++ style comments
    cat > test.txt << 'EOF'
// MAINTENANCE: test-item
// Last updated: 2026-07-15
// Next review: 2026-08-15
EOF

    run_isolated "$BATS_TEST_DIRNAME/../../scripts/maintenance-bump.sh" "test-item"
    [ "$status" -eq 0 ]

    grep -q '// Last updated: 2026-08-15' test.txt
    grep -q '// Next review: 2026-09-14' test.txt
}
