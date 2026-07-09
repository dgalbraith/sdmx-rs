#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/release-push.sh
#
# Testing approach: the script first re-validates HEAD against the CI-verified
# staging branch (fetch origin <staging>, compare rev-parse HEAD == FETCH_HEAD),
# then performs three git push calls (main fast-forward, tags, staging delete).
# A `git` stub on PATH records push invocations, returns "main" for
# `branch --show-current`, and serves controllable SHAs for `rev-parse HEAD` and
# `rev-parse FETCH_HEAD` (via HEAD_SHA / STAGING_SHA env, default-equal so the
# re-validation passes). Tests assert the SHA gate, then the correct push
# sequence, refspecs, and remote; error paths (missing argument, wrong branch,
# SHA drift, missing staging branch, push failure) use the same infrastructure.
#
# Run with: bats tests/bats/release-push.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    cp "$BATS_TEST_DIRNAME/../../scripts/release-push.sh" .
    mkdir -p lib
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    # Recording git stub: dispatches on subcommand.
    #   branch --show-current  → prints "main" (satisfies the on-main guard)
    #   rev-parse HEAD         → prints $HEAD_SHA (default "validsha")
    #   rev-parse FETCH_HEAD   → prints $STAGING_SHA (default "validsha" → match)
    #   fetch ...              → exits per $FETCH_RC (default 0)
    #   push ...               → appends the full argument vector to GIT_CALLS
    #   anything else          → exits 0 silently
    # HEAD_SHA == STAGING_SHA by default so the SHA re-validation passes; tests
    # override them to exercise drift, and FETCH_RC to exercise a missing branch.
    GIT_CALLS="$BATS_TEST_TMPDIR/git-calls.log"
    export GIT_CALLS
    export HEAD_SHA="validsha"
    export STAGING_SHA="validsha"
    export FETCH_RC=0
    mkdir -p "$BATS_TEST_TMPDIR/bin"
    cat > "$BATS_TEST_TMPDIR/bin/git" << 'EOF'
#!/bin/sh
case "$1" in
    branch) echo "main" ;;
    rev-parse)
        case "$2" in
            HEAD)       echo "${HEAD_SHA}" ;;
            FETCH_HEAD) echo "${STAGING_SHA}" ;;
        esac
        ;;
    fetch)  exit "${FETCH_RC}" ;;
    # `tag -l <glob>` drives the scoped tag-push refspec loop. Return one
    # per-crate tag for the requested version so TAG_REFSPECS is non-empty and
    # the script proceeds past its empty-guard. The version is the last arg of
    # the glob (sdmx-*/v<version>); echo a representative crate tag for it.
    tag)
        glob=$(eval echo "\${$#}")           # last positional = the -l pattern
        ver=${glob#sdmx-*/v}
        echo "sdmx-types/v${ver}"
        ;;
    push)   printf '%s\n' "$*" >> "$GIT_CALLS" ;;
esac
exit 0
EOF
    chmod +x "$BATS_TEST_TMPDIR/bin/git"
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# ---------------------------------------------------------------------------
# Error paths
# ---------------------------------------------------------------------------

@test "release-push: fails when not on main branch" {
    cat > "$BATS_TEST_TMPDIR/bin/git" << 'EOF'
#!/bin/sh
case "$1" in
    branch) echo "release/sdmx-rs/0.2.0" ;;
    push)   printf '%s\n' "$*" >> "$GIT_CALLS" ;;
esac
exit 0
EOF
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error:"* ]]
    [[ "$output" == *"main"* ]]
    [ ! -f "$GIT_CALLS" ] || [ ! -s "$GIT_CALLS" ]
}

@test "release-push: fails with exit 1 when version argument is missing" {
    run_isolated ./release-push.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"Error:"* ]]
    [[ "$output" == *"version"* ]]
}

@test "release-push: prints usage hint on missing version" {
    run_isolated ./release-push.sh
    [[ "$output" == *"Usage:"* ]]
}

# ---------------------------------------------------------------------------
# Happy path — correct push sequence
# ---------------------------------------------------------------------------

@test "release-push: first push fast-forwards main" {
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -eq 0 ]
    first_call=$(head -n 1 "$GIT_CALLS")
    [[ "$first_call" == *"HEAD:refs/heads/main"* ]]
}

@test "release-push: second push sends scoped tag refspecs, not --tags" {
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -eq 0 ]
    second_call=$(sed -n '2p' "$GIT_CALLS")
    # Scoped to this release's tags via explicit refspecs (see release-push.sh):
    # a bare --tags would push stray signed tags from a prior aborted session.
    [[ "$second_call" == *"refs/tags/sdmx-types/v0.2.0:refs/tags/sdmx-types/v0.2.0"* ]]
    [[ "$second_call" != *"--tags"* ]]
}

@test "release-push: aborts before tag push when no matching tags exist" {
    # Empty-guard: main is fast-forwarded first, so zero matching tags must fail
    # LOUDLY rather than silently push nothing and leave a half-landed release.
    cat > "$BATS_TEST_TMPDIR/bin/git" << 'EOF'
#!/bin/sh
case "$1" in
    branch) echo "main" ;;
    rev-parse)
        case "$2" in
            HEAD)       echo "validsha" ;;
            FETCH_HEAD) echo "validsha" ;;
        esac
        ;;
    fetch)  exit 0 ;;
    tag)    : ;;                 # no tags match → empty output
    push)   printf '%s\n' "$*" >> "$GIT_CALLS" ;;
esac
exit 0
EOF
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -eq 1 ]
    [[ "$output" == *"No release tags matching"* ]]
    # main was pushed (call 1); the tag push must NOT have happened.
    ! grep -q "refs/tags/" "$GIT_CALLS"
}

@test "release-push: third push deletes staging branch from origin" {
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -eq 0 ]
    third_call=$(sed -n '3p' "$GIT_CALLS")
    [[ "$third_call" == *"--delete"* ]]
    [[ "$third_call" == *"staging-release-sdmx-rs-0.2.0"* ]]
    # Deletes from origin specifically (where the staging branch lives), NOT
    # ${REMOTE}, which may be a fan-out mirror the branch never reached.
    [[ "$third_call" == "push origin "* ]]
}

@test "release-push: exactly three git push calls are made" {
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -eq 0 ]
    call_count=$(wc -l < "$GIT_CALLS")
    [ "$call_count" -eq 3 ]
}

@test "release-push: uses default remote origin when SDMX_MAIN_REMOTE is unset" {
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -eq 0 ]
    grep -qv "^push origin " "$GIT_CALLS" && return 1
    return 0
}

@test "release-push: respects SDMX_MAIN_REMOTE override for code and tag pushes" {
    unset GITHUB_ACTIONS CI GITHUB_EVENT_NAME
    export SDMX_MAIN_REMOTE=all
    run sh ./release-push.sh 0.2.0
    unset SDMX_MAIN_REMOTE
    [ "$status" -eq 0 ]
    # The main fast-forward and tag push honour the override (push to `all`)...
    [[ "$(sed -n '1p' "$GIT_CALLS")" == "push all "* ]]
    [[ "$(sed -n '2p' "$GIT_CALLS")" == "push all "* ]]
    # ...but the staging-branch cleanup deletes from origin regardless, because
    # the branch only ever lived on origin, not on the mirror fan-out.
    [[ "$(sed -n '3p' "$GIT_CALLS")" == "push origin "* ]]
}

@test "release-push: staging branch name embeds version for prerelease tags" {
    run_isolated ./release-push.sh 1.0.0-alpha.1
    [ "$status" -eq 0 ]
    third_call=$(sed -n '3p' "$GIT_CALLS")
    [[ "$third_call" == *"staging-release-sdmx-rs-1.0.0-alpha.1"* ]]
}

@test "release-push: emits gate-summary log_ok line" {
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -eq 0 ]
    [[ "$output" == *"release-push:"* ]]
}

# ---------------------------------------------------------------------------
# Push failure propagation
# ---------------------------------------------------------------------------

@test "release-push: exits non-zero when main push fails" {
    cat > "$BATS_TEST_TMPDIR/bin/git" << 'EOF'
#!/bin/sh
case "$1" in
    branch) echo "main"; exit 0 ;;
    rev-parse) echo "validsha" ;;
    fetch)  exit 0 ;;
    push)   printf '%s\n' "$*" >> "$GIT_CALLS"; exit 1 ;;
esac
exit 0
EOF
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -ne 0 ]
}

@test "release-push: exits non-zero when tag push fails" {
    cat > "$BATS_TEST_TMPDIR/bin/git" << 'EOF'
#!/bin/sh
case "$1" in
    branch) echo "main"; exit 0 ;;
    rev-parse) echo "validsha" ;;
    fetch)  exit 0 ;;
    tag)    echo "sdmx-types/v0.2.0" ;;
    push)
        printf '%s\n' "$*" >> "$GIT_CALLS"
        [ "$(wc -l < "$GIT_CALLS")" -eq 2 ] && exit 1
        exit 0
        ;;
esac
exit 0
EOF
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -ne 0 ]
}

@test "release-push: continues when staging branch deletion fails" {
    cat > "$BATS_TEST_TMPDIR/bin/git" << 'EOF'
#!/bin/sh
case "$1" in
    branch) echo "main"; exit 0 ;;
    rev-parse) echo "validsha" ;;
    fetch)  exit 0 ;;
    tag)    echo "sdmx-types/v0.2.0" ;;
    push)
        printf '%s\n' "$*" >> "$GIT_CALLS"
        case "$*" in
            *--delete*) exit 1 ;;
            *) exit 0 ;;
        esac
        ;;
esac
exit 0
EOF
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -eq 0 ]
}

# ---------------------------------------------------------------------------
# SHA re-validation (step 0) — bind the push to the CI-verified staging commit
# ---------------------------------------------------------------------------

@test "release-push: fetches the staging branch before any push" {
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -eq 0 ]
    # The re-validation must run BEFORE pushes: the first push only happens once
    # the SHA matched. Confirm the gate's success line is emitted.
    [[ "$output" == *"HEAD matches CI-verified staging commit"* ]]
}

@test "release-push: aborts when HEAD has drifted from the staging commit" {
    export HEAD_SHA="driftedsha"
    export STAGING_SHA="validsha"
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -eq 1 ]
    [[ "$output" == *"HEAD does not match"* ]]
    # Fail-closed: nothing pushed.
    [ ! -f "$GIT_CALLS" ] || [ ! -s "$GIT_CALLS" ]
}

@test "release-push: aborts when the staging branch cannot be fetched" {
    export FETCH_RC=1
    run_isolated ./release-push.sh 0.2.0
    [ "$status" -eq 1 ]
    [[ "$output" == *"Could not fetch staging branch"* ]]
    [[ "$output" == *"stage-merge"* ]]
    # Fail-closed: nothing pushed.
    [ ! -f "$GIT_CALLS" ] || [ ! -s "$GIT_CALLS" ]
}

@test "release-push: re-validates against origin, not SDMX_MAIN_REMOTE" {
    # CI always runs on GitHub (origin); the fetch must target origin even when
    # pushes fan out to a mirror remote. Record fetch args to assert the remote.
    cat > "$BATS_TEST_TMPDIR/bin/git" << 'EOF'
#!/bin/sh
case "$1" in
    branch) echo "main" ;;
    rev-parse) echo "validsha" ;;
    fetch)  printf 'FETCH %s\n' "$*" >> "$GIT_CALLS" ;;
    tag)    echo "sdmx-types/v0.2.0" ;;
    push)   printf '%s\n' "$*" >> "$GIT_CALLS" ;;
esac
exit 0
EOF
    unset GITHUB_ACTIONS CI GITHUB_EVENT_NAME
    export SDMX_MAIN_REMOTE=all
    run sh ./release-push.sh 0.2.0
    unset SDMX_MAIN_REMOTE
    [ "$status" -eq 0 ]
    grep -q "^FETCH fetch origin refs/heads/staging-release-sdmx-rs-0.2.0" "$GIT_CALLS"
    # Code/tag pushes still fan out to the mirror remote.
    grep -q "^push all " "$GIT_CALLS"
}
