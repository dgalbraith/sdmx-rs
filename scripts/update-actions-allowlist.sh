#!/bin/sh
# ==============================================================================
# scripts/update-actions-allowlist.sh
# Idempotent actions allowlist apply (single PUT from committed file)
#
# Pushes the committed forge/github/actions-allowlist.json to the live GitHub
# repository as the selected-actions allowlist. This is the update path for
# adding or removing workflow action entries after the initial bootstrap:
#
#   1. Edit forge/github/actions-allowlist.json (add/remove patterns_allowed)
#   2. Commit the change
#   3. Run this script to push it live
#   4. Run `just doctor-forge` to confirm live == spec
#
# This is a pure PUT — the allowlist is replaced wholesale with the committed
# file. Partial updates are not supported (GitHub's PUT replaces the resource).
#
# PREREQUISITE: allowed_actions=selected must already be active on the repo
# (set by scripts/forge-apply.sh during initial bootstrap). This script does
# NOT flip the mode — it only updates the allowlist body. If the mode is not
# `selected`, the PUT will succeed but have no effect until the mode is
# switched. `just doctor-forge` will surface a mode mismatch.
#
# Usage:
#   scripts/update-actions-allowlist.sh [--dry-run]
#
#   --dry-run   Print what would PUT without calling the forge API.
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
log_section "Actions Allowlist Update"
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

allowlist_file="$(forge_spec_actions_allowlist_file)"
if [ ! -f "$allowlist_file" ]; then
    log_fatal "Committed allowlist file missing: $allowlist_file"
fi
log_info "Allowlist: $allowlist_file" 1

if [ "$dry_run" = "1" ]; then
    log_warn "DRY RUN — no forge API calls will be made" 1
    echo ""
fi

# --- Apply the allowlist ------------------------------------------------------
log_section "Applying allowlist"
echo ""

if [ "$dry_run" = "1" ]; then
    log_info "[dry-run] PUT repos/$owner_repo/actions/permissions/selected-actions from $allowlist_file" 1
else
    if gh api --method PUT "repos/$owner_repo/actions/permissions/selected-actions" \
           --input "$allowlist_file" >/dev/null; then
        log_ok "PUT selected-actions from $allowlist_file" 1
    else
        log_fail "Failed to PUT selected-actions" 1
        exit 1
    fi
fi

echo ""

# --- Summary ------------------------------------------------------------------
if [ "$dry_run" = "1" ]; then
    log_ok "update-actions-allowlist: dry run complete — no changes made"
else
    log_ok "update-actions-allowlist: allowlist applied — run 'just doctor-forge' to verify"
fi
