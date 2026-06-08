#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/check-maintenance.sh
#
# Testing approach: Integration tests for maintenance deadline enforcement.
# Validates deadline tracking, comment verification, and overdue detection.
#
# Run with: bats tests/bats/check-maintenance.bats
# ==============================================================================
setup() {
    # Load common test helpers
    source "tests/bats/common.sh"

    # Change to test directory
    cd "$BATS_TEST_TMPDIR" || exit 1

    # Inject "today" for maintenance deadline checks (deterministic, no PATH manipulation)
    export MAINTENANCE_TODAY="2026-08-15"
}

teardown() {
    # BATS automatically cleans up $BATS_TEST_TMPDIR
    :
}

# ==============================================================================
# Test: Valid maintenance item passes
# ==============================================================================

@test "valid maintenance item passes check" {
    create_test_maintenance_toml
    create_test_source_file "test.txt" "2026-07-15" "2026-08-15"

    run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force

    [ "$status" -eq 0 ]
    [[ "$output" =~ "✅ test-item" ]]
    [[ "$output" =~ "maintenance: all obligations tracked and current" ]]
}

# ==============================================================================
# Test: Overdue item fails
# ==============================================================================

@test "overdue maintenance item fails check" {
    cat > maintenance.toml << 'EOF'
[[maintenance]]
item = "overdue-item"
file = "test.txt"
marker = "# MAINTENANCE: overdue-item"
review_cadence = 30
warn_threshold = 45
fail_threshold = 60
last_updated = "2026-06-01"
next_review = "2026-08-14"
EOF

    create_test_source_file "test.txt" "2026-06-01" "2026-08-14" "overdue-item"

    #run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force
    run_isolated "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force

    echo "DEBUG test 51: status=$status, MAINTENANCE_TODAY=$MAINTENANCE_TODAY, today=$(date +%Y-%m-%d)" >&2
    echo "Output: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" =~ "❌ overdue-item: OVERDUE" ]]
    [[ "$output" =~ "failed: 1 item" ]]
}

# ==============================================================================
# Test: Stale item warns
# ==============================================================================

@test "stale maintenance item warns but passes" {
    cat > maintenance.toml << 'EOF'
[[maintenance]]
item = "stale-item"
file = "test.txt"
marker = "# MAINTENANCE: stale-item"
review_cadence = 30
warn_threshold = 45
fail_threshold = 60
last_updated = "2026-06-01"
next_review = "2026-09-15"
EOF

    create_test_source_file "test.txt" "2026-06-01" "2026-09-15" "stale-item"

    run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force

    [ "$status" -eq 0 ]
    [[ "$output" =~ "⚠️" ]]
    [[ "$output" =~ "stale-item" ]]
}

# ==============================================================================
# Test: Missing inline comment fails
# ==============================================================================

@test "missing inline comment fails check" {
    create_test_maintenance_toml
    create_test_source_file_no_comment "test.txt"

    run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force

    [ "$status" -eq 1 ]
    [[ "$output" =~ "❌ test-item: marker not found" ]]
}

# ==============================================================================
# Test: Date mismatch (drift) fails
# ==============================================================================

@test "date mismatch in inline comment fails" {
    create_test_maintenance_toml
    # Create file with DIFFERENT dates
    create_test_source_file "test.txt" "2026-07-14" "2026-08-14"

    run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force

    [ "$status" -eq 1 ]
    [[ "$output" =~ "❌ test-item: last_updated mismatch" ]]
}

# ==============================================================================
# Test: Missing source file fails
# ==============================================================================

@test "missing source file fails check" {
    create_test_maintenance_toml
    # Don't create the source file

    run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force

    [ "$status" -eq 1 ]
    [[ "$output" =~ "❌ test-item: source file not found" ]]
}

# ==============================================================================
# Test: Multiple items, one fails
# ==============================================================================

@test "one failing item fails overall check" {
    cat > maintenance.toml << 'EOF'
[[maintenance]]
item = "item-1"
file = "file1.txt"
marker = "# MAINTENANCE: item-1"
review_cadence = 30
warn_threshold = 45
fail_threshold = 60
last_updated = "2026-07-15"
next_review = "2026-08-15"

[[maintenance]]
item = "item-2"
file = "file2.txt"
marker = "# MAINTENANCE: item-2"
review_cadence = 30
warn_threshold = 45
fail_threshold = 60
last_updated = "2026-06-01"
next_review = "2026-08-14"
EOF

    create_test_source_file "file1.txt" "2026-07-15" "2026-08-15" "item-1"
    create_test_source_file "file2.txt" "2026-06-01" "2026-08-14" "item-2"

    #run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force
    run_isolated "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force

    echo "DEBUG test 56: status=$status, MAINTENANCE_TODAY=$MAINTENANCE_TODAY, today=$(date +%Y-%m-%d)" >&2
    echo "Output: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" =~ "✅ item-1" ]]
    [[ "$output" =~ "❌ item-2: OVERDUE" ]]
}

# ==============================================================================
# Test: --dry-run flag
# ==============================================================================

@test "--dry-run mode exits 0 even if items are overdue" {
    cat > maintenance.toml << 'EOF'
[[maintenance]]
item = "overdue-item"
file = "test.txt"
marker = "# MAINTENANCE: overdue-item"
review_cadence = 30
warn_threshold = 45
fail_threshold = 60
last_updated = "2026-06-01"
next_review = "2026-08-14"
EOF

    create_test_source_file "test.txt" "2026-06-01" "2026-08-14" "overdue-item"

    #run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --dry-run --force
    run env -i PATH="$BATS_TEST_TMPDIR/bin:$PATH" MAINTENANCE_TODAY="$MAINTENANCE_TODAY" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --dry-run --force

    echo "DEBUG test 57: status=$status, MAINTENANCE_TODAY=$MAINTENANCE_TODAY, today=$(date +%Y-%m-%d)" >&2
    echo "Output: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" =~ "would fail" ]]
}

# ==============================================================================
# Test: Invalid date format
# ==============================================================================

@test "invalid date format fails" {
    cat > maintenance.toml << 'EOF'
[[maintenance]]
item = "bad-date-item"
file = "test.txt"
marker = "# MAINTENANCE: bad-date-item"
review_cadence = 30
warn_threshold = 45
fail_threshold = 60
last_updated = "2026/07/15"
next_review = "2026-08-15"
EOF

    create_test_source_file "test.txt" "2026-07-15" "2026-08-15" "bad-date-item"

    run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force

    [ "$status" -eq 1 ]
    [[ "$output" =~ "invalid" ]] || [[ "$output" =~ "format" ]]
}

# ==============================================================================
# Test: Maintenance.toml not found
# ==============================================================================

@test "missing maintenance.toml fails with clear error" {
    run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force

    [ "$status" -eq 1 ]
    [[ "$output" == *"maintenance.toml not found"* ]]
}

# ==============================================================================
# Dirty Working Tree Safeguard
# ==============================================================================

@test "dirty working tree fails without --force but bypasses with --force" {
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"

    create_test_maintenance_toml
    create_test_source_file "test.txt" "2026-07-15" "2026-08-15"

    git add maintenance.toml test.txt
    git commit -m "initial commit" -q

    # Make working tree dirty
    echo "dirty edit" >> test.txt

    # 1. Without --force, should fail
    run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh"
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Git working tree is dirty" ]]

    # 2. With --force, should bypass and succeed
    run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Dirty tree check bypassed" ]]
}

# ==============================================================================
# Overdue Warnings and Pull Request Bypass
# ==============================================================================

@test "overdue item fails by default but warns and exits 0 with --warn-overdue" {
    # 1. Setup clean git tree and write an overdue item
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"

    create_test_maintenance_toml_with_dates "2026-05-01" "2026-05-15"
    create_test_source_file "test.txt" "2026-05-01" "2026-05-15"

    git add maintenance.toml test.txt
    git commit -m "initial commit" -q

    # 2. Default run: should fail (exit 1) on overdue item
    #run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force
    run_isolated "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force
    echo "DEBUG test 61 (part 1): status=$status, MAINTENANCE_TODAY=$MAINTENANCE_TODAY, today=$(date +%Y-%m-%d)" >&2
    echo "Output: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" =~ "OVERDUE" ]]

    # 3. Warn-overdue run: should demote to warning and succeed (exit 0)
    PATH="$BATS_TEST_TMPDIR/bin:$PATH" run bash -c "MAINTENANCE_TODAY='$MAINTENANCE_TODAY' bash '$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh' --force --warn-overdue"
    echo "DEBUG test 61 (part 2): status=$status, MAINTENANCE_TODAY=$MAINTENANCE_TODAY, today=$(date +%Y-%m-%d)" >&2
    echo "Output: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" =~ "OVERDUE WARNING" ]]
}

@test "overdue item warns and exits 0 in GITHUB_EVENT_NAME=pull_request context" {
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"

    create_test_maintenance_toml_with_dates "2026-05-01" "2026-05-15"
    create_test_source_file "test.txt" "2026-05-01" "2026-05-15"

    git add maintenance.toml test.txt
    git commit -m "initial commit" -q

    # Run in PR environment: should exit 0 and warn
    run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" GITHUB_EVENT_NAME="pull_request" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force
    [ "$status" -eq 0 ]
    [[ "$output" =~ "OVERDUE WARNING" ]]
}

@test "structural errors still trigger exit 1 in warn-overdue mode" {
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"

    # Create config mismatch (config has 2026-05-15, but comment has 2026-05-20)
    create_test_maintenance_toml_with_dates "2026-05-01" "2026-05-15"
    create_test_source_file "test.txt" "2026-05-01" "2026-05-20"

    git add maintenance.toml test.txt
    git commit -m "initial commit" -q

    # Run with --warn-overdue: should still exit 1 because mismatch is a structural/drift error, not just overdue
    run env PATH="$BATS_TEST_TMPDIR/bin:$PATH" bash "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force --warn-overdue
    [ "$status" -eq 1 ]
    [[ "$output" =~ "mismatch" ]]
}
