#!/bin/sh
# ==============================================================================
# scripts/doctor-forge.sh
# Forge configuration diagnostics (READ-ONLY)
#
# Asserts that the LIVE forge configuration matches the desired spec in
# scripts/lib/forge-spec.sh (the machine-readable realization of the declarative
# parts of docs/project/forge-setup.md). This script NEVER mutates forge state —
# it only reads. To converge drift, run scripts/forge-apply.sh (guarded one-shot
# setup) or deliberately re-baseline a committed ruleset file from live.
#
# Two tiers:
#   OFFLINE — local/git/file checks that need no forge auth. ALWAYS run:
#     key file present, primary fingerprint in the CI trust root, local git
#     signing config, signed root commit, fan-out remotes.
#   ONLINE  — checks that query the forge via `gh`. Gated behind a `gh auth`
#     probe: if auth is absent, this tier is SKIPPED with a warning + hint and
#     the script still exits 0 (the offline tier having run). With auth, online
#     drift fails the run (exit 1).
#
# Exit: 0 = all run checks matched (or online tier skipped for missing auth);
#       1 = a check that ran found drift.
#
# Usage: scripts/doctor-forge.sh
#   FORGE_RELEASE_REQUIRED=1   treat a missing `release` environment as a failure
#                              (default: warn only — it may be intentionally
#                              deferred until publishing goes live).
#   FORGE_SECURITY_REQUIRED=1  treat disabled secret scanning / push protection as
#                              a failure (default: warn only — these are free only
#                              on public repos, so they are deferred until the repo
#                              goes public at the release go-live window).
#   FORGE_WORKFLOWS_DIR=<dir>  override the workflows directory scanned by the
#                              actions allowlist crosscheck (default: .github/workflows).
#                              Also scans .github/actions/**/action.yml in the same
#                              parent. Set in tests to point at a fixture workflow tree.
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/forge-spec.sh"

log_section "Forge Configuration Diagnostics"
echo ""

# Track overall status across all checks.
failed=0

# ==============================================================================
# OFFLINE TIER — no forge auth required; always runs.
# ==============================================================================
log_section "Offline checks (local / git / files)"
echo ""

# --- Resolve OWNER/REPO (needed for messaging; not a forge call) --------------
if owner_repo="$(forge_spec_owner_repo)"; then
    log_ok "Repository slug: $owner_repo" 1
else
    log_fail "Could not derive OWNER/REPO from origin remote" 1
    owner_repo=""
    failed=1
fi

# --- Offline 1: maintainer key file present -----------------------------------
key_file=".github/maintainer-keys/dgalbraith.asc"
if [ -f "$key_file" ]; then
    log_ok "Maintainer key file present: $key_file" 1
else
    log_fail "Maintainer key file missing: $key_file" 1
    failed=1
fi

# --- Offline 2: primary fingerprint anchored in the CI trust root -------------
if primary_fpr="$(forge_spec_primary_fpr)"; then
    log_ok "Primary fingerprint in verify-signature.yml: $primary_fpr" 1
else
    log_fail "Primary fingerprint not found in .github/workflows/verify-signature.yml" 1
    primary_fpr=""
    failed=1
fi

# --- Offline 3: local git signing configuration -------------------------------
spec_email="$(forge_spec_signing_email)"
git_email="$(git config --get user.email 2>/dev/null || true)"
if [ "$git_email" = "$spec_email" ]; then
    log_ok "git user.email matches signing identity: $git_email" 1
else
    log_warn "git user.email is '$git_email', spec signing email is '$spec_email'" 1
fi

git_signingkey="$(git config --get user.signingkey 2>/dev/null || true)"
# The spec pins the signing subkey; the local config carries a trailing '!' to
# pin the subkey specifically (forge-setup.md). Strip it before comparing.
# git accepts either the full 40-hex fingerprint or a short key ID (the last
# 16 hex chars), so match by SUFFIX rather than exact equality — a configured
# short ID like '7D4A0D2EE2E2ECD7' is the tail of the full spec fingerprint and
# is the same key. This is a local-config sanity hint, not the trust root (that
# is the PRIMARY fingerprint allowlist in verify-signature.yml), so a suffix
# match is acceptable. The empty case MUST be checked first: an empty string is
# a suffix of everything, so the glob below would otherwise match "not set".
_signingkey="${git_signingkey%!}"
if [ -z "$_signingkey" ]; then
    log_warn "git user.signingkey not configured" 1
else
    case "$FORGE_SUBKEY_FPR" in
        *"$_signingkey")
            log_ok "git user.signingkey pins the signing subkey" 1
            ;;
        *)
            log_warn "git user.signingkey '$git_signingkey' != spec subkey '$FORGE_SUBKEY_FPR'" 1
            ;;
    esac
fi
unset _signingkey

if [ "$(git config --get commit.gpgsign 2>/dev/null || true)" = "true" ]; then
    log_ok "commit.gpgsign enabled" 1
else
    log_warn "commit.gpgsign not enabled" 1
fi

# --- Offline 4: signed root commit --------------------------------------------
# The root commit is the immutable signed history anchor (forge-setup.md step 3).
# `git verify-commit` exits 0 only for a good signature; it is the same plumbing
# the verify-signature tooling uses, so it mocks cleanly.
if root_commit="$(git rev-list --max-parents=0 HEAD 2>/dev/null | tail -n1)" && [ -n "$root_commit" ]; then
    if git verify-commit "$root_commit" >/dev/null 2>&1; then
        log_ok "Root commit is signed ($root_commit)" 1
    else
        log_fail "Root commit is NOT signed/verifiable ($root_commit)" 1
        failed=1
    fi
else
    log_warn "Could not resolve root commit" 1
fi

# --- Offline 5: fan-out remotes -----------------------------------------------
# origin (GitHub) + codeberg (mirror) + 'all' fan-out push remote per
# forge-setup.md. origin is mandatory; the mirror/fan-out are advisory (a fresh
# clone may not have them yet).
if git remote get-url origin >/dev/null 2>&1; then
    log_ok "Remote 'origin' configured" 1
else
    log_fail "Remote 'origin' not configured" 1
    failed=1
fi
for remote in codeberg all; do
    if git remote get-url "$remote" >/dev/null 2>&1; then
        log_ok "Remote '$remote' configured" 1
    else
        log_warn "Remote '$remote' not configured (mirror/fan-out)" 1
    fi
done

echo ""

# ==============================================================================
# ONLINE TIER — gated behind a gh-auth probe.
# ==============================================================================
log_section "Online checks (live forge state)"
echo ""

if ! command -v gh >/dev/null 2>&1; then
    log_warn "gh CLI not found — skipping online forge checks"
    log_hint "Install gh and run 'gh auth login' to verify live forge state"
    echo ""
    # Offline tier already ran; honour its result without online drift.
    if [ "$failed" -eq 0 ]; then
        log_ok "doctor-forge: offline checks passed (online tier skipped — no gh)"
        exit 0
    fi
    log_fail "doctor-forge: offline checks found drift — see above"
    exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
    log_warn "gh is not authenticated — skipping online forge checks"
    log_hint "Run 'gh auth login' to verify live forge state, then re-run"
    echo ""
    if [ "$failed" -eq 0 ]; then
        log_ok "doctor-forge: offline checks passed (online tier skipped — no auth)"
        exit 0
    fi
    log_fail "doctor-forge: offline checks found drift — see above"
    exit 1
fi

# From here on we have auth. A missing slug is fatal — every online call needs it.
if [ -z "$owner_repo" ]; then
    log_fatal "Cannot run online checks without a resolved OWNER/REPO"
fi

# Per-item drift sink. The online checks emit each spec item inside a
# `... | while read` pipeline, which POSIX runs in a SUBSHELL — a `failed=1`
# there would not survive to the top level. So drifting items append a line
# here; after the online tier we fold a non-empty sink back into $failed.
drift_sink="$(mktemp "${TMPDIR:-/tmp}/doctor-forge.drift.XXXXXX")"
# Managed scratch dir for the per-ruleset diff temp files (below). Holding them
# under one dir lets the trap clean them even if the run is interrupted mid-loop
# — an inline `rm -f` alone would leak on INT/TERM between create and remove.
diff_dir="$(mktemp -d "${TMPDIR:-/tmp}/doctor-forge.diff.XXXXXX")"
trap 'rm -f "$drift_sink"; rm -rf "$diff_dir"' EXIT INT TERM

# --- Online 1 + 2: merge flags & repo settings --------------------------------
# Both read the SAME repos/{o}/{r} body, so they share one fetch and one failure
# path: if the fetch fails, repo_json would be "" and every jq lookup would yield
# empty — emitting a spurious "drift" line per setting on top of the real error.
# Nesting both loops inside the single `if` avoids that misleading cascade.
log_info "Merge methods" 1
if repo_json="$(gh api "repos/$owner_repo" 2>/dev/null)"; then
    forge_spec_merge_flags | while IFS="$FORGE_TAB" read -r key want; do
        got="$(printf '%s' "$repo_json" | jq -r ".$key")"
        if [ "$got" = "$want" ]; then
            log_ok "$key = $got" 2
        else
            log_fail "$key = $got (want $want)" 2
            echo "drift" >> "$drift_sink"
        fi
    done

    log_info "Repository settings" 1
    forge_spec_repo_settings | while IFS="$FORGE_TAB" read -r key want; do
        got="$(printf '%s' "$repo_json" | jq -r ".$key")"
        if [ "$got" = "$want" ]; then
            log_ok "$key = $got" 2
        else
            log_fail "$key = $got (want $want)" 2
            echo "drift" >> "$drift_sink"
        fi
    done

    # security_and_analysis.* (secret scanning + push protection). PUBLIC-ONLY:
    # free only on public repos, so a "disabled" reading is a deferred WARN by
    # default (the repo is private until go-live) and a FAILURE only under the
    # FORGE_SECURITY_REQUIRED opt-in. Read off the same repo_json; the nested
    # status is .security_and_analysis.<key>.status ("enabled"/"disabled"/absent).
    log_info "Security & analysis (public-only)" 1
    forge_spec_security_analysis | while IFS="$FORGE_TAB" read -r key want; do
        got="$(printf '%s' "$repo_json" | jq -r ".security_and_analysis.${key}.status // \"absent\"")"
        if [ "$got" = "$want" ]; then
            log_ok "$key = $got" 2
        elif [ "${FORGE_SECURITY_REQUIRED:-0}" = "1" ]; then
            log_fail "$key = $got (want $want; FORGE_SECURITY_REQUIRED=1)" 2
            echo "drift" >> "$drift_sink"
        else
            log_warn "$key = $got (want $want — deferred until repo is public)" 2
        fi
    done
else
    log_fail "Could not query repos/$owner_repo (merge flags & repo settings)" 2
    failed=1
fi

# --- Online 3: actions permissions --------------------------------------------
log_info "Actions permissions" 1
if actions_json="$(gh api "repos/$owner_repo/actions/permissions" 2>/dev/null)"; then
    forge_spec_actions | while IFS="$FORGE_TAB" read -r key want; do
        got="$(printf '%s' "$actions_json" | jq -r ".$key")"
        if [ "$got" = "$want" ]; then
            log_ok "$key = $got" 2
        else
            log_fail "$key = $got (want $want)" 2
            echo "drift" >> "$drift_sink"
        fi
    done
else
    log_fail "Could not query actions/permissions" 2
    echo "drift" >> "$drift_sink"
fi

# --- Online 3a: selected-actions body diff ------------------------------------
# Only meaningful once allowed_actions=selected; when the live setting is still
# "all" the selected-actions sub-resource is empty/absent — report as deferred
# (consistent with the allowed_actions drift line above) rather than double-
# counting. When selected: project both sides and diff like the rulesets block.
_live_allowed="$(printf '%s' "${actions_json:-}" | jq -r '.allowed_actions // "all"')"
if [ "$_live_allowed" = "selected" ]; then
    log_info "Actions allowlist (selected-actions)" 1
    _allowlist_file="$(forge_spec_actions_allowlist_file)"
    _allowlist_proj_jq="$(forge_spec_actions_allowlist_projection_jq)"
    if [ ! -f "$_allowlist_file" ]; then
        log_fail "Committed allowlist file missing: $_allowlist_file" 2
        echo "drift" >> "$drift_sink"
    elif sa_json="$(gh api "repos/$owner_repo/actions/permissions/selected-actions" 2>/dev/null)"; then
        _live_sa_proj="$(printf '%s' "$sa_json" | jq -S "$_allowlist_proj_jq")"
        _file_sa_proj="$(jq -S "$_allowlist_proj_jq" "$_allowlist_file")"
        if [ "$_live_sa_proj" = "$_file_sa_proj" ]; then
            log_ok "selected-actions matches committed file: $_allowlist_file" 2
        else
            log_fail "selected-actions drifts from $_allowlist_file" 2
            _df="$(mktemp "$diff_dir/file.XXXXXX")"
            _dl="$(mktemp "$diff_dir/live.XXXXXX")"
            printf '%s\n' "$_file_sa_proj" > "$_df"
            printf '%s\n' "$_live_sa_proj" > "$_dl"
            diff -u "$_df" "$_dl" | sed 's/^/      /' || true
            rm -f "$_df" "$_dl"
            echo "drift" >> "$drift_sink"
        fi
    else
        log_fail "Could not query selected-actions" 2
        echo "drift" >> "$drift_sink"
    fi
    unset _allowlist_file _allowlist_proj_jq _live_sa_proj _file_sa_proj sa_json
else
    log_info "Actions allowlist (selected-actions)" 1
    log_warn "Deferred: allowed_actions=$_live_allowed (want selected) — apply first, then re-run" 2
fi
unset _live_allowed

# --- Online 3b: actions allowlist uses: crosscheck ----------------------------
# Enumerate all third-party uses: from the workflow files and check each is
# covered by the committed allowlist. Uncovered → FAIL (it's the CI-breaking
# direction). Stale patterns in the allowlist (matching no uses:) → WARN only.
# This check reads only committed files — it does not call the forge API — but
# sits here for output coherence with its siblings.
log_info "Actions allowlist coverage (uses: crosscheck)" 1
_allowlist_file="$(forge_spec_actions_allowlist_file)"
_wf_dir="${FORGE_WORKFLOWS_DIR:-.github/workflows}"
# Derive the parent of the workflows dir so we can also scan composite actions.
_actions_parent="${_wf_dir%/workflows}"
[ "$_actions_parent" = "$_wf_dir" ] && _actions_parent="$(dirname "$_wf_dir")"

if [ ! -f "$_allowlist_file" ]; then
    log_fail "Committed allowlist file missing: $_allowlist_file — cannot crosscheck" 2
    echo "drift" >> "$drift_sink"
else
    # Read allowlist fields into temp files so the coverage checks can read them
    # without nested subshells fighting set -eu.
    _cc_tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/doctor-forge.cc.XXXXXX")"
    jq -r '.github_owned_allowed' "$_allowlist_file" > "$_cc_tmpdir/gh_owned"
    jq -r '.patterns_allowed[] | sub("@.*"; "")' "$_allowlist_file" > "$_cc_tmpdir/patterns"
    _gh_owned="$(cat "$_cc_tmpdir/gh_owned")"

    # Collect distinct third-party uses: (local ./... refs excluded).
    # Scan both workflows dir and composite actions under the same parent.
    { [ -d "$_wf_dir" ] && grep -hoE 'uses:[[:space:]]*[^[:space:]]+' "$_wf_dir"/*.yml 2>/dev/null || true
      [ -d "$_actions_parent/actions" ] && \
          find "$_actions_parent/actions" -name "action.yml" -exec \
          grep -hoE 'uses:[[:space:]]*[^[:space:]]+' {} \; 2>/dev/null || true
    } | sed 's/uses:[[:space:]]*//' | grep -v '^\.' | sed 's/@.*//' | sort -u \
      > "$_cc_tmpdir/uses"

    if [ ! -s "$_cc_tmpdir/uses" ]; then
        log_warn "No workflow uses: lines found — check FORGE_WORKFLOWS_DIR" 2
    else
        # Forward pass: each uses: must be covered.
        while read -r _cc_ref; do
            _cc_org="${_cc_ref%%/*}"
            # github-owned?
            if [ "$_gh_owned" = "true" ] && [ "$_cc_org" = "actions" ]; then
                log_ok "covered: $_cc_ref" 2
                unset _cc_org; continue
            fi
            # exact or org/* pattern match?
            if grep -qxF "$_cc_ref" "$_cc_tmpdir/patterns" || \
               grep -qxF "${_cc_org}/*" "$_cc_tmpdir/patterns"; then
                log_ok "covered: $_cc_ref" 2
            else
                log_fail "Uncovered action: $_cc_ref — add \"${_cc_ref}@*\" to forge/github/actions-allowlist.json (see forge/README.md)" 2
                echo "drift" >> "$drift_sink"
            fi
            unset _cc_org
        done < "$_cc_tmpdir/uses"

        # Reverse pass: stale patterns (matching no current uses:) → WARN only.
        while read -r _cc_pat; do
            [ -z "$_cc_pat" ] && continue
            _cc_pat_org="${_cc_pat%%/*}"
            # Does any uses: match this pattern?
            if grep -qxF "$_cc_pat" "$_cc_tmpdir/uses" || \
               grep -qE "^${_cc_pat_org}/" "$_cc_tmpdir/uses"; then
                : # covered — not stale
            else
                log_warn "Stale allowlist pattern (no matching uses:): ${_cc_pat}@*" 2
            fi
            unset _cc_pat_org
        done < "$_cc_tmpdir/patterns"
    fi

    rm -rf "$_cc_tmpdir"
    unset _cc_tmpdir _gh_owned
fi
unset _allowlist_file _wf_dir _actions_parent

# --- Online 3c: default workflow-token permissions ----------------------------
# Distinct sub-resource from actions/permissions: least-privilege for GITHUB_TOKEN
# (default read; bot may not approve PRs).
log_info "Workflow token permissions" 1
if wfperm_json="$(gh api "repos/$owner_repo/actions/permissions/workflow" 2>/dev/null)"; then
    forge_spec_workflow_permissions | while IFS="$FORGE_TAB" read -r key want; do
        got="$(printf '%s' "$wfperm_json" | jq -r ".$key")"
        if [ "$got" = "$want" ]; then
            log_ok "$key = $got" 2
        else
            log_fail "$key = $got (want $want)" 2
            echo "drift" >> "$drift_sink"
        fi
    done
else
    log_fail "Could not query actions/permissions/workflow" 2
    echo "drift" >> "$drift_sink"
fi

# --- Online 3b: security toggle endpoints -------------------------------------
# Each is its own sub-resource, not a repo-object field. Two probe shapes:
#   presence — `gh api --silent` exits 0 when the resource exists (enabled, 204)
#              and non-zero on 404 (disabled). No body to parse.
#   enabled  — the resource returns a JSON body; read `.enabled` (true/false).
# Desired value per spec ("true"/"false"); a mismatch is drift.
log_info "Security settings" 1
forge_spec_security_toggles | while IFS="$FORGE_TAB" read -r key want endpoint probe; do
    case "$probe" in
        presence)
            if gh api --silent "repos/$owner_repo/$endpoint" >/dev/null 2>&1; then
                got="true"
            else
                got="false"
            fi
            ;;
        enabled)
            _body="$(gh api "repos/$owner_repo/$endpoint" 2>/dev/null || true)"
            got="$(printf '%s' "$_body" | jq -r '.enabled // false' 2>/dev/null || echo "false")"
            ;;
        *)
            got="?"
            ;;
    esac
    if [ "$got" = "$want" ]; then
        log_ok "$key = $got" 2
    else
        # private-vulnerability-reporting is free only on public repos — treat as
        # deferred (consistent with secret scanning) until FORGE_SECURITY_REQUIRED=1.
        case "$key" in
            private-vulnerability-reporting)
                if [ "${FORGE_SECURITY_REQUIRED:-0}" = "1" ]; then
                    log_fail "$key = $got (want $want; FORGE_SECURITY_REQUIRED=1)" 2
                    echo "drift" >> "$drift_sink"
                else
                    log_warn "$key = $got (want $want — deferred until repo is public)" 2
                fi
                ;;
            *)
                log_fail "$key = $got (want $want)" 2
                echo "drift" >> "$drift_sink"
                ;;
        esac
    fi
done

# --- Online 4: rulesets (file-diff + signing bypass INVARIANT) ----------------
log_info "Branch / tag rulesets" 1
proj_jq="$(forge_spec_ruleset_projection_jq)"
if rulesets_json="$(gh api "repos/$owner_repo/rulesets" 2>/dev/null)"; then
    forge_spec_rulesets | while IFS="$FORGE_TAB" read -r rs_name rs_target rs_file; do
        # GitHub does not enforce unique ruleset names. Count matches first: a
        # duplicate (which a botched re-apply can create) would make the id pick
        # below silently take one and ignore the rest, hiding a permissive twin.
        rs_n="$(printf '%s' "$rulesets_json" | jq -r --arg n "$rs_name" 'map(select(.name == $n)) | length')"
        if [ "$rs_n" = "0" ]; then
            log_fail "Ruleset missing: '$rs_name' ($rs_target)" 2
            echo "drift" >> "$drift_sink"
            continue
        fi
        if [ "$rs_n" != "1" ]; then
            log_fail "Duplicate ruleset name '$rs_name' ($rs_n found) — expected exactly one" 2
            echo "drift" >> "$drift_sink"
            continue
        fi
        # Exactly one match by here, so this resolves a single id. The null/empty
        # guard stays as defence against a malformed entry that matched by name
        # but carries no .id (head -n1 keeps it harmless either way).
        rs_id="$(printf '%s' "$rulesets_json" | jq -r --arg n "$rs_name" '.[] | select(.name == $n) | .id' | head -n1)"
        if [ -z "$rs_id" ] || [ "$rs_id" = "null" ]; then
            log_fail "Ruleset missing: '$rs_name' ($rs_target)" 2
            echo "drift" >> "$drift_sink"
            continue
        fi
        if [ ! -f "$rs_file" ]; then
            log_fail "Committed ruleset file missing: $rs_file" 2
            echo "drift" >> "$drift_sink"
            continue
        fi
        # Project both sides into the same canonical shape and diff.
        live_proj="$(gh api "repos/$owner_repo/rulesets/$rs_id" 2>/dev/null | jq -S "$proj_jq")"
        file_proj="$(jq -S "$proj_jq" "$rs_file")"
        if [ "$live_proj" = "$file_proj" ]; then
            log_ok "Ruleset matches committed file: '$rs_name'" 2
        else
            log_fail "Ruleset drifts from $rs_file: '$rs_name'" 2
            # POSIX-portable diff (no process substitution): write both
            # projections to temp files under the trap-managed $diff_dir, diff,
            # clean. The inline rm handles the happy path; the trap's `rm -rf
            # "$diff_dir"` reclaims them if the run is interrupted mid-loop.
            _df="$(mktemp "$diff_dir/file.XXXXXX")"
            _dl="$(mktemp "$diff_dir/live.XXXXXX")"
            printf '%s\n' "$file_proj" > "$_df"
            printf '%s\n' "$live_proj" > "$_dl"
            diff -u "$_df" "$_dl" | sed 's/^/      /' || true
            rm -f "$_df" "$_dl"
            echo "drift" >> "$drift_sink"
        fi
        # INVARIANT: the signing and push-restriction rulesets MUST keep empty
        # bypass lists. A bypass on signing lets unsigned commits onto main; a
        # bypass on push-restriction lets an unverified SHA past the CI gate.
        # Both are checked explicitly, independent of the whole-file diff above.
        case "$rs_file" in
            forge/github/ruleset-signing.json|forge/github/ruleset-push-restriction.json)
                bypass_n="$(gh api "repos/$owner_repo/rulesets/$rs_id" 2>/dev/null | jq '.bypass_actors | length')"
                if [ "$bypass_n" = "0" ]; then
                    log_ok "$(basename "$rs_file" .json) bypass list is empty" 2
                else
                    log_fail "SECURITY: $(basename "$rs_file" .json) has $bypass_n bypass actor(s) — must be 0" 2
                    echo "drift" >> "$drift_sink"
                fi
                ;;
        esac
    done
else
    log_fail "Could not query rulesets" 2
    echo "drift" >> "$drift_sink"
fi

# --- Online 5: labels ---------------------------------------------------------
log_info "Labels" 1
# 14 spec labels sit well under one page (per_page=100), so a single call
# returns one JSON array — no pagination merge needed.
if labels_json="$(gh api "repos/$owner_repo/labels?per_page=100" 2>/dev/null)"; then
    forge_spec_labels | while IFS="$FORGE_TAB" read -r name color desc; do
        # Downcase the live colour before comparing: the spec stores lowercase hex
        # and GitHub returns lowercase today, but comparing case-insensitively makes
        # the check robust to a label set elsewhere in uppercase (which would
        # otherwise read as perpetual, unexplained drift).
        match="$(printf '%s' "$labels_json" | jq -r --arg n "$name" --arg c "$color" --arg d "$desc" \
            'map(select(.name == $n))
             | if length == 0 then "absent"
               elif ((.[0].color | ascii_downcase) == $c and (.[0].description // "") == $d) then "match"
               else "drift" end')"
        case "$match" in
            match)  log_ok "$name" 2 ;;
            absent) log_fail "Label absent: $name" 2; echo "drift" >> "$drift_sink" ;;
            *)      log_fail "Label drifts (color/description): $name" 2; echo "drift" >> "$drift_sink" ;;
        esac
    done
else
    log_fail "Could not query labels" 2
    echo "drift" >> "$drift_sink"
fi

# --- Online 6: release environment --------------------------------------------
log_info "Release environment" 1
if env_json="$(gh api "repos/$owner_repo/environments/$(forge_spec_release_env_name)" 2>/dev/null)"; then
    psr_want="$(forge_spec_release_env_prevent_self_review)"
    psr_got="$(printf '%s' "$env_json" | jq -r '.prevent_self_review')"
    if [ "$psr_got" = "$psr_want" ]; then
        log_ok "release environment exists; prevent_self_review=$psr_got" 2
    else
        # Required reviewers (and prevent_self_review) on environments require
        # Team/Enterprise or a public repo — deferred until go-live visibility flip.
        if [ "${FORGE_SECURITY_REQUIRED:-0}" = "1" ]; then
            log_fail "release environment prevent_self_review=$psr_got (want $psr_want; FORGE_SECURITY_REQUIRED=1)" 2
            echo "drift" >> "$drift_sink"
        else
            log_warn "release environment prevent_self_review=$psr_got (want $psr_want — deferred until repo is public)" 2
        fi
    fi
else
    if [ "${FORGE_RELEASE_REQUIRED:-0}" = "1" ]; then
        log_fail "release environment missing (FORGE_RELEASE_REQUIRED=1)" 2
        echo "drift" >> "$drift_sink"
    else
        log_warn "release environment missing (may be deferred until go-live)" 2
        log_hint "Set FORGE_RELEASE_REQUIRED=1 to treat this as a failure" 2
    fi
fi

echo ""

# Fold any per-item drift (recorded in the subshell-safe sink) into $failed.
if [ -s "$drift_sink" ]; then
    failed=1
fi

# ==============================================================================
# Summary
# ==============================================================================
if [ "$failed" -eq 0 ]; then
    log_ok "doctor-forge: live forge configuration matches spec"
    exit 0
else
    log_fail "doctor-forge: live forge configuration drifts from spec — see above"
    exit 1
fi
