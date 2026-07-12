#!/bin/sh
# ==============================================================================
# scripts/registry-tp.sh
# Trusted Publishing setup command helper (PRINT-ONLY — NEVER mutates)
#
# Prints the exact commands to register Trusted Publishers and to enable
# enforcement on crates.io, plus the manual name-reservation commands, for the
# crates named in scripts/lib/registry-spec.sh. It checks preconditions and emits
# ready-to-run snippets — the MAINTAINER runs them by hand.
#
# This tool NEVER issues a mutating request and NEVER holds a long-lived crates.io
# token: by design our tooling is incapable of touching crates.io state. The
# irreversible acts (register / publish / enable enforcement) stay entirely in the
# human's hands at a shell prompt — the strongest posture for a public registry
# where a published version cannot be unpublished and enforcement removes the
# token escape hatch. There is NO --execute path.
#
# It MAY use a read-only ${CRATES_IO_TOKEN}, if present, to query existing TP
# configs so it can skip already-registered crates — but it issues no POST/PATCH.
#
# Maintainer-only, runbook-documented (docs/project/registry-setup.md), NOT a
# `just` recipe — same class as scripts/forge-apply.sh (a guarded one-shot
# bootstrap tool kept off the routine recipe surface; see the tooling-placement
# policy in docs/dev/tooling.md).
#
# Usage:
#   scripts/registry-tp.sh [--print-register | --print-enforce]
#
#   --print-register  (default) Print the `cargo publish` name-reservation
#                     commands for unreserved crates, and the TP-registration curl
#                     for crates without a matching config. Skips crates already
#                     correctly registered (requires CRATES_IO_TOKEN to detect).
#   --print-enforce   Print the enforcement (trustpub_only) curl per crate — only
#                     after asserting the preconditions: the crate is published
#                     (>=1 indexed version) and a matching Trusted Publisher is
#                     registered.
#
# Requires: jq; curl. CRATES_IO_TOKEN optional (improves --print-register skip
# accuracy; mint a minimal-scope, short-lived token and revoke it as soon as the
# task is done).
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"
# registry-spec borrows repo/env identity from forge-spec — source forge-spec first.
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/forge-spec.sh"
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/registry-spec.sh"

CRATES_API="https://crates.io/api/v1"
CRATES_UA="sdmx-rs-registry-tp (https://github.com/dgalbraith/sdmx-rs)"

# --- Argument parsing ---------------------------------------------------------
mode="register"
for _arg in "$@"; do
    case "$_arg" in
        --print-register) mode="register" ;;
        --print-enforce)  mode="enforce" ;;
        *) log_fatal "Unknown argument: $_arg (use --print-register or --print-enforce)" ;;
    esac
done
unset _arg

command -v jq   >/dev/null 2>&1 || log_fatal "jq not found — required to read TP configs"
command -v curl >/dev/null 2>&1 || log_fatal "curl not found — required to query crates.io"

if ! owner_repo="$(registry_spec_tp_repo)"; then
    log_fatal "Could not derive OWNER/REPO from origin remote"
fi
spec_owner="${owner_repo%%/*}"
spec_name="${owner_repo##*/}"
tp_workflow="$(registry_spec_tp_workflow)"
tp_environment="$(registry_spec_tp_environment)"

# --- Helpers (READ-ONLY) ------------------------------------------------------

# crate_indexed <crate> — true if the crate has >=1 published version (name
# reserved). Public sparse index, no auth (same source as ci/check-published.sh).
crate_indexed() {
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

# has_matching_config <crate> — echo the count of TP configs matching the full
# spec binding. Echoes "?" if it cannot tell (no token / read error), so callers
# never treat "unknown" as "absent". READ-ONLY (GET).
has_matching_config() {
    [ -n "${CRATES_IO_TOKEN:-}" ] || { printf '?'; return 0; }
    _body="$(curl -sS \
        -H "Authorization: ${CRATES_IO_TOKEN}" \
        -H "User-Agent: ${CRATES_UA}" \
        "${CRATES_API}/trusted_publishing/github_configs?crate=$1" 2>/dev/null || true)"
    _n="$(printf '%s' "$_body" | jq -r --arg o "$spec_owner" --arg n "$spec_name" \
        --arg w "$tp_workflow" --arg e "$tp_environment" \
        '[.github_configs[]? | select(.repository_owner == $o and .repository_name == $n
            and .workflow_filename == $w and (.environment // "") == $e)] | length' 2>/dev/null || echo '?')"
    case "$_n" in ''|*[!0-9]*) printf '?' ;; *) printf '%s' "$_n" ;; esac
}

# --- register mode ------------------------------------------------------------
print_register() {
    log_section "Trusted Publisher registration commands"
    echo ""
    log_info "For each crate: reserve the name (if needed), then register the publisher." 1
    log_hint "Run the printed commands yourself — this script never mutates crates.io." 1
    [ -n "${CRATES_IO_TOKEN:-}" ] || log_hint "Set CRATES_IO_TOKEN to skip already-registered crates." 1
    echo ""

    registry_spec_crates | while IFS= read -r crate; do
        log_item "$crate" 1
        if ! crate_indexed "$crate"; then
            echo "      # 1. Reserve the name (manual, long-lived token — one-time):"
            echo "      cargo publish --manifest-path crates/${crate}/Cargo.toml --token <TOKEN>"
        fi
        _have="$(has_matching_config "$crate")"
        if [ "$_have" = "1" ]; then
            echo "      # Trusted Publisher already registered — nothing to do."
            continue
        fi
        if [ "$_have" != "0" ]; then
            echo "      # (Could not confirm existing config — set CRATES_IO_TOKEN to check; command shown for completeness.)"
        fi
        echo "      # 2. Register the Trusted Publisher (run in your authenticated crates.io session):"
        echo "      curl -X POST \"${CRATES_API}/trusted_publishing/github_configs\" \\"
        echo "        -H \"Authorization: \$CRATES_IO_TOKEN\" \\"
        echo "        -H \"User-Agent: ${CRATES_UA}\" \\"
        echo "        -H 'Content-Type: application/json' \\"
        echo "        -d '{\"github_config\":{\"crate\":\"${crate}\",\"repository_owner\":\"${spec_owner}\",\"repository_name\":\"${spec_name}\",\"workflow_filename\":\"${tp_workflow}\",\"environment\":\"${tp_environment}\"}}'"
    done
    echo ""
    log_hint "Then run 'just doctor-registry' to verify each registration." 1
}

# --- enforce mode -------------------------------------------------------------
print_enforce() {
    log_section "Trusted Publishing enforcement commands"
    echo ""
    log_warn "Enforcement is already enabled for the family's crates. Enforcing a" 1
    log_warn "future crate disables its API-token publishing; the recovery path is the" 1
    log_warn "owner toggling the setting off in the crates.io web UI (documented in" 1
    log_warn "the releasing.md bootstrap record)." 1
    echo ""

    registry_spec_crates | while IFS= read -r crate; do
        log_item "$crate" 1
        # Preconditions: the crate is published (>=1 indexed version) AND a
        # matching Trusted Publisher is registered. Refuse to print otherwise.
        if ! crate_indexed "$crate"; then
            echo "      # SKIP — not yet published; publish the crate first."
            continue
        fi
        _have="$(has_matching_config "$crate")"
        if [ "$_have" = "0" ]; then
            echo "      # SKIP — no matching Trusted Publisher registered yet (register first)."
            continue
        fi
        if [ "$_have" = "?" ]; then
            echo "      # NOTE — could not confirm the TP config (set CRATES_IO_TOKEN). Verify before running:"
        fi
        echo "      # Enable enforcement (run in your authenticated crates.io session):"
        echo "      curl -X PATCH \"${CRATES_API}/crates/${crate}\" \\"
        echo "        -H \"Authorization: \$CRATES_IO_TOKEN\" \\"
        echo "        -H \"User-Agent: ${CRATES_UA}\" \\"
        echo "        -H 'Content-Type: application/json' \\"
        echo "        -d '{\"crate\":{\"trustpub_only\":true}}'"
    done
    echo ""
    log_hint "Then verify with: just doctor-registry" 1
}

case "$mode" in
    register) print_register ;;
    enforce)  print_enforce ;;
esac
