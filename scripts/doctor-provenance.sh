#!/bin/sh
# ==============================================================================
# scripts/doctor-provenance.sh
# Repo-provenance audit for main's signed history.
#
# Two tiers, distinct trust properties (see docs/dev/tooling.md, SECURITY.md):
#
#   Tier 1 - signature provenance (portable; git + committed keys)
#     Every commit reachable from `main` (and every annotated tag) must be
#     signed by a maintainer PRIMARY key that was allowlisted AS OF that object
#     -- checked against the maintainer-keys ledger as it existed at that point
#     in history, never the current tip. The empty signed root is anchored to a
#     founding primary fingerprint; the ledger self-authenticates forward.
#       --root-fpr <FPR>  independent audit: the chain must terminate at an
#                         externally-obtained primary fingerprint.
#       (default)         self-consistency: anchor on the repo's OWN declared
#                         founding key -- useful, but NOT an independent verdict,
#                         since a full history rewrite rewrites that key too.
#
#   Tier 2 - CI round-trip (GitHub query; degrades gracefully)
#     For each first-parent merge, report whether it reached `main` through the
#     staging CI round-trip or by direct push -- detected by whether a
#     Continuous Integration run exists whose head_branch is a staging-* branch
#     (CI runs green on direct pushes too, so the gate result cannot tell them
#     apart). Skipped with a warning when `gh` is unavailable/unauthenticated.
#
# Matching is on the maintainer's PRIMARY key (identity), not the signing key,
# so it is agnostic to whether a maintainer signs with their primary or a
# subkey: the effective primary is %GP when present (subkey signature) and
# falls back to %GF (a primary signature, where the signer IS the primary).
# Rotating a signing subkey then needs no allowlist change, only a refreshed .asc.
# Signature status is read from `git log` %G?: G/U (good), X/Y (good, key/sig
# expired SINCE signing -> accepted as-of), R (revoked -> flagged), else fail.
#
# Usage: scripts/doctor-provenance.sh [--root-fpr <FPR>] [--since <ref>] [--no-ci]
# Exit:  0 = clean; 1 = a provenance violation was found; 2 = usage/config error.
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

KEYS_PATH=".github/maintainer-keys"
BRANCH="main"
CI_WORKFLOW="Continuous Integration"
ROOT_FPR=""
SINCE=""
RUN_TIER2=1

usage() {
    echo "Usage: scripts/doctor-provenance.sh [--root-fpr <FPR>] [--since <ref>] [--no-ci]" >&2
}

while [ $# -gt 0 ]; do
    case "$1" in
        --root-fpr) [ $# -ge 2 ] || { usage; exit 2; }
                    ROOT_FPR=$(printf '%s' "$2" | tr -d ' ' | tr '[:lower:]' '[:upper:]'); shift 2 ;;
        --since)    [ $# -ge 2 ] || { usage; exit 2; }
                    SINCE="$2"; shift 2 ;;
        --no-ci)    RUN_TIER2=0; shift ;;
        -h|--help)  usage; exit 0 ;;
        *)          log_err "unknown argument: $1"; usage; exit 2 ;;
    esac
done

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

# --- gpg helpers --------------------------------------------------------------
# Import every .asc blob in the ledger at tree-ish $2 into GNUPGHOME $1.
import_ledger() {  # $1 = gnupghome ; $2 = commit/tree
    git ls-tree -r "$2" -- "$KEYS_PATH" 2>/dev/null | awk '$2=="blob"{print $3}' \
    | while read -r _blob; do
        [ -n "$_blob" ] || continue
        git cat-file blob "$_blob" | GNUPGHOME="$1" gpg --batch --quiet --import 2>/dev/null || true
    done
}

# Print PRIMARY key fingerprints held in GNUPGHOME $1 (the fpr line after pub:).
primary_fprs() {  # $1 = gnupghome
    GNUPGHOME="$1" gpg --batch --with-colons --list-keys 2>/dev/null | awk -F: '
        /^pub:/ { want=1; next }
        /^sub:/ { want=0; next }
        /^fpr:/ { if (want) { print $10; want=0 } }
    '
}

# ==============================================================================
# Tier 1
# ==============================================================================
log_section "Tier 1 - signature provenance (as-of the maintainer allowlist)"
echo ""

# Epoch boundaries: commits that changed the ledger. Oldest-first and newest-first.
BOUNDARIES_OLD=$(git rev-list --reverse "$BRANCH" -- "$KEYS_PATH")
BOUNDARIES_NEW=$(git rev-list "$BRANCH" -- "$KEYS_PATH")

if [ -z "$BOUNDARIES_OLD" ]; then
    log_fatal "No ${KEYS_PATH} history on ${BRANCH}; cannot reconstruct the roster."
fi
FIRST_BOUNDARY=$(printf '%s\n' "$BOUNDARIES_OLD" | head -1)

# Union keyring: every key ever in the ledger, so gpg can crypto-verify any
# commit. Authorisation is enforced separately, per-epoch, on the primary fpr.
UNION="$WORK/union"; mkdir -m 700 "$UNION"
for b in $BOUNDARIES_OLD; do
    import_ledger "$UNION" "$b"
done

# Per-epoch allowed PRIMARY fingerprints: "<boundary> <fpr>" lines.
: > "$WORK/allowed"
for b in $BOUNDARIES_OLD; do
    ering="$WORK/ering-$b"; mkdir -m 700 "$ering"
    import_ledger "$ering" "$b"
    primary_fprs "$ering" | sed "s/^/$b /" >> "$WORK/allowed"
done

# The trust anchor. Independent audit pins an out-of-band fpr; otherwise the
# self-declared founding key (the root commit's own primary signer).
ROOT_COMMIT=$(git rev-list --max-parents=0 "$BRANCH" | head -1)
_rp=$(GNUPGHOME="$UNION" git log -1 --format='%GP' "$ROOT_COMMIT" 2>/dev/null || true)
_rf=$(GNUPGHOME="$UNION" git log -1 --format='%GF' "$ROOT_COMMIT" 2>/dev/null || true)
ROOT_SIGNER=${_rp:-$_rf}
if [ -n "$ROOT_FPR" ]; then
    AXIOM="$ROOT_FPR"; MODE="independent audit (--root-fpr)"
else
    AXIOM="$ROOT_SIGNER"; MODE="self-consistency (repo's self-declared root)"
fi
printf 'PRELEDGER %s\n' "$AXIOM" >> "$WORK/allowed"

if [ -z "$AXIOM" ]; then
    log_fatal "Could not determine a trust anchor for the root commit."
fi

# Continuity: the founding ledger must contain the anchor, so trust is unbroken
# from the empty root into the versioned key ledger.
if ! grep -qxF "$FIRST_BOUNDARY $AXIOM" "$WORK/allowed"; then
    log_fail "Continuity broken: founding ledger (${FIRST_BOUNDARY}) does not contain anchor ${AXIOM}."
    exit 1
fi

# Map every commit to its epoch: the LATEST ledger-change that is an ancestor of
# it (or itself). Process boundaries newest-first and claim each boundary plus
# its descendants (`^b` excludes b's ancestors, leaving the commits at/after b);
# the newest claim wins, so a commit lands in the epoch whose ledger was current
# when it was made. The remainder precede the ledger and anchor to the axiom.
: > "$WORK/claimed"; : > "$WORK/map"
for b in $BOUNDARIES_NEW; do
    { echo "$b"; git rev-list "$BRANCH" "^$b"; } | grep -vxF -f "$WORK/claimed" > "$WORK/e" || true
    cat "$WORK/e" >> "$WORK/claimed"
    sed "s/\$/ $b/" "$WORK/e" >> "$WORK/map"
done
git rev-list "$BRANCH" | grep -vxF -f "$WORK/claimed" | sed 's/$/ PRELEDGER/' >> "$WORK/map"

# One batched verify pass: sha|status|primary|signer. Effective primary = the
# primary field when present, else the signer (a direct primary signature).
RANGE="$BRANCH"
[ -n "$SINCE" ] && RANGE="${SINCE}..${BRANCH}"
GNUPGHOME="$UNION" git log --format='%H|%G?|%GP|%GF' "$RANGE" > "$WORK/sig"

AUDITED=$(wc -l < "$WORK/sig" | tr -d ' ')
log_info "Anchor: ${AXIOM}" 1
log_info "Mode:   ${MODE}" 1
log_info "Scope:  ${AUDITED} commits, $(printf '%s\n' "$BOUNDARIES_OLD" | wc -l | tr -d ' ') roster epoch(s)" 1
echo ""

# Revocation reasons for revoked keys (primary fpr -> RFC-4880 reason code).
# NB the RFC packet codes are NOT gpg's interactive menu numbers: 0x01=superseded
# and 0x03=retired are historic (the key was legitimately the maintainer's, so its
# in-epoch commits stay as-of valid); 0x00=unspecified and 0x02=compromised are
# flagged for investigation. Revoked keys are rare, so this loop is usually empty.
: > "$WORK/revreason"
GNUPGHOME="$UNION" gpg --batch --with-colons --list-keys 2>/dev/null | awk -F: '
    /^pub:/ { rvk = ($2 == "r") }
    /^sub:/ { rvk = 0 }
    /^fpr:/ && rvk { print $10; rvk = 0 }
' | while read -r rf; do
    rc=$(GNUPGHOME="$UNION" gpg --export "$rf" 2>/dev/null | GNUPGHOME="$UNION" gpg --list-packets 2>/dev/null \
         | sed -n 's/.*revocation reason 0x\([0-9A-Fa-f][0-9A-Fa-f]\).*/\1/p' | head -1)
    printf '%s %s\n' "$rf" "${rc:-00}" >> "$WORK/revreason"
done

# Adjudicate: allowed[boundary" "fpr] gates authorisation; %G? gates crypto; a
# revocation is as-of valid only for a superseded/retired reason, else flagged.
awk '
    FILENAME ~ /allowed$/   { allow[$1" "$2]=1; next }
    FILENAME ~ /map$/       { epoch[$1]=$2; next }
    FILENAME ~ /revreason$/ { revreason[$1]=$2; next }
    {
        split($0, a, "|"); c=a[1]; s=a[2]; f=(a[3] != "" ? a[3] : a[4]); b=epoch[c]
        good_sig = (s=="G" || s=="U" || s=="X" || s=="Y")
        if (s=="R") {
            rc = revreason[f]
            if (rc == "01" || rc == "03") {          # superseded / retired: as-of
                if (allow[b" "f]) { ok++; next }
                bad++; print "UNAUTHORISED "substr(c,1,12)" (superseded key, not allowlisted as-of)" > "/dev/stderr"; next
            }
            rev++; print "REVOKED    "substr(c,1,12)" (reason 0x"rc"; investigate)" > "/dev/stderr"; next
        }
        if (good_sig && allow[b" "f]) { ok++; next }
        bad++
        if (!good_sig) print "UNVERIFIED "substr(c,1,12)" (status "s")" > "/dev/stderr"
        else           print "UNAUTHORISED "substr(c,1,12)" (primary "f" not allowlisted as-of)" > "/dev/stderr"
    }
    END { printf "%d %d %d\n", ok+0, rev+0, bad+0 }
' "$WORK/allowed" "$WORK/map" "$WORK/revreason" "$WORK/sig" > "$WORK/tally"

read -r T1_OK T1_REV T1_BAD < "$WORK/tally"

# Annotated tags carry their own signed object (the publish chain keys off signed
# release tags), so each must be verified in its own right; lightweight tags are
# just refs to already-audited commits. Verify each annotated tag against the
# allowlist as of its target commit's epoch, via the VALIDSIG primary fingerprint.
: > "$WORK/tagbad"
git for-each-ref --format='%(refname:short) %(objecttype)' refs/tags 2>/dev/null | while read -r tag ttype; do
    [ "$ttype" = "tag" ] || continue
    ttarget=$(git rev-list -n 1 "$tag" 2>/dev/null)
    tepoch=$(awk -v c="$ttarget" '$1==c{print $2}' "$WORK/map")
    traw=$(GNUPGHOME="$UNION" git verify-tag --raw "$tag" 2>&1 || true)
    tprimary=$(printf '%s\n' "$traw" | awk '/VALIDSIG/{print $NF}' | head -1)
    if [ -n "$tprimary" ] && grep -qxF "${tepoch} ${tprimary}" "$WORK/allowed"; then
        continue
    fi
    log_fail "tag ${tag}: not signed by an as-of-allowlisted maintainer key" 1
    echo "$tag" >> "$WORK/tagbad"
done
TAG_N=$(git for-each-ref --format='%(objecttype)' refs/tags 2>/dev/null | grep -c '^tag$' || true)
TAG_BAD=$(wc -l < "$WORK/tagbad" | tr -d ' ')
T1_BAD=$((T1_BAD + TAG_BAD))

if [ "$T1_BAD" -eq 0 ] && [ "$T1_REV" -eq 0 ]; then
    log_ok "Tier 1: ${T1_OK} commit(s) + ${TAG_N} tag(s) signed by an as-of-allowlisted primary key" 1
else
    [ "$T1_REV" -gt 0 ] && log_warn "${T1_REV} commit(s) signed by a revoked key (see above; apply revocation policy)" 1
    [ "$T1_BAD" -gt 0 ] && log_fail "${T1_BAD} object(s) failed the as-of signature check (see above)" 1
fi
echo ""

# ==============================================================================
# Tier 2
# ==============================================================================
T2_BYPASS=0
if [ "$RUN_TIER2" -eq 1 ]; then
    log_section "Tier 2 - CI round-trip (staging vs direct push)"
    echo ""
    if ! command -v gh >/dev/null 2>&1 || ! gh auth status >/dev/null 2>&1; then
        log_warn "gh unavailable or unauthenticated; skipping Tier 2 (Tier 1 stands)." 1
    else
        SLUG=$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || true)
        if [ -z "$SLUG" ]; then
            log_warn "Could not resolve the GitHub repository; skipping Tier 2." 1
        else
            git rev-list --first-parent --merges "$RANGE" > "$WORK/merges"
            export SLUG CI_WORKFLOW
            # One gh call per merge, parallel; a staging-* CI run => proper.
            # $-refs below expand in the child sh (env + positional), by design.
            # shellcheck disable=SC2016
            xargs -P 8 -n1 sh -c '
                sha="$1"
                st=$(gh api "repos/${SLUG}/actions/runs?head_sha=${sha}&per_page=50" \
                     --jq "[.workflow_runs[] | select(.name==\"${CI_WORKFLOW}\") | .head_branch | select(startswith(\"staging-\"))] | .[0] // \"\"" \
                     2>/dev/null || true)
                [ -n "$st" ] && printf "%s PROPER\n" "$sha" || printf "%s BYPASS\n" "$sha"
            ' _ < "$WORK/merges" > "$WORK/t2"
            T2_TOTAL=$(wc -l < "$WORK/t2" | tr -d ' ')
            T2_BYPASS=$(grep -c ' BYPASS$' "$WORK/t2" || true)
            grep ' BYPASS$' "$WORK/t2" | while read -r sha _; do
                log_item "direct push, no staging CI round-trip: $(git log -1 --format='%h %s' "$sha")" 1
            done
            if [ "$T2_BYPASS" -eq 0 ]; then
                log_ok "Tier 2: all ${T2_TOTAL} merges reached main via the staging CI round-trip" 1
            else
                log_info "Tier 2: ${T2_BYPASS} of ${T2_TOTAL} merges reached main by direct push (disclosed above; not a failure)" 1
            fi
        fi
    fi
    echo ""
fi

# ==============================================================================
# Summary
# ==============================================================================
if [ "$T1_BAD" -gt 0 ]; then
    log_fail "doctor-provenance: ${T1_BAD} signature violation(s) on main (see above)"
    exit 1
fi
if [ "$T1_REV" -gt 0 ] || [ "$T2_BYPASS" -gt 0 ]; then
    log_ok "doctor-provenance: history cryptographically sound; disclosed above: ${T1_REV} revoked-key commit(s), ${T2_BYPASS} CI-bypass merge(s)"
    exit 0
fi
log_ok "doctor-provenance: main's history is signed as-of the allowlist and fully CI-routed"
exit 0
