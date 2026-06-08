#!/bin/sh
# ==============================================================================
# scripts/doctor-env.sh
# Environment overview diagnostics
#
# Quick overview of the core local toolchain: Nix, direnv, the Rust compiler,
# and pre-commit hooks. Surfaced by `just doctor-env` (and `just setup`) as the
# one-glance "is my environment wired up?" check.
#
# Usage: scripts/doctor-env.sh
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

log_section "Environment Check"
echo ""

if command -v nix >/dev/null 2>&1; then
    log_ok "Nix installed: $(nix --version)"
else
    log_fail "Nix not found — install from https://nixos.org/download.html"
fi

if command -v direnv >/dev/null 2>&1; then
    log_ok "direnv installed: $(direnv --version)"
else
    log_fail "direnv not found — required for automatic environment activation"
fi

echo ""
echo "Rust toolchain:"
cargo --version
rustc --version
echo ""

if [ -f .git/hooks/pre-commit ]; then
    log_ok "Pre-commit hooks installed"
else
    log_fail "Pre-commit hooks not installed — run 'just hook-install'"
fi

echo ""
