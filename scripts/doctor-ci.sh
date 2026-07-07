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

# `just` has no transitive-dependency query, so we walk the graph ourselves.
# resolve_recipe accumulates into RESOLVED every recipe name reachable from the
# root (leaves and intermediates; the root itself is not added). A recipe's
# direct deps are the space-separated tokens after `<name>:` on its definition
# line; a recipe whose definition line has no trailing tokens is a leaf.
resolve_recipe() {
    for _dep in $(sed -n "s/^$1:[[:space:]]*//p" Justfile | head -1); do
        case " ${RESOLVED} " in
            *" ${_dep} "*) continue ;;
        esac
        RESOLVED="${RESOLVED} ${_dep}"
        resolve_recipe "${_dep}"
    done
}

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
    # verify's direct dependencies are its `verify-*` sub-gates (informational
    # listing only; the alignment check below resolves the full graph).
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

# Resolve `verify` transitively to its leaf recipes, then match the key checks
# against that set with an EXACT (word-boundary) match. A substring match would
# spuriously pass `doc` against `verify-docs`, and matching against verify's
# DIRECT deps (its sub-gates) misses every leaf recipe.
RESOLVED=""
resolve_recipe verify

# Key checks a developer expects `just verify` to run. Overridable for tests.
KEY_CHECKS="${DOCTOR_CI_KEY_CHECKS:-check-format clippy check-conventions check-wasm test-wasm docs semver-check coverage-gate release-dry-run shellcheck md-check link-check deny machete secrets-scan}"

echo "Key Checks Status:"
missing=0
for check in $KEY_CHECKS; do
    case " ${RESOLVED} " in
        *" ${check} "*) log_ok "${check} (in local verify)" 1 ;;
        *) log_warn "${check} (not in local verify)" 1; missing=$((missing + 1)) ;;
    esac
done

echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ "$missing" -eq 0 ]; then
    log_ok "doctor-ci: local 'just verify' covers all key CI checks"
else
    log_fail "doctor-ci: ${missing} key check(s) not covered by local 'just verify'"
fi
echo ""
echo "Maintenance Notes:"
log_item "Keep CI jobs aligned with 'just verify' recipe" 1
log_item "Add new checks to both CI and 'just verify'" 1
echo ""

[ "$missing" -eq 0 ] || exit 1
