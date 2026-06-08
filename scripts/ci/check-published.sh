#!/bin/sh
set -eu

# ==============================================================================
# scripts/ci/check-published.sh
# Determines whether an exact crate version is already published to crates.io,
# so publish.yml can skip re-publishing on a re-run (idempotent release).
#
# Queries the crates.io sparse index, which lists every published version of a
# crate as newline-delimited JSON (one object per version, with a `vers` field).
# `cargo search` is unsuitable here — it reports only the latest version, so it
# cannot confirm the presence of a specific older version.
#
# Usage: scripts/ci/check-published.sh <crate-name> <version>
#
# Output (to $GITHUB_OUTPUT, or stdout when run outside Actions):
#   exists=true   — <version> is already on crates.io
#   exists=false  — <version> is not yet published
#
# Exit codes:
#   0 = check completed (regardless of exists true/false)
#   1 = bad arguments, or the index responded with an unexpected HTTP status
# ==============================================================================

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/../lib/log.sh"

CRATE="${1:?usage: check-published.sh <crate-name> <version>}"
VERSION="${2:?usage: check-published.sh <crate-name> <version>}"

# Sparse-index path layout: for names >=4 chars, {first2}/{next2}/{name}.
# All workspace crates are "sdmx-*", i.e. >=4 chars, so no 1/2/3-char special
# casing is needed; derive it generally anyway to stay correct if names change.
len=${#CRATE}
case "$len" in
    1) path="1/${CRATE}" ;;
    2) path="2/${CRATE}" ;;
    3) path="3/$(printf '%s' "$CRATE" | cut -c1)/${CRATE}" ;;
    *) path="$(printf '%s' "$CRATE" | cut -c1-2)/$(printf '%s' "$CRATE" | cut -c3-4)/${CRATE}" ;;
esac

URL="https://index.crates.io/${path}"

BODY=$(mktemp)
trap 'rm -f "$BODY"' EXIT

STATUS=$(curl -sS -o "$BODY" -w '%{http_code}' "$URL") || status=$?

if [ "${status:-0}" -ne 0 ] || [ "$STATUS" = "000" ]; then
    log_err_ci "Curl request failed (exit: ${status:-0}, HTTP status: ${STATUS}). Treating as transient network error."
    exit 3
fi

emit() {
    if [ -n "${GITHUB_OUTPUT:-}" ]; then
        echo "exists=$1" >> "$GITHUB_OUTPUT"
    fi
    echo "exists=$1"
}

case "$STATUS" in
    404)
        # Crate not in the index at all — no versions published yet.
        log_info "${CRATE} not found in index (HTTP 404) — treating as unpublished."
        emit false
        exit 0
        ;;
    200) ;;
    400|401|403)
        log_err_ci "Permanent client error HTTP ${STATUS} from ${URL} — check configuration/auth."
        exit 2
        ;;
    500|502|503|504)
        log_err_ci "Transient server error HTTP ${STATUS} from ${URL} — will retry."
        exit 3
        ;;
    *)
        log_err_ci "Unexpected HTTP ${STATUS} from ${URL}"
        exit 1
        ;;
esac

# Exact-match the version field against the index entries. The index is
# newline-delimited JSON (one object per version); slurp into an array and use
# `any` for an unambiguous boolean exit status (a bare `select` under `jq -e`
# reflects only the last line's value, which is fragile).
if jq -s -e --arg v "$VERSION" 'any(.[]; .vers == $v)' "$BODY" > /dev/null 2>&1; then
    log_ok "${CRATE} ${VERSION} is already published — publish will be skipped."
    emit true
else
    log_info "${CRATE} ${VERSION} is not yet published — publish will proceed."
    emit false
fi
