#!/bin/sh
# ==============================================================================
# scripts/run-fmt.sh
#
# Format Rust code with the project-sanctioned NIGHTLY rustfmt. rustfmt.toml uses
# nightly-only options, so formatting MUST run under the pinned nightly rustfmt
# (provided by the Nix devShell as $RUSTFMT) — stable rustfmt would silently skip
# those options and produce a different result. This script enforces that: it
# hard-fails with guidance if $RUSTFMT is unset, then runs `cargo fmt`.
#
# TOML formatting is handled separately by the `toml-fmt` recipe, which the `fmt`
# recipe depends on; this script is only the Rust half.
#
# Extracted from the `fmt` Justfile recipe so the env guard is a testable unit
# and its error goes through log.sh's `log_fatal` (the sanctioned `❌ Error:` +
# exit) instead of a hand-rolled `echo "❌ Error:"` that bypassed the log library
# and could drift from its format. The recipe now delegates its body here.
#
# `log_fatal` is called at the TOP LEVEL of this script (never in a subshell), as
# log.sh's subshell contract requires for its `exit 1` to actually terminate.
#
# POSIX sh only.
#
# Environment:
#   RUSTFMT  path to the nightly rustfmt binary (set by the Nix devShell). REQUIRED.
#   CARGO    cargo invocation to use (default: cargo) — indirection for tests,
#            which stub it so the guard can be exercised without a real format.
#
# Exit codes:
#   0 = formatting completed
#   1 = $RUSTFMT not set (cannot guarantee nightly rustfmt)
#   N = cargo fmt failed (its own exit code)
# ==============================================================================

set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

CARGO="${CARGO:-cargo}"

if [ -z "${RUSTFMT:-}" ]; then
    log_err "RUSTFMT environment variable not set — cannot guarantee the nightly rustfmt that rustfmt.toml requires."
    log_err_detail "Enter the Nix devShell, or run:"
    log_err_detail "RUSTFMT=\$(nix build --print-out-paths --no-link .#nightly-fmt)/bin/rustfmt just fmt"
    exit 1
fi

"$CARGO" fmt -- --config-path rustfmt.toml

log_ok "fmt: Rust code formatted with nightly rustfmt"
