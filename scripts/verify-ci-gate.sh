#!/bin/sh
# ==============================================================================
# scripts/verify-ci-gate.sh
#
# Cross-checks the CI Quality Gate's coverage on three fronts:
#   1. the `ci-gate` job's `needs:` list in .github/workflows/ci.yml MUST equal
#      the gating set declared in forge/github/ci-gating-jobs.json;
#   2. every job named in the manifest MUST actually exist as a job in ci.yml
#      (no ghost gates); and
#   3. every gating job MUST be documented in docs/project/ci-gating.md as a
#      `#### <job>` Check Details heading (manifest is a SUBSET of documented —
#      the doc legitimately documents more, e.g. the deliberately-excluded jobs).
#
# WHY a separate manifest: the manifest is the declared INTENT (what must gate
# main); the workflow `needs:` is the EXECUTION (what actually gates). Verifying
# them against each other means a stray edit to `needs:` — dropping a security
# gate, or adding an undeclared one — fails CI instead of silently weakening the
# Zero Trust Gate. The duplication is the safety net, not an accident.
#
# `changes` IS part of the gating set (the gate asserts changes==success), so it
# appears in BOTH the manifest and `needs:`. `ci-gate` itself appears in NEITHER
# (a gate cannot gate itself).
#
# NOTE: ci.yml job-key extraction depends on the canonical 2-space job
# indentation under `jobs:`. The `needs:` extraction reads the literal block
# between `  ci-gate:` and the job's `    steps:`.
#
# POSIX sh; uses jq (manifest parse) + grep/sed/sort/comm. No bashisms.
#
# Exit codes:
#   0 = needs:, manifest, and ci-gating.md all in sync; no ghost gates
#   1 = drift, ghost gate, undocumented gating job, or a missing input
# ==============================================================================

set -u

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

MANIFEST="forge/github/ci-gating-jobs.json"
CI=".github/workflows/ci.yml"
DOC="docs/project/ci-gating.md"

log_section "Verifying CI Quality Gate coverage..."

if [ ! -f "$MANIFEST" ]; then
    log_fail "Manifest not found: $MANIFEST"
    exit 1
fi
if [ ! -f "$CI" ]; then
    log_fail "Workflow not found: $CI"
    exit 1
fi
if [ ! -f "$DOC" ]; then
    log_fail "Gate documentation not found: $DOC"
    exit 1
fi

# Temp files (POSIX: no process substitution). LC_ALL=C pins sort collation so
# `comm` cannot report phantom drift from a locale-dependent ordering mismatch.
TMP_WANT=$(mktemp)
TMP_HAVE=$(mktemp)
TMP_ALL=$(mktemp)
TMP_DOC=$(mktemp)
trap 'rm -f "$TMP_WANT" "$TMP_HAVE" "$TMP_ALL" "$TMP_DOC"' EXIT

# WANT — the declared gating set from the manifest.
if ! jq -e '.jobs | type == "array" and length > 0' "$MANIFEST" > /dev/null 2>&1; then
    log_fail "Manifest $MANIFEST has no non-empty .jobs array"
    exit 1
fi
jq -r '.jobs[].job' "$MANIFEST" | LC_ALL=C sort -u > "$TMP_WANT"

# HAVE — the jobs listed in the ci-gate job's `needs:` block. Slice the block
# from `  ci-gate:` to the job's `    steps:`, then take the `- jobname` lines
# (stripping any trailing comment).
sed -n '/^  ci-gate:/,/^    steps:/p' "$CI" \
    | grep -E '^[[:space:]]+-[[:space:]]+' \
    | sed -E 's/^[[:space:]]+-[[:space:]]+//; s/[[:space:]]*#.*$//; s/[[:space:]]+$//' \
    | LC_ALL=C sort -u > "$TMP_HAVE"

# ALL — every top-level job key in ci.yml (2-space indented `key:`), used to
# detect manifest entries that name a job which does not exist.
grep -E '^  [A-Za-z0-9_-]+:[[:space:]]*$' "$CI" \
    | sed -E 's/^  ([A-Za-z0-9_-]+):.*$/\1/' \
    | LC_ALL=C sort -u > "$TMP_ALL"

rc=0

# Assertion A — no ghost gates: every manifest entry must be a real job in ci.yml.
ghosts=$(LC_ALL=C comm -23 "$TMP_WANT" "$TMP_ALL")
if [ -n "$ghosts" ]; then
    log_fail "Ghost gate(s): manifest names job(s) absent from $CI:"
    printf '%s\n' "$ghosts" | while IFS= read -r g; do
        [ -n "$g" ] && log_err_detail "$g"
    done
    rc=1
fi

# Assertion B — no drift: the manifest set must equal the ci-gate needs: set.
# comm -3 prints lines unique to either file: column 1 = in manifest but missing
# from needs: (an UNGATED job); column 2 = in needs: but not declared in the
# manifest (an UNDECLARED gate). Either is drift.
missing_from_gate=$(LC_ALL=C comm -23 "$TMP_WANT" "$TMP_HAVE")
undeclared_in_manifest=$(LC_ALL=C comm -13 "$TMP_WANT" "$TMP_HAVE")
if [ -n "$missing_from_gate" ] || [ -n "$undeclared_in_manifest" ]; then
    log_fail "CI gate drift between $MANIFEST and the ci-gate 'needs:' list:"
    if [ -n "$missing_from_gate" ]; then
        log_err_detail "Declared in manifest but MISSING from ci-gate needs: (ungated):"
        printf '%s\n' "$missing_from_gate" | while IFS= read -r j; do
            [ -n "$j" ] && log_err_detail "  $j" 2
        done
    fi
    if [ -n "$undeclared_in_manifest" ]; then
        log_err_detail "Present in ci-gate needs: but NOT declared in manifest (undeclared gate):"
        printf '%s\n' "$undeclared_in_manifest" | while IFS= read -r j; do
            [ -n "$j" ] && log_err_detail "  $j" 2
        done
    fi
    rc=1
fi

# Assertion C — every gating job is DOCUMENTED in ci-gating.md as a
# `#### <job>` Check Details heading. The doc legitimately documents MORE than
# the gating set (the deliberately-excluded jobs — semver-check, check-msrv, …
# — each have a heading too), so this is a subset assertion (manifest ⊆
# documented), NOT equality. It catches a gating job added to the manifest +
# ci-gate needs: but never written up in the prose doc — the drift class the
# `needs:` ↔ manifest check above cannot see.
grep -E '^#### [A-Za-z0-9_-]+[[:space:]]*$' "$DOC" \
    | sed -E 's/^#### //; s/[[:space:]]+$//' \
    | LC_ALL=C sort -u > "$TMP_DOC"
undocumented=$(LC_ALL=C comm -23 "$TMP_WANT" "$TMP_DOC")
if [ -n "$undocumented" ]; then
    log_fail "Gating job(s) not documented in $DOC (need a '#### <job>' Check Details entry):"
    printf '%s\n' "$undocumented" | while IFS= read -r j; do
        [ -n "$j" ] && log_err_detail "  $j" 2
    done
    rc=1
fi

if [ "$rc" -eq 0 ]; then
    log_ok "verify-ci-gate: $(wc -l < "$TMP_WANT" | tr -d ' ') gating jobs in sync across the manifest, ci-gate needs:, and ci-gating.md"
fi

exit "$rc"
