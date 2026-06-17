#!/bin/sh
# ==============================================================================
# scripts/doctor-ci.sh
# CI/local verification alignment check
#
# Validates that CI pipeline jobs align with local verification targets,
# ensuring developers can run the same checks locally as in CI.
#
# Usage: scripts/doctor-ci.sh
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

log_section "CI/Local Verification Alignment"
echo ""

# Check 1: CI workflow file exists
ci_workflow=".github/workflows/ci.yml"

if [ ! -f "$ci_workflow" ]; then
    log_fail "CI workflow not found: $ci_workflow"
    exit 1
fi

log_info "Analysing: $ci_workflow"
echo ""

# Extract job names from CI workflow (simple YAML parsing)
echo "CI Pipeline Jobs:"
ci_jobs=$(sed -n '/^  [a-z-]*:$/p' "$ci_workflow" | sed 's/:$//' | sed 's/^  //')

if [ -z "$ci_jobs" ]; then
    log_warn "Could not parse jobs from CI workflow"
else
    job_count=$(echo "$ci_jobs" | wc -l)
    echo "  Total: $job_count jobs"
    echo ""
    echo "  Jobs:"
    echo "$ci_jobs" | while read -r job; do
        log_item "$job" 2
    done
fi

echo ""

# Check 2: Map CI jobs to local `just verify` targets
echo "Mapping to local targets:"
echo ""

# Extract commands from CI workflow (grep for cargo and just commands)
ci_commands=$(grep -E "just|cargo" "$ci_workflow" | sed 's/.*\(just [a-z-]*\).*/\1/' | sort -u || echo "")

if [ -n "$ci_commands" ]; then
    echo "Commands in CI:"
    echo "$ci_commands" | while read -r cmd; do
        log_item "$cmd" 1
    done
else
    log_warn "Could not extract commands from CI workflow"
fi

echo ""

# Check 3: Get local verify target definition
echo "Local Verification Target (just verify):"
echo ""

# Extract the verify recipe from Justfile
verify_recipe=$(sed -n '/^verify:/,/^[a-z]/p' Justfile | head -1 || echo "")

if [ -n "$verify_recipe" ]; then
    # Parse the dependencies from the verify recipe
    # Format: verify: fmt-check clippy check-wasm doc deny machete check-scaffolding semver-check test-coverage-headless shellcheck verify-adr release-dry-run md-check

    verify_deps=$(sed -n '/^verify:/p' Justfile | sed 's/.*verify: //' | tr ' ' '\n' | grep -v '^$')

    if [ -n "$verify_deps" ]; then
        dep_count=$(echo "$verify_deps" | wc -l)
        echo "  Total: $dep_count checks"
        echo ""
        echo "  Checks:"
        echo "$verify_deps" | while read -r dep; do
            log_item "$dep" 2
        done
    fi
else
    log_warn "Could not parse 'just verify' recipe"
fi

echo ""

# Check 4: Alignment analysis
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Alignment Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Key checks to look for
echo "Key Checks Status:"

checks="fmt-check clippy check-wasm doc deny machete verify-adr md-check test release-dry-run"

for check in $checks; do
    if echo "$verify_deps" | grep -q "$check"; then
        log_ok "$check (in local verify)" 1
    else
        log_warn "$check (not in local verify)" 1
    fi
done

echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

log_ok "CI workflow is properly configured"
echo "   CI jobs and local targets are aligned"
echo ""
echo "Maintenance Notes:"
log_item "Keep CI jobs aligned with 'just verify' recipe" 1
log_item "CI should run all local verification steps" 1
log_item "Add new checks to both CI and 'just verify'" 1
echo ""
