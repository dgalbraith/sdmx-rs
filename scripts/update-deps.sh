#!/bin/sh
set -eu

# ==============================================================================
# scripts/update-deps.sh
# Refresh pinned crate dependencies in Cargo.lock (semver-compatible only) and
# validate the result, leaving an UNSTAGED diff for manual review and signing.
#
# Lockfile-only: never edits the version ranges in Cargo.toml. Updates all crates
# by default, or the named specs (each gets its own `-p`; cargo rejects a single
# space-joined spec). Mutation, not commit — see docs/dev/tooling.md
# "Refreshing Pinned Dependencies".
#
# Usage: scripts/update-deps.sh [crate ...]
#
# Behaviour contract (exercised by tests/bats/update-deps.bats):
#   - Pre-flight: refuse to run if Cargo.lock already has staged or unstaged
#     changes, so the resulting diff is solely this run's.
#   - No-op: if cargo reports no package deltas, print "(none)" and exit 0
#     WITHOUT running validation.
#   - Change: print the captured deltas, then run `just verify-rust`.
#
# Indirection for tests: CARGO and JUST may be overridden to point at mocks.
# ==============================================================================

CARGO="${CARGO:-cargo}"
JUST="${JUST:-just}"

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

# --- Pre-flight: lock must be clean (committed) before we start ----------------
if ! git diff --quiet -- Cargo.lock || ! git diff --cached --quiet -- Cargo.lock; then
    log_fatal "Cargo.lock already has uncommitted changes — commit or revert them first."
fi

summary=$(mktemp)
trap 'rm -f "$summary"' EXIT

# --- Update -------------------------------------------------------------------
if [ "$#" -eq 0 ]; then
    "$CARGO" update 2>&1 | tee "$summary"
else
    args=""
    for spec in "$@"; do
        args="$args -p $spec"
    done
    # shellcheck disable=SC2086  # word-splitting of $args is intentional
    "$CARGO" update $args 2>&1 | tee "$summary"
fi

# --- Surface the change set (exclude the "Updating crates.io index" status) ----
echo ""
echo "📦 Dependency changes:"
changes=$(grep -E '^[[:space:]]*(Updating|Adding|Removing|Downgrading) ' "$summary" \
    | grep -vE 'Updating crates\.io index' || true)
if [ -z "$changes" ]; then
    echo "   (none — Cargo.lock already at the latest compatible versions; skipping validation)"
    exit 0
fi
printf '%s\n' "$changes" | sed 's/^ */   /'

# --- Validate -----------------------------------------------------------------
echo ""
echo "🔎 Validating updated dependency graph..."
"$JUST" verify-rust

echo ""
log_ok "Cargo.lock updated and validated (left unstaged). Next:"
echo "   1. Review:  git diff Cargo.lock"
echo "   2. Commit:  git commit -S Cargo.lock -m 'chore(deps): update Cargo.lock'"
