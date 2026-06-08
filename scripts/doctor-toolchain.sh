#!/bin/sh
# ==============================================================================
# scripts/doctor-toolchain.sh
# Rust toolchain and dependencies validation
#
# Validates that the Rust toolchain is properly installed and all dependencies
# are available for the project.
#
# Usage: scripts/doctor-toolchain.sh
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

log_section "Rust Toolchain Validation"
echo ""

# Track overall status
failed=0

# Read MSRV from Cargo.toml [workspace.package]
msrv=$(sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | grep "^rust-version" | sed 's/rust-version = "\([^"]*\)".*/\1/' | head -1 || true)

if [ -z "$msrv" ]; then
    log_warn "MSRV not found in Cargo.toml"
else
    log_info "Declared MSRV: $msrv"
fi

echo ""

# Check 1: rustc version matches MSRV
if command -v rustc >/dev/null 2>&1; then
    rustc_version=$(rustc --version | awk '{print $2}')
    log_ok "rustc: $rustc_version"

    if [ -n "$msrv" ]; then
        if [ "$rustc_version" = "$msrv" ]; then
            log_ok "Matches declared MSRV" 1
        else
            log_warn "Does not match MSRV ($msrv)" 1
            echo "      Upgrade: rustup update or use Nix devshell"
        fi
    fi
else
    log_fail "rustc not found"
    failed=1
fi

echo ""

# Check 2: cargo version
if command -v cargo >/dev/null 2>&1; then
    cargo_version=$(cargo --version | awk '{print $2}')
    log_ok "cargo: $cargo_version"
else
    log_fail "cargo not found"
    failed=1
fi

echo ""

# Check 3: rustfmt (nightly from RUSTFMT env var)
echo "Formatting & Linting Tools:"

if [ -n "${RUSTFMT:-}" ]; then
    if [ -f "$RUSTFMT" ]; then
        rustfmt_version=$("$RUSTFMT" --version 2>/dev/null | head -1 || echo "")
        log_ok "rustfmt (nightly): $rustfmt_version" 1
    else
        log_fail "RUSTFMT env var set but file not found: $RUSTFMT" 1
        failed=1
    fi
else
    log_warn "RUSTFMT env var not set (needed for formatting)" 1
    echo "     Load Nix devshell: direnv allow && direnv reload"
fi

# Check for stable rustfmt too
if command -v rustfmt >/dev/null 2>&1; then
    rustfmt_stable=$(rustfmt --version 2>/dev/null | head -1 || echo "")
    log_info "rustfmt (stable): $rustfmt_stable" 1
fi

# Check 4: clippy
if command -v cargo-clippy >/dev/null 2>&1; then
    log_ok "clippy available" 1
else
    log_warn "clippy not found (usually bundled with rustc)" 1
fi

echo ""

# Check 5: Essential cargo tools
echo "Required Cargo Tools:"

required_tools="cargo-machete cargo-deny cargo-nextest"

for tool in $required_tools; do
    if command -v "$tool" >/dev/null 2>&1; then
        version=$("$tool" --version 2>/dev/null | head -1 || echo "")
        log_ok "$tool: $version" 1
    else
        log_fail "$tool not found" 1
        failed=1
    fi
done

echo ""

# Check 6: Optional but important tools
echo "Optional Tools:"

optional_tools="cargo-llvm-cov cargo-semver-checks cargo-udeps"

for tool in $optional_tools; do
    if command -v "$tool" >/dev/null 2>&1; then
        version=$("$tool" --version 2>/dev/null | head -1 || echo "")
        log_ok "$tool: $version" 1
    else
        log_warn "$tool not found (optional for Phase 1)" 1
    fi
done

echo ""

# Check 7: WASM target
echo "WASM Support:"

if rustup target list 2>/dev/null | grep -q "wasm32-unknown-unknown (installed)"; then
    log_ok "wasm32-unknown-unknown target installed" 1
else
    log_warn "wasm32-unknown-unknown target not installed" 1
    echo "     Run: rustup target add wasm32-unknown-unknown"
fi

echo ""

# Check 8: Workspace linting configuration
echo "Workspace Linting:"

lint_config=$(sed -n '/^\[workspace\.lints\.rust\]/,/^\[/p' Cargo.toml | grep "unsafe_code" | head -1 || true)

if [ -n "$lint_config" ]; then
    log_ok "unsafe_code lint configured" 1
else
    log_warn "unsafe_code lint not configured in [workspace.lints.rust]" 1
fi

echo ""

# Summary
if [ "$failed" -eq 0 ]; then
    log_ok "Rust toolchain is configured correctly"
    exit 0
else
    log_fail "Toolchain has missing components — see above"
    exit 1
fi
