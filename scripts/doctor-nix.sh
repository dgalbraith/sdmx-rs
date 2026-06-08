#!/bin/sh
# ==============================================================================
# scripts/doctor-nix.sh
# Nix flake diagnostics
#
# Validates that the Nix flake configuration is correct and all declared
# inputs and outputs are properly configured.
#
# Usage: scripts/doctor-nix.sh
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

log_section "Nix Flake Diagnostics"
echo ""

# Track overall status
failed=0

# Check 1: Nix installed
if command -v nix >/dev/null 2>&1; then
    nix_version=$(nix --version 2>/dev/null | head -1)
    log_ok "Nix installed: $nix_version"
else
    log_fail "Nix not found"
    echo "   Install from: https://nixos.org/download.html"
    failed=1
fi

echo ""

# Check 2: Flakes experimental feature enabled
if [ -d ~/.config/nix ]; then
    if grep -q "experimental-features" ~/.config/nix/nix.conf 2>/dev/null && \
       grep "experimental-features" ~/.config/nix/nix.conf | grep -q "nix-command"; then
        log_ok "Nix Flakes experimental feature enabled"
    else
        log_warn "Nix Flakes may not be enabled in nix.conf"
        echo "   Add to ~/.config/nix/nix.conf:"
        echo "   experimental-features = nix-command flakes"
    fi
elif command -v nix >/dev/null 2>&1; then
    # Check via nix flake command itself
    if nix flake --version >/dev/null 2>&1; then
        log_ok "Nix Flakes experimental feature enabled (detected via nix flake)"
    else
        log_warn "Cannot verify Flakes are enabled"
    fi
fi

echo ""

# Check 3: flake.nix exists and is valid
if [ -f flake.nix ]; then
    if nix flake check --offline >/dev/null 2>&1; then
        log_ok "flake.nix syntax is valid"
    else
        log_fail "flake.nix has syntax errors"
        nix flake check --offline 2>&1 | head -5
        failed=1
    fi
else
    log_fail "flake.nix not found in current directory"
    failed=1
fi

echo ""

# Check 4: flake.lock age
if [ -f flake.lock ]; then
    # Get file modification time (portable)
    lock_mtime=$(stat -f%m flake.lock 2>/dev/null || stat -c%Y flake.lock 2>/dev/null)
    current_time=$(date +%s)
    age_days=$(( (current_time - lock_mtime) / 86400 ))

    if [ "$age_days" -lt 7 ]; then
        log_ok "flake.lock is current ($age_days days old)"
    elif [ "$age_days" -lt 30 ]; then
        log_info "flake.lock is $age_days days old (consider updating)"
    else
        log_warn "flake.lock is $age_days days old — consider updating"
        echo "   Run: just update-flake  (stages + validates; review and sign manually)"
    fi
else
    log_fail "flake.lock not found"
    failed=1
fi

echo ""

required_inputs="nixpkgs rust-overlay crane"
missing_inputs=""

for input in $required_inputs; do
    # Match input name in the inputs section (before 'outputs')
    if sed -n '/^  inputs = {/,/^  };/p' flake.nix | grep -q "$input"; then
        # Found in flake.nix inputs section
        continue
    else
        missing_inputs="$missing_inputs $input"
    fi
done

if [ -z "$missing_inputs" ]; then
    log_ok "All required flake inputs present (nixpkgs, rust-overlay, crane)"
else
    log_fail "Missing required inputs:$missing_inputs"
    failed=1
fi

echo ""

# Summary
if [ "$failed" -eq 0 ]; then
    log_ok "Nix flake setup is healthy"
    exit 0
else
    log_fail "Nix flake setup has issues — see above"
    exit 1
fi
