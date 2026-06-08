#!/bin/sh
# ==============================================================================
# scripts/release-dry-run.sh
# Dry-run release simulation for one, several, or all workspace crates.
#
# With no arguments, simulates every crate at once via `--workspace` (dry-run
# ONLY — the real release must never use --workspace; see below). With crate
# names, simulates just those, each passed as its own `-p`.
#
# Pre-flight: the working tree must be clean, so the simulation reflects exactly
# what would be released. A dirty tree is FATAL in CI (the release gate must not
# pass on uncommitted state) but only a local advisory otherwise (so a developer
# mid-edit isn't blocked from running other checks).
#
# NOTE: `--workspace` is used for the no-arg DRY-RUN only — it simulates every
# crate at once with no side effects. The actual release (`cargo release
# --execute`) must NEVER use --workspace: a mid-run failure leaves partial state
# with no clean resume point, so crates are released one at a time in
# topological order. See docs/project/releasing.md "What Not to Do".
#
# Usage: scripts/release-dry-run.sh [crate ...]
#
# Exit codes:
#   0 = dry-run completed, or skipped locally on a dirty tree
#   1 = dirty working tree in CI (release gate refuses to proceed)
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

if [ -z "$(git status --porcelain)" ]; then
    if [ "$#" -eq 0 ]; then
        cargo release --workspace --no-confirm
    else
        pkgs=""
        for c in "$@"; do
            pkgs="${pkgs} -p ${c}"
        done
        # shellcheck disable=SC2086  # word-splitting of $pkgs is intentional
        cargo release ${pkgs} --no-confirm
    fi
    log_ok "release-dry-run: release simulation passed"
elif [ -n "${GITHUB_ACTIONS:-}" ]; then
    log_err_ci "Git tree is dirty in CI environment. Release dry-run cannot proceed."
    exit 1
else
    # A warning, not an info note — but for the footgun, not the deferral. The
    # skip itself is safe (CI re-runs the dry-run on the committed tree; the
    # GITHUB_ACTIONS branch above hard-fails there), so this is NOT about a gate
    # losing protection. It is about the uncommitted tree: a developer who sees
    # verify go green can forget the release path was never exercised against the
    # changes they are about to push. Word it around that CONSEQUENCE, not the
    # neutral state — a consequence-named warning survives habituation (the tree
    # is often dirty mid-iteration) better than a bare "tree is dirty" shrug.
    log_warn "release dry-run SKIPPED — your working tree is dirty, so the release path was NOT verified against the changes you're about to push. Commit and re-run before pushing (CI will hard-fail if the committed tree breaks it)."
fi
