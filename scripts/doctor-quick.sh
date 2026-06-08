#!/bin/sh
# ==============================================================================
# scripts/doctor-quick.sh
# Quick triage (fast verification)
#
# Runs fast verification checks: cargo check, clippy, and tests.
# Useful for rapid feedback during development.
#
# Usage: scripts/doctor-quick.sh
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

echo "🏃 Quick Triage (Fast Verification)"
echo ""

# Track overall status
failed=0
total_time_start=$(date +%s)

# Check 1: cargo check
echo "1️⃣  Running: cargo check --workspace"
check_time_start=$(date +%s)

if cargo check --workspace 2>&1 | tail -5; then
    check_time_end=$(date +%s)
    check_time=$((check_time_end - check_time_start))
    log_ok "Check passed ($check_time seconds)" 1
else
    check_time_end=$(date +%s)
    check_time=$((check_time_end - check_time_start))
    log_fail "Check failed ($check_time seconds)" 1
    failed=1
fi

echo ""

# Check 2: cargo clippy
echo "2️⃣  Running: cargo clippy --workspace -- -D warnings"
clippy_time_start=$(date +%s)

if cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5; then
    clippy_time_end=$(date +%s)
    clippy_time=$((clippy_time_end - clippy_time_start))
    log_ok "Clippy passed ($clippy_time seconds)" 1
else
    clippy_time_end=$(date +%s)
    clippy_time=$((clippy_time_end - clippy_time_start))
    log_fail "Clippy failed ($clippy_time seconds)" 1
    failed=1
fi

echo ""

# Check 3: cargo test
echo "3️⃣  Running: cargo nextest run --workspace"
test_time_start=$(date +%s)

# Capture test output for summary
test_output=$(cargo nextest run --workspace 2>&1 | tail -20 || echo "")

if echo "$test_output" | grep -q "passed"; then
    test_time_end=$(date +%s)
    test_time=$((test_time_end - test_time_start))
    # Extract summary line if available
    summary=$(echo "$test_output" | grep -E "passed|failed" | tail -1 || echo "")
    log_ok "Tests passed ($test_time seconds)" 1
    if [ -n "$summary" ]; then
        echo "      $summary"
    fi
else
    test_time_end=$(date +%s)
    test_time=$((test_time_end - test_time_start))
    log_fail "Tests failed ($test_time seconds)" 1
    echo "$test_output"
    failed=1
fi

echo ""

# Summary
total_time_end=$(date +%s)
total_time=$((total_time_end - total_time_start))

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Quick Triage Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ "$failed" -eq 0 ]; then
    log_ok "All checks passed"
    echo "   Total time: $total_time seconds"
    echo ""
    echo "Next steps:"
    log_item "Run 'just verify' for complete validation" 1
    log_item "Run 'just coverage' for coverage report" 1
    exit 0
else
    log_fail "Some checks failed"
    echo "   Total time: $total_time seconds"
    echo ""
    echo "Next steps:"
    log_item "Fix failing tests/lints above" 1
    log_item "Run 'just verify' for full diagnostics" 1
    exit 1
fi
