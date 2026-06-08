#!/bin/sh
# ==============================================================================
# scripts/doctor-direnv.sh
# direnv integration diagnostics
#
# Validates that direnv is properly configured and integrated with the project,
# ensuring automatic environment loading for developers.
#
# Usage: scripts/doctor-direnv.sh
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

log_section "direnv Integration Diagnostics"
echo ""

# Track overall status
failed=0

# Check 1: direnv installed
if command -v direnv >/dev/null 2>&1; then
    direnv_version=$(direnv version 2>/dev/null)
    log_ok "direnv installed: $direnv_version"
else
    log_fail "direnv not found"
    echo "   Install from: https://direnv.net/docs/installation.html"
    failed=1
    echo ""
    exit 1
fi

echo ""

# Check 2: .envrc exists
if [ -f .envrc ]; then
    log_ok ".envrc exists"
else
    log_fail ".envrc not found in current directory"
    failed=1
fi

echo ""

# Check 3: .envrc is trusted
if [ -f .envrc ]; then
    # Check if .envrc is in the allowlist
    allowlist_file="${XDG_CONFIG_HOME:-$HOME/.config}/direnv/allow"
    envrc_path=$(cd "$(dirname .envrc)" && pwd)/.envrc

    if [ -f "$allowlist_file" ] && grep -q "$envrc_path" "$allowlist_file" 2>/dev/null; then
        log_ok ".envrc is trusted by direnv"
    else
        # Try direnv status to check trust status
        status_output=$(direnv status 2>&1 || true)
        if echo "$status_output" | grep -qiE "allowed|trusted"; then
            log_ok ".envrc is trusted by direnv"
        else
            log_warn ".envrc may not be trusted"
            echo "   Run: direnv allow"
        fi
    fi
fi

echo ""

# Check 4: Can load Nix shell environment
if [ -f .envrc ]; then
    # Test if direnv can evaluate the environment without actually loading it
    if direnv exec . true >/dev/null 2>&1; then
        log_ok "Nix shell environment loads successfully"
    else
        log_warn "Nix shell environment has issues"
        echo "   Try: direnv allow && direnv reload"
        # Not a hard failure — may be trust/permission issue
    fi
fi

echo ""

# Check 5: RUSTFMT environment variable is set
if [ -n "${RUSTFMT:-}" ]; then
    if [ -f "$RUSTFMT" ]; then
        log_ok "RUSTFMT env var set: $RUSTFMT"
    else
        log_warn "RUSTFMT env var is set but points to non-existent path: $RUSTFMT"
        echo "   Try: direnv reload"
    fi
else
    log_warn "RUSTFMT environment variable not set"
    echo "   Load the Nix environment: direnv allow && direnv reload"
    echo "   Or manually: export RUSTFMT=\$(nix build --print-out-paths --no-link .#nightly-fmt)/bin/rustfmt"
fi

echo ""

# Check 6: Standard tools available
echo "Checking standard tools in loaded environment:"

tools_ok=0
for tool in rustc cargo rustfmt taplo; do
    if command -v "$tool" >/dev/null 2>&1; then
        log_ok "$tool" 1
        tools_ok=$((tools_ok + 1))
    else
        log_fail "$tool not found" 1
    fi
done

if [ "$tools_ok" -lt 4 ]; then
    echo ""
    log_warn "Some tools are missing — ensure Nix environment is loaded"
    echo "   Try: direnv allow && direnv reload"
fi

echo ""

# Summary
if [ "$failed" -eq 0 ]; then
    log_ok "direnv integration is healthy"
    exit 0
else
    log_warn "direnv integration has issues — see above"
    exit 0  # Don't fail hard; allow user to continue without full setup
fi
