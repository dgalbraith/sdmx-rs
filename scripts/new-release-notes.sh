#!/bin/sh
# ==============================================================================
# scripts/new-release-notes.sh
#
# Scaffold a curated facade release-notes file for <version> from the template,
# so the maintainer starts from the structured form (9 sections, empty-state
# conventions) rather than a blank file. The facade GitHub Release body is driven
# by this curated file (design-0004 §9); a curated copy is a mandatory pre-tag
# gate (check-release-notes.sh).
#
# Copies crates/sdmx-rs/release-notes/templates/template.md to
# crates/sdmx-rs/release-notes/<version>.md. The curated file leads with summary
# prose; the GitHub Release title is a plain "sdmx-rs v<version>".
#
# Refuses to overwrite an existing <version>.md — a curated file must never be
# clobbered by a re-scaffold.
#
# This only SCAFFOLDS; it does not satisfy the gate. A fresh scaffold still
# carries the template guidance, which check-release-notes rejects until curated.
#
# POSIX sh only.
#
# Usage: scripts/new-release-notes.sh <version>
#   e.g. scripts/new-release-notes.sh 0.2.0
#
# Environment:
#   FACADE_CRATE  facade crate name (default: sdmx-rs) — indirection for tests.
#
# Exit codes:
#   0 = scaffolded
#   1 = no <version>, template missing, or <version>.md already exists
# ==============================================================================

set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

FACADE_CRATE="${FACADE_CRATE:-sdmx-rs}"

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    log_err "new-release-notes: no version given."
    log_err_detail "Usage: new-release-notes.sh <version>"
    exit 1
fi

NOTES_DIR="crates/${FACADE_CRATE}/release-notes"
TEMPLATE="${NOTES_DIR}/templates/template.md"
TARGET="${NOTES_DIR}/${VERSION}.md"

if [ ! -f "$TEMPLATE" ]; then
    log_err "new-release-notes: template not found at ${TEMPLATE}."
    exit 1
fi

if [ -e "$TARGET" ]; then
    log_err "new-release-notes: ${TARGET} already exists — refusing to overwrite a curated file."
    log_err_detail "Edit it directly, or remove it first if you really mean to start over."
    exit 1
fi

log_section "Scaffolding facade release notes for ${VERSION}"

cp "$TEMPLATE" "$TARGET"

log_ok "new-release-notes: scaffolded ${TARGET}"
log_hint "Curate every section (remove the template guidance), then: just check-release-notes ${VERSION}"
