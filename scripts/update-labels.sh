#!/bin/sh
# ==============================================================================
# scripts/update-labels.sh
# Idempotent label apply (PATCH-or-POST by name)
#
# Applies the 14 labels declared in scripts/lib/forge-spec.sh to the live
# GitHub repository. For each spec label it attempts PATCH (update colour +
# description on an existing label); if the label does not exist (404), falls
# through to POST (create). Any other PATCH failure (5xx, permission error)
# is reported immediately rather than falling through to POST, preventing
# accidental duplicate creation from transient errors.
#
# SCOPE — this script handles creates and attribute updates (colour,
# description) only. It does NOT:
#
#   - Delete labels: the delete pass (removing the 6 non-spec GitHub defaults)
#     is confined to scripts/forge-apply.sh because it is inherently
#     destructive and only safe on a repo with no live issues.
#
#   - Rename labels: a rename appears as a missing spec label + an orphaned
#     live label. Perform renames manually:
#
#       gh api --method PATCH "repos/{o}/{r}/labels/{old-name}" \
#           -f new_name="{new-name}" -f color="{color}" -f description="{desc}"
#
#     After the rename, `just doctor-forge` will confirm the live state matches
#     the spec. No spec change is needed for the rename itself — the spec
#     already carries the new name.
#
# Usage:
#   scripts/update-labels.sh [--dry-run]
#
#   --dry-run   Print what would PATCH/POST without calling the forge API.
#
# Environment overrides:
#   FORGE_DRY_RUN=1   Same as --dry-run.
#
# Maintainer-only. Requires: gh (authenticated with repo scope).
# NOT in `just verify` — CI has no gh auth and must never mutate forge state.
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/forge-spec.sh"

# --- Argument parsing ---------------------------------------------------------
dry_run=0

for _arg in "$@"; do
    case "$_arg" in
        --dry-run) dry_run=1 ;;
        *) log_fatal "Unknown argument: $_arg" ;;
    esac
done
unset _arg

[ "${FORGE_DRY_RUN:-0}" = "1" ] && dry_run=1

# --- Prerequisites ------------------------------------------------------------
log_section "Label Update"
echo ""

if ! command -v gh >/dev/null 2>&1; then
    log_fatal "gh CLI not found — install it and run 'gh auth login'"
fi
if ! gh auth status >/dev/null 2>&1; then
    log_fatal "gh is not authenticated — run 'gh auth login' first"
fi
if ! owner_repo="$(forge_spec_owner_repo)"; then
    log_fatal "Could not derive OWNER/REPO from origin remote"
fi
log_info "Target: $owner_repo" 1

if [ "$dry_run" = "1" ]; then
    log_warn "DRY RUN — no forge API calls will be made" 1
    echo ""
fi

# Scratch directory for all temporary files: the failure sink and the per-label
# _patch_err capture file. A single trap cleans everything up on exit or signal,
# including files created mid-loop on early abort (Ctrl+C, set -e).
SCRATCH_DIR="$(mktemp -d "${TMPDIR:-/tmp}/update-labels.XXXXXX")"
trap 'rm -rf "$SCRATCH_DIR"' EXIT INT TERM
fail_sink="${SCRATCH_DIR}/fail_sink"

# --- Apply each spec label ----------------------------------------------------
log_section "Applying labels"
echo ""

forge_spec_labels | while IFS="$FORGE_TAB" read -r lname lcolor ldesc; do
    if [ "$dry_run" = "1" ]; then
        log_info "[dry-run] UPSERT label: $lname (#$lcolor)" 1
        continue
    fi

    # URI-encode the label name for use in the endpoint path.
    _label_uri="$(printf '%s' "$lname" | jq -sRr @uri)"

    # Try PATCH first. We need to distinguish a genuine 404 (label absent —
    # fall through to POST) from any other failure (5xx, permission error —
    # report and skip, do NOT fall through to POST which would create a
    # duplicate). `gh api` exits non-zero on 4xx/5xx; we cannot interrogate the
    # HTTP status directly, so we capture stderr and check the exit code:
    #   - exit 0 → updated, done.
    #   - exit non-zero + "404" in stderr → label absent, try POST.
    #   - exit non-zero + anything else   → real error, record failure.
    _patch_err="${SCRATCH_DIR}/patch_err"
    if gh api --method PATCH "repos/$owner_repo/labels/$_label_uri" \
           -f "name=$lname" -f "color=$lcolor" -f "description=$ldesc" \
           >/dev/null 2>"$_patch_err"; then
        log_ok "Updated: $lname" 1
        continue
    fi

    # PATCH failed — inspect the error.
    if grep -qiE "404|not found" "$_patch_err" 2>/dev/null; then
        # Label absent — POST to create.
        if gh api --method POST "repos/$owner_repo/labels" \
               -f "name=$lname" -f "color=$lcolor" -f "description=$ldesc" \
               >/dev/null 2>&1; then
            log_ok "Created: $lname" 1
        else
            log_fail "Failed to create label: $lname" 1
            echo "fail" >> "$fail_sink"
        fi
    else
        # Non-404 error — report and skip.
        _err_msg="$(cat "$_patch_err" 2>/dev/null || true)"
        log_fail "Failed to update label: $lname (${_err_msg:-unknown error})" 1
        echo "fail" >> "$fail_sink"
    fi
    unset _label_uri _err_msg
done

echo ""

# --- Summary ------------------------------------------------------------------
if [ -s "$fail_sink" ]; then
    log_fail "update-labels: one or more labels could not be applied — see above"
    exit 1
fi

if [ "$dry_run" = "1" ]; then
    log_ok "update-labels: dry run complete — no changes made"
else
    log_ok "update-labels: all labels applied — run 'just doctor-forge' to verify"
fi
