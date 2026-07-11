#!/bin/sh
set -eu

# ==============================================================================
# scripts/ci/compare-registry-crate.sh
# Compares the SHA-256 of the locally packaged .crate (the attestation subject)
# against the .crate crates.io actually serves for the same name and version.
#
# publish.yml attests build provenance for, and attaches to a GitHub Release,
# the .crate that `cargo package` produced on the runner. This guard proves
# those exact bytes are the bytes the registry serves, so the attestation and
# the Release asset cannot describe an artefact that differs from what a
# consumer downloads from crates.io. It runs after the version is indexed and
# BEFORE the attestation step; a byte mismatch fails the run before anything is
# signed.
#
# The registry-served .crate can lag the sparse index by a short CDN
# propagation window, so a not-yet-served response is retried with linear
# backoff (mirroring wait-for-index.sh). That transient case is reported
# distinctly from a genuine SHA-256 MISMATCH, which never retries: a mismatch
# is a hard, immediate failure.
#
# Usage: scripts/ci/compare-registry-crate.sh <crate-name> <version> <local-crate-file>
#
# Exit codes:
#   0 = the served .crate is byte-identical to the local artefact
#   1 = bad arguments, SHA-256 MISMATCH, or the .crate was not served in time
# ==============================================================================

CRATE="${1:?usage: compare-registry-crate.sh <crate-name> <version> <local-crate-file>}"
VERSION="${2:?usage: compare-registry-crate.sh <crate-name> <version> <local-crate-file>}"
LOCAL_FILE="${3:?usage: compare-registry-crate.sh <crate-name> <version> <local-crate-file>}"

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/../lib/log.sh"

if [ ! -f "$LOCAL_FILE" ]; then
    log_err_ci "Local crate file not found: ${LOCAL_FILE}"
    exit 1
fi

# SHA-256 of the artefact we packaged and are about to attest. The hasher is
# overridable for tests (mirrors update-specs.sh).
LOCAL_SHA=$("${SHA256SUM:-sha256sum}" "$LOCAL_FILE" | cut -d' ' -f1)

# The registry-served download. static.crates.io is the CDN origin that the
# crates.io download endpoint redirects to; addressing it directly avoids a
# redirect hop while -L still follows one if the layout ever changes.
URL="https://static.crates.io/crates/${CRATE}/${CRATE}-${VERSION}.crate"

# Linear backoff (sleep = attempt * 10s), checking BEFORE each sleep: the same
# shape and budget as wait-for-index.sh. The sparse index is already confirmed
# by the time this runs, so this only covers the shorter CDN propagation
# window, but the generous budget keeps a transient lag from red-lighting a
# crate that already published. A byte MISMATCH is not a transient state and
# exits immediately below without consuming the retry budget.
MAX_RETRIES=16
ATTEMPT=0

log_section "Comparing ${CRATE} ${VERSION} against the registry-served .crate:"
log_item "local artefact: ${LOCAL_FILE}" 1
log_item "local SHA-256:  ${LOCAL_SHA}" 1

BODY=$(mktemp)
trap 'rm -f "$BODY"' EXIT

while true; do
    set +e
    STATUS=$(curl -sSL -o "$BODY" -w '%{http_code}' "$URL")
    rc=$?
    set -e

    if [ "$rc" -ne 0 ] || [ "$STATUS" = "000" ]; then
        log_info "Download of ${URL} failed (curl exit ${rc}, HTTP ${STATUS}); treating as transient."
    else
        case "$STATUS" in
            200)
                SERVED_SHA=$("${SHA256SUM:-sha256sum}" "$BODY" | cut -d' ' -f1)
                # Relies on `cargo package` normalising mtimes (byte-deterministic repack); otherwise a skipped-publish re-run would false-mismatch here.
                if [ "$SERVED_SHA" = "$LOCAL_SHA" ]; then
                    log_ok "compare-registry-crate: ${LOCAL_FILE} matches the registry-served .crate (${LOCAL_SHA})."
                    exit 0
                fi
                # Served, but the bytes differ: a hard, non-retryable failure.
                log_err_ci "SHA-256 MISMATCH for ${LOCAL_FILE}: the .crate crates.io serves for ${CRATE} ${VERSION} differs from the packaged artefact."
                log_err_detail "local  (attestation subject): ${LOCAL_SHA}"
                log_err_detail "served (crates.io):          ${SERVED_SHA}"
                exit 1
                ;;
            403|404)
                log_info "${CRATE} ${VERSION} is not yet served by the registry (HTTP ${STATUS})."
                ;;
            500|502|503|504)
                log_info "Transient server error HTTP ${STATUS} from ${URL}; will retry."
                ;;
            *)
                log_err_ci "Unexpected HTTP ${STATUS} from ${URL}; aborting."
                exit 1
                ;;
        esac
    fi

    ATTEMPT=$((ATTEMPT + 1))
    if [ "$ATTEMPT" -ge "$MAX_RETRIES" ]; then
        log_err_ci "${CRATE} ${VERSION} .crate was not served by the registry after ${MAX_RETRIES} attempts."
        exit 1
    fi
    SLEEP=$((ATTEMPT * 10))
    log_info "Attempt ${ATTEMPT}/${MAX_RETRIES}: waiting ${SLEEP}s before re-checking the served .crate."
    sleep "$SLEEP"
done
