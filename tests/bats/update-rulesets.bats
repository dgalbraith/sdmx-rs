#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/update-rulesets.sh
#
# Testing approach: update-rulesets never touches a live forge — `gh` is
# replaced by mock_gh_update (a recording shim that logs every call to
# $GH_CALL_LOG without making network requests). Guard-state control mirrors
# the mock_gh_apply pattern in forge-apply.bats.
#
# Test contracts:
#   - All rulesets absent (fresh repo)  → POST for each spec ruleset.
#   - All rulesets present (re-run)     → PUT for each, with correct ID.
#   - Mixed state (partial apply)       → correct verb per ruleset.
#   - Duplicate name found              → abort, no mutations.
#   - Signing ruleset PUT               → bypass invariant re-asserted.
#   - Signing ruleset bypass non-empty  → fail after PUT.
#   - Missing committed ruleset file    → abort before any mutation.
#   - --dry-run                         → no mutating calls.
#
# Run with: bats tests/bats/update-rulesets.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    REPO_ROOT="$BATS_TEST_DIRNAME/../.."

    cd "$BATS_TEST_TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$REPO_ROOT/scripts/update-rulesets.sh" scripts/
    cp "$REPO_ROOT/scripts/lib/forge-spec.sh" scripts/lib/
    cp "$REPO_ROOT/scripts/lib/log.sh" scripts/lib/

    mkdir -p forge/github
    cp "$REPO_ROOT"/forge/github/ruleset-*.json forge/github/

    git init --initial-branch=main -q
    git config user.name "Test User"
    git config user.email "test@example.com"
    git remote add origin "git@github.com:dgalbraith/sdmx-rs.git"

    GH_CALL_LOG="$BATS_TEST_TMPDIR/gh-calls.log"
    export GH_CALL_LOG
    touch "$GH_CALL_LOG"

    command -v jq >/dev/null || skip "jq not available"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# Inject a recording `gh` shim for update-rulesets tests.
#
# State control (set before calling mock_gh_update):
#   GH_MOCK_RULESETS_STATE  "none"   → /rulesets returns [] (all absent)
#                           "all"    → /rulesets returns all 3 spec rulesets
#                           "mixed"  → /rulesets returns only the signing ruleset
#                           "dup"    → /rulesets returns 2 copies of signing ruleset
#   GH_MOCK_BYPASS_COUNT              "0" → signing ruleset has empty bypass (default)
#                                     "1" → signing ruleset has 1 bypass actor (invariant fail)
#   GH_MOCK_PUSH_RESTRICT_BYPASS_COUNT "0" → push-restriction has empty bypass (default)
#                                      "1" → push-restriction has 1 bypass actor (invariant fail)
mock_gh_update() {
    : "${GH_CALL_LOG:?mock_gh_update requires GH_CALL_LOG to be set}"
    mkdir -p "$BATS_TEST_TMPDIR/bin"

    cat > "$BATS_TEST_TMPDIR/bin/gh" << EOF
#!/bin/sh
GH_CALL_LOG="$GH_CALL_LOG"
GH_MOCK_RULESETS_STATE="${GH_MOCK_RULESETS_STATE:-none}"
GH_MOCK_BYPASS_COUNT="${GH_MOCK_BYPASS_COUNT:-0}"
GH_MOCK_PUSH_RESTRICT_BYPASS_COUNT="${GH_MOCK_PUSH_RESTRICT_BYPASS_COUNT:-0}"
EOF
    cat >> "$BATS_TEST_TMPDIR/bin/gh" << 'EOF'

printf '%s\n' "$*" >> "$GH_CALL_LOG"

if [ "$1" = "auth" ] && [ "$2" = "status" ]; then exit 0; fi

if [ "$1" = "api" ]; then
    shift
    method="GET"
    endpoint=""
    for arg in "$@"; do
        case "$arg" in
            --method|-X) method="" ;;
            --input|-f|-F|--jq|--silent) ;;
            *)
                if [ -z "$method" ]; then
                    method="$arg"
                elif [ -z "$endpoint" ]; then
                    case "$arg" in -*) ;; *) endpoint="$arg" ;; esac
                fi
                ;;
        esac
    done

    case "$method" in
        POST) printf '{"id":1}\n'; exit 0 ;;
        PUT|DELETE) echo '{}'; exit 0 ;;
    esac

    case "$endpoint" in
        repos/*/rulesets/2)
            # Single-ruleset fetch for push-restriction (id 2) bypass-invariant check.
            if [ "$GH_MOCK_PUSH_RESTRICT_BYPASS_COUNT" = "0" ]; then
                printf '{"id":2,"name":"Zero Trust Gate","bypass_actors":[]}\n'
            else
                printf '{"id":2,"name":"Zero Trust Gate","bypass_actors":[{"actor_id":5}]}\n'
            fi
            exit 0 ;;
        repos/*/rulesets/*)
            # Single-ruleset fetch for signing (id 1) bypass-invariant check.
            if [ "$GH_MOCK_BYPASS_COUNT" = "0" ]; then
                printf '{"id":1,"name":"Enforce High Integrity Development","bypass_actors":[]}\n'
            else
                printf '{"id":1,"name":"Enforce High Integrity Development","bypass_actors":[{"actor_id":5}]}\n'
            fi
            exit 0 ;;
        repos/*/rulesets)
            case "$GH_MOCK_RULESETS_STATE" in
                none)
                    printf '[]\n' ;;
                all)
                    printf '[{"id":1,"name":"Enforce High Integrity Development","bypass_actors":[]},{"id":2,"name":"Zero Trust Gate","bypass_actors":[]},{"id":3,"name":"Protect Release Tags","bypass_actors":[]}]\n' ;;
                mixed)
                    printf '[{"id":1,"name":"Enforce High Integrity Development","bypass_actors":[]}]\n' ;;
                dup)
                    printf '[{"id":1,"name":"Enforce High Integrity Development","bypass_actors":[]},{"id":9,"name":"Enforce High Integrity Development","bypass_actors":[]}]\n' ;;
                bad_json)
                    printf '<html>502 Bad Gateway</html>\n' ;;
                *)
                    printf '[]\n' ;;
            esac
            exit 0 ;;
        repos/*)
            echo "mock gh (update-rulesets): unmapped repos endpoint: $endpoint" >&2
            exit 1 ;;
    esac

    echo "mock gh (update-rulesets): unmapped endpoint: $endpoint" >&2
    exit 1
fi

echo "mock gh (update-rulesets): unhandled: $*" >&2
exit 1
EOF

    chmod +x "$BATS_TEST_TMPDIR/bin/gh"
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

run_update() {
    unset FORGE_DRY_RUN
    run sh scripts/update-rulesets.sh "$@"
}

# ==============================================================================
# All rulesets absent — POST for each
# ==============================================================================

@test "update-rulesets: no existing rulesets -> POST for each spec ruleset" {
    GH_MOCK_RULESETS_STATE=none mock_gh_update
    run_update
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    post_count="$(grep -c -- "--method POST repos/dgalbraith/sdmx-rs/rulesets " "$GH_CALL_LOG" || true)"
    [ "$post_count" -ge 3 ] \
        || { echo "Expected >=3 POSTs, got $post_count"; cat "$GH_CALL_LOG" >&2; false; }

    # No PUTs to rulesets/{id}
    if grep -qE -- "--method PUT repos/dgalbraith/sdmx-rs/rulesets/[0-9]+" "$GH_CALL_LOG"; then
        echo "Unexpected PUT found when all rulesets were absent" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# All rulesets present — PUT for each
# ==============================================================================

@test "update-rulesets: all rulesets present -> PUT for each with correct ID" {
    GH_MOCK_RULESETS_STATE=all mock_gh_update
    run_update
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    # One PUT per spec ruleset (IDs 1, 2, 3)
    for id in 1 2 3; do
        grep -q -- "--method PUT repos/dgalbraith/sdmx-rs/rulesets/$id " "$GH_CALL_LOG" \
            || { echo "Missing PUT rulesets/$id"; cat "$GH_CALL_LOG" >&2; false; }
    done

    # No POSTs to /rulesets (create path)
    if grep -q -- "--method POST repos/dgalbraith/sdmx-rs/rulesets " "$GH_CALL_LOG"; then
        echo "Unexpected POST found when all rulesets were present" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# Mixed state — correct verb per ruleset
# ==============================================================================

@test "update-rulesets: mixed state -> POST for absent, PUT for present" {
    # Only signing ruleset (id=1) exists; the other two are absent.
    GH_MOCK_RULESETS_STATE=mixed mock_gh_update
    run_update
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    grep -q -- "--method PUT repos/dgalbraith/sdmx-rs/rulesets/1 " "$GH_CALL_LOG" \
        || { echo "Missing PUT for existing signing ruleset"; cat "$GH_CALL_LOG" >&2; false; }

    post_count="$(grep -c -- "--method POST repos/dgalbraith/sdmx-rs/rulesets " "$GH_CALL_LOG" || true)"
    [ "$post_count" -ge 2 ] \
        || { echo "Expected >=2 POSTs for absent rulesets, got $post_count"; cat "$GH_CALL_LOG" >&2; false; }
}

# ==============================================================================
# Duplicate ruleset name — abort, no mutations
# ==============================================================================

@test "update-rulesets: duplicate ruleset name -> aborts, no mutations" {
    GH_MOCK_RULESETS_STATE=dup mock_gh_update
    run_update
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -ne 0 ]
    [[ "$output" == *"Duplicate ruleset"* ]]

    # No POST or PUT to rulesets endpoint (no mutation after duplicate detected).
    if grep -qE -- "--method (POST|PUT) repos/dgalbraith/sdmx-rs/rulesets" "$GH_CALL_LOG"; then
        echo "Mutating calls found after duplicate detection" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# Signing ruleset bypass invariant — empty bypass passes
# ==============================================================================

@test "update-rulesets: signing ruleset PUT with empty bypass -> passes invariant check" {
    GH_MOCK_RULESETS_STATE=all GH_MOCK_BYPASS_COUNT=0 mock_gh_update
    run_update
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"bypass list is empty"* ]]
}

# ==============================================================================
# Signing ruleset bypass invariant — non-empty bypass fails
# ==============================================================================

@test "update-rulesets: signing ruleset PUT with non-empty bypass -> fails invariant check" {
    GH_MOCK_RULESETS_STATE=all GH_MOCK_BYPASS_COUNT=1 mock_gh_update
    run_update
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -ne 0 ]
    [[ "$output" == *"bypass actor"* ]]
}

# ==============================================================================
# Push-restriction ruleset bypass invariant — empty bypass passes
# ==============================================================================

@test "update-rulesets: push-restriction ruleset PUT with empty bypass -> passes invariant check" {
    GH_MOCK_RULESETS_STATE=all GH_MOCK_PUSH_RESTRICT_BYPASS_COUNT=0 mock_gh_update
    run_update
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"bypass list is empty"* ]]
}

# ==============================================================================
# Push-restriction ruleset bypass invariant — non-empty bypass fails
# ==============================================================================

@test "update-rulesets: push-restriction ruleset PUT with non-empty bypass -> fails invariant check" {
    GH_MOCK_RULESETS_STATE=all GH_MOCK_PUSH_RESTRICT_BYPASS_COUNT=1 mock_gh_update
    run_update
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -ne 0 ]
    [[ "$output" == *"bypass actor"* ]]
}

# ==============================================================================
# Missing committed ruleset file — abort before mutations
# ==============================================================================

@test "update-rulesets: missing ruleset file -> aborts, no mutations" {
    GH_MOCK_RULESETS_STATE=none mock_gh_update
    rm -f forge/github/ruleset-signing.json
    run_update
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -ne 0 ]
    [[ "$output" == *"missing"* ]]

    if grep -qE -- "--method (POST|PUT) " "$GH_CALL_LOG"; then
        echo "Mutating calls found after missing-file abort" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# Invalid JSON response from rulesets list — fatal, no mutations
# ==============================================================================

@test "update-rulesets: invalid JSON rulesets response -> fatal, no mutations" {
    GH_MOCK_RULESETS_STATE=bad_json mock_gh_update
    run_update
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -ne 0 ]
    [[ "$output" == *"not a valid JSON array"* ]]

    if grep -qE -- "--method (POST|PUT) " "$GH_CALL_LOG"; then
        echo "Mutating calls found after invalid JSON fatal" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# --dry-run — no mutating calls
# ==============================================================================

@test "update-rulesets: --dry-run makes no mutating calls" {
    GH_MOCK_RULESETS_STATE=none mock_gh_update
    run_update --dry-run
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"dry-run"* ]]

    if grep -qE -- "--method (POST|PATCH|PUT|DELETE) " "$GH_CALL_LOG"; then
        echo "Mutating calls found in dry-run mode" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# FORGE_DRY_RUN=1 env override
# ==============================================================================

@test "update-rulesets: FORGE_DRY_RUN=1 makes no mutating calls" {
    GH_MOCK_RULESETS_STATE=none mock_gh_update
    unset FORGE_DRY_RUN
    FORGE_DRY_RUN=1 run sh scripts/update-rulesets.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    if grep -qE -- "--method (POST|PATCH|PUT|DELETE) " "$GH_CALL_LOG"; then
        echo "Mutating calls found when FORGE_DRY_RUN=1" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}
