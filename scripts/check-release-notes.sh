#!/bin/sh
# ==============================================================================
# scripts/check-release-notes.sh
#
# MANDATORY pre-tag gate: the facade crate (sdmx-rs) cannot be released without
# a curated, human-written release-notes file for the target version at
# crates/sdmx-rs/release-notes/<version>.md.
#
# WHY this exists (the model, see docs/design/0004 §9 and docs/project/
# releasing.md §1): the five per-crate CHANGELOG.md files are STRICT git-cliff
# output — the machine record — and are gated byte-for-byte by check-changelog.sh.
# That gate has no facade exemption, so the facade CHANGELOG.md cannot also be
# hand-curated (the two would fight: curation makes it diverge from git-cliff and
# check-changelog fails). The user-facing prose therefore lives in a SEPARATE,
# curated file per facade version, and THIS gate makes that file a precondition
# of cutting the release. create-release.sh then prefers it over the machine
# CHANGELOG section when materialising the facade's GitHub Release body.
#
# WHY it must run BEFORE `cargo release --execute` (the irreversibility argument):
# a GitHub Release attaches to a PUSHED tag ref, so it can only be created in CI
# after the tag exists — and a pushed tag triggers the irreversible crates.io
# publish. The guarantee "no facade release without curated prose" cannot be
# enforced at Release-creation time (too late — the tag is already public). It is
# enforced HERE, locally, pre-tag: this gate is wired into releasing.md §0 and
# folded into prepublish-check, with a CI backstop re-check in create-release.sh.
#
# Scope: facade ONLY. Leaf crates (sdmx-types/parsers/writers/client) do NOT
# require curated notes — their GitHub Release body is the auto CHANGELOG section
# (or a provenance placeholder when genuinely empty); per-crate provenance is the
# settled compliance decision in design-0004, so leaves are never gated on prose.
#
# A curated file must be NON-EMPTY (ignoring whitespace): an empty or
# whitespace-only file is treated as missing, so `touch`-ing the path cannot
# satisfy the gate.
#
# POSIX sh only.
#
# Usage: scripts/check-release-notes.sh <version>     (e.g. 0.2.0 or 1.0.0-rc.1)
#
# Environment:
#   FACADE_CRATE  facade crate name (default: sdmx-rs) — indirection for tests.
#
# Exit codes:
#   0 = curated notes for <version> exist, are non-empty, carry every required
#       section, and contain no surviving template guidance (i.e. actually curated)
#   1 = no <version> argument; OR the curated file is missing/empty; OR a required
#       section is absent; OR unedited template guidance sentinels remain
# ==============================================================================

set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

FACADE_CRATE="${FACADE_CRATE:-sdmx-rs}"

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    log_err "check-release-notes: no version given."
    log_err_detail "Usage: check-release-notes.sh <version>   (e.g. 0.2.0 or 1.0.0-rc.1)"
    exit 1
fi

NOTES_FILE="crates/${FACADE_CRATE}/release-notes/${VERSION}.md"

log_section "Checking curated facade release notes for ${VERSION}"

if [ ! -f "$NOTES_FILE" ]; then
    log_err "check-release-notes: required curated notes file is missing."
    log_err_detail "Expected: ${NOTES_FILE}"
    log_err_detail "The facade (${FACADE_CRATE}) GitHub Release body is driven by curated prose,"
    log_err_detail "not the machine CHANGELOG. Write user-facing notes for ${VERSION} there"
    log_err_detail "(breaking changes, new capabilities, notable dependency updates), then re-run."
    exit 1
fi

# Normalise line endings before any line-based check. A file edited on Windows (or
# under git `core.autocrlf=true`) carries CRLF, so a physical line reads "## X\r";
# the line-anchored `grep -xF "## X"` (LF) would then never match and every section
# would be reported missing. Strip CR once into a temp copy and run all checks
# against it — the committed file is untouched; we only normalise what we inspect.
CLEANED=$(mktemp)
trap 'rm -f "$CLEANED"' EXIT
tr -d '\r' < "$NOTES_FILE" > "$CLEANED"

# Non-empty test: strip all whitespace; if nothing remains, the file is a stub.
if [ -z "$(tr -d '[:space:]' < "$CLEANED")" ]; then
    log_err "check-release-notes: curated notes file exists but is empty."
    log_err_detail "File: ${NOTES_FILE}"
    log_err_detail "An empty file does not satisfy the gate — write the user-facing notes for ${VERSION}."
    exit 1
fi

# --- Required sections (structure) --------------------------------------------
# The curated file must carry EVERY section the template defines — a consumer
# making an upgrade decision must not have to guess whether an absent section
# means "nothing changed" or "the author forgot". Keep this list in sync with the
# headings in release-notes/templates/template.md (and create-release.sh's body).
REQUIRED_SECTIONS="## Breaking Changes & Migration
## Bug Fixes
## New Features & Enhancements
## Deprecations
## Minimum Supported Rust Version (MSRV)
## Feature Flags
## Security
## Dependency Updates
## Verifying Release Provenance"

missing=0
# Feed the loop via a here-doc (NOT a pipe) so `missing` is set in THIS shell, not
# a subshell — a pipe would lose the flag and let a gap slip through.
while IFS= read -r section; do
    [ -n "$section" ] || continue
    if ! grep -qxF "$section" "$CLEANED"; then
        log_err "check-release-notes: missing required section: ${section}"
        missing=1
    fi
done <<EOF
$REQUIRED_SECTIONS
EOF

if [ "$missing" -ne 0 ]; then
    log_err_detail "File: ${NOTES_FILE}"
    log_err_detail "Every template section must be present (state the negative if a section is empty);"
    log_err_detail "do not delete sections. Re-scaffold with 'just new-release-notes ${VERSION}' to compare."
    exit 1
fi

# --- Sentinel rejection (content) ---------------------------------------------
# A curated copy must contain NO surviving template guidance — neither the header
# "TEMPLATE GUIDANCE" block nor any per-section/theme "GUIDANCE:" comment. These
# are sentinel phrases the template guarantees (not arbitrary HTML comments, so a
# maintainer's own `<!-- note -->` is fine). Their presence means the file was
# scaffolded but not curated. Keep these sentinels in sync with template.md.
if grep -qF 'TEMPLATE GUIDANCE' "$CLEANED" || grep -qF 'GUIDANCE:' "$CLEANED"; then
    log_err "check-release-notes: unedited template guidance remains in ${NOTES_FILE}."
    log_err_detail "Curate every section — replace each 'GUIDANCE:' comment with real content"
    log_err_detail "(or its stated 'If none' line) and delete the 'TEMPLATE GUIDANCE' header block."
    log_err_detail "A scaffolded-but-uncurated file does not satisfy the gate."
    exit 1
fi

# --- MSRV literal check -------------------------------------------------------
# The curated file must carry a "* **Current MSRV**: `<version>`" line that
# matches the rust-version field in the crate's Cargo.toml exactly. The template
# is a convenience copy on a different branch and is not authoritative; the only
# source of truth is the Cargo.toml on the branch being released.
CARGO_TOML="crates/${FACADE_CRATE}/Cargo.toml"
if [ ! -f "$CARGO_TOML" ]; then
    log_err "check-release-notes: cannot locate ${CARGO_TOML} to verify MSRV."
    log_err_detail "Expected crate Cargo.toml at: ${CARGO_TOML}"
    exit 1
fi

ACTUAL_MSRV=$(grep '^rust-version' "$CARGO_TOML" | sed 's/.*"\(.*\)".*/\1/')
if [ -z "$ACTUAL_MSRV" ]; then
    log_err "check-release-notes: no rust-version field found in ${CARGO_TOML}."
    exit 1
fi

EXPECTED_MSRV_LINE="* **Current MSRV**: \`${ACTUAL_MSRV}\`"
if ! grep -qxF "$EXPECTED_MSRV_LINE" "$CLEANED"; then
    log_err "check-release-notes: MSRV literal in ${NOTES_FILE} does not match ${CARGO_TOML}."
    log_err_detail "Expected line (from ${CARGO_TOML} rust-version): ${EXPECTED_MSRV_LINE}"
    log_err_detail "Update the '* **Current MSRV**' line in the curated notes to match."
    exit 1
fi

log_ok "check-release-notes: curated facade notes present, complete, and curated for ${VERSION} (${NOTES_FILE})"
