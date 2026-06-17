#!/bin/sh
# ==============================================================================
# scripts/doctor-registry.sh
# Registry (crates.io) configuration diagnostics (READ-ONLY)
#
# Asserts that the LIVE crates.io Trusted Publishing (TP) configuration matches
# the desired spec in scripts/lib/registry-spec.sh (the machine-readable
# realisation of docs/project/registry-setup.md). This script NEVER mutates
# registry state — it only reads. To converge drift, use scripts/registry-tp.sh,
# which PRINTS the exact register/enforce commands for a human to run (this
# tooling never holds a crates.io token).
#
# PLANE: crates.io is a REGISTRY, not a forge. This is the registry sibling of
# doctor-forge.sh and shares its two-tier shape.
#
# Two tiers:
#   OFFLINE — local/git/file checks that need no registry auth. ALWAYS run:
#     publish.yml present + its environment is the gating `release` env; each spec
#     crate manifest exists, is publishable, and declares the publish-required
#     metadata (license / repository / rust-version); the release.toml tag glob
#     still matches publish.yml's trigger.
#   ONLINE  — checks that query crates.io. Gated behind a token probe: if
#     CRATES_IO_TOKEN is absent, this tier is SKIPPED with a warning + hint and
#     the script still exits 0 (the offline tier having run). With a token,
#     online drift fails the run (exit 1).
#
# Exit: 0 = all run checks matched (or online tier skipped for missing token);
#       1 = a check that ran found drift.
#
# Usage: scripts/doctor-registry.sh
#   CRATES_IO_TOKEN              a crates.io API token (Account -> API Tokens),
#                               read-only use here; absent -> online tier skipped.
#                               NOT CARGO_REGISTRY_TOKEN (that holds the pipeline's
#                               publish-only OIDC token, which the management API
#                               rejects).
#   REGISTRY_ENFORCEMENT_REQUIRED=1  treat a crate whose trustpub_only is still
#                               false as a FAILURE (default: warn — enforcement is
#                               intentionally the last bootstrap step, enabled only
#                               after a TP publish is proven).
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"
# registry-spec borrows repo/env identity from forge-spec, so source forge-spec
# FIRST (registry_spec_tp_repo/_environment call forge_spec_* helpers).
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/forge-spec.sh"
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/registry-spec.sh"

CRATES_API="https://crates.io/api/v1"
# crates.io requires a descriptive User-Agent on every API request.
CRATES_UA="sdmx-rs-doctor-registry (https://github.com/dgalbraith/sdmx-rs)"

log_section "Registry Configuration Diagnostics"
echo ""

failed=0

# Drift sinks for the per-crate `... | while read` loops (POSIX runs each loop
# body in a SUBSHELL, so a flag set there would not survive). Offline and online
# tiers each fold their sink into $failed after the loop. mktemp + trap (same
# idiom as doctor-forge); created up front so both tiers can use them.
offline_sink="$(mktemp "${TMPDIR:-/tmp}/doctor-registry.offline.XXXXXX")"
drift_sink="$(mktemp "${TMPDIR:-/tmp}/doctor-registry.drift.XXXXXX")"
trap 'rm -f "$offline_sink" "$drift_sink"' EXIT INT TERM

# ==============================================================================
# OFFLINE TIER — no registry auth required; always runs.
# ==============================================================================
log_section "Offline checks (local / git / files)"
echo ""

# --- Resolve OWNER/REPO + environment (for messaging + later TP matching) ------
if owner_repo="$(registry_spec_tp_repo)"; then
    log_ok "Repository slug: $owner_repo" 1
else
    log_fail "Could not derive OWNER/REPO from origin remote" 1
    owner_repo=""
    failed=1
fi
tp_workflow="$(registry_spec_tp_workflow)"
tp_environment="$(registry_spec_tp_environment)"

# --- Offline 1: publish.yml present + bound to the gating environment ----------
publish_wf=".github/workflows/publish.yml"
if [ -f "$publish_wf" ]; then
    log_ok "Publish workflow present: $publish_wf" 1
    # The TP environment must be the gating `release` env — assert publish.yml's
    # publish job declares exactly that environment.
    if grep -qE "^[[:space:]]*environment:[[:space:]]*${tp_environment}([[:space:]]|\$)" "$publish_wf"; then
        log_ok "publish.yml binds environment: $tp_environment" 1
    else
        log_fail "publish.yml does not declare 'environment: $tp_environment'" 1
        failed=1
    fi
else
    log_fail "Publish workflow missing: $publish_wf" 1
    failed=1
fi

# --- Offline 2: release.toml tag glob still matches publish.yml's trigger ------
# publish.yml triggers on 'sdmx-*/v*'; release.toml stamps '{{crate_name}}/v{{version}}'.
# A drift between them means tags would be pushed that never trigger a publish.
if [ -f release.toml ] && [ -f "$publish_wf" ]; then
    if grep -qE 'tag-name[[:space:]]*=[[:space:]]*"\{\{crate_name\}\}/v\{\{version\}\}"' release.toml \
        && grep -qE "tags:[[:space:]]*\[[[:space:]]*'sdmx-\*/v\*'" "$publish_wf"; then
        log_ok "release.toml tag convention matches publish.yml trigger glob" 1
    else
        log_warn "release.toml tag-name / publish.yml tag glob may have drifted — verify by hand" 1
    fi
fi

# --- Offline 3: per-crate manifest sanity (publishable + required metadata) ----
# Every spec crate must have a manifest, must NOT be publish=false (it is meant to
# go to the registry), and must carry the publish-required metadata that crates.io
# enforces. Workspace-inherited keys (license/repository) count, so accept either
# a direct key or a `<key>.workspace = true`.
registry_spec_crates | while IFS= read -r crate; do
    manifest="crates/${crate}/Cargo.toml"
    if [ ! -f "$manifest" ]; then
        log_fail "Crate manifest missing: $manifest" 2
        echo "drift" >> "$offline_sink"
        continue
    fi
    if grep -qE '^[[:space:]]*publish[[:space:]]*=[[:space:]]*false' "$manifest"; then
        log_fail "$crate is publish=false but is in the registry spec" 2
        echo "drift" >> "$offline_sink"
        continue
    fi
    _missing=""
    for key in license repository rust-version; do
        if ! grep -qE "^[[:space:]]*${key}([[:space:]]*=|\\.workspace[[:space:]]*=[[:space:]]*true)" "$manifest"; then
            _missing="${_missing} ${key}"
        fi
    done
    if [ -n "$_missing" ]; then
        log_fail "$crate missing publish metadata:${_missing}" 2
        echo "drift" >> "$offline_sink"
    else
        log_ok "$crate: publishable, metadata present" 2
    fi
done

# Fold offline per-crate drift (recorded in the subshell-safe sink) into $failed.
if [ -s "$offline_sink" ]; then
    failed=1
fi

echo ""

# ==============================================================================
# ONLINE TIER — gated behind a CRATES_IO_TOKEN probe.
# ==============================================================================
log_section "Online checks (live crates.io state)"
echo ""

if ! command -v curl >/dev/null 2>&1; then
    log_warn "curl not found — skipping online registry checks"
    log_hint "Install curl to verify live crates.io Trusted Publishing state"
    echo ""
    if [ "$failed" -eq 0 ]; then
        log_ok "doctor-registry: offline checks passed (online tier skipped — no curl)"
        exit 0
    fi
    log_fail "doctor-registry: offline checks found drift — see above"
    exit 1
fi

if [ -z "${CRATES_IO_TOKEN:-}" ]; then
    log_warn "CRATES_IO_TOKEN not set — skipping online registry checks"
    log_hint "Mint a token at https://crates.io/settings/tokens and export CRATES_IO_TOKEN"
    log_hint "Use a minimal-scope, short-lived token and revoke it after setup"
    echo ""
    if [ "$failed" -eq 0 ]; then
        log_ok "doctor-registry: offline checks passed (online tier skipped — no token)"
        exit 0
    fi
    log_fail "doctor-registry: offline checks found drift — see above"
    exit 1
fi

if [ -z "$owner_repo" ]; then
    log_fatal "Cannot run online checks without a resolved OWNER/REPO"
fi
spec_owner="${owner_repo%%/*}"
spec_name="${owner_repo##*/}"

# crates_api <path> — GET the crates.io API, echo the body, return curl's status.
# Authenticated, descriptive UA. Read-only: this function never POSTs/PATCHes.
crates_api() {
    curl -sS \
        -H "Authorization: ${CRATES_IO_TOKEN}" \
        -H "User-Agent: ${CRATES_UA}" \
        "${CRATES_API}/$1"
}

# crate_is_indexed <crate> — true if the crate has >=1 published version (i.e.
# the name is reserved). Uses the public sparse index (no auth), same source as
# scripts/ci/check-published.sh.
crate_is_indexed() {
    _c="$1"
    case "${#_c}" in
        1) _p="1/${_c}" ;;
        2) _p="2/${_c}" ;;
        3) _p="3/$(printf '%s' "$_c" | cut -c1)/${_c}" ;;
        *) _p="$(printf '%s' "$_c" | cut -c1-2)/$(printf '%s' "$_c" | cut -c3-4)/${_c}" ;;
    esac
    _st="$(curl -sS -o /dev/null -w '%{http_code}' \
        -H "User-Agent: ${CRATES_UA}" "https://index.crates.io/${_p}" 2>/dev/null || echo 000)"
    [ "$_st" = "200" ]
}

want_enforce="$(registry_spec_enforcement)"

log_info "Trusted Publishing configs + enforcement" 1
registry_spec_crates | while IFS= read -r crate; do
    # Name must exist before a TP config can be attached (crates.io has no
    # pending-publisher feature). Unreserved is the EXPECTED pre-bootstrap state.
    if ! crate_is_indexed "$crate"; then
        log_warn "$crate: not yet reserved on crates.io (run the bootstrap publish)" 2
        continue
    fi

    cfg_json="$(crates_api "trusted_publishing/github_configs?crate=${crate}" 2>/dev/null || true)"
    # Count configs matching the FULL spec binding (owner/name/workflow/env). A
    # stray/extra config pointing at the wrong repo or workflow is the real
    # supply-chain threat, so assert EXACTLY ONE correct config.
    match_n="$(printf '%s' "$cfg_json" | jq -r --arg o "$spec_owner" --arg n "$spec_name" \
        --arg w "$tp_workflow" --arg e "$tp_environment" \
        '[.github_configs[]? | select(.repository_owner == $o and .repository_name == $n
            and .workflow_filename == $w and (.environment // "") == $e)] | length' 2>/dev/null || echo "")"
    total_n="$(printf '%s' "$cfg_json" | jq -r '(.github_configs // []) | length' 2>/dev/null || echo "")"

    case "$match_n" in
        ''|*[!0-9]*)
            log_fail "$crate: could not read Trusted Publishing configs" 2
            echo "drift" >> "$drift_sink" ;;
        0)
            log_fail "$crate: no matching Trusted Publisher (want $spec_owner/$spec_name $tp_workflow @ $tp_environment)" 2
            echo "drift" >> "$drift_sink" ;;
        1)
            if [ "$total_n" = "1" ]; then
                log_ok "$crate: Trusted Publisher matches spec" 2
            else
                log_fail "$crate: extra Trusted Publishing config(s) present ($total_n total) — expected exactly one" 2
                echo "drift" >> "$drift_sink"
            fi ;;
        *)
            log_fail "$crate: $match_n matching TP configs — expected exactly one" 2
            echo "drift" >> "$drift_sink" ;;
    esac

    # Enforcement state (trustpub_only). Not-yet-enforced is a warn by default
    # (it is intentionally the LAST bootstrap step), a failure under the opt-in.
    crate_json="$(crates_api "crates/${crate}" 2>/dev/null || true)"
    # Use an explicit null test, NOT `// empty`: in jq `false // empty` yields
    # empty, which would erase a legitimately-unenforced `false` into "".
    got_enforce="$(printf '%s' "$crate_json" \
        | jq -r 'if (.crate.trustpub_only == null) then "" else (.crate.trustpub_only | tostring) end' 2>/dev/null || echo "")"
    if [ "$got_enforce" = "$want_enforce" ]; then
        log_ok "$crate: trustpub_only=$got_enforce (enforced)" 2
    elif [ "${REGISTRY_ENFORCEMENT_REQUIRED:-0}" = "1" ]; then
        log_fail "$crate: trustpub_only=$got_enforce (want $want_enforce; REGISTRY_ENFORCEMENT_REQUIRED=1)" 2
        echo "drift" >> "$drift_sink"
    else
        log_warn "$crate: trustpub_only=$got_enforce — enforcement deferred until a TP publish is proven" 2
        log_hint "Set REGISTRY_ENFORCEMENT_REQUIRED=1 to treat this as a failure" 2
    fi
done

echo ""

if [ -s "$drift_sink" ]; then
    failed=1
fi

# ==============================================================================
# Summary
# ==============================================================================
if [ "$failed" -eq 0 ]; then
    log_ok "doctor-registry: live registry configuration matches spec"
    exit 0
else
    log_fail "doctor-registry: live registry configuration drifts from spec — see above"
    exit 1
fi
