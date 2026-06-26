#!/bin/sh
# ==============================================================================
# fetch-specs.sh
#
# VERIFY driver for fetch-on-demand SDMX schemas (the read-only
# sibling of update-specs.sh). Materialises the pinned schema tree from the Nix
# fixed-output derivation and re-checks every file's sha256 against
# specs/sources.toml. Idempotent: a re-run with the tree already present and the
# pin unchanged is a no-op, never a re-fetch.
#
# This is the "built once, left in place" entry point: the same fetch ->
# materialise -> sha-verify chain that the gated CI jobs run, so local mirrors CI
# 1:1. A thin POSIX-sh wrapper over `nix build .#sdmxSpecs`.
#
#   SDMX_SPECS_DIR  where to materialise (default: <repo>/specs). During the
#                   migration the bootstrap points this at a TEMP dir so the
#                   still-tracked specs/ tree is never overwritten.
#
# Overridable for tests: NIX, SHA256SUM, SPECS_FLAKE (see lib/specs-fetch.sh).
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/specs-fetch.sh"

ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
SPECS_SOURCES="${SPECS_SOURCES:-$ROOT/specs/sources.toml}"
SDMX_SPECS_DIR="${SDMX_SPECS_DIR:-$ROOT/specs}"
SPECS_FLAKE="${SPECS_FLAKE:-$ROOT}"
export SPECS_SOURCES SPECS_FLAKE
STAMP="$SDMX_SPECS_DIR/.sha256.stamp"

if [ ! -f "$SPECS_SOURCES" ]; then
    log_fatal "fetch-specs: pin file not found: ${SPECS_SOURCES} (run update-specs first)"
fi

WANT=$(specs_stamp_value)

# 1. Fast path: this exact pin is already materialised here. Require the tree
#    to be present, not just the stamp: a rebase across the untrack commit
#    deletes the once-tracked .xsd while the gitignored stamp survives, and a
#    stamp-only check would then wedge a false no-op over an empty tree.
if [ -f "$STAMP" ] && [ "$(cat "$STAMP")" = "$WANT" ] && specs_present "$SDMX_SPECS_DIR"; then
    log_ok "fetch-specs: schemas already materialised for the current pin (idempotent no-op)"
    exit 0
fi

# 2. Present and sha-valid but unstamped (e.g. a fresh tree from another path):
#    re-stamp, no fetch.
if specs_verify "$SDMX_SPECS_DIR" 2>/dev/null; then
    printf '%s\n' "$WANT" > "$STAMP"
    log_ok "fetch-specs: schemas present and verified"
    exit 0
fi

# 3. Fetch from the pinned upstream (Nix FOD), then verify per-file sha256.
log_info "Materialising pinned SDMX schemas into ${SDMX_SPECS_DIR#"$ROOT"/} ..."
if ! specs_build "$SDMX_SPECS_DIR"; then
    log_fatal "fetch-specs: 'nix build .#sdmxSpecs' failed (network, or a stale pin in sources.toml)"
fi
if ! specs_verify "$SDMX_SPECS_DIR"; then
    log_fatal "fetch-specs: materialised tree failed sha256 verification against sources.toml"
fi
printf '%s\n' "$WANT" > "$STAMP"

_count=$(find "$SDMX_SPECS_DIR" -name '*.xsd' | wc -l | tr -d ' ')
log_ok "fetch-specs: $_count schema files materialised and verified"
