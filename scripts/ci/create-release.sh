#!/bin/sh
set -eu

# ==============================================================================
# scripts/ci/create-release.sh
# Creates (or, on a re-run, updates) the GitHub Release for a crate tag and
# attaches the given asset files. Idempotent: if the release already exists,
# its notes are refreshed and assets are re-uploaded with --clobber, so a
# re-triggered publish run does not fail on "release already exists".
#
# Release-notes source depends on the crate (design-0004 §9):
#
#   FACADE (sdmx-rs): the body is the CURATED prose at
#     crates/sdmx-rs/release-notes/<version>.md — human-written, user-facing.
#     This file is a MANDATORY pre-tag gate (check-release-notes.sh / folded into
#     prepublish-check), so by the time this CI step runs it should exist; the
#     machine CHANGELOG section is only a BACKSTOP if it somehow does not. An
#     empty facade body is still fatal (a facade release must say something).
#
#   LEAF (sdmx-types/parsers/writers/client): the body is the auto CHANGELOG.md
#     section for the version. When that section is genuinely empty (a pre-1.0
#     lockstep no-op crate), the Release is NOT failed — instead a provenance
#     placeholder is emitted: the leaf Release exists to host that crate's own
#     SLSA + SBOM attestations and .crate asset (the settled per-crate-provenance
#     decision), so it must be createable even with no user-facing changes. The
#     placeholder names it as a provenance container and points at the crate's
#     CHANGELOG. This must fire ONLY on a truly-empty section, so post-1.0 leaves
#     releasing independently with real notes are unaffected.
#
# Release TITLE is a plain "<crate> v<version>".
#
# crates.io publishing is the irreversible step; the GitHub Release is a
# presentation layer, hence safe to overwrite.
#
# Usage: scripts/ci/create-release.sh <crate-name> <version> <asset>...
#
# Requires: GH_TOKEN in the environment (GitHub CLI auth).
#
# Environment:
#   FACADE_CRATE  facade crate name (default: sdmx-rs) — indirection for tests.
#
# Exit codes:
#   0 = release created or updated
#   1 = bad arguments, missing curated facade notes with no CHANGELOG backstop,
#       or an empty facade body
# ==============================================================================

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/../lib/log.sh"

CRATE="${1:?usage: create-release.sh <crate-name> <version> <asset>...}"
VERSION="${2:?usage: create-release.sh <crate-name> <version> <asset>...}"
shift 2
# Remaining args are asset paths.
if [ "$#" -eq 0 ]; then
    log_err_ci "No release assets supplied."
    exit 1
fi

FACADE_CRATE="${FACADE_CRATE:-sdmx-rs}"
TAG="${CRATE}/v${VERSION}"
CHANGELOG="crates/${CRATE}/CHANGELOG.md"
CURATED="crates/${CRATE}/release-notes/${VERSION}.md"

if [ ! -f "$CHANGELOG" ]; then
    log_err_ci "${CHANGELOG} not found."
    exit 1
fi

# Extract the CHANGELOG section under "## [<version>]" up to the next "## " header.
# Match the header as a LITERAL string, not a regex: a semver version contains `.`
# (and, for pre-releases like 1.0.0-alpha.1, `-` and further `.`) — all regex
# metacharacters. Building the target header in awk and comparing with index()==1
# avoids any metacharacter interpretation, so e.g. `1.0.0` cannot loosely match
# `1x0x0`. This is the machine-record body (and the leaf source / facade backstop).
CHANGELOG_NOTES=$(awk -v ver="$VERSION" '
    BEGIN { header = "## [" ver "]" }
    index($0, header) == 1 { found = 1; next }
    found && /^## / { exit }
    found { print }
' "$CHANGELOG")

# Is the changelog section effectively empty (whitespace only)?
changelog_empty=0
[ -z "$(printf '%s' "$CHANGELOG_NOTES" | tr -d '[:space:]')" ] && changelog_empty=1

# --- Resolve the release-notes body per crate kind (design-0004 §9) -----------
if [ "$CRATE" = "$FACADE_CRATE" ]; then
    # FACADE: curated prose drives the body; CHANGELOG is only a backstop.
    if [ -f "$CURATED" ] && [ -n "$(tr -d '[:space:]' < "$CURATED")" ]; then
        log_info "Facade body: curated ${CURATED}."
        NOTES=$(cat "$CURATED")
    elif [ "$changelog_empty" -eq 0 ]; then
        # Curated file absent/empty in CI, but the pre-tag gate should have caught
        # that. Fall back to the machine section rather than ship an empty body.
        log_warn "Curated ${CURATED} missing/empty — falling back to CHANGELOG section. The check-release-notes gate should have blocked this pre-tag."
        NOTES="$CHANGELOG_NOTES"
    else
        # No curated prose AND no changelog content: a facade release must say
        # something. Fail closed.
        log_err_ci "No facade release notes for ${CRATE} ${VERSION}: curated ${CURATED} is missing/empty and the CHANGELOG '## [${VERSION}]' section is empty."
        exit 1
    fi
else
    # LEAF: CHANGELOG section, or a provenance placeholder when genuinely empty.
    if [ "$changelog_empty" -eq 0 ]; then
        NOTES="$CHANGELOG_NOTES"
    else
        log_info "Leaf ${CRATE} ${VERSION} has an empty changelog section — emitting provenance placeholder."
        # Placeholder: name the Release as a provenance container and point at the
        # crate's own CHANGELOG. Emit the facade-batch link unconditionally (the
        # facade release is guaranteed to exist by the time the whole publish
        # run completes, and absolute URLs avoid relative path depth issues).
        facade_line="

See the [\`${FACADE_CRATE}\` v${VERSION} release notes](https://github.com/dgalbraith/sdmx-rs/releases/tag/${FACADE_CRATE}/v${VERSION}) for the user-facing summary of this batch."
        NOTES="No user-facing changes in this release.

This GitHub Release exists to host the build-provenance and SBOM attestations and the packaged \`.crate\` for \`${CRATE}\` v${VERSION}. See the [\`${CRATE}\` CHANGELOG](https://github.com/dgalbraith/sdmx-rs/blob/${CRATE}/v${VERSION}/crates/${CRATE}/CHANGELOG.md) for the full machine-generated history.${facade_line}"
    fi
fi

if gh release view "$TAG" >/dev/null 2>&1; then
    log_info "Release ${TAG} already exists — updating notes and assets."
    printf '%s' "$NOTES" | gh release edit "$TAG" \
        --title "${CRATE} v${VERSION}" \
        --notes-file -
    gh release upload "$TAG" "$@" --clobber
else
    log_info "Creating release ${TAG}."
    printf '%s' "$NOTES" | gh release create "$TAG" \
        --title "${CRATE} v${VERSION}" \
        --notes-file - \
        "$@"
fi

log_ok "Release ${TAG} is up to date."
