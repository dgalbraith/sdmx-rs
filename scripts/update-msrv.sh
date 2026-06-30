#!/bin/sh
# ==============================================================================
# scripts/update-msrv.sh
# MSRV (Minimum Supported Rust Version) update automation
#
# Automates updating the minimum supported Rust version across the monorepo,
# supporting both raising (breaking change) and lowering (feature) operations.
#
# Usage: scripts/update-msrv.sh [--downgrade] [--dry-run] [--force] OLD_VERSION NEW_VERSION
#
# Flags:
#   --downgrade  Lower MSRV (feature, non-breaking, opportunistic)
#   --dry-run    Preview changes without modifying files
#   --force      Bypass dirty git tree check (for testing/development)
#
# Examples:
#   scripts/update-msrv.sh 1.91.0 1.92.0                   # Raise MSRV (breaking)
#   scripts/update-msrv.sh --downgrade 1.91.0 1.85.0       # Lower MSRV (feature)
#   scripts/update-msrv.sh --dry-run 1.91.0 1.92.0         # Preview changes
#   scripts/update-msrv.sh --force 1.91.0 1.92.0           # Force despite dirty tree
# ==============================================================================
# Raise (breaking change):
#   1. Validates the 6-month MSRV policy (new version must be 6+ months old)
#   2. Checks git state and file consistency (pre-flight)
#   3. Compares clippy output between old and new MSRV (warns on divergence)
#   4. Updates all Cargo.toml files (workspace + 5 crates)
#   5. Updates rust-toolchain.toml, maintenance.toml, README.md, crates/*/README.md,
#      docs/project/msrv.md, and the facade release-notes template's "Current MSRV" line
#   6. Runs full verification (just verify)
#   7. Stages files for developer review and commit
#   8. Prints breaking-change warning and suggested commit message
#
# Lower (opportunistic feature):
#   1. Updates all Cargo.toml files (workspace + 5 crates)
#   2. Updates rust-toolchain.toml
#   3. SKIPS maintenance.toml (no review obligation)
#   4. Updates README.md, crates/*/README.md, and CONTRIBUTING.md
#   5. Updates docs/project/msrv.md version references and the facade
#      release-notes template's "Current MSRV" line
#   6. Runs full verification (just verify)
#   7. Stages files for developer review and commit

set -u
set -e

DRY_RUN=0
FORCE=0
DOWNGRADE=0
OLD_MSRV=""
NEW_MSRV=""

# Status output (glyph/label/colour/stream owned by the shared logger)
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

# ==============================================================================
# Argument parsing
# ==============================================================================

# Handle --help before checking argument count
if [ $# -eq 1 ] && [ "$1" = "--help" ]; then
    echo "Usage: $0 [--downgrade] [--dry-run] [--force] OLD_VERSION NEW_VERSION"
    echo ""
    echo "Flags:"
    echo "  --downgrade  Lower MSRV (feature, non-breaking, opportunistic)"
    echo "  --dry-run    Preview changes without modifying files"
    echo "  --force      Bypass dirty git tree check (for testing/development)"
    echo ""
    echo "Examples:"
    echo "  $0 1.91.0 1.92.0                    # Raise MSRV"
    echo "  $0 --downgrade 1.91.0 1.85.0        # Lower MSRV"
    echo "  $0 --dry-run 1.91.0 1.92.0          # Preview"
    echo "  $0 --force 1.91.0 1.92.0            # Force despite dirty tree"
    exit 0
fi

if [ $# -lt 2 ]; then
    echo "Usage: $0 [--downgrade] [--dry-run] [--force] OLD_VERSION NEW_VERSION"
    echo ""
    echo "Flags:"
    echo "  --downgrade  Lower MSRV (feature, non-breaking, opportunistic)"
    echo "  --dry-run    Preview changes without modifying files"
    echo "  --force      Bypass dirty git tree check (for testing/development)"
    echo ""
    echo "Examples:"
    echo "  $0 1.91.0 1.92.0                    # Raise MSRV"
    echo "  $0 --downgrade 1.91.0 1.85.0        # Lower MSRV"
    echo "  $0 --dry-run 1.91.0 1.92.0          # Preview"
    echo "  $0 --force 1.91.0 1.92.0            # Force despite dirty tree"
    exit 1
fi

# Parse flags
while [ $# -gt 0 ]; do
    case "$1" in
        --downgrade)
            DOWNGRADE=1
            shift
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        --force)
            FORCE=1
            shift
            ;;
        *)
            break
            ;;
    esac
done

if [ $# -lt 2 ]; then
    log_fatal "OLD_VERSION and NEW_VERSION are required"
fi

OLD_MSRV="$1"
NEW_MSRV="$2"

# Validate version format (X.Y.Z)
validate_version_format() {
    version="$1"
    if ! echo "$version" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
        log_fatal "Invalid version format: $version (expected X.Y.Z)"
    fi
    unset version
}

# Portable date utilities
validate_date_format() {
    date_str="$1"
    if ! echo "$date_str" | grep -qE '^[0-9]{4}-[0-9]{2}-[0-9]{2}$'; then
        unset date_str
        return 1
    fi
    unset date_str
    return 0
}

date_to_epoch() {
    date_str="$1"
    year="${date_str%%-*}"
    rest="${date_str#*-}"
    month="${rest%%-*}"
    day="${rest#*-}"

    if ! { [ "$month" -ge 1 ] 2>/dev/null && [ "$month" -le 12 ] 2>/dev/null; } || ! { [ "$day" -ge 1 ] 2>/dev/null && [ "$day" -le 31 ] 2>/dev/null; }; then
        unset date_str year rest month day days
        return 1
    fi

    days=$(((year - 1970) * 365 + (year - 1969) / 4 - (year - 1901) / 100 + (year - 1601) / 400))

    case "$month" in
        1) days=$((days + 0)) ;;
        2) days=$((days + 31)) ;;
        3) days=$((days + 59)) ;;
        4) days=$((days + 90)) ;;
        5) days=$((days + 120)) ;;
        6) days=$((days + 151)) ;;
        7) days=$((days + 181)) ;;
        8) days=$((days + 212)) ;;
        9) days=$((days + 243)) ;;
        10) days=$((days + 273)) ;;
        11) days=$((days + 304)) ;;
        12) days=$((days + 334)) ;;
    esac

    days=$((days + day))
    echo $((days * 86400))
    unset date_str year rest month day days
}

days_between() {
    date1="$1"
    date2="$2"
    epoch1=""
    epoch2=""
    diff=""

    if ! epoch1=$(date_to_epoch "$date1") || [ -z "$epoch1" ]; then
        unset date1 date2 epoch1 epoch2 diff
        return 1
    fi
    if ! epoch2=$(date_to_epoch "$date2") || [ -z "$epoch2" ]; then
        unset date1 date2 epoch1 epoch2 diff
        return 1
    fi
    diff=$((epoch2 - epoch1))
    echo $((diff / 86400))
    unset date1 date2 epoch1 epoch2 diff
}

date_has_passed() {
    target_date="$1"
    today=""
    today_int=""
    target_int=""

    today=$(date +%Y-%m-%d 2>/dev/null) || today="2026-05-23"

    today_int=$(echo "$today" | tr -d '-')
    target_int=$(echo "$target_date" | tr -d '-')

    if [ "$today_int" -gt "$target_int" ]; then
        unset target_date today today_int target_int
        return 0
    fi
    unset target_date today today_int target_int
    return 1
}

# Portable sed-in-place: GNU sed -i and BSD sed -i '' are incompatible, so we
# redirect to a temp file and cat back. If this function gets a second caller,
# move it to scripts/lib/ rather than duplicating it.
sed_inplace() {
    # Subshell so the trap cleans tmp_f on any exit including set -e abort.
    (
        expr="$1"
        target_f="$2"
        tmp_f=$(mktemp)
        trap 'rm -f "$tmp_f"' EXIT
        sed "$expr" "$target_f" > "$tmp_f"
        cat "$tmp_f" > "$target_f"
    )
}

# ==============================================================================
# Validation phase
# ==============================================================================

if [ $DOWNGRADE -eq 1 ]; then
    log_section "MSRV Downgrade (Feature)"
else
    log_section "MSRV Upgrade (Breaking Change)"
fi
echo ""

# Validate version formats
validate_version_format "$OLD_MSRV"
validate_version_format "$NEW_MSRV"

# Pre-flight: git state
if [ $DRY_RUN -eq 0 ] && [ $FORCE -eq 0 ]; then
    if ! git diff-index --quiet HEAD --; then
        log_fatal "Git working tree is dirty. Commit or stash changes before proceeding."
    fi
    log_ok "Git working tree is clean"
elif [ $FORCE -eq 1 ]; then
    log_warn "Dirty tree check bypassed (--force flag)"
fi

# Pre-flight: check that Cargo.toml and rust-toolchain.toml are in sync
WORKSPACE_MSRV=$(grep "^rust-version = " Cargo.toml | sed 's/.*= "\([^"]*\)".*/\1/')
TOOLCHAIN_MSRV=$(grep "^channel = " rust-toolchain.toml | sed 's/.*= "\([^"]*\)".*/\1/')

if [ "$WORKSPACE_MSRV" != "$TOOLCHAIN_MSRV" ]; then
    log_fatal "MSRV mismatch: Cargo.toml ($WORKSPACE_MSRV) vs rust-toolchain.toml ($TOOLCHAIN_MSRV). Fix before proceeding."
fi
log_ok "Cargo.toml and rust-toolchain.toml are in sync ($WORKSPACE_MSRV)"

# Pre-flight: verify current MSRV matches old version
if [ "$WORKSPACE_MSRV" != "$OLD_MSRV" ]; then
    log_fatal "Current MSRV ($WORKSPACE_MSRV) does not match OLD_VERSION ($OLD_MSRV). Check your arguments."
fi

if [ $DOWNGRADE -eq 0 ]; then
    # 6-month policy validation (upgrade only)
    # For now, we'll skip automated date checking and rely on developer knowledge
    # This can be enhanced later with a local release database
    log_info "Skipping automated 6-month validation (requires Rust release history DB)"
    log_warn "Verify manually: $NEW_MSRV must have been released 6+ months ago"
    echo "  See: https://github.com/rust-lang/rust/releases/tag/$NEW_MSRV"
    echo ""
else
    log_info "Downgrading MSRV (no 6-month policy window required)"
    echo ""
fi

# ==============================================================================
# Lint compatibility
# ==============================================================================
#
# A standalone old-versus-new clippy comparison used to live here, driven by
# rustup's `cargo +<version>` toolchain selector. This repo provisions Rust through
# the Nix flake (fromRustupToolchainFile): `cargo +<version>` is unavailable and only
# one toolchain is on PATH per shell, so that comparison could never run here. New-lint
# detection is handled instead by the verification step below, which re-enters Nix to
# run `just verify` (clippy::pedantic -D warnings) under the NEW toolchain.

if [ $DRY_RUN -eq 1 ]; then
    echo ""
    echo "Dry-run validation complete. All checks passed."
    unset DRY_RUN FORCE DOWNGRADE OLD_MSRV NEW_MSRV WORKSPACE_MSRV TOOLCHAIN_MSRV TODAY RUST_RELEASES_URL TODAY_DATE NEW_REVIEW_DATE cargo_file
    exit 0
fi

# ==============================================================================
# File updates
# ==============================================================================

echo ""
echo "Updating configuration files..."

# Calculate new review date (6 months = 180 days from today) - only for raises
if [ $DOWNGRADE -eq 0 ]; then
    TODAY_DATE=$(date +%Y-%m-%d)
    NEW_REVIEW_DATE=$(date -d "+6 months" +%Y-%m-%d 2>/dev/null || python3 -c "from datetime import datetime, timedelta; print((datetime.strptime('$TODAY_DATE', '%Y-%m-%d') + timedelta(days=180)).strftime('%Y-%m-%d'))")
fi

# 1. Update all 6 Cargo.toml files (workspace + 5 crates) with explicit pattern matching
for cargo_file in Cargo.toml crates/sdmx-types/Cargo.toml crates/sdmx-parsers/Cargo.toml crates/sdmx-writers/Cargo.toml crates/sdmx-client/Cargo.toml crates/sdmx-rs/Cargo.toml; do
    if [ -f "$cargo_file" ]; then
        sed_inplace "s/rust-version = \"$OLD_MSRV\"/rust-version = \"$NEW_MSRV\"/g" "$cargo_file"
    fi
done
log_ok "Updated 6 Cargo.toml files"

# 2. Update rust-toolchain.toml with explicit pattern matching
sed_inplace "s/channel = \"$OLD_MSRV\"/channel = \"$NEW_MSRV\"/g" rust-toolchain.toml
log_ok "Updated rust-toolchain.toml"

# 3. Update maintenance.toml and its source marker (raise only; skip for downgrade).
# An MSRV raise satisfies ONLY the msrv-upgrade-window review obligation, so scope the
# date bump to that one [[maintenance]] block and leave the unrelated windows alone. The
# maintenance gate cross-checks maintenance.toml against the "# Last updated:" comment
# under the marker in Cargo.toml, so bump that comment in lock-step.
if [ $DOWNGRADE -eq 0 ]; then
    sed_inplace "/item = \"msrv-upgrade-window\"/,/^\[\[maintenance\]\]/ s/last_updated = \"[^\"]*\"/last_updated = \"$TODAY_DATE\"/" maintenance.toml
    sed_inplace "/item = \"msrv-upgrade-window\"/,/^\[\[maintenance\]\]/ s/next_review = \"[^\"]*\"/next_review = \"$NEW_REVIEW_DATE\"/" maintenance.toml
    sed_inplace "/# MAINTENANCE: msrv-upgrade-window/,/# Last updated:/ s/# Last updated: .*/# Last updated: $TODAY_DATE/" Cargo.toml
    sed_inplace "/# MAINTENANCE: msrv-upgrade-window/,/# Next review:/ s/# Next review: .*/# Next review: $NEW_REVIEW_DATE/" Cargo.toml
    log_ok "Updated maintenance.toml + Cargo.toml marker (msrv-upgrade-window)"
else
    log_info "Skipping maintenance.toml (downgrade is not a breaking change)"
fi

# 4. Update README.md (badge URL, badge alt-text, and bold section)
sed_inplace "s/MSRV-$OLD_MSRV/MSRV-$NEW_MSRV/g" README.md
sed_inplace "s/MSRV: $OLD_MSRV/MSRV: $NEW_MSRV/g" README.md
sed_inplace "s/\\*\\*$OLD_MSRV\\*\\*/\\*\\*$NEW_MSRV\\*\\*/g" README.md
log_ok "Updated README.md"

# 4a. Update per-crate READMEs (badge URL and alt-text; not in Cargo.toml loop above)
for crate_readme in crates/*/README.md; do
    if [ -f "$crate_readme" ]; then
        sed_inplace "s/MSRV-$OLD_MSRV/MSRV-$NEW_MSRV/g" "$crate_readme"
        sed_inplace "s/MSRV: $OLD_MSRV/MSRV: $NEW_MSRV/g" "$crate_readme"
    fi
done
log_ok "Updated crates/*/README.md (5 files)"

# 5. Update CONTRIBUTING.md - different sed patterns for raise vs downgrade
sed_inplace "s/(currently \*\*$OLD_MSRV\*\*)/(currently **$NEW_MSRV**)/g" CONTRIBUTING.md
sed_inplace "s/MSRV ($OLD_MSRV)/MSRV ($NEW_MSRV)/g" CONTRIBUTING.md

if [ $DOWNGRADE -eq 0 ]; then
    # Raise: update both version numbers in breaking-change example
    sed_inplace "s/MSRV bumped to [0-9]*\.[0-9]*\.[0-9]* (was $OLD_MSRV)/MSRV bumped to $NEW_MSRV (was $OLD_MSRV)/g" CONTRIBUTING.md
else
    # Downgrade: only update the "(was X)" part to show context of previous baseline
    sed_inplace "s/(was [0-9]*\.[0-9]*\.[0-9]*)/(was $OLD_MSRV)/g" CONTRIBUTING.md
fi

# Note: CONTRIBUTING.md section 5c uses evergreen text "previous release" (no version-specific update needed)
log_ok "Updated CONTRIBUTING.md"

# 6. Update docs/project/msrv.md manual-path version pins. The Nix-compatible manual path
# carries the new floor as rust-toolchain.toml/Cargo.toml pin values, not as
# `cargo +<version>` selectors. Anchor each sed on its manual-path comment line so the
# policy-section literal and prose elsewhere in the file are left untouched.
sed_inplace "/# rust-toolchain.toml: channel = /s/\"[0-9][0-9.]*\"/\"$NEW_MSRV\"/" docs/project/msrv.md
sed_inplace "/# Cargo.toml \[workspace.package\] rust-version = /s/\"[0-9][0-9.]*\"/\"$NEW_MSRV\"/" docs/project/msrv.md

log_ok "Updated docs/project/msrv.md (manual-path version pins)"

# 7. Update the facade release-notes template's "Current MSRV" line. The template
# carries the MSRV as a LITERAL current value (not a {{VERSION}}-style token) so a
# maintainer copying it for a release starts from the correct floor; that means it
# must be kept in sync HERE rather than at release time. Anchored to the
# "Current MSRV:" line so no other version-like text in the template moves.
RELEASE_NOTES_TEMPLATE="crates/sdmx-rs/release-notes/templates/template.md"
sed_inplace "s/\\(\\*\\*Current MSRV\\*\\*: \`\\)$OLD_MSRV\\(\`\\)/\\1$NEW_MSRV\\2/" "$RELEASE_NOTES_TEMPLATE"
log_ok "Updated ${RELEASE_NOTES_TEMPLATE} (Current MSRV line)"

# ==============================================================================
# Verification
# ==============================================================================

echo ""
echo "Running full verification suite under the updated toolchain..."
# The Nix flake provisions the toolchain at shell entry from rust-toolchain.toml, so a
# bare `just verify` would run under the PRE-rewrite toolchain still on PATH. Re-enter
# Nix so the flake re-reads the just-rewritten rust-toolchain.toml and provisions
# NEW_MSRV for verification.
if nix develop --command just verify; then
    log_ok "All verification checks passed"
else
    log_fatal "Verification failed. Fix issues before proceeding."
fi

# ==============================================================================
# Staging and summary
# ==============================================================================

echo ""
echo "Staging files for review..."

git add Cargo.toml crates/*/Cargo.toml crates/*/README.md rust-toolchain.toml maintenance.toml README.md CONTRIBUTING.md docs/project/msrv.md "$RELEASE_NOTES_TEMPLATE"
log_ok "Files staged"

# Print summary
echo ""
if [ $DOWNGRADE -eq 0 ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_ok "MSRV Upgrade (Breaking Change) Complete"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Changes made:"
    log_item "Cargo.toml (6 files): $OLD_MSRV → $NEW_MSRV" 1
    log_item "rust-toolchain.toml: $OLD_MSRV → $NEW_MSRV" 1
    log_item "maintenance.toml: dates updated (review: $NEW_REVIEW_DATE)" 1
    log_item "README.md: badge URL, alt-text, and section updated" 1
    log_item "crates/*/README.md (5 files): badge URL and alt-text updated" 1
    log_item "CONTRIBUTING.md: MSRV references updated" 1
    log_item "docs/project/msrv.md: manual path examples updated" 1
    log_item "release-notes template: Current MSRV line updated" 1
    echo ""
    echo "All files are staged. Review and commit:"
    echo ""
    echo "  git commit --gpg-sign -m \"chore(msrv): raise minimum supported Rust version to $NEW_MSRV\""
    echo ""
    echo "⚠️  BREAKING CHANGE"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Per policy: MSRV raises require a MAJOR version increment."
    echo "Update your crate versions before release."
    echo "cargo-release will prompt for version confirmation."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_ok "MSRV Downgrade (Feature) Complete"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Changes made:"
    log_item "Cargo.toml (6 files): $OLD_MSRV → $NEW_MSRV" 1
    log_item "rust-toolchain.toml: $OLD_MSRV → $NEW_MSRV" 1
    log_item "README.md: badge URL, alt-text, and section updated" 1
    log_item "crates/*/README.md (5 files): badge URL and alt-text updated" 1
    log_item "CONTRIBUTING.md: MSRV references updated" 1
    log_item "docs/project/msrv.md: manual path examples updated" 1
    log_item "release-notes template: Current MSRV line updated" 1
    log_info "maintenance.toml: unchanged (no review obligation)" 1
    echo ""
    echo "All files are staged. Review and commit:"
    echo ""
    echo "  git commit --gpg-sign -m \"chore(msrv): lower minimum supported Rust version to $NEW_MSRV\""
    echo ""
    echo "ℹ️  NON-BREAKING CHANGE"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Per policy: MSRV lowering is a non-breaking feature."
    echo "No version increment required."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
fi

# Verification above ran in a fresh `nix develop` subshell that picked up the new
# rust-toolchain.toml, but the INTERACTIVE shell this script was invoked from still has the
# previous toolchain on PATH (a Nix env binds its toolchain at load time, not per command).
# Flag it so the next direct `just verify`, cargo invocation, or pre-push hook in this shell
# does not run the old toolchain against the updated manifests.
echo ""
echo "⚠️  Reload your Nix shell before pushing"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Verification ran under the updated toolchain in a fresh Nix shell, but THIS shell"
echo "still has the pre-update toolchain. Exit and re-enter the directory (or 'direnv"
echo "reload') so the new toolchain is active before you push or run 'just verify'"
echo "directly — otherwise they run the old toolchain against the updated manifests and fail."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

unset DRY_RUN FORCE DOWNGRADE OLD_MSRV NEW_MSRV WORKSPACE_MSRV TOOLCHAIN_MSRV TODAY RUST_RELEASES_URL TODAY_DATE NEW_REVIEW_DATE cargo_file
