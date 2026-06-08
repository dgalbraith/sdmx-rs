#!/bin/sh
# ==============================================================================
# scripts/release-push.sh
#
# Final step of the release pipeline: push the CI-validated merge commit to
# main, push all per-crate release tags (triggering publish.yml), and clean up
# the staging branch.
#
# Called by `just release-push <version>` AFTER the maintainer has verified
# the CI Quality Gate is green on the staging branch (pushed via stage-merge).
#
# Sequence:
#   0. Re-validate HEAD against the CI-verified staging branch (see below).
#   1. Fast-forward main (HEAD:refs/heads/main — no --follow-tags, avoiding
#      the race between the code push and the publish.yml verify-tag-on-main
#      gate that fires on tag push).
#   2. Push per-crate release tags (triggers publish.yml).
#   3. Delete the staging branch from origin (where it lives — not ${REMOTE},
#      which may be a mirror fan-out). || true — deletion failures are
#      non-fatal; the branch is ephemeral and will age out.
#
# SHA RE-VALIDATION (step 0): stage-merge earned a green CI Quality Gate on a
# SPECIFIC commit — the tip it pushed to staging-release-sdmx-rs-<version>. The
# gap between that and this push is a window where local HEAD could drift (a new
# commit, an amend, a different branch checked out). Pushing a drifted HEAD would
# land UNVERIFIED source on main and fire the irreversible tag-triggered publish
# on a commit CI never saw. So before any push, fetch the staging ref and assert
# local HEAD still equals it. The staging branch IS the CI-verified SHA — binding
# to it (not a local state file) means the assertion tracks exactly what CI ran.
# Fail CLOSED: if the staging ref cannot be resolved (never staged, or remote
# unreachable) we refuse rather than push something unvalidated.
#
# POSIX sh only.
#
# Usage: scripts/release-push.sh <version>   (e.g. 0.2.0  or  0.2.0-alpha.1)
#
# Exit codes:
#   0 = release landed successfully
#   1 = missing argument or a git push failed
# ==============================================================================

set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    log_err "Missing required argument: <version>"
    log_err_detail "Usage: release-push.sh <version>   (e.g. 0.2.0)"
    exit 1
fi

CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    log_err "Must be on 'main' to push the release (currently on '${CURRENT_BRANCH}')."
    log_err_detail "Run 'just release-merge' first, which checks out main, then re-run."
    exit 1
fi

REMOTE="${SDMX_MAIN_REMOTE:-origin}"
STAGING="staging-release-sdmx-rs-${VERSION}"

# ---------------------------------------------------------------------------
# Step 0: re-validate HEAD against the CI-verified staging branch.
#
# CI runs on GitHub regardless of SDMX_MAIN_REMOTE, and stage-merge always pushes
# the staging branch to `origin`, so validate against origin specifically — not
# ${REMOTE}, which may be a fan-out mirror remote (e.g. `all`) that cannot be
# fetched from. Fetch the ref fresh into FETCH_HEAD rather than trusting a stale
# local remote-tracking ref.
# ---------------------------------------------------------------------------
HEAD_SHA=$(git rev-parse HEAD)

log_info "Re-validating HEAD against CI-verified staging branch ${STAGING}..."
if ! git fetch origin "refs/heads/${STAGING}" >/dev/null 2>&1; then
    log_err "Could not fetch staging branch '${STAGING}' from origin."
    log_err_detail "release-push refuses to push source that CI has not verified."
    log_err_detail "Run 'just stage-merge ${VERSION}' first (it pushes the staging branch"
    log_err_detail "and waits for the CI Quality Gate); then re-run release-push."
    exit 1
fi

STAGING_SHA=$(git rev-parse FETCH_HEAD)
if [ "$HEAD_SHA" != "$STAGING_SHA" ]; then
    log_err "HEAD does not match the CI-verified staging commit."
    log_err_detail "  HEAD:                ${HEAD_SHA}"
    log_err_detail "  ${STAGING}: ${STAGING_SHA}"
    log_err_detail "Local HEAD drifted since stage-merge validated it. Pushing now would"
    log_err_detail "land source CI never saw and fire an irreversible publish on it."
    log_err_detail "Re-run 'just stage-merge ${VERSION}' to validate the current HEAD,"
    log_err_detail "then release-push."
    exit 1
fi
log_ok "release-push: HEAD matches CI-verified staging commit ${STAGING_SHA}"

log_info "Fast-forwarding main..."
git push "${REMOTE}" HEAD:refs/heads/main

log_info "Pushing per-crate release tags (triggers publish.yml)..."
# Scope strictly to THIS release's tags. A bare `--tags` pushes every local tag
# not yet on the remote, so a signed tag stranded by a prior aborted release
# session would fire publish.yml — burning CI and cutting a confusing GitHub
# Release even though check-published.sh skips the actual publish. Pushing
# explicit per-crate refspecs for sdmx-*/v${VERSION} closes that cross-version
# leak. A same-version stale tag is benign (same source we intend to publish)
# and is handled by check-published.sh + the verify-tag-on-main gate.
TAG_REFSPECS=""
for tag in $(git tag -l "sdmx-*/v${VERSION}"); do
    TAG_REFSPECS="${TAG_REFSPECS} refs/tags/${tag}:refs/tags/${tag}"
done

# Empty-guard: main was fast-forwarded above. If no release tags exist we must
# fail LOUDLY rather than push nothing and leave a silent half-release (main
# advanced, nothing published). The old `--tags` would have masked this.
if [ -z "$TAG_REFSPECS" ]; then
    log_err "No release tags matching 'sdmx-*/v${VERSION}' found locally."
    log_err_detail "Expected per-crate tags created by 'just release-stage ${VERSION}'."
    log_err_detail "main was fast-forwarded but there are no tags to publish."
    exit 1
fi

# shellcheck disable=SC2086 # intentional word-splitting of the refspec list
git push "${REMOTE}" $TAG_REFSPECS

log_info "Cleaning up staging branch..."
# Delete from origin specifically, not ${REMOTE}: release-stage pushes the
# staging branch to origin and step 0 above validates against origin (because
# ${REMOTE} may be a fan-out mirror like `all` that the branch never reached).
# Deleting from ${REMOTE} could therefore miss the real branch on origin while
# appearing to succeed. `|| true` keeps cleanup best-effort — a throwaway
# staging branch lingering is harmless (CI's staging-* run has long finished;
# the branch ages out), and a deletion failure must never fail a landed release.
git push origin --delete "refs/heads/${STAGING}" || true

log_ok "release-push: landed verified release and tags on ${REMOTE}"
