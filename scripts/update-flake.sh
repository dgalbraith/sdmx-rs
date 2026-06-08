#!/bin/sh
set -eu

# ==============================================================================
# scripts/update-flake.sh
# Refresh pinned Nix flake inputs (nixpkgs, rust-overlay, crane) in flake.lock
# and validate the result, leaving an UNSTAGED diff for manual review/signing.
#
# Governs the toolchain & dev tools, NOT the crate graph. Mutation, not commit —
# see docs/dev/tooling.md "Refreshing Pinned Dependencies".
#
# Staging dance: the Nix sandbox copies only git-tracked files, so flake.lock
# MUST be staged for `nix flake check` (via verify-infra) to see the new inputs.
# We therefore stage it transiently for validation, then `git restore --staged`
# to return the index to its pre-run (clean) state, leaving only an unstaged
# working-tree diff. The pre-flight dirty guard guarantees the lock was clean in
# HEAD beforehand, which makes restore-staged provably non-destructive: it
# reverts the index entry to the HEAD blob and never touches the working file.
# The unstage runs from the EXIT trap, so it fires even if validation fails.
#
# Usage: scripts/update-flake.sh
#
# Indirection for tests: NIX and JUST may be overridden to point at mocks.
# ==============================================================================

NIX="${NIX:-nix}"
JUST="${JUST:-just}"

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

# --- Pre-flight: lock must be clean; also the precondition that makes the -------
# --- post-validation `git restore --staged` safe. -----------------------------
if ! git diff --quiet -- flake.lock || ! git diff --cached --quiet -- flake.lock; then
    log_fatal "flake.lock already has uncommitted changes — commit or revert them first."
fi

summary=$(mktemp)
# Trap is extended after staging to also unstage; both arms clean $summary.
trap 'rm -f "$summary"' EXIT

# --- Update -------------------------------------------------------------------
"$NIX" flake update 2>&1 | tee "$summary"

# --- Surface the change set ----------------------------------------------------
echo ""
echo "📦 Flake input changes:"
if ! grep -qiE 'Updated input' "$summary"; then
    echo "   (none — flake.lock already at the pinned inputs' latest; skipping validation)"
    exit 0
fi
grep -iE 'Updated input' "$summary" | sed 's/^/   /'

# --- Validate (stage transiently; unstage via trap regardless of outcome) ------
echo ""
echo "🔎 Validating updated flake (staged transiently for the Nix sandbox)..."
git add flake.lock
# Extend trap to also unstage flake.lock; overwrite is safe — $summary is still cleaned.
trap 'git restore --staged flake.lock 2>/dev/null || true; rm -f "$summary"' EXIT
"$JUST" verify-infra

echo ""
log_ok "flake.lock updated and validated (left unstaged). Next:"
echo "   1. Review:  git diff flake.lock"
echo "   2. Commit:  git commit -S flake.lock -m 'chore(deps): update flake.lock'"
