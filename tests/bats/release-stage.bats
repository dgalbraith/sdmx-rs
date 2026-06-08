#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/release-stage.sh
#
# Testing approach:
#   Pre-push guards (branch, version) — exercised with no gh stub; the script
#   exits before reaching the poll block so no forge interaction occurs.
#
#   Push behaviour — a recording git stub captures refspecs. No github.com
#   remote is configured in the sandbox, so forge_spec_owner_repo() returns 1
#   and the script degrades gracefully (warns, exits 0) after the push — the
#   poll block is skipped, which is correct for an offline/non-github fixture.
#
#   Poll behaviour — a full stub set (git + gh) is wired up with a github.com
#   remote URL so forge_spec_owner_repo() resolves, and gh is controlled via
#   GH_CHECK_RUNS to serve canned check-run payloads. Each poll-path scenario
#   (all green, one failed, pending-then-green, no-gh-auth, timeout) is tested
#   against the full script execution.
#
# Run with: bats tests/bats/release-stage.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    cp "$BATS_TEST_DIRNAME/../../scripts/release-stage.sh" .
    mkdir -p lib
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/forge-spec.sh" lib/

    GIT_CALLS="$TMPDIR/git-calls.log"
    export GIT_CALLS

    mkdir -p "$TMPDIR/bin"

    # Default git stub: branch → "main", push → record, rev-parse → fixed SHA,
    # remote get-url → a github.com URL (used by forge_spec_owner_repo).
    install_git_stub "main" "abc1234"

    # Default gh stub: auth succeeds, check-runs returns all-green immediately.
    install_gh_stub "green"

    export PATH="$TMPDIR/bin:$PATH"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# ---------------------------------------------------------------------------
# Stub helpers
# ---------------------------------------------------------------------------

# install_git_stub <branch> <sha>
# Writes a git stub that returns <branch> for `branch --show-current`,
# <sha> for `rev-parse HEAD`, a github.com URL for `remote get-url origin`,
# and records push calls to GIT_CALLS.
install_git_stub() {
    local branch="$1" sha="$2"
    cat > "$TMPDIR/bin/git" << EOF
#!/bin/sh
case "\$1" in
    branch)     echo "$branch" ;;
    rev-parse)  echo "$sha" ;;
    remote)     echo "https://github.com/testowner/testrepo.git" ;;
    push)       printf '%s\n' "\$*" >> "\$GIT_CALLS" ;;
esac
exit 0
EOF
    chmod +x "$TMPDIR/bin/git"
}

# install_gh_stub <mode>
# Writes a gh stub. <mode> controls check-run responses:
#   green        — all checks completed+success (immediate)
#   green_with_skips — gate success + path-filtered jobs reporting `skipped`
#                  (the realistic release case: a release commit touches crates
#                  but not docs/scripts, so those gating jobs skip)
#   failed       — one check completed+failure
#   failed_with_pending — gate failure while a non-gating job is still in_progress
#                  (the gate uses if: always(), so it can conclude before others)
#   gate_absent  — only NON-gating runs exist; the "CI Quality Gate" context is
#                  not present at all (fail-open trap: must NOT pass)
#   pending_once — first call returns in_progress, second returns success
#   no_auth      — auth status fails (unauthenticated)
#   api_error    — API call exits non-zero
install_gh_stub() {
    local mode="$1"
    local call_count_file="$TMPDIR/gh-calls.count"
    printf '0' > "$call_count_file"
    cat > "$TMPDIR/bin/gh" << EOF
#!/bin/sh
MODE="$mode"
CALL_COUNT_FILE="$call_count_file"
EOF
    cat >> "$TMPDIR/bin/gh" << 'EOF'
if [ "$1" = "auth" ] && [ "$2" = "status" ]; then
    [ "$MODE" = "no_auth" ] && exit 1
    exit 0
fi

if [ "$1" = "api" ]; then
    [ "$MODE" = "api_error" ] && exit 1

    # Increment call counter (used for pending_once mode).
    count=$(cat "$CALL_COUNT_FILE")
    count=$((count + 1))
    printf '%d' "$count" > "$CALL_COUNT_FILE"

    case "$MODE" in
        green)
            printf '{"check_runs":[{"status":"completed","conclusion":"success","name":"CI Quality Gate"}]}\n'
            ;;
        green_with_skips)
            printf '{"check_runs":[{"status":"completed","conclusion":"success","name":"CI Quality Gate"},{"status":"completed","conclusion":"skipped","name":"Documentation & ADR Validation"},{"status":"completed","conclusion":"skipped","name":"Shell Script Lint & Test Check"}]}\n'
            ;;
        failed)
            printf '{"check_runs":[{"status":"completed","conclusion":"failure","name":"CI Quality Gate"},{"status":"completed","conclusion":"success","name":"Other Check"}]}\n'
            ;;
        failed_with_pending)
            printf '{"check_runs":[{"status":"completed","conclusion":"failure","name":"CI Quality Gate"},{"status":"in_progress","conclusion":null,"name":"Clippy Lint Check"}]}\n'
            ;;
        gate_absent)
            printf '{"check_runs":[{"status":"completed","conclusion":"success","name":"Some Non-Gating Check"},{"status":"completed","conclusion":"skipped","name":"Documentation & ADR Validation"}]}\n'
            ;;
        pending_once)
            if [ "$count" -eq 1 ]; then
                printf '{"check_runs":[{"status":"in_progress","conclusion":null,"name":"CI Quality Gate"}]}\n'
            else
                printf '{"check_runs":[{"status":"completed","conclusion":"success","name":"CI Quality Gate"}]}\n'
            fi
            ;;
        *)
            printf '{"check_runs":[]}\n'
            ;;
    esac
    exit 0
fi

echo "gh stub: unhandled: $*" >&2
exit 1
EOF
    chmod +x "$TMPDIR/bin/gh"
}

# ---------------------------------------------------------------------------
# Error paths — before push
# ---------------------------------------------------------------------------

@test "release-stage: fails when not on main branch" {
    install_git_stub "release/sdmx-rs/0.2.0" "abc1234"
    run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error:"* ]]
    [[ "$output" == *"main"* ]]
    [ ! -f "$GIT_CALLS" ] || [ ! -s "$GIT_CALLS" ]
}

@test "release-stage: fails with exit 1 when version argument is missing" {
    run_isolated ./release-stage.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error:"* ]]
    [[ "$output" == *"version"* ]]
}

@test "release-stage: prints usage hint on missing version" {
    run_isolated ./release-stage.sh
    [[ "$output" == *"Usage:"* ]]
}

# ---------------------------------------------------------------------------
# Push behaviour
# ---------------------------------------------------------------------------

@test "release-stage: pushes HEAD to correct staging refspec on default remote" {
    run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 0 ]
    grep -q "push origin HEAD:refs/heads/staging-release-sdmx-rs-0.2.0" "$GIT_CALLS"
}

@test "release-stage: respects SDMX_MAIN_REMOTE override" {
    unset GITHUB_ACTIONS CI GITHUB_EVENT_NAME
    export SDMX_MAIN_REMOTE=all
    run sh ./release-stage.sh 0.2.0
    unset SDMX_MAIN_REMOTE
    [ "$status" -eq 0 ]
    grep -q "push all HEAD:refs/heads/staging-release-sdmx-rs-0.2.0" "$GIT_CALLS"
}

@test "release-stage: staging branch name embeds version correctly" {
    run_isolated ./release-stage.sh 1.0.0-alpha.1
    [ "$status" -eq 0 ]
    grep -q "push origin HEAD:refs/heads/staging-release-sdmx-rs-1.0.0-alpha.1" "$GIT_CALLS"
}

@test "release-stage: exits non-zero when git push fails" {
    cat > "$TMPDIR/bin/git" << 'EOF'
#!/bin/sh
case "$1" in
    branch) echo "main"; exit 0 ;;
    push)   exit 1 ;;
esac
exit 0
EOF
    run_isolated ./release-stage.sh 0.2.0
    [ "$status" -ne 0 ]
}

# ---------------------------------------------------------------------------
# Poll — success paths
# ---------------------------------------------------------------------------

@test "release-stage: exits 0 and emits gate-summary when all checks pass" {
    install_gh_stub "green"
    run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 0 ]
    [[ "$output" == *"stage-merge:"* ]]
    [[ "$output" == *"CI Quality Gate passed"* ]]
}

@test "release-stage: emits release-push hint on success" {
    install_gh_stub "green"
    run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 0 ]
    [[ "$output" == *"release-push"* ]]
    [[ "$output" == *"0.2.0"* ]]
}

@test "release-stage: passes when gate is green and path-filtered jobs are skipped" {
    # The realistic release case: a release commit touches crates/** but not
    # docs/** or scripts/**, so those gating jobs report `skipped`. The gate
    # itself treats skipped deps as success; the poll must agree and exit 0.
    install_gh_stub "green_with_skips"
    run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 0 ]
    [[ "$output" == *"CI Quality Gate passed"* ]]
}

@test "release-stage: shows skipped jobs in the breakdown without failing" {
    install_gh_stub "green_with_skips"
    run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 0 ]
    [[ "$output" == *"skipped"* ]]
}

@test "release-stage: times out fail-closed when the gate context is absent" {
    # Only non-gating runs exist; "CI Quality Gate" is missing entirely. An
    # all-runs aggregate would read green/skipped and pass FAIL-OPEN. Keying the
    # decision off the gate alone means an absent gate stays pending → timeout.
    install_gh_stub "gate_absent"
    RELEASE_STAGE_MAX_ATTEMPTS=2 RELEASE_STAGE_POLL_INTERVAL=0 \
        run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 1 ]
    [[ "$output" == *"did not complete"* ]]
}

@test "release-stage: gate failure still prints the per-job breakdown" {
    # Decision keys off the gate; display iterates all runs. A non-gating job
    # still in_progress at failure time must appear in the breakdown.
    install_gh_stub "failed_with_pending"
    run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 1 ]
    [[ "$output" == *"CI Quality Gate failed"* ]]
    [[ "$output" == *"Clippy Lint Check"* ]]
}

@test "release-stage: polls multiple times until checks complete" {
    install_gh_stub "pending_once"
    RELEASE_STAGE_POLL_INTERVAL=0 run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 0 ]
    [[ "$output" == *"CI Quality Gate passed"* ]]
    # Confirm the stub was called more than once (pending on first, green on second).
    count=$(cat "$TMPDIR/gh-calls.count")
    [ "$count" -ge 2 ]
}

# ---------------------------------------------------------------------------
# Poll — failure paths
# ---------------------------------------------------------------------------

@test "release-stage: exits 1 when a check run fails" {
    install_gh_stub "failed"
    run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error:"* ]]
    [[ "$output" == *"CI Quality Gate failed"* ]]
}

@test "release-stage: failure output names the failed check" {
    install_gh_stub "failed"
    run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 1 ]
    [[ "$output" == *"CI Quality Gate"* ]]
    [[ "$output" == *"failure"* ]]
}

@test "release-stage: fails closed when gh is not authenticated" {
    # Unauthenticated gh is a fixable local precondition on a real GitHub repo:
    # CI is running and polling is possible, so the script must NOT exit 0
    # (which its contract reserves for "gate is green"). It fails closed and
    # tells the maintainer to authenticate and re-run. Contrast the non-github
    # remote case below, which is a genuine skip (no GitHub to poll at all).
    install_gh_stub "no_auth"
    run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 1 ]
    [[ "$output" == *"not authenticated"* ]]
    [[ "$output" == *"gh auth login"* ]]
    [[ "$output" == *"stage-merge"* ]]
}

@test "release-stage: exits 0 with warning when origin is not a github.com remote" {
    # Override git stub so remote get-url returns a non-github URL.
    cat > "$TMPDIR/bin/git" << 'EOF'
#!/bin/sh
case "$1" in
    branch)    echo "main" ;;
    rev-parse) echo "abc1234" ;;
    remote)    echo "https://codeberg.org/testowner/testrepo.git" ;;
    push)      printf '%s\n' "$*" >> "$GIT_CALLS" ;;
esac
exit 0
EOF
    run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 0 ]
    [[ "$output" == *"Warning:"* ]]
    [[ "$output" == *"release-push"* ]]
}

@test "release-stage: exits 1 when API call fails persistently" {
    install_gh_stub "api_error"
    RELEASE_STAGE_MAX_ATTEMPTS=2 RELEASE_STAGE_POLL_INTERVAL=0 \
        run_isolated ./release-stage.sh 0.2.0
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error:"* ]]
}
