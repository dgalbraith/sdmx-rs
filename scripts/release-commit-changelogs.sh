#!/bin/sh
# ==============================================================================
# scripts/release-commit-changelogs.sh
#
# Stage the per-crate CHANGELOG.md files and record them as ONE signed
# `chore(release): prepare release batch for <date>` checkpoint — the commit a
# maintainer makes AFTER `just changelog-generate` and a manual review, BEFORE
# running `cargo release --execute`. Captures the reviewed changelog state as a
# discrete, signed, revertable point in the release sequence (see
# docs/project/releasing.md §0).
#
# Assumes changelogs have already been generated and reviewed: this only stages
# `crates/*/CHANGELOG.md` and commits. If no changelog actually changed, `git
# commit` finds nothing staged and fails — that failure is framed (rather than a
# bare git error) so the operator knows to run `just changelog-generate` first.
#
# Extracted from the `release-commit-changelogs` Justfile recipe so the
# add+commit logic is a testable unit behind a GIT seam (matching prep-release.sh)
# and its result is framed by log.sh, closing the last inline-multi-line release
# recipe. The recipe now delegates here.
#
# POSIX sh only.
#
# Usage: scripts/release-commit-changelogs.sh
#
# Environment:
#   GIT   git invocation to use (default: git) — indirection for tests, which
#         stub it so no real (signed) commit is created.
#   DATE  command producing the checkpoint date (default: `date +%Y-%m-%d`) —
#         indirection so tests can assert a byte-exact commit message.
#
# Exit codes:
#   0 = changelogs staged and the signed checkpoint commit created
#   N = a delegated git step failed (its own exit code) — e.g. nothing staged
#       because no changelog changed, or the GPG signature failed
# ==============================================================================

set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/common.sh"

GIT="${GIT:-git}"
DATE="${DATE:-date +%Y-%m-%d}"

CHECKPOINT_DATE=$($DATE)

log_section "Committing reviewed changelogs as a signed checkpoint"

log_item "Staging crates/*/CHANGELOG.md" 1
"$GIT" add crates/*/CHANGELOG.md

log_item "Creating signed checkpoint commit" 1
"$GIT" commit --gpg-sign -m "chore(release): prepare release batch for ${CHECKPOINT_DATE}"

log_ok "release-commit-changelogs: changelogs committed as a signed checkpoint"
