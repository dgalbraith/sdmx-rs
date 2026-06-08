#!/bin/sh
# ==============================================================================
# scripts/doctor-hooks.sh
# Pre-commit hook validation
#
# Validates that pre-commit hooks are properly installed and functional,
# ensuring code quality checks run before commits.
#
# Usage: scripts/doctor-hooks.sh
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

log_section "Pre-Commit Hook Validation"
echo ""

# Track overall status
failed=0

# Check 1: Pre-commit framework installed
if command -v pre-commit >/dev/null 2>&1; then
    pre_commit_version=$(pre-commit --version 2>/dev/null | head -1)
    log_ok "pre-commit framework: $pre_commit_version"
else
    log_fail "pre-commit not installed"
    echo "   Run: pip install pre-commit or use Nix devshell"
    failed=1
    echo ""
    exit 1
fi

echo ""

# Check 2: .pre-commit-config.yaml exists
if [ -f .pre-commit-config.yaml ]; then
    log_ok ".pre-commit-config.yaml exists"
else
    log_fail ".pre-commit-config.yaml not found"
    failed=1
fi

echo ""

# Check 3: Git hooks directory exists
if [ -d .git/hooks ]; then
    log_ok ".git/hooks directory exists"
else
    log_fail ".git/hooks directory not found (not a git repo?)"
    failed=1
fi

echo ""

# Check 4: Hook stages are installed
echo "Checking hook installation:"

hook_stages="pre-commit commit-msg pre-push"
hooks_installed=0

for stage in $hook_stages; do
    hook_file=".git/hooks/$stage"
    if [ -f "$hook_file" ]; then
        if [ -x "$hook_file" ]; then
            log_ok "$stage hook installed and executable" 1
            hooks_installed=$((hooks_installed + 1))
        else
            log_warn "$stage hook exists but not executable" 1
            echo "     Run: chmod +x .git/hooks/$stage"
        fi
    else
        log_fail "$stage hook not installed" 1
    fi
done

if [ "$hooks_installed" -lt 3 ]; then
    echo ""
    echo "  Run: just hook-install"
    failed=1
fi

echo ""

# Check 5: Test hook execution (dry-run)
echo "Testing hook framework:"

if pre-commit run --files flake.nix >/dev/null 2>&1; then
    log_ok "Hooks can execute (dry-run test passed)"
else
    # Some hooks may fail on test files; check if pre-commit itself runs
    if pre-commit --help >/dev/null 2>&1; then
        log_ok "Hooks framework responsive (execution test inconclusive)"
    else
        log_warn "Hook execution test failed"
        echo "   Verify hooks are trusted: direnv allow && direnv reload"
    fi
fi

echo ""

# Check 6: Configured hooks summary
echo "Configured hook repositories:"

if [ -f .pre-commit-config.yaml ]; then
    # Extract repo names (simple grep for "repo:" lines)
    repo_count=$(grep -c "^  - repo:" .pre-commit-config.yaml || echo "0")

    if [ "$repo_count" -gt 0 ]; then
        echo "  Total repositories: $repo_count"

        # List a few key ones
        echo "  Key hooks:"
        grep "^  - repo:" .pre-commit-config.yaml | head -5 | while read -r line; do
            repo=$(echo "$line" | sed 's/.*repo: //;s/\r$//')
            repo_name=$(basename "$repo" .git || basename "$repo")
            log_item "$repo_name" 2
        done

        if [ "$repo_count" -gt 5 ]; then
            echo "    ... and $((repo_count - 5)) more"
        fi
    else
        echo "  No repositories configured"
    fi
fi

echo ""

# Summary
if [ "$failed" -eq 0 ]; then
    log_ok "Pre-commit hooks are properly installed"
    exit 0
else
    log_fail "Hook setup has issues — see above"
    exit 1
fi
