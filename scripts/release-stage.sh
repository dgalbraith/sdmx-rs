#!/bin/sh
# ==============================================================================
# scripts/release-stage.sh
#
# Push the local release merge commit to a CI staging branch so the Quality
# Gate runs on this exact SHA before anything touches main. The staging branch
# name follows the repo convention: `staging-release-sdmx-rs-<version>`, which
# the CI trigger pattern `staging-*` already covers (.github/workflows/ci.yml).
#
# Called by `just stage-merge <version>`. Blocks until the CI Quality Gate is
# green on the staging SHA, then exits 0 so the maintainer can immediately run
# `just release-push <version>` to land on main. Exits 1 on CI failure or
# poll timeout so the maintainer investigates before anything irreversible runs.
#
# WHY a separate staging branch: the Zero Trust Gate on main requires a
# CI-verified SHA. Pushing directly to main would be rejected if the commit
# hasn't been validated. The staging branch earns the CI context; a subsequent
# fast-forward of main carries that context over without re-running CI.
#
# Poll behaviour:
#   - Uses `gh api repos/<owner>/<repo>/commits/<sha>/check-runs` (GitHub only;
#     always reads from `origin` regardless of SDMX_MAIN_REMOTE because gh is
#     GitHub-scoped and CI always runs there).
#   - If the repo slug cannot be derived from the origin remote (not a GitHub
#     repo), polling is impossible and is skipped with a manual hint (exit 0).
#   - If `gh` is not authenticated (a GitHub repo, but a fixable local
#     precondition), the script fails closed (exit 1): exit 0 is reserved for a
#     verified-green gate, so it must not signal success without polling.
#   - Linear backoff: POLL_INTERVAL seconds between attempts, up to MAX_ATTEMPTS.
#   - Any check run with conclusion failure/cancelled/timed_out → exit 1
#     immediately (fail red and loud; human investigates).
#   - All check runs completed with conclusion success → exit 0.
#   - Exhausted retries without a terminal state → exit 1.
#
# POSIX sh only.
#
# Usage: scripts/release-stage.sh <version>   (e.g. 0.2.0  or  0.2.0-alpha.1)
#
# Exit codes:
#   0 = staging branch pushed and CI Quality Gate is green
#   1 = missing argument, git push failed, gh unauthenticated, CI gate failed,
#       or poll timed out
# ==============================================================================

set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/forge-spec.sh"

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    log_err "Missing required argument: <version>"
    log_err_detail "Usage: release-stage.sh <version>   (e.g. 0.2.0)"
    exit 1
fi

CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    log_err "Must be on 'main' to stage the release (currently on '${CURRENT_BRANCH}')."
    log_err_detail "Run 'just release-merge' first, which checks out main, then re-run."
    exit 1
fi

REMOTE="${SDMX_MAIN_REMOTE:-origin}"
STAGING="staging-release-sdmx-rs-${VERSION}"

log_info "Pushing merge commit to ${STAGING} for CI validation..."
git push "${REMOTE}" HEAD:refs/heads/"${STAGING}"
log_ok "stage-merge: pushed merge commit to ${STAGING} on ${REMOTE}"

# Capture the SHA so the poll targets this exact commit, not wherever HEAD
# moves between push and first poll.
SHA=$(git rev-parse HEAD)

# ---------------------------------------------------------------------------
# CI Quality Gate poll
# ---------------------------------------------------------------------------

# Override these in tests (e.g. MAX_ATTEMPTS=2 POLL_INTERVAL=0) to avoid
# waiting. Production defaults: 24 attempts × 30s = 12 minute ceiling.
MAX_ATTEMPTS="${RELEASE_STAGE_MAX_ATTEMPTS:-24}"
POLL_INTERVAL="${RELEASE_STAGE_POLL_INTERVAL:-30}"

# Resolve the GitHub owner/repo slug from the origin remote. Polling is always
# GitHub-scoped (gh is GitHub-only); SDMX_MAIN_REMOTE may point elsewhere for
# pushes, but CI always runs on GitHub.
if ! OWNER_REPO="$(forge_spec_owner_repo 2>/dev/null)"; then
    log_warn "Could not derive GitHub repo slug from origin remote — skipping CI poll."
    log_hint "Monitor CI manually, then run: just release-push ${VERSION}"
    exit 0
fi

# Unauthenticated gh is a FIXABLE local precondition, not an environmental
# impossibility like the missing-slug case above (which means "not a GitHub
# repo at all" — polling can never work there, so skipping is correct). Here CI
# *is* running on GitHub and polling *is* possible; gh just needs a login. Fail
# CLOSED rather than exit 0: this script's contract is "exit 0 ⟹ CI Quality Gate
# is green", and exiting 0 here would verify nothing while signalling success.
# The staging branch was already pushed (a no-op to re-push), and nothing
# irreversible runs in this script, so failing here costs only a re-run.
if ! gh auth status >/dev/null 2>&1; then
    log_err "gh is not authenticated — cannot poll the CI Quality Gate."
    log_err_detail "This script must confirm CI is green before you run release-push."
    log_err_detail "Run 'gh auth login', then re-run: just stage-merge ${VERSION}"
    log_err_detail "(The staging branch is already pushed; re-running just re-polls.)"
    exit 1
fi

log_section "Polling CI Quality Gate for ${SHA}..."

ATTEMPT=0
while true; do
    ATTEMPT=$((ATTEMPT + 1))

    # Fetch all check runs for this SHA, then extract the fields we need.
    # Two-step (gh | jq) rather than gh --jq so the gh stub in tests does not
    # need to implement jq itself — it just returns raw JSON.
    set +e
    RUNS=$(gh api "repos/${OWNER_REPO}/commits/${SHA}/check-runs" 2>/dev/null \
        | jq -r '.check_runs[] | "\(.status) \(.conclusion // "null") \(.name)"')
    api_rc=$?
    set -e

    if [ "$api_rc" -ne 0 ]; then
        log_err "GitHub API call failed (attempt ${ATTEMPT}/${MAX_ATTEMPTS})."
        if [ "$ATTEMPT" -ge "$MAX_ATTEMPTS" ]; then
            log_err "Giving up after ${MAX_ATTEMPTS} attempts — investigate and retry."
            exit 1
        fi
        log_info "Retrying in ${POLL_INTERVAL}s..."
        sleep "$POLL_INTERVAL"
        continue
    fi

    if [ -z "$RUNS" ]; then
        log_info "Attempt ${ATTEMPT}/${MAX_ATTEMPTS} — no check runs yet, waiting ${POLL_INTERVAL}s..."
        if [ "$ATTEMPT" -ge "$MAX_ATTEMPTS" ]; then
            log_err "No check runs appeared after ${MAX_ATTEMPTS} attempts — investigate and retry."
            exit 1
        fi
        sleep "$POLL_INTERVAL"
        continue
    fi

    # Two concerns, one payload — keep them separate:
    #   DECISION (pass/fail/pending) keys ONLY off the "CI Quality Gate" run, the
    #     single context the Zero Trust ruleset requires (ci.yml). Matching the poll
    #     to that one context — not an all-runs aggregate — is load-bearing:
    #       - A non-gating check that fails must NOT red-light a release the ruleset
    #         would merge (an all-runs aggregate was too strict).
    #       - A gate that was never created must NOT pass because the other runs are
    #         green/skipped (an all-runs aggregate was fail-OPEN). Absent gate →
    #         pending → timeout, which is the correct refusal.
    #       - `skipped` is the gate's own success-equivalent for path-filtered jobs;
    #         it never reaches us here because we read the gate's own conclusion, not
    #         its dependencies'.
    #   DISPLAY (the per-job breakdown below) iterates ALL runs for the maintainer's
    #     benefit only. It is cosmetic: a surprising state in a non-gating job changes
    #     what is printed, never the verdict. Do not fold these two back together.
    GATE_NAME="CI Quality Gate"

    # DECISION: extract the gate run's conclusion (empty if the gate run does not
    # exist yet on this SHA). Reads from $RUNS so the same payload drives both halves.
    GATE_CONCLUSION=$(printf '%s\n' "$RUNS" \
        | while IFS=' ' read -r _status conclusion name rest; do
              # $name..$rest reassembles the gate name (it contains a space).
              if [ "${name} ${rest}" = "$GATE_NAME" ]; then
                  printf '%s' "$conclusion"
                  break
              fi
          done)

    # DISPLAY: render every run's state so the maintainer sees per-job progress on
    # each poll iteration (not just on failure). Cosmetic only — never gates.
    print_run_breakdown() {
        printf '%s\n' "$RUNS" | while IFS=' ' read -r _status conclusion name; do
            case "$conclusion" in
                success)                     log_ok   "  ${name}" 1 ;;
                skipped)                     log_item "  ${name}: skipped (not applicable)" 1 ;;
                failure|cancelled|timed_out) log_fail "  ${name}: ${conclusion}" ;;
                *)                           log_item "  ${name}: in progress" 1 ;;
            esac
        done
    }

    case "$GATE_CONCLUSION" in
        failure|cancelled|timed_out)
            print_run_breakdown
            log_err "CI Quality Gate failed on ${SHA} (conclusion: ${GATE_CONCLUSION})."
            log_err_detail "The states above are context at time of failure — the gate is the verdict."
            log_err_detail "Investigate the failed check(s), fix on a new branch, merge to main,"
            log_err_detail "create a fresh release branch, and restart from the beginning."
            exit 1
            ;;
        success)
            print_run_breakdown
            log_ok "stage-merge: CI Quality Gate passed on ${SHA}"
            log_hint "Run: just release-push ${VERSION}"
            exit 0
            ;;
        *)
            # Pending OR gate not yet created. Show progress and keep polling; an
            # absent gate correctly times out below rather than passing fail-open.
            print_run_breakdown
            ;;
    esac

    if [ "$ATTEMPT" -ge "$MAX_ATTEMPTS" ]; then
        log_err "CI Quality Gate did not complete within $((MAX_ATTEMPTS * POLL_INTERVAL))s."
        log_err_detail "SHA: ${SHA}"
        log_err_detail "Check CI directly and run 'just release-push ${VERSION}' only when green."
        exit 1
    fi

    log_info "Attempt ${ATTEMPT}/${MAX_ATTEMPTS} — checks still running, waiting ${POLL_INTERVAL}s..."
    sleep "$POLL_INTERVAL"
done
