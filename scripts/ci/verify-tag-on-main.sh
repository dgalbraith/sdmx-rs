#!/bin/sh
set -eu

# ==============================================================================
# scripts/ci/verify-tag-on-main.sh
# Asserts that the commit a release tag points at is reachable from the canonical
# main branch — i.e. the source being published already lives in main.
#
# Why this gate exists: publishing keys off a pushed "<crate>/v<version>" tag via
# Trusted Publishing, and crates.io publication is irreversible. The tag commit
# can be pushed (and thus trigger publish) BEFORE the release branch is merged
# into main. Without this check, a crate could go live — fully attested — whose
# source commit is an orphan relative to origin/main. This gate makes "the source
# is in main" a precondition of going live, enforced by the trusted publisher
# itself rather than by a maintainer-run post-publish merge that may never land.
#
# Usage: scripts/ci/verify-tag-on-main.sh <tag-commit-sha>
#   In CI, pass "$GITHUB_SHA" — for a tag push that is the tag's commit.
#
# Environment:
#   SDMX_MAIN_REMOTE  remote holding canonical main (default: origin)
#
# Exit codes:
#   0 = tag commit is an ancestor of (or equal to) <remote>/main
#   1 = tag commit is NOT reachable from main, or git/fetch failed
# ==============================================================================

# This is a CI gate; keep stdout clean by sending all script output to stderr.
# Done once here (before sourcing the logger, so its [ -t 1 ] colour check sees the
# redirected fd 1) rather than appending `>&2` to every line.
exec 1>&2

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/../lib/log.sh"

COMMIT="${1:?usage: verify-tag-on-main.sh <tag-commit-sha>}"
REMOTE="${SDMX_MAIN_REMOTE:-origin}"

# Resolve the commit the tag names to a concrete object. Accept a raw SHA
# (GITHUB_SHA) or any other revision spec; peel annotated tags to their commit.
TAG_COMMIT=$(git rev-parse --verify --quiet "${COMMIT}^{commit}") || \
    log_fatal "'${COMMIT}' does not resolve to a commit."

# Fetch main explicitly into a known ref. A tag-triggered checkout fetches the
# tag's history but does NOT guarantee a remote-tracking ref for main exists, so
# we cannot rely on ${REMOTE}/main already being present. Use an explicit refspec
# so the ref is populated regardless of the remote's configured fetch refspec.
log_info "Verifying release source is in main (remote '${REMOTE}')..."
git fetch --quiet "$REMOTE" "main:refs/remotes/${REMOTE}/main" || \
    log_fatal "Could not fetch 'main' from remote '${REMOTE}'."

MAIN_COMMIT=$(git rev-parse --verify --quiet "refs/remotes/${REMOTE}/main^{commit}") || \
    log_fatal "'${REMOTE}/main' did not resolve after fetch."

# The core invariant: the tag commit must be reachable from main. --is-ancestor
# is true when TAG_COMMIT == MAIN_COMMIT or lies anywhere in main's history.
# main is fetched live here (not a cached SHA) on purpose: the gate may pass long
# before the human-approved publish step runs, but main only moves forward, so a
# proven ancestor stays an ancestor — re-resolving against newer main cannot lose
# the property. Do NOT "optimise" this into a pinned SHA: that would only narrow
# what counts as in-main and reintroduce the pre-merge-tag race this gate closes.
if git merge-base --is-ancestor "$TAG_COMMIT" "$MAIN_COMMIT"; then
    log_ok "Tag commit ${TAG_COMMIT} is reachable from ${REMOTE}/main."
    exit 0
fi

log_err "the release tag's commit is NOT in main."
echo ""
echo "   Tag commit:    ${TAG_COMMIT}"
echo "   ${REMOTE}/main: ${MAIN_COMMIT}"
echo ""
echo "   Publishing would put source on crates.io that is not reachable from"
echo "   the canonical main branch. Merge the release branch into main and"
echo "   ensure it is pushed before the publish tag, then re-trigger."
exit 1
