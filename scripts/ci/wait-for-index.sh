#!/bin/sh
set -eu

# ==============================================================================
# scripts/ci/wait-for-index.sh
# Polls the crates.io sparse index until an exact crate version appears, with
# linear backoff. Run after `cargo publish` so downstream crates in the
# topological chain resolve the freshly published dependency.
#
# Uses the same exact-version sparse-index check as check-published.sh rather
# than `cargo search`, which reports only the latest version and so cannot
# confirm a specific version reliably.
#
# Usage: scripts/ci/wait-for-index.sh <crate-name> <version>
#
# Exit codes:
#   0 = version is indexed
#   1 = bad arguments, or version did not appear within the retry budget
# ==============================================================================

CRATE="${1:?usage: wait-for-index.sh <crate-name> <version>}"
VERSION="${2:?usage: wait-for-index.sh <crate-name> <version>}"

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

# shellcheck disable=SC1091
. "${SCRIPT_DIR}/../lib/log.sh"

# Linear backoff (sleep = attempt * 10s), checking BEFORE each sleep, so the
# actual wait budget is sum(10..(MAX_RETRIES-1)*10). At 16 that is
# 10+20+…+150 = 1200s ≈ 20 min — chosen to cover the upper end of historical
# crates.io indexing slowdowns (15–20 min during outages) so a transient lag
# does not red-light a crate that already published successfully. The common
# case still exits on attempt 1. Beyond this budget the step fails cleanly and
# the workflow is safely re-runnable: cargo publish already succeeded, so
# check-published.sh skips republish on the re-run, and the attestation and
# GitHub Release steps that follow this one in publish.yml complete then.
MAX_RETRIES=16
ATTEMPT=0

while true; do
    set +e
    out=$(env -u GITHUB_OUTPUT "${SCRIPT_DIR}/check-published.sh" "$CRATE" "$VERSION")
    rc=$?
    set -e

    if [ "$rc" -eq 0 ]; then
        if printf '%s\n' "$out" | grep -q '^exists=true$'; then
            break
        fi
        log_info "${CRATE} ${VERSION} is not yet indexed."
    elif [ "$rc" -eq 3 ]; then
        : # check-published.sh already emitted log_err_ci to stderr; nothing to add
    else
        log_err_ci "Permanent error (exit code ${rc}) from check-published.sh — aborting."
        exit 1
    fi

    ATTEMPT=$((ATTEMPT + 1))
    if [ "$ATTEMPT" -ge "$MAX_RETRIES" ]; then
        log_err_ci "${CRATE} ${VERSION} did not appear in the index after ${MAX_RETRIES} attempts."
        exit 1
    fi
    SLEEP=$((ATTEMPT * 10))
    log_info "Attempt ${ATTEMPT}/${MAX_RETRIES} — waiting ${SLEEP}s before next check."
    sleep "$SLEEP"
done

log_ok "${CRATE} ${VERSION} is indexed."
