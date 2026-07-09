#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/forge-apply.sh
#
# Testing approach: forge-apply is a GUARDED ONE-SHOT SETUP script. These tests
# NEVER touch a live forge — `gh` is replaced by mock_gh_apply (a recording shim
# that logs every call to $GH_CALL_LOG without making network requests).
#
# Test contracts:
#   - The guard rejects runs when rulesets or issues already exist.
#   - --force bypasses the guard and the script proceeds to apply.
#   - A clean-state run calls the correct gh endpoints in the correct order.
#   - --skip-release-env suppresses the PUT environments call.
#   - --dry-run makes no gh api calls beyond the auth/guard probes.
#
# Run with: bats tests/bats/forge-apply.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    REPO_ROOT="$BATS_TEST_DIRNAME/../.."

    cd "$BATS_TEST_TMPDIR" || exit 1

    # Mirror the scripts/ layout the script resolves relative to $0.
    mkdir -p scripts/lib lib
    cp "$REPO_ROOT/scripts/forge-apply.sh" scripts/
    cp "$REPO_ROOT/scripts/lib/forge-spec.sh" scripts/lib/
    cp "$REPO_ROOT/scripts/lib/log.sh" scripts/lib/

    # Committed forge/github JSON bodies (forge-apply feeds these to gh api --input).
    mkdir -p forge/github
    cp "$REPO_ROOT"/forge/github/ruleset-*.json forge/github/
    cp "$REPO_ROOT/forge/github/actions-allowlist.json" forge/github/

    # Minimal git repo + origin remote so forge_spec_owner_repo works.
    git init --initial-branch=main -q
    git config user.name "Test User"
    git config user.email "test@example.com"
    git remote add origin "git@github.com:dgalbraith/sdmx-rs.git"

    # Call log for recording gh invocations.
    GH_CALL_LOG="$BATS_TEST_TMPDIR/gh-calls.log"
    export GH_CALL_LOG
    touch "$GH_CALL_LOG"

    # jq must be present (forge-apply.sh prerequisite check).
    # The devshell provides it; this ensures a clear error if somehow absent.
    command -v jq >/dev/null || skip "jq not available"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# Scrub ambient CI / tooling env variables, set up the apply shim, then run.
# FORGE_APPLY_YES=1 makes the run NON-INTERACTIVE: every caller of this helper is
# exercising the apply LOGIC, not the confirmation prompt, and must not block on
# `read` when the suite happens to run with a real TTY on stdin (e.g. `just
# test-scripts` in a terminal). The confirmation prompt has its own dedicated
# tests below that drive it explicitly via FORGE_APPLY_FORCE_PROMPT + piped input.
run_apply() {
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE FORGE_APPLY_FORCE
    unset FORGE_SKIP_RELEASE_ENV FORGE_DRY_RUN
    FORGE_APPLY_YES=1 run sh scripts/forge-apply.sh "$@"
}

# ==============================================================================
# Guard: blocks when rulesets already exist
# ==============================================================================

@test "forge-apply: guard blocks when rulesets present, exits 1" {
    GH_MOCK_HAS_RULESETS=1 mock_gh_apply
    run_apply
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Guard blocked"* ]]
    [[ "$output" == *"ruleset"* ]]
}

# ==============================================================================
# Guard: blocks when issues/PRs present
# ==============================================================================

@test "forge-apply: guard blocks when issues present, exits 1" {
    GH_MOCK_HAS_ISSUES=1 mock_gh_apply
    run_apply
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Guard blocked"* ]]
    [[ "$output" == *"issue"* ]]
}

# ==============================================================================
# Guard FAILS CLOSED: a forge error while probing must block, not apply
# ==============================================================================

@test "forge-apply: guard blocks when ruleset probe errors (fail-closed)" {
    GH_MOCK_RULESETS_FAIL=1 mock_gh_apply
    run_apply
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Guard blocked"* ]]
    [[ "$output" == *"could not determine"* ]]
    # And it must NOT have proceeded to mutate anything.
    if grep -qE "^(POST|PATCH|PUT|DELETE) " "$GH_CALL_LOG"; then
        echo "Mutating calls found after a fail-closed guard" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# --force overrides the guard
# ==============================================================================

@test "forge-apply: --force overrides guard, proceeds to apply" {
    GH_MOCK_HAS_RULESETS=1 mock_gh_apply
    run_apply --force
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"BYPASSED"* ]]
    [[ "$output" == *"bootstrap complete"* ]]
}

# ==============================================================================
# Clean-state run: correct endpoints called
# ==============================================================================

@test "forge-apply: clean state calls PATCH repos, POST rulesets, PATCH labels, PUT env" {
    mock_gh_apply
    run_apply
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    # PATCH /repos/{o}/{r} for merge flags + repo settings
    grep -q "PATCH repos/dgalbraith/sdmx-rs" "$GH_CALL_LOG" \
        || { echo "CALL LOG:"; cat "$GH_CALL_LOG" >&2; false; }

    # POST rulesets (3 times — one per spec ruleset)
    ruleset_posts="$(grep -c "POST repos/dgalbraith/sdmx-rs/rulesets " "$GH_CALL_LOG" || true)"
    [ "$ruleset_posts" -ge 3 ] \
        || { echo "Expected >=3 ruleset POSTs, got $ruleset_posts"; cat "$GH_CALL_LOG" >&2; false; }

    # PUT /actions/permissions/selected-actions (populate allowlist before mode flip)
    grep -q "PUT repos/dgalbraith/sdmx-rs/actions/permissions/selected-actions" "$GH_CALL_LOG" \
        || { echo "Missing selected-actions PUT"; cat "$GH_CALL_LOG" >&2; false; }

    # PUT /actions/permissions for enabled + allowed_actions=selected
    grep -q "PUT repos/dgalbraith/sdmx-rs/actions/permissions " "$GH_CALL_LOG" \
        || { echo "Missing actions/permissions PUT"; cat "$GH_CALL_LOG" >&2; false; }

    # PUT /actions/permissions/workflow for default-token least-privilege
    grep -q "PUT repos/dgalbraith/sdmx-rs/actions/permissions/workflow" "$GH_CALL_LOG" \
        || { echo "Missing workflow-permissions PUT"; cat "$GH_CALL_LOG" >&2; false; }

    # PUT /environments/release
    grep -q "repos/dgalbraith/sdmx-rs/environments/release" "$GH_CALL_LOG" \
        || { echo "Missing environments/release PUT"; cat "$GH_CALL_LOG" >&2; false; }

    # Security settings: enable vuln alerts (PUT), DISABLE Dependabot auto-fixes
    # (DELETE — signing invariant), enable private vuln reporting (PUT).
    grep -q "PUT repos/dgalbraith/sdmx-rs/vulnerability-alerts" "$GH_CALL_LOG" \
        || { echo "Missing vulnerability-alerts PUT"; cat "$GH_CALL_LOG" >&2; false; }
    grep -q "DELETE repos/dgalbraith/sdmx-rs/automated-security-fixes" "$GH_CALL_LOG" \
        || { echo "Missing automated-security-fixes DELETE"; cat "$GH_CALL_LOG" >&2; false; }
    grep -q "PUT repos/dgalbraith/sdmx-rs/private-vulnerability-reporting" "$GH_CALL_LOG" \
        || { echo "Missing private-vulnerability-reporting PUT"; cat "$GH_CALL_LOG" >&2; false; }
}

# ==============================================================================
# Actions allowlist: PUT allowed_actions=selected BEFORE PUT selected-actions
# GitHub rejects PUT /selected-actions while the mode is still "all", so the
# mode flip must happen first.
# ==============================================================================

@test "forge-apply: PUT allowed_actions=selected appears before PUT selected-actions in call log" {
    mock_gh_apply
    run_apply
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    # Capture line numbers for the two calls to verify ordering.
    permissions_line="$(grep -n "PUT repos/dgalbraith/sdmx-rs/actions/permissions " "$GH_CALL_LOG" | head -1 | cut -d: -f1)"
    selected_line="$(grep -n "PUT repos/dgalbraith/sdmx-rs/actions/permissions/selected-actions" "$GH_CALL_LOG" | head -1 | cut -d: -f1)"

    [ -n "$permissions_line" ] || { echo "PUT actions/permissions not found in call log"; cat "$GH_CALL_LOG" >&2; false; }
    [ -n "$selected_line" ] || { echo "PUT selected-actions not found in call log"; cat "$GH_CALL_LOG" >&2; false; }
    [ "$permissions_line" -lt "$selected_line" ] \
        || { echo "PUT permissions ($permissions_line) must precede PUT selected-actions ($selected_line)"; cat "$GH_CALL_LOG" >&2; false; }
}

# ==============================================================================
# Actions allowlist: missing allowlist file -> fatal before any mutation
# ==============================================================================

@test "forge-apply: missing actions-allowlist.json -> fatal, no mutations" {
    mock_gh_apply
    # Remove the allowlist so the missing-file guard fires.
    rm -f forge/github/actions-allowlist.json
    run_apply
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -ne 0 ]
    [[ "$output" == *"actions-allowlist.json"* ]]
    # No mutating call may have been made (guard fires in Step 2, before writes).
    if grep -qE "^(POST|PATCH|PUT|DELETE) " "$GH_CALL_LOG"; then
        echo "Mutating calls found after missing-file fatal" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# Actions allowlist: --dry-run logs selected-actions PUT but makes no real calls
# ==============================================================================

@test "forge-apply: --dry-run logs selected-actions PUT but makes no real call" {
    mock_gh_apply
    run_apply --dry-run
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"[dry-run] PUT actions/permissions/selected-actions"* ]]
    [[ "$output" == *"[dry-run] PUT actions/permissions: enabled=true, allowed_actions=selected"* ]]
    # No mutating API calls.
    if grep -qE "^(POST|PATCH|PUT|DELETE) " "$GH_CALL_LOG"; then
        echo "Mutating calls found in dry-run mode" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# --skip-release-env: no PUT environments call
# ==============================================================================

@test "forge-apply: --skip-release-env suppresses PUT environments call" {
    mock_gh_apply
    run_apply --skip-release-env
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    # No PUT environments call in the log.
    if grep -q "environments/release" "$GH_CALL_LOG"; then
        echo "Unexpected environments/release call found" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# --dry-run: no mutating gh api calls
# ==============================================================================

@test "forge-apply: --dry-run makes no gh api calls" {
    mock_gh_apply
    run_apply --dry-run
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"dry-run"* ]]

    # The call log should contain auth + guard GET probes only (no POST/PATCH/PUT).
    if grep -qE "^(POST|PATCH|PUT|DELETE) " "$GH_CALL_LOG"; then
        echo "Mutating calls found in dry-run mode" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# Mutation confirmation (§4b)
# ==============================================================================

# Non-TTY stdin (the bats default) auto-skips the prompt so scripted/CI runs do
# not hang — proven by every clean-state test above proceeding to apply. These
# add the explicit override + abort paths.

@test "forge-apply: --yes proceeds without prompting" {
    mock_gh_apply
    run_apply --yes
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"bootstrap complete"* ]]
    # The confirmation prompt must not appear.
    [[ "$output" != *"proceed?"* ]]
}

@test "forge-apply: FORGE_APPLY_YES=1 proceeds without prompting" {
    mock_gh_apply
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE FORGE_APPLY_FORCE
    unset FORGE_SKIP_RELEASE_ENV FORGE_DRY_RUN
    FORGE_APPLY_YES=1 run sh scripts/forge-apply.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"bootstrap complete"* ]]
}

@test "forge-apply: 'n' at the prompt aborts before any mutation" {
    mock_gh_apply
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE FORGE_APPLY_FORCE
    unset FORGE_SKIP_RELEASE_ENV FORGE_DRY_RUN
    # FORGE_APPLY_FORCE_PROMPT forces the prompt branch without a real TTY, so the
    # read+abort logic is exercised by a piped answer — NO pty allocation (a stray
    # pty can grab the terminal and hang `just verify`; that is why this is hook-
    # driven, not `script`-driven). Feed 'n' on stdin.
    run sh -c 'printf "n\n" | FORGE_APPLY_FORCE_PROMPT=1 sh scripts/forge-apply.sh'
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -ne 0 ]
    [[ "$output" == *"proceed?"* ]]
    [[ "$output" == *"Aborted by user"* ]]
    # No mutating call may have been made.
    if grep -qE "^(POST|PATCH|PUT|DELETE) " "$GH_CALL_LOG"; then
        echo "Mutating calls found after an aborted confirmation" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

@test "forge-apply: 'y' at the prompt proceeds to apply" {
    mock_gh_apply
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE FORGE_APPLY_FORCE
    unset FORGE_SKIP_RELEASE_ENV FORGE_DRY_RUN
    run sh -c 'printf "y\n" | FORGE_APPLY_FORCE_PROMPT=1 sh scripts/forge-apply.sh'
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"proceed?"* ]]
    [[ "$output" == *"bootstrap complete"* ]]
}
