#!/bin/sh
# ==============================================================================
# scripts/doctor-devshell.sh
# Nix devshell package validation
#
# Validates that required packages are declared in the Nix devshell environment,
# ensuring all build dependencies are available to developers.
#
# Usage: scripts/doctor-devshell.sh
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

log_section "Nix Devshell Package Validation"
echo ""

# Track overall status
failed=0

# Extract package list from flake.nix
# Match the nativeBuildInputs section in devShells and extract all pkgs.* declarations
packages=$(sed -n '/nativeBuildInputs = \[/,/\];/p' flake.nix | \
           grep -E "^\s+pkgs\." | \
           sed 's/.*pkgs\.\([a-zA-Z0-9_-]*\).*/\1/')

if [ -z "$packages" ]; then
    log_warn "Could not parse packages from flake.nix buildInputs"
    echo ""
    exit 1
fi

# Count packages
total_packages=$(echo "$packages" | wc -l)
found_packages=0
missing_packages=""

echo "Checking declared packages ($total_packages total):"
echo ""

for pkg in $packages; do
    # Extract the command name (usually the same as package name, but some differ)
    # For example: cargo-release → cargo-release, git-cliff → git-cliff
    # Some packages have different command names (e.g., adr-tools → adr)
    cmd_name="$pkg"

    # Handle special cases where package name ≠ command name
    case "$pkg" in
        "git-cliff")
            cmd_name="git-cliff"
            ;;
        "adr-tools")
            cmd_name="adr"
            ;;
        "nodejs")
            cmd_name="node"
            ;;
    esac

    if command -v "$cmd_name" >/dev/null 2>&1; then
        # Get version
        version_output=""
        case "$cmd_name" in
            "cargo-release")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            "cargo-nextest")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            "git-cliff")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            "just")
                version_output=$($cmd_name --version 2>/dev/null || echo "")
                ;;
            "cargo-deny")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            "cargo-machete")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            "cargo-llvm-cov")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            "cargo-semver-checks")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            "git")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            "gh")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            "shellcheck")
                version_output=$($cmd_name --version 2>/dev/null | grep "version" | head -1 || echo "")
                ;;
            "taplo")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            "markdownlint-cli2")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            "wasm-pack")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            "node")
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
            *)
                version_output=$($cmd_name --version 2>/dev/null | head -1 || echo "")
                ;;
        esac

        if [ -n "$version_output" ]; then
            log_ok "$pkg ($version_output)" 1
        else
            log_ok "$pkg" 1
        fi
        found_packages=$((found_packages + 1))
    else
        log_fail "$pkg (command '$cmd_name' not found)" 1
        missing_packages="$missing_packages $pkg"
        failed=1
    fi
done

echo ""
echo "Summary: $found_packages / $total_packages packages available"

echo ""

# Check if we're in a Nix devshell
if [ -n "${IN_NIX_SHELL:-}" ]; then
    log_info "Nix devshell is active"
else
    log_warn "Not running in Nix devshell"
    echo "   Run: direnv allow && direnv reload"
    echo "   Or: nix develop"
fi

echo ""

# Summary
if [ "$failed" -eq 0 ]; then
    log_ok "All devshell packages are available"
    exit 0
else
    log_fail "Missing packages:$missing_packages"
    echo "   Try: direnv reload or nix develop"
    exit 1
fi
