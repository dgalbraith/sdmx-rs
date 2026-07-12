#!/bin/sh
# ==============================================================================
# scripts/forge-apply.sh
# Forge configuration bootstrap (GUARDED ONE-SHOT SETUP)
#
# Applies the desired forge configuration from scripts/lib/forge-spec.sh to the
# live GitHub repository — the machine-executable form of the declarative setup
# steps in docs/project/forge-setup.md. Consumes the committed JSON bodies under
# forge/github/ directly as request payloads (no shell reconstruction).
#
# GUARD: refuses to run if the repo already looks configured (rulesets exist OR
# issues/PRs exist). Both are "configured / has data" signals — running apply
# on a live repo would duplicate rulesets and risk label-wiping issues. Override
# with --force or FORGE_APPLY_FORCE=1 when you know what you are doing (e.g.
# re-applying individual steps after a partial run).
#
# IDEMPOTENCY: this is INITIAL SETUP, not an idempotent reconcile. Running it
# twice without --force is blocked by the guard. For drift correction after the
# initial setup, use `just doctor-forge` to identify drift, then make targeted
# changes. Do NOT re-run forge-apply wholesale against a repo with live data.
#
# Usage:
#   scripts/forge-apply.sh [--force] [--skip-release-env] [--dry-run] [--yes]
#
#   --force            Override the pre-run guard (rulesets/issues present).
#   --skip-release-env Skip creating the `release` GitHub environment. Use this
#                      if you are intentionally deferring environment setup.
#   --dry-run          Print what would be applied without calling the forge API.
#   --yes              Skip the interactive "proceed?" confirmation (for scripted
#                      / non-interactive use). The confirmation is also auto-
#                      skipped when stdin is not a TTY.
#
# Environment overrides:
#   FORGE_APPLY_FORCE=1       Same as --force.
#   FORGE_SKIP_RELEASE_ENV=1  Same as --skip-release-env.
#   FORGE_DRY_RUN=1           Same as --dry-run.
#   FORGE_APPLY_YES=1         Same as --yes.
#   FORGE_APPLY_FORCE_PROMPT=1  TEST-ONLY: force the confirmation prompt even when
#                             stdin is not a TTY, so the read+abort path is testable
#                             without allocating a pty. Not for normal use.
#
# Maintainer-only. Requires: `gh` authenticated with repo scope; `jq`.
# NOT in `just verify` (CI has no gh auth and must never mutate forge state).
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/forge-spec.sh"

# --- Argument parsing ---------------------------------------------------------
force=0
skip_release_env=0
dry_run=0
assume_yes=0

for _arg in "$@"; do
    case "$_arg" in
        --force)            force=1 ;;
        --skip-release-env) skip_release_env=1 ;;
        --dry-run)          dry_run=1 ;;
        --yes)              assume_yes=1 ;;
        *) log_fatal "Unknown argument: $_arg" ;;
    esac
done
unset _arg

# Environment variable overrides.
[ "${FORGE_APPLY_FORCE:-0}" = "1" ] && force=1
[ "${FORGE_SKIP_RELEASE_ENV:-0}" = "1" ] && skip_release_env=1
[ "${FORGE_DRY_RUN:-0}" = "1" ] && dry_run=1
[ "${FORGE_APPLY_YES:-0}" = "1" ] && assume_yes=1

# --- Prerequisites ------------------------------------------------------------
log_section "Forge Configuration Bootstrap"
echo ""

if ! command -v gh >/dev/null 2>&1; then
    log_fatal "gh CLI not found — install it and run 'gh auth login'"
fi
if ! gh auth status >/dev/null 2>&1; then
    log_fatal "gh is not authenticated — run 'gh auth login' first"
fi
if ! command -v jq >/dev/null 2>&1; then
    log_fatal "jq not found — it is required for ruleset projection"
fi

if ! owner_repo="$(forge_spec_owner_repo)"; then
    log_fatal "Could not derive OWNER/REPO from origin remote"
fi
log_info "Target: $owner_repo" 1

if [ "$dry_run" = "1" ]; then
    log_warn "DRY RUN — no forge API calls will be made" 1
    echo ""
fi

# Per-step failure sinks. The ruleset/label apply loops run inside a `... | while`
# pipeline — POSIX runs that body in a SUBSHELL, so a flag set there would not
# survive to the top level. Each loop records a per-item failure by appending to
# its sink; the top level folds a non-empty sink into a fatal afterwards. mktemp
# (not a predictable /tmp/...$$ path) avoids a stale-file false positive on PID
# reuse and the classic /tmp symlink race. Cleaned by the EXIT/INT/TERM trap.
ruleset_fail_sink="$(mktemp "${TMPDIR:-/tmp}/forge-apply.rs.XXXXXX")"
label_fail_sink="$(mktemp "${TMPDIR:-/tmp}/forge-apply.lbl.XXXXXX")"
trap 'rm -f "$ruleset_fail_sink" "$label_fail_sink"' EXIT INT TERM

# --- Pre-run guard ------------------------------------------------------------
# Refuse if the repo already looks configured. Two signals checked independently:
#   1. Rulesets exist: a configured repo already has rulesets; posting again
#      would create duplicate entries.
#   2. Issues or PRs exist: the label DELETE+CREATE pass would strip labels from
#      live issues, losing triage state irreversibly. NOTE: GitHub's /issues
#      endpoint returns PRs too, so a lone bot PR (e.g. Dependabot) also blocks
#      apply — intentional (conservative), but it means the only escape is
#      --force, which re-enables the label DELETE pass wholesale.
#
# FAIL CLOSED: the guard exists to prevent duplicate rulesets / a label wipe, so
# any inability to DETERMINE the live state (gh error, unparseable body) must
# BLOCK, never be read as "empty / safe to apply". guard_probe_count returns a
# numeric count on success and non-zero on any failure; a non-zero return is
# treated as "configured / unknown" and blocks.
log_section "Pre-run guard"
echo ""

# guard_probe_count <endpoint> — echo a numeric element count for the JSON array
# the endpoint returns, or return 1 (no output) on ANY failure. Captures the gh
# body and exit status SEPARATELY from the jq parse: piping `gh | jq length`
# hides gh's exit code, and `jq length` on empty stdin exits 0 with empty output
# — so a failed fetch would silently yield "" and a naive `-gt 0` test would
# misread it as zero. The numeric-only case guard rejects "" and non-numbers.
guard_probe_count() {
    if ! _body="$(gh api "$1" 2>/dev/null)"; then unset _body; return 1; fi
    _n="$(printf '%s' "$_body" | jq 'length' 2>/dev/null)"
    case "$_n" in
        ''|*[!0-9]*) unset _body _n; return 1 ;;
    esac
    printf '%s\n' "$_n"
    unset _body _n
}

if [ "$force" = "0" ]; then
    guard_blocked=0

    if ruleset_count="$(guard_probe_count "repos/$owner_repo/rulesets")"; then
        if [ "$ruleset_count" -gt 0 ]; then
            log_fail "Guard blocked: $ruleset_count ruleset(s) already exist" 1
            guard_blocked=1
        fi
    else
        log_fail "Guard blocked: could not determine ruleset count — refusing to apply" 1
        guard_blocked=1
    fi

    if issue_count="$(guard_probe_count "repos/$owner_repo/issues?state=all&per_page=1")"; then
        if [ "$issue_count" -gt 0 ]; then
            log_fail "Guard blocked: issues/PRs present — label wipe would strip live data" 1
            guard_blocked=1
        fi
    else
        log_fail "Guard blocked: could not determine issue/PR count — refusing to apply" 1
        guard_blocked=1
    fi

    if [ "$guard_blocked" = "1" ]; then
        log_err "forge-apply is initial setup only — the repo appears already configured."
        log_err_detail "To apply individual changes, target them manually."
        log_err_detail "To override the guard (you know what you are doing), pass --force."
        exit 1
    fi
    log_ok "Guard passed — no rulesets and no issues found" 1
else
    log_warn "Guard BYPASSED via --force / FORGE_APPLY_FORCE=1" 1
fi

echo ""

# --- Mutation confirmation ----------------------------------------------------
# Everything below this point mutates LIVE forge configuration. Confirm before the
# first write, UNLESS: --dry-run (nothing to confirm), --yes / FORGE_APPLY_YES=1
# (caller opted out), or stdin is not a TTY (CI / piped / scripted) — so an
# unattended run never blocks on a prompt. The pre-run guard + --force are
# unchanged; this is an ADDITIONAL human gate, not a replacement.
#
# Whether to prompt is decided here; the prompt+read itself lives in
# confirm_or_abort() so the abort logic is unit-testable WITHOUT allocating a real
# pty (which is unsafe in a verification gate — a stray pty can grab the terminal
# and hang `just verify`). FORGE_APPLY_FORCE_PROMPT=1 is a TEST-ONLY hook that
# forces the prompt branch so a piped answer on stdin exercises read+abort.
confirm_or_abort() {
    printf 'About to apply forge configuration to %s — proceed? [y/N] ' "$owner_repo"
    read -r _confirm || _confirm=""
    case "$_confirm" in
        y|Y|yes|Yes|YES) ;;
        *) log_fatal "Aborted by user — no changes made." ;;
    esac
    unset _confirm
    echo ""
}

if [ "$dry_run" = "0" ] && [ "$assume_yes" = "0" ]; then
    if [ -t 0 ] || [ "${FORGE_APPLY_FORCE_PROMPT:-0}" = "1" ]; then
        confirm_or_abort
    fi
fi

# --- Step 1: Merge flags + repo settings --------------------------------------
log_section "Step 1 — Merge flags & repository settings"
echo ""

# Each spec function emits key\tvalue records. Apply them one PATCH per key so
# the loop stays simple and failures are per-key-visible. These are BOOLEAN flags
# (allow_merge_commit, has_projects, …), so -F (--field) is correct: its magic
# typing sends `true`/`false` as JSON booleans. (Label STRINGS in Step 4 use -f
# for the opposite reason — see the note there.)
for _spec_fn in forge_spec_merge_flags forge_spec_repo_settings; do
    "$_spec_fn" | while IFS="$FORGE_TAB" read -r key val; do
        if [ "$dry_run" = "1" ]; then
            log_info "[dry-run] PATCH repos/$owner_repo: $key = $val" 1
        else
            gh api --method PATCH "repos/$owner_repo" -F "${key}=${val}" >/dev/null
            log_ok "PATCH $key = $val" 1
        fi
    done
done
unset _spec_fn

echo ""

# --- Step 2: Actions permissions ----------------------------------------------
log_section "Step 2 — Actions permissions"
echo ""

# PUT allowed_actions=selected BEFORE PUT-ing the allowlist: GitHub rejects
# PUT /selected-actions while the mode is still "all". Flip the mode first, then
# populate the allowlist. Ordering is critical.
# Uses PUT (not PATCH) — the correct method for this endpoint on standard accounts.
# sha_pinning_required is read-only on standard GitHub plans; omit it from the body.
if [ "$dry_run" = "1" ]; then
    log_info "[dry-run] PUT actions/permissions: enabled=true, allowed_actions=selected" 1
else
    printf '{"enabled":true,"allowed_actions":"selected"}' \
        | gh api --method PUT "repos/$owner_repo/actions/permissions" --input - >/dev/null
    log_ok "PUT actions/permissions: enabled=true, allowed_actions=selected" 1
fi

_allowlist_file="$(forge_spec_actions_allowlist_file)"
if [ ! -f "$_allowlist_file" ]; then
    log_fatal "Actions allowlist file missing: $_allowlist_file — cannot apply"
fi
if [ "$dry_run" = "1" ]; then
    log_info "[dry-run] PUT actions/permissions/selected-actions from $_allowlist_file" 1
else
    gh api --method PUT "repos/$owner_repo/actions/permissions/selected-actions" \
        --input "$_allowlist_file" >/dev/null
    log_ok "PUT selected-actions from $_allowlist_file" 1
fi
unset _allowlist_file

# Default workflow-token permissions (distinct sub-resource). Single PUT carries
# both fields: default_workflow_permissions is a STRING, can_approve_pull_request_
# reviews a BOOL, so build the JSON body rather than -F per key.
if [ "$dry_run" = "1" ]; then
    forge_spec_workflow_permissions | while IFS="$FORGE_TAB" read -r key val; do
        log_info "[dry-run] PUT actions/permissions/workflow: $key = $val" 1
    done
else
    # Extract the two fields. Use a case (not `[ ] &&`) so the subshell's exit
    # status is always 0 — a trailing failed test would trip `set -e` here.
    _dwp="$(forge_spec_workflow_permissions | while IFS="$FORGE_TAB" read -r k v; do
        case "$k" in default_workflow_permissions) printf '%s' "$v" ;; esac
    done)"
    _capr="$(forge_spec_workflow_permissions | while IFS="$FORGE_TAB" read -r k v; do
        case "$k" in can_approve_pull_request_reviews) printf '%s' "$v" ;; esac
    done)"
    _wfbody="$(printf '{"default_workflow_permissions":"%s","can_approve_pull_request_reviews":%s}' "$_dwp" "$_capr")"
    printf '%s' "$_wfbody" \
        | gh api --method PUT "repos/$owner_repo/actions/permissions/workflow" --input - >/dev/null
    log_ok "PUT workflow permissions: default=$_dwp, can_approve_pr=$_capr" 1
    unset _dwp _capr _wfbody
fi

echo ""

# --- Step 2b: Security settings -----------------------------------------------
log_section "Step 2b — Security settings"
echo ""

# Toggle endpoints (vulnerability-alerts, automated-security-fixes, private-
# vulnerability-reporting): each its own sub-resource. want=true -> PUT (enable),
# want=false -> DELETE (disable). automated-security-fixes is deliberately
# DISABLED (its auto-PRs would introduce unsigned commits — SECURITY.md).
# The record's 4th field (<probe>) is for doctor's read shape; apply only needs
# <want> to choose the verb. shellcheck disable=SC2034 — _probe is read to consume
# the field but intentionally unused here.
# shellcheck disable=SC2034
forge_spec_security_toggles | while IFS="$FORGE_TAB" read -r key want endpoint _probe; do
    if [ "$want" = "true" ]; then _verb="PUT"; else _verb="DELETE"; fi
    if [ "$dry_run" = "1" ]; then
        log_info "[dry-run] $_verb repos/$owner_repo/$endpoint ($key -> $want)" 1
    elif gh api --method "$_verb" "repos/$owner_repo/$endpoint" >/dev/null 2>&1; then
        log_ok "$_verb $key (=$want)" 1
    else
        log_warn "Could not set $key (=$want) — verify manually" 1
    fi
done

# security_and_analysis.* (secret scanning + push protection) via PATCH /repos
# with a nested object. A failure here is a WARN, not fatal — forge-setup.md is
# the reference for these settings.
forge_spec_security_analysis | while IFS="$FORGE_TAB" read -r key want; do
    if [ "$dry_run" = "1" ]; then
        log_info "[dry-run] PATCH repos/$owner_repo security_and_analysis.$key=$want" 1
    else
        _body="$(printf '{"security_and_analysis":{"%s":{"status":"%s"}}}' "$key" "$want")"
        if printf '%s' "$_body" | gh api --method PATCH "repos/$owner_repo" --input - >/dev/null 2>&1; then
            log_ok "PATCH security_and_analysis.$key=$want" 1
        else
            log_warn "Could not set $key=$want — verify manually" 1
        fi
    fi
done

echo ""

# --- Step 3: Rulesets ---------------------------------------------------------
log_section "Step 3 — Rulesets"
echo ""

if [ "$force" = "1" ]; then
    log_warn "Force mode: skipping ruleset dedup check — caller is responsible for avoiding duplicates" 1
fi

forge_spec_rulesets | while IFS="$FORGE_TAB" read -r rs_name rs_target rs_file; do
    if [ ! -f "$rs_file" ]; then
        log_fail "Ruleset file missing: $rs_file" 1
        echo "fail" >> "$ruleset_fail_sink"
        continue
    fi
    if [ "$dry_run" = "1" ]; then
        log_info "[dry-run] POST rulesets — '$rs_name' (target: $rs_target) from $rs_file" 1
        continue
    fi
    gh api --method POST "repos/$owner_repo/rulesets" --input "$rs_file" >/dev/null
    log_ok "POST ruleset: '$rs_name' (target: $rs_target)" 1
done

if [ -s "$ruleset_fail_sink" ]; then
    log_fatal "One or more ruleset files missing — see above"
fi

echo ""

# --- Step 4: Labels -----------------------------------------------------------
log_section "Step 4 — Labels"
echo ""

# Delete the 6 non-spec GitHub default labels.
log_info "Deleting non-spec default labels" 1
forge_spec_default_labels_to_delete | while read -r lname; do
    if [ "$dry_run" = "1" ]; then
        log_info "[dry-run] DELETE label: $lname" 2
        continue
    fi
    if gh api --method DELETE "repos/$owner_repo/labels/$(printf '%s' "$lname" | jq -sRr @uri)" >/dev/null 2>&1; then
        log_ok "Deleted: $lname" 2
    else
        log_warn "Not found (already absent?): $lname" 2
    fi
done

echo ""

# Create/upsert the 14 spec labels. PATCH if already present (idempotent for
# the 3 shared labels — duplicate/good first issue/help wanted — that GitHub
# creates by default and are also in our spec). POST otherwise.
log_info "Creating spec labels" 1
forge_spec_labels | while IFS="$FORGE_TAB" read -r lname lcolor ldesc; do
    if [ "$dry_run" = "1" ]; then
        log_info "[dry-run] UPSERT label: $lname (#$lcolor)" 2
        continue
    fi
    # Try PATCH (update) first; if the label does not exist, fall through to POST.
    # Label fields are STRINGS, so use -f (--raw-field, always a string). NOT -F
    # (--field), whose magic typing would coerce a value of true/false/null/a bare
    # integer into the wrong JSON type — e.g. a future description "null" would be
    # sent as JSON null. (The boolean PATCH calls in Steps 1-2 deliberately use -F
    # for exactly that typed coercion; the split is intentional.)
    _label_uri="$(printf '%s' "$lname" | jq -sRr @uri)"
    if gh api --method PATCH "repos/$owner_repo/labels/$_label_uri" \
           -f "name=$lname" -f "color=$lcolor" -f "description=$ldesc" >/dev/null 2>&1; then
        log_ok "Upserted: $lname" 2
    elif gh api --method POST "repos/$owner_repo/labels" \
           -f "name=$lname" -f "color=$lcolor" -f "description=$ldesc" >/dev/null 2>&1; then
        log_ok "Created: $lname" 2
    else
        log_fail "Failed to create/update label: $lname" 2
        echo "fail" >> "$label_fail_sink"
    fi
done

if [ -s "$label_fail_sink" ]; then
    log_fatal "One or more labels could not be created — see above"
fi

echo ""

# --- Step 5: Release environment ----------------------------------------------
if [ "$skip_release_env" = "1" ]; then
    log_warn "Skipping release environment creation (--skip-release-env)" 1
    echo ""
else
    log_section "Step 5 — Release environment"
    echo ""

    env_name="$(forge_spec_release_env_name)"
    psr="$(forge_spec_release_env_prevent_self_review)"

    # Derive the maintainer id at runtime — it is account-specific.
    if [ "$dry_run" = "1" ]; then
        log_info "[dry-run] GET user.id for maintainer reviewer" 1
        log_info "[dry-run] PUT environments/$env_name" 1
        log_item "prevent_self_review = $psr" 2
    else
        # A failing `gh api` in this assignment does NOT trip `set -e` (the exit
        # status of a command substitution in a simple assignment is not checked),
        # so maintainer_id becomes "" and the explicit guard below catches both the
        # error case and a genuine null/empty id. 2>/dev/null keeps gh's stderr out
        # of the captured value; the guard provides the actionable message.
        # Create the environment first without reviewers — required reviewers on
        # environments require GitHub Team/Enterprise; Pro plan on a private repo
        # returns 422. Add reviewers in a second call so the environment itself
        # always succeeds and the reviewer step degrades gracefully.
        gh api --method PUT "repos/$owner_repo/environments/$env_name" >/dev/null
        log_ok "PUT environment '$env_name'" 1

        maintainer_id="$(gh api user --jq '.id' 2>/dev/null)"
        if [ -z "$maintainer_id" ] || [ "$maintainer_id" = "null" ]; then
            log_warn "Could not derive maintainer user id — skipping reviewer assignment" 1
        else
            env_body="$(printf '{"reviewers":[{"type":"User","id":%s}],"prevent_self_review":%s}' \
                "$maintainer_id" "$psr")"
            if printf '%s' "$env_body" \
                | gh api --method PUT "repos/$owner_repo/environments/$env_name" --input - >/dev/null 2>&1; then
                log_ok "PUT reviewer for '$env_name' (prevent_self_review=$psr)" 1
            else
                log_warn "Could not set required reviewer (requires Team/Enterprise or public repo) — add manually when available" 1
            fi
        fi
        log_hint "Trusted Publisher registration (crates.io) remains deferred until the bootstrap publish" 1
    fi
    echo ""
fi

# ==============================================================================
# Summary
# ==============================================================================
log_ok "forge-apply: bootstrap complete — run 'just doctor-forge' to verify"
