#!/bin/sh
# ==============================================================================
# scripts/check-test-manifests.sh
# Verifies that BATS test fixture manifests stay in sync with the workspace.
#
# Currently checks: tests/bats/fixtures/update-msrv-manifest.txt
# Ensures every crates/*/Cargo.toml in the workspace is listed, so that adding
# a new crate doesn't silently exclude it from the MSRV test environment.
#
# Usage: scripts/check-test-manifests.sh
# Exit code: 0 = all manifests up-to-date, 1 = one or more files missing
# ==============================================================================

set -u

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

MANIFEST="tests/bats/fixtures/update-msrv-manifest.txt"

failed=0

for cargo_file in crates/*/Cargo.toml; do
    if ! grep -qF "$cargo_file" "$MANIFEST"; then
        log_err "$cargo_file missing from $MANIFEST"
        failed=1
    fi
done

if [ "$failed" -eq 0 ]; then
    log_ok "update-msrv: test fixture manifest up to date"
fi

exit "$failed"
