#!/bin/sh
# ==============================================================================
# scripts/guard-main-push.sh
# Pre-push accident-guard: refuse a direct push to `main` that skipped staging.
#
# The canonical path to `main` (docs/project/merging.md) pushes a locally-signed
# merge commit to a `staging-*` branch first, lets CI verify that exact SHA, then
# fast-forwards `main` to it. A `git push <remote> main` typed from muscle memory
# bypasses that round-trip: the commit lands on `main` without ever having earned
# the CI seal. The forge's `required_status_checks` ruleset does NOT stop this —
# it gates pull-request merges, not direct ref updates (see SECURITY.md).
#
# This hook is the local safety net for that gap. When a push targets `main` it
# checks whether the pushed SHA already sits on a `staging-*` branch of the
# canonical remote (evidence it went through the round-trip); if not, it aborts.
# It is an ACCIDENT guard, not a security control: the maintainer is the root of
# trust and can consciously override (SDMX_ALLOW_DIRECT_MAIN=1, or the built-in
# `git push --no-verify`). It cannot, and does not claim to, stop a determined
# actor with valid credentials — only a slip of the fingers.
#
# Invoked by pre-commit at the pre-push stage (see .pre-commit-config.yaml),
# which exports the push destination and SHA as PRE_COMMIT_* variables. Run
# standalone for tests by setting those variables directly.
#
# Exit codes: 0 = push allowed (not main, staged, or overridden); 1 = blocked.
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

# pre-commit exports these at the pre-push stage. REMOTE_BRANCH is the
# DESTINATION ref; TO_REF is the local SHA being pushed. Absent them (not a
# pre-push context, or a pre-commit too old to set them) we cannot judge the
# destination — an accident guard must never wedge an otherwise-valid push, so
# fail open.
remote_branch="${PRE_COMMIT_REMOTE_BRANCH:-}"
pushed_sha="${PRE_COMMIT_TO_REF:-}"

# Only pushes to main are guarded. Feature branches and the staging-* push itself
# (`main:staging-<slug>`) have a non-main destination and pass silently.
case "$remote_branch" in
    refs/heads/main | main) ;;
    *) exit 0 ;;
esac

# A branch deletion (all-zero SHA) or an unset SHA is not a content push.
case "$pushed_sha" in
    "" | 0000000000000000000000000000000000000000) exit 0 ;;
esac

# Conscious override for the rare intentional direct push to main.
if [ "${SDMX_ALLOW_DIRECT_MAIN:-}" = "1" ]; then
    log_warn "SDMX_ALLOW_DIRECT_MAIN=1 — skipping the staging round-trip check for this push to main."
    exit 0
fi

REMOTE="${SDMX_MAIN_REMOTE:-origin}"

# The pushed SHA is legitimate iff it already sits on a staging-* branch of the
# canonical remote, where CI verified it. ls-remote may fail (offline) — but a
# real push needs the network too, so a failure here means the push itself would
# also fail; treat it as "cannot verify" and fail open rather than block a push
# that the network is about to reject anyway.
staging_refs=$(git ls-remote --heads "$REMOTE" 'staging-*' 2>/dev/null) || {
    log_warn "Could not query '${REMOTE}' for staging-* branches; skipping the round-trip check."
    exit 0
}

if printf '%s\n' "$staging_refs" | grep -q "^${pushed_sha}[[:space:]]"; then
    exit 0
fi

short=$(git rev-parse --short "$pushed_sha" 2>/dev/null || printf '%s' "$pushed_sha")
log_err "Refusing to push ${short} directly to main: it is not on any staging-* branch."
log_err_detail "The canonical path stages the commit for CI first (docs/project/merging.md):"
log_err_detail "  git push ${REMOTE} main:staging-<slug>   # CI verifies this exact SHA"
log_err_detail "  git push all main                        # fast-forward once green"
log_err_detail "If this direct push is intentional, override for this push only:"
log_err_detail "  SDMX_ALLOW_DIRECT_MAIN=1 git push ..."
exit 1
