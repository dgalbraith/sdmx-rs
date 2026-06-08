#!/bin/sh
# ==============================================================================
# scripts/ci/secrets-scan.sh
#
# Scan for committed secrets (API keys, tokens, private keys) with gitleaks.
# Two modes, selected by environment — same tool, different surface:
#
#   CI (GITHUB_ACTIONS set)  → `gitleaks detect`: scans the full commit HISTORY.
#                              The merge gate; a leak anywhere in history fails.
#   Local (default)          → `gitleaks protect`: scans STAGED/uncommitted work
#                              only. A pre-commit-style guard that catches a
#                              secret before it is ever committed, without paying
#                              the cost of a full-history scan on every run.
#
# Extracted from the `secrets-scan` Justfile recipe so the env-branch is a
# testable unit (see tests/bats/secrets-scan.bats) and its output is framed by
# the shared log library rather than raw gitleaks text — closing the
# script-vs-recipe logging parity gap. The recipe is now a one-line delegate.
#
# POSIX sh only (runs under dash/busybox in CI containers).
#
# Environment:
#   GITHUB_ACTIONS  when "true" (CI), run the full-history `detect`; otherwise
#                   the working-tree `protect`. This is the ONLY mode switch.
#   GITLEAKS        gitleaks invocation to use (default: gitleaks) — indirection
#                   for tests, which point it at a stub so no real scan runs.
#
# Exit codes:
#   0 = no leaks found
#   N = gitleaks found leaks (or itself failed); N is gitleaks' own exit code,
#       propagated so the gate fails exactly when gitleaks does.
# ==============================================================================

set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/../lib/log.sh"

GITLEAKS="${GITLEAKS:-gitleaks}"

log_section "Scanning for committed secrets"

# `|| status=$?` so `set -e` does not abort before we can frame the result —
# we want a log_fail with context on a leak, not a bare non-zero exit.
status=0
if [ "${GITHUB_ACTIONS:-}" = "true" ]; then
    log_item "CI mode: scanning full commit history (gitleaks detect)" 1
    "$GITLEAKS" detect --source . --redact --verbose || status=$?
else
    log_item "Local mode: scanning staged/uncommitted changes (gitleaks protect)" 1
    "$GITLEAKS" protect --redact --verbose || status=$?
fi

if [ "$status" -ne 0 ]; then
    log_fail "secrets-scan: gitleaks reported findings (exit ${status}). Review the redacted report above."
    exit "$status"
fi

log_ok "secrets-scan: no secrets found"
