#!/bin/sh
# ==============================================================================
# scripts/check-semver.sh
#
# Verify semantic-versioning compliance across the workspace with
# cargo-semver-checks. Whether the check is APPLICABLE is decided from the
# crates' own versions (read locally via `cargo metadata` — authoritative and
# offline), NOT from a crates.io probe:
#
#   - Pre-1.0 (every crate < 1.0.0): the API is still evolving and there is no
#     stability promise to diff against, so the check is structurally
#     inapplicable and is WARN-skipped (skip is not a failure). A network probe
#     is irrelevant here.
#   - 1.0+ (any crate >= 1.0.0, INCLUDING 1.0 pre-releases like 1.0.0-rc.1):
#     breaking-change detection is MANDATORY. The rc cycle IS the 1.0 cycle — by
#     the time you cut 1.0.0-rc.1 you have declared the API you intend to
#     stabilise, so the gate must already be live to catch an unintended break
#     before the promise becomes permanent. A crate at >= 1.0.0-0 is therefore
#     "post-1.0" for this gate's purposes (a numeric major-version test gets this
#     right: 1.0.0-rc.1 has major 1).
#
# WHY phase is read locally, not probed (the bug this design closes):
#   The previous implementation inferred phase from `cargo search <crate>`: a hit
#   meant "baseline exists, run the check"; a miss meant "Phase 0, skip". But a
#   MISS and a PROBE FAILURE are indistinguishable through a `| grep -q` pipe with
#   2>/dev/null — so a transient crates.io outage, an offline runner, or a 5xx
#   would fall through to the skip branch and SILENTLY DISABLE breaking-change
#   detection on a post-1.0 release, fail-OPEN on the one axis a supply-chain gate
#   must fail closed (and print a misleading "Phase 0" while doing it). Phase is a
#   known property of the repo (ADR-0004, ROADMAP), not something to rediscover
#   from the network each run. The crate version is self-describing, cannot drift
#   from itself, and is already the canonical 1.0 pivot the rest of the pipeline
#   (prep-release, releasing.md) turns on — so it, not a registry round-trip, is
#   the phase signal. See [[project_prerelease_lockstep_boundary]].
#
# Post-1.0, the registry is consulted ONLY to confirm a baseline is actually
# published before running the diff — and there a probe FAILURE is FATAL, never a
# skip (fail closed). The single legitimate post-1.0 skip — the very first 1.0.0
# publish, before its own 1.0 baseline exists on the index — is an EXPLICIT,
# greppable opt-in (SEMVER_ALLOW_NO_BASELINE=1), so the gate can never be disabled
# by an accident, only by a deliberate, auditable decision.
#
# POSIX sh only.
#
# Environment:
#   CARGO  cargo invocation to use (default: cargo) — indirection for tests,
#          which stub it to fake versions / published state / check outcome
#          without hitting crates.io.
#   SEMVER_PROBE_CRATE  crate name whose published baseline is probed post-1.0
#          (default sdmx-types, the workspace's foundational crate — first to be
#          published).
#   SEMVER_ALLOW_NO_BASELINE  if "1", a post-1.0 run with NO published baseline
#          warn-skips instead of failing. The explicit escape hatch for the first
#          1.0.0 publish ONLY. Has no effect pre-1.0 (already skipped) and does
#          NOT suppress a probe ERROR (that stays fatal — "I could not check" is
#          not "there is nothing to check").
#
# Exit codes:
#   0 = check passed, OR pre-1.0 skip, OR explicit-opt-in first-1.0 skip
#   1 = a post-1.0 baseline probe failed (fail closed), or no baseline without
#       the explicit opt-in, or could not determine versions
#   N = cargo semver-checks found a violation (its own exit code, propagated)
# ==============================================================================

set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

CARGO="${CARGO:-cargo}"
PROBE_CRATE="${SEMVER_PROBE_CRATE:-sdmx-types}"

log_section "Checking semantic-versioning compliance"

# --- Phase decision: read crate versions locally (no network) -----------------
# The highest MAJOR version across workspace members decides the era. During
# lockstep every crate shares a version, so this is unambiguous; post-1.0 it
# correctly flips to mandatory the moment ANY crate crosses 1.0. cargo metadata
# is offline and authoritative; its stdout is clean (the devShell banner goes to
# stderr — see flake.nix shellHook).
VERSIONS=$("$CARGO" metadata --no-deps --format-version 1 2>/dev/null \
    | jq -r '.packages[] | select(.name | startswith("sdmx-")) | .version') || true

if [ -z "$VERSIONS" ]; then
    log_fatal "semver-check: could not read workspace crate versions from cargo metadata. Refusing to guess the release phase — fix the metadata error and re-run."
fi

# Highest major component. A version's major is the text before the first '.'.
MAX_MAJOR=0
for v in $VERSIONS; do
    major="${v%%.*}"
    case "$major" in
        ''|*[!0-9]*)
            log_fatal "semver-check: unparseable crate version '${v}' — cannot determine release phase."
            ;;
    esac
    [ "$major" -gt "$MAX_MAJOR" ] && MAX_MAJOR="$major"
done

if [ "$MAX_MAJOR" -lt 1 ]; then
    # Pre-1.0: structurally inapplicable. Warn (not info): while skipped,
    # breaking-change detection is OFF — a property that tends to outlive the
    # phase that justified it. A probe failure is irrelevant here, so we never
    # touch the network.
    log_warn "semver-checks SKIPPED — workspace is pre-1.0 (highest crate major is 0). Breaking-change detection is OFF until the first version with major 1; it becomes a MANDATORY, fail-closed gate from 1.0.0-rc onward (the rc cycle is the 1.0 cycle — any 1.x, pre-release included)."
    exit 0
fi

# --- 1.0+ : the check is MANDATORY. Confirm a baseline exists, fail closed. ----
log_item "Workspace is 1.0+ (major ${MAX_MAJOR}) — semver-checks is mandatory" 1

# Probe for a published baseline. Capture status and output SEPARATELY so a probe
# FAILURE (network/registry error) is distinguishable from a genuine MISS (no
# version published). The old code's `2>/dev/null | grep -q` collapsed these.
probe_out=$("$CARGO" search "$PROBE_CRATE" --limit 1 2>/dev/null) || probe_rc=$?
probe_rc="${probe_rc:-0}"

if [ "$probe_rc" -ne 0 ]; then
    # We could not query the registry. Post-1.0 this MUST NOT degrade to a skip:
    # "I couldn't check" is not "there's nothing to check". Fail closed.
    log_fatal "semver-check: could not query crates.io for the baseline (cargo search exited ${probe_rc}). Refusing to skip breaking-change detection on a 1.0+ release. Retry once the registry is reachable."
fi

# Anchored match: cargo search prints `name = "x.y.z"` at line start. Match the
# exact crate name followed by ` =` so a sibling/substring crate (e.g.
# sdmx-types-extra) cannot satisfy the baseline check.
if printf '%s\n' "$probe_out" | grep -q "^${PROBE_CRATE} ="; then
    log_item "Published baseline found — running cargo semver-checks" 1
    status=0
    "$CARGO" semver-checks check-release || status=$?
    if [ "$status" -ne 0 ]; then
        log_fail "semver-check: cargo semver-checks reported a violation (exit ${status})."
        exit "$status"
    fi
    log_ok "semver-check: no semver violations"
elif [ "${SEMVER_ALLOW_NO_BASELINE:-}" = "1" ]; then
    # Explicit, auditable opt-in for the ONE legitimate post-1.0 skip: the first
    # 1.0.0 publish, before its own 1.0 baseline is on the index. Greppable in CI
    # logs and config — never an inferred accident.
    log_warn "semver-checks SKIPPED — 1.0+ but no published baseline, and SEMVER_ALLOW_NO_BASELINE=1 is set (first-1.0 bootstrap). Remove this opt-in once the 1.0 baseline is published; it must NOT persist."
    exit 0
else
    # 1.0+ with no baseline and no explicit opt-in. Fail closed: this is either
    # the first-1.0 case (set SEMVER_ALLOW_NO_BASELINE=1 deliberately) or a real
    # problem (the baseline that should exist does not).
    log_fatal "semver-check: workspace is 1.0+ but no published baseline for '${PROBE_CRATE}' was found. If this is the FIRST 1.0.0 publish, set SEMVER_ALLOW_NO_BASELINE=1 to opt in explicitly. Otherwise the expected baseline is missing — investigate before releasing."
fi
