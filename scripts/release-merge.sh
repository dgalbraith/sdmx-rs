#!/bin/sh
# shellcheck disable=SC2015
# ==============================================================================
# scripts/release-merge.sh
# Release branch merge to main with auto-generated commit message
#
# Orchestrates the final merge of a release branch to main after per-crate
# versions have been published. Generates a structured commit message listing
# which crates were released and which were left unchanged.
#
# This script:
#   1. Extracts version for each crate from Cargo.toml via cargo metadata
#   2. Checks if that crate was tagged (released) in this batch
#   3. Builds a structured commit message with Released and Unchanged lists
#   4. Merges the release branch to main with a signed merge commit
#
# Usage: scripts/release-merge.sh [<branch-or-version>]
#
# Assumptions: You are on a release/sdmx-rs/<version> branch after
# cargo-release --execute (the version-named branch convention, matching the
# CI staging branch staging-release-sdmx-rs-<version>; see docs/project/
# releasing.md §5). With no argument the current branch is used if it is a
# release/* branch; otherwise the script aborts rather than guessing a name.
#
# Exit codes:
#   0 = merge succeeded
#   1 = release tags not found, cargo metadata failed, or git operations failed
# ==============================================================================

set -eu

# Source shared configuration
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/common.sh"

# Determine branch name
BRANCH=""
if [ $# -gt 0 ] && [ -n "$1" ]; then
    case "$1" in
        release/*)
            BRANCH="$1"
            ;;
        *)
            BRANCH="release/$1"
            ;;
    esac
else
    CURRENT_BRANCH=$(git symbolic-ref --short -q HEAD || git branch --show-current || true)
    case "$CURRENT_BRANCH" in
        release/*)
            BRANCH="$CURRENT_BRANCH"
            ;;
        *)
            # Do NOT fabricate a branch name. The release branch is version-named
            # (release/sdmx-rs/<version>); guessing a date-based name here would
            # silently target a branch that does not exist and fail confusingly
            # later, or worse, match an unrelated one. Make the caller be explicit.
            log_err "Not on a release/* branch (currently on '${CURRENT_BRANCH:-detached HEAD}')."
            log_err_detail "Pass the release branch or version explicitly, e.g.:"
            log_err_detail "  scripts/release-merge.sh release/sdmx-rs/0.2.0"
            log_err_detail "  scripts/release-merge.sh sdmx-rs/0.2.0"
            exit 1
            ;;
    esac
fi

# The portion after "release/" labels the merge commit (e.g. "sdmx-rs/0.2.0").
RELEASE_LABEL="${BRANCH#release/}"
RELEASED=""
UNCHANGED=""

log_section "Release Merge Orchestration"

# Guard against polluting the signed release merge with un-pushed local work.
# The script ends with `git checkout main` + `git merge --no-ff -S`. A dirty
# tracked tree at that point can ride across the checkout and be swept into the
# *signed* merge commit — lending release provenance to code that was never
# reviewed, pushed, or on a PR. Untracked files are intentionally NOT blocked:
# git refuses to overwrite them on checkout (it errors instead of merging them
# in), so editor scratch and notes cannot enter the commit. Only staged/unstaged
# modifications to tracked files are the real exposure. Check fast, before the
# per-crate cargo metadata loop makes the maintainer wait.
if ! git diff --quiet || ! git diff --cached --quiet; then
    log_err "Working tree has uncommitted changes to tracked files."
    log_err_detail "These could be swept into the signed release merge commit."
    echo "" >&2
    git -c color.status=always status --short --untracked-files=no >&2
    echo "" >&2
    log_err_detail "Action required: commit, stash, or discard these before releasing."
    exit 1
fi

# Verify the release branch exists locally before proceeding
if ! git show-ref --verify --quiet "refs/heads/$BRANCH"; then
    log_fatal "Release branch '$BRANCH' does not exist locally."
fi

REMOTE="${SDMX_MAIN_REMOTE:-origin}"

# Fetch latest main from remote to ensure tag detection is accurate
# and not polluted by stale local main state. Use an explicit refspec so the
# remote-tracking ref is updated regardless of the remote's configured fetch
# refspec — a bare `git fetch <remote> main` only guarantees FETCH_HEAD, so a
# non-default ${SDMX_MAIN_REMOTE} could otherwise leave ${REMOTE}/main stale.
echo "   Fetching latest main from remote '${REMOTE}'..."
git fetch "$REMOTE" "main:refs/remotes/${REMOTE}/main" -q || \
    log_fatal "Could not fetch 'main' from remote '${REMOTE}'."

# Guard against silent misclassification: if ${REMOTE}/main does not resolve,
# `git tag --no-merged "${REMOTE}/main"` errors, the error is swallowed below,
# and every crate falls through to Unchanged. Fail loudly instead.
git rev-parse --verify --quiet "${REMOTE}/main^{commit}" >/dev/null || \
    log_fatal "'${REMOTE}/main' did not resolve after fetch."

# Temp files — one for cargo metadata per iteration, one for the commit message
META_FILE=$(mktemp)
MSG_FILE=$(mktemp)
trap 'rm -f "$META_FILE" "$MSG_FILE"' EXIT

for crate in $CRATES; do
    # Write to temp file so cargo failure is caught without pipefail
    cargo metadata --no-deps \
        --manifest-path "crates/${crate}/Cargo.toml" \
        --format-version 1 > "$META_FILE" || \
        log_fatal "Could not read metadata for ${crate} — aborting"

    VERSION=$(jq -r '.packages[0].version' "$META_FILE") || \
        log_fatal "Could not parse version from metadata for ${crate} — aborting"

    if [ -z "$VERSION" ]; then
        log_fatal "Could not read version for ${crate} — aborting"
    fi

    # Check if this crate was tagged in THIS batch on the release branch.
    # cargo-release stamps one release commit + tag per crate sequentially, so
    # only the last crate's tag sits on the branch tip — `--points-at "$BRANCH"`
    # would miss every earlier crate. Instead, find tags reachable from the
    # release branch but not yet from remote main: that is exactly this batch's
    # tags, and it correctly excludes prior-batch tags already merged into main.
    # shellcheck disable=SC2086
    TAG=$(git tag --sort=-v:refname --merged "$BRANCH" --no-merged "${REMOTE}/main" "${crate}/v*" 2>/dev/null | head -n 1) || true

    if [ -n "$TAG" ]; then
        RELEASED="${RELEASED}- ${crate}: v${VERSION}
"
    else
        UNCHANGED="${UNCHANGED}- ${crate}: v${VERSION}
"
    fi
done

if [ -z "$RELEASED" ]; then
    log_fatal "No release tags found on $BRANCH — did cargo release --execute run?"
fi

# Build commit message into temp file. RELEASED/UNCHANGED already carry a
# trailing newline per entry, so use printf with %s (no extra newline) — a
# heredoc whose terminator would land on the same source line as an expanded
# ${VAR} does NOT terminate, silently swallowing the rest of the script.
printf 'chore(release): merge release branch %s\n\nReleased:\n%s' \
    "$RELEASE_LABEL" "$RELEASED" > "$MSG_FILE"

if [ -n "$UNCHANGED" ]; then
    printf 'Unchanged:\n%s' "$UNCHANGED" >> "$MSG_FILE"
fi

# Closing note.
printf '\nAll commits and tags are cryptographically signed.\n' >> "$MSG_FILE"

# For testing, allow skipping GPG signing
SIGN_FLAG="-S"
if [ "${RELEASE_MERGE_NO_SIGN:-}" = "1" ]; then
    SIGN_FLAG=""
fi

git checkout main
# Canonical main remote (default: origin; override with SDMX_MAIN_REMOTE).
# Distinct from the early fetch above: that one updates the remote-tracking ref
# for *tag classification*; this fast-forwards local main for the *merge base*.
# Do not dedupe — both are needed.
git pull "$REMOTE" main
# shellcheck disable=SC2086
git merge --no-ff $SIGN_FLAG "$BRANCH" -m "$(cat "$MSG_FILE")"

# DELIBERATE: this script stops at a LOCAL merge and does NOT push. The gap
# between merge and push is a review gate — the maintainer inspects the proposed
# merge commit (`git show HEAD`: message, diff, signature) before anything
# reaches the remote, and can abort cheaply (`git reset --hard @{1}`) since a
# local merge is free to discard. Pushing is the first irreversible step toward
# the tag-triggered publish chain, so a human checkpoint precedes it by design.
# Do NOT add `git push` here. The CI `verify-tag-on-main` gate is the backstop,
# not a substitute for this proactive review.
log_ok "Release merge completed (LOCAL). Review, then push to release:"
echo "   git show HEAD                    # inspect the merge commit before it leaves your machine"
echo "   just stage-merge <version>       # push to staging, poll CI, then:"
echo "   just release-push <version>      # fast-forward main and push tags"
