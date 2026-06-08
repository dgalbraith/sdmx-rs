#!/bin/sh
# ==============================================================================
# scripts/update-rulesets.sh
# Idempotent ruleset apply (POST-or-PUT by name)
#
# Applies the rulesets declared in scripts/lib/forge-spec.sh to the live GitHub
# repository. Unlike scripts/forge-apply.sh (a guarded one-shot bootstrap),
# this script is SAFE TO RE-RUN: for each spec ruleset it queries the live forge
# by name, then POSTs (create) or PUTs (update) as appropriate.
#
# DUPLICATE NAMES: GitHub does not enforce unique ruleset names. If two live
# rulesets share a spec name (e.g. from a botched --force re-apply), the script
# ABORTS rather than picking one arbitrarily. Resolve the duplicate manually
# (list IDs with `gh api repos/{o}/{r}/rulesets`, delete the unwanted one with
# `gh api --method DELETE repos/{o}/{r}/rulesets/{id}`), then re-run.
#
# SIGNING RULESET INVARIANT: after any PUT to the signing ruleset, the bypass
# list is re-asserted to be empty. A non-empty bypass allows unsigned commits
# onto the default branch — this check is an immediate post-write guard, not
# deferred to the next doctor-forge run.
#
# LABEL RENAMES: this script does not handle label renames (name is the lookup
# key; a rename appears as a new label + orphaned old label). See
# docs/project/forge-setup.md for the documented manual rename procedure.
#
# Usage:
#   scripts/update-rulesets.sh [--dry-run]
#
#   --dry-run   Print what would POST/PUT without calling the forge API.
#
# Environment overrides:
#   FORGE_DRY_RUN=1   Same as --dry-run.
#
# Maintainer-only. Requires: gh (authenticated with repo scope), jq.
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
log_section "Ruleset Update"
echo ""

if ! command -v gh >/dev/null 2>&1; then
    log_fatal "gh CLI not found — install it and run 'gh auth login'"
fi
if ! gh auth status >/dev/null 2>&1; then
    log_fatal "gh is not authenticated — run 'gh auth login' first"
fi
if ! command -v jq >/dev/null 2>&1; then
    log_fatal "jq not found — it is required for ruleset ID lookup"
fi
if ! owner_repo="$(forge_spec_owner_repo)"; then
    log_fatal "Could not derive OWNER/REPO from origin remote"
fi
log_info "Target: $owner_repo" 1

if [ "$dry_run" = "1" ]; then
    log_warn "DRY RUN — no forge API calls will be made" 1
    echo ""
fi

# Scratch directory for all temporary files: the failure sink and any
# per-iteration files (_post_tmp). A single trap cleans everything up on exit
# or signal, including files created mid-loop on early abort (Ctrl+C, set -e).
SCRATCH_DIR="$(mktemp -d "${TMPDIR:-/tmp}/update-rulesets.XXXXXX")"
trap 'rm -rf "$SCRATCH_DIR"' EXIT INT TERM
fail_sink="${SCRATCH_DIR}/fail_sink"

# --- Fetch live rulesets once -------------------------------------------------
# All name lookups work off this single fetch; each ruleset PUT uses its own
# GET for the bypass-invariant check (to reflect the just-written state).
log_info "Fetching live rulesets" 1
if ! rulesets_json="$(gh api "repos/$owner_repo/rulesets" 2>/dev/null)"; then
    log_fatal "Could not query live rulesets — check gh auth and repo access"
fi
if ! printf '%s' "$rulesets_json" | jq -e 'if type == "array" then true else error end' >/dev/null 2>&1; then
    log_fatal "Live rulesets response is not a valid JSON array — possible proxy or auth issue"
fi

echo ""

# --- Pre-validate: files exist + no duplicate names (abort before any write) --
# The apply loop uses `continue` to skip failed rulesets, but duplicate-name and
# missing-file errors must abort the entire run — not just skip one ruleset — to
# satisfy the "no mutations" contract. We validate all rulesets here first; the
# apply loop only runs if every spec ruleset passes.

forge_spec_rulesets | while IFS="$FORGE_TAB" read -r rs_name rs_target rs_file; do
    if [ ! -f "$rs_file" ]; then
        log_fail "Committed ruleset file missing: $rs_file" 1
        echo "fail" >> "$fail_sink"
    fi
    rs_count="$(printf '%s' "$rulesets_json" | \
        jq -r --arg n "$rs_name" 'map(select(.name == $n)) | length')"
    if [ "$rs_count" -gt 1 ]; then
        log_fail "Duplicate ruleset name '$rs_name' ($rs_count found) — resolve manually:" 1
        log_hint "  gh api repos/$owner_repo/rulesets | jq '.[] | select(.name == \"$rs_name\") | {id, name}'" 1
        log_hint "  gh api --method DELETE repos/$owner_repo/rulesets/{id}" 1
        echo "fail" >> "$fail_sink"
    fi
done

if [ -s "$fail_sink" ]; then
    echo ""
    log_fail "update-rulesets: pre-validation failed — no rulesets were applied"
    exit 1
fi

# --- Apply each spec ruleset --------------------------------------------------
log_section "Applying rulesets"
echo ""

forge_spec_rulesets | while IFS="$FORGE_TAB" read -r rs_name rs_target rs_file; do
    log_info "$rs_name ($rs_target)" 1

    # Count live rulesets matching this name.
    rs_count="$(printf '%s' "$rulesets_json" | \
        jq -r --arg n "$rs_name" 'map(select(.name == $n)) | length')"

    # rs_written_id is set to the live ruleset ID after a successful POST or PUT.
    # Used by the bypass-invariant check to fetch the single just-written ruleset.
    rs_written_id=""

    if [ "$rs_count" = "0" ]; then
        # Ruleset does not exist — POST to create.
        if [ "$dry_run" = "1" ]; then
            log_info "[dry-run] POST rulesets — '$rs_name' from $rs_file" 2
        else
            _post_tmp="${SCRATCH_DIR}/post_resp"
            if gh api --method POST "repos/$owner_repo/rulesets" --input "$rs_file" >"$_post_tmp" 2>/dev/null; then
                rs_written_id="$(jq -r '.id // empty' "$_post_tmp")"
                log_ok "Created: '$rs_name'" 2
            else
                log_fail "Failed to create: '$rs_name'" 2
                echo "fail" >> "$fail_sink"
                continue
            fi
        fi
    else
        # Exactly one match — PUT to update.
        rs_id="$(printf '%s' "$rulesets_json" | \
            jq -r --arg n "$rs_name" '.[] | select(.name == $n) | .id')"
        if [ -z "$rs_id" ] || [ "$rs_id" = "null" ]; then
            log_fail "Could not extract id for '$rs_name'" 2
            echo "fail" >> "$fail_sink"
            continue
        fi
        if [ "$dry_run" = "1" ]; then
            log_info "[dry-run] PUT rulesets/$rs_id — '$rs_name' from $rs_file" 2
        else
            if gh api --method PUT "repos/$owner_repo/rulesets/$rs_id" --input "$rs_file" >/dev/null; then
                rs_written_id="$rs_id"
                log_ok "Updated: '$rs_name' (id=$rs_id)" 2
            else
                log_fail "Failed to update: '$rs_name' (id=$rs_id)" 2
                echo "fail" >> "$fail_sink"
                continue
            fi
        fi
    fi

    # INVARIANT: signing and push-restriction rulesets bypass lists must remain
    # empty. Re-assert by fetching the single just-written ruleset via its ID.
    # Skip in dry-run (no write was made) or if we have no ID (should not happen).
    case "$rs_file" in
        forge/github/ruleset-signing.json|forge/github/ruleset-push-restriction.json)
            if [ "$dry_run" = "0" ] && [ -n "$rs_written_id" ]; then
                bypass_n="$(gh api "repos/$owner_repo/rulesets/$rs_written_id" 2>/dev/null | \
                    jq -r '.bypass_actors | length')"
                if [ "$bypass_n" = "0" ]; then
                    log_ok "$(basename "$rs_file" .json) bypass list is empty" 2
                else
                    log_fail "SECURITY: $(basename "$rs_file" .json) has $bypass_n bypass actor(s) — must be 0" 2
                    log_hint "Remove bypass actors from $rs_file and re-run" 2
                    echo "fail" >> "$fail_sink"
                fi
            fi
            ;;
    esac
done

echo ""

# --- Summary ------------------------------------------------------------------
if [ -s "$fail_sink" ]; then
    log_fail "update-rulesets: one or more rulesets could not be applied — see above"
    exit 1
fi

if [ "$dry_run" = "1" ]; then
    log_ok "update-rulesets: dry run complete — no changes made"
else
    log_ok "update-rulesets: all rulesets applied — run 'just doctor-forge' to verify"
fi
