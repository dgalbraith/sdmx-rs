#!/bin/sh
# ==============================================================================
# scripts/doctor-git.sh
# Git configuration diagnostics
#
# Validates that Git is properly configured for the project, including hooks,
# signing, and authentication settings.
#
# Usage: scripts/doctor-git.sh
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

log_section "Git Configuration Diagnostics"
echo ""

# Track overall status
failed=0

# Check 1: user.name configured
if git_name=$(git config --get user.name 2>/dev/null); then
    log_ok "user.name: $git_name"
else
    log_fail "user.name not configured"
    echo "   Run: git config --global user.name 'Your Name'"
    failed=1
fi

echo ""

# Check 2: user.email configured
if git_email=$(git config --get user.email 2>/dev/null); then
    log_ok "user.email: $git_email"
else
    log_fail "user.email not configured"
    echo "   Run: git config --global user.email 'your.email@example.com'"
    failed=1
fi

echo ""

# Check 3: GPG signing key configured
if gpg_key=$(git config --get user.signingkey 2>/dev/null); then
    log_ok "user.signingkey: $gpg_key"
else
    log_fail "user.signingkey not configured"
    echo "   Run: git config --global user.signingkey <KEY_ID>"
    echo "   Get KEY_ID from: gpg --list-secret-keys --keyid-format=long"
    failed=1
fi

echo ""

# Check 4: commit.gpgsign enabled
if gpg_sign=$(git config --get commit.gpgsign 2>/dev/null); then
    if [ "$gpg_sign" = "true" ]; then
        log_ok "commit.gpgsign: enabled"
    else
        log_warn "commit.gpgsign: $gpg_sign (not enabled)"
        echo "   Run: git config --global commit.gpgsign true"
    fi
else
    log_warn "commit.gpgsign not configured (automatic signing disabled)"
    echo "   Run: git config --global commit.gpgsign true"
fi

echo ""

# Check 5: GPG key exists and is valid
if [ -n "${gpg_key:-}" ]; then
    if gpg --list-secret-keys "$gpg_key" >/dev/null 2>&1; then
        log_ok "GPG key exists and is accessible"
    else
        log_fail "GPG key not found: $gpg_key"
        echo "   Verify with: gpg --list-secret-keys --keyid-format=long"
        failed=1
    fi
fi

echo ""

# Check 6: GPG key not expired
if [ -n "${gpg_key:-}" ]; then
    if gpg --list-secret-keys "$gpg_key" >/dev/null 2>&1; then
        # Check expiration date in GPG key listing
        expiry=$(gpg --list-secret-keys --with-colons "$gpg_key" 2>/dev/null | grep "^sec" | cut -d: -f7 || true)
        if [ -z "$expiry" ] || [ "$expiry" = "0" ]; then
            log_ok "GPG key has no expiration"
        else
            expiry_epoch="$expiry"
            current_epoch=$(date +%s)
            if [ "$expiry_epoch" -gt "$current_epoch" ]; then
                expiry_date=$(date -d "@$expiry_epoch" "+%Y-%m-%d" 2>/dev/null || date -r "$expiry_epoch" "+%Y-%m-%d" 2>/dev/null || echo "unknown")
                log_ok "GPG key expires: $expiry_date"
            else
                log_fail "GPG key has expired"
                failed=1
            fi
        fi
    fi
fi

echo ""

# Check 7: GPG can sign (functional test)
if [ -n "${gpg_key:-}" ]; then
    if echo "test message" | gpg --batch --quiet --sign --default-key "$gpg_key" >/dev/null 2>&1; then
        log_ok "GPG can sign commits"
    else
        log_warn "GPG signing test failed"
        echo "   Check GPG agent is running and key is accessible"
        echo "   Try: gpg-agent --daemon"
    fi
fi

echo ""

# Check 8: Pre-commit hooks installed
if [ -f .git/hooks/pre-commit ] && [ -x .git/hooks/pre-commit ]; then
    log_ok "Pre-commit hooks installed"
else
    log_warn "Pre-commit hooks not installed"
    echo "   Run: just hook-install"
fi

echo ""

# Check 9: Current branch status
if git rev-parse --git-dir >/dev/null 2>&1; then
    current_branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)
    if [ "$current_branch" = "main" ] || [ "$current_branch" = "master" ]; then
        log_info "Current branch: $current_branch (main branch)"
    else
        log_info "Current branch: $current_branch (feature branch)"
    fi
fi

echo ""

# Check 10: Dirty working tree
if [ -z "$(git status --porcelain 2>/dev/null)" ]; then
    log_ok "Working tree is clean"
else
    log_warn "Working tree has uncommitted changes"
    git_status=$(git status --short 2>/dev/null | wc -l)
    echo "   Files with changes: $git_status"
fi

echo ""

# Check 11: Commits ahead of the canonical main remote (default: origin;
# override with SDMX_MAIN_REMOTE for mirrored-forge setups).
main_remote="${SDMX_MAIN_REMOTE:-origin}"
if git rev-parse "$main_remote/main" >/dev/null 2>&1; then
    commits_ahead=$(git rev-list --count "$main_remote/main..HEAD" 2>/dev/null || echo "0")
    if [ "$commits_ahead" -gt 0 ]; then
        log_info "Commits ahead of $main_remote/main: $commits_ahead"
    else
        log_ok "In sync with $main_remote/main"
    fi
fi

echo ""

# Summary
if [ "$failed" -eq 0 ]; then
    log_ok "Git configuration is ready for secure commits"
    exit 0
else
    log_fail "Git configuration has issues — see above"
    exit 1
fi
