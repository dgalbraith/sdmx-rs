#!/bin/sh
set -e

# ==============================================================================
# scripts/check-changelog.sh
# Validates that crate CHANGELOG.md files are synchronized with git history.
# Uses git-cliff to generate expected changelogs and compares them to the committed files.
#
# Usage: scripts/check-changelog.sh [crate1 crate2 ...] or scripts/check-changelog.sh all
# If no arguments provided, checks all crates.
# ==============================================================================

# Source shared configuration
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/common.sh"

log_section "Changelog Synchronization Validation"

TEMP_FILE=$(mktemp)
trap 'rm -f "$TEMP_FILE"' EXIT

# Get crates to check (default: all)
CRATES_TO_CHECK=$(get_crates "$@")

for crate in $CRATES_TO_CHECK; do
    CHANGELOG_PATH="crates/${crate}/CHANGELOG.md"

    if [ ! -f "$CHANGELOG_PATH" ]; then
        log_fatal "${CHANGELOG_PATH} does not exist."
    fi

    # Guard against uncommitted edits to the changelog file itself — those would
    # skew the diff against the cliff-generated output, producing false results.
    # Unrelated working-tree changes are irrelevant to this check.
    if ! git diff --quiet HEAD -- "$CHANGELOG_PATH"; then
        log_fatal "${CHANGELOG_PATH} has uncommitted changes. Commit or stash before running check."
    fi

    # Generate what git-cliff thinks the changelog should be. Must mirror the
    # generation scoping in `just changelog-generate` exactly
    git-cliff --config cliff.toml --tag-pattern "^${crate}/v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$" --include-path "crates/${crate}/**" --output "$TEMP_FILE"

    # Compare to committed CHANGELOG.md
    if ! diff "$CHANGELOG_PATH" "$TEMP_FILE" > /dev/null; then
        log_fatal "${CHANGELOG_PATH} is out of sync with history. Run 'just changelog-generate' to sync."
    fi
done

log_ok "changelog: all crate changelogs synchronized with commit history"
exit 0
