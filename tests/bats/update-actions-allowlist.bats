#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/update-actions-allowlist.sh
#
# Testing approach: update-actions-allowlist never touches a live forge — `gh`
# is replaced by mock_gh_allowlist (a recording shim that logs every call to
# $GH_CALL_LOG).
#
# Test contracts:
#   - Clean run → PUT selected-actions from the committed file.
#   - Missing committed allowlist file → fatal before any mutation.
#   - --dry-run → no mutating calls, logs the intended PUT.
#   - FORGE_DRY_RUN=1 → same as --dry-run.
#   - gh PUT failure → exits non-zero.
#
# Run with: bats tests/bats/update-actions-allowlist.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    REPO_ROOT="$BATS_TEST_DIRNAME/../.."

    cd "$BATS_TEST_TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$REPO_ROOT/scripts/update-actions-allowlist.sh" scripts/
    cp "$REPO_ROOT/scripts/lib/forge-spec.sh" scripts/lib/
    cp "$REPO_ROOT/scripts/lib/log.sh" scripts/lib/

    mkdir -p forge/github
    cp "$REPO_ROOT/forge/github/actions-allowlist.json" forge/github/

    git init --initial-branch=main -q
    git config user.name "Test User"
    git config user.email "test@example.com"
    git remote add origin "git@github.com:dgalbraith/sdmx-rs.git"

    GH_CALL_LOG="$BATS_TEST_TMPDIR/gh-calls.log"
    export GH_CALL_LOG
    touch "$GH_CALL_LOG"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# Inject a recording `gh` shim for update-actions-allowlist tests.
#
# State control (set before calling mock_gh_allowlist):
#   GH_MOCK_PUT_FAIL=1  → PUT exits non-zero (models a forge error)
mock_gh_allowlist() {
    : "${GH_CALL_LOG:?mock_gh_allowlist requires GH_CALL_LOG to be set}"
    mkdir -p "$BATS_TEST_TMPDIR/bin"

    cat > "$BATS_TEST_TMPDIR/bin/gh" << EOF
#!/bin/sh
GH_CALL_LOG="$GH_CALL_LOG"
GH_MOCK_PUT_FAIL="${GH_MOCK_PUT_FAIL:-0}"
EOF
    cat >> "$BATS_TEST_TMPDIR/bin/gh" << 'EOF'

printf '%s\n' "$*" >> "$GH_CALL_LOG"

if [ "$1" = "auth" ] && [ "$2" = "status" ]; then exit 0; fi

if [ "$1" = "api" ]; then
    shift
    method="GET"
    for arg in "$@"; do
        case "$arg" in
            --method|-X) method="" ;;
            --input|-f|-F|--jq|--silent) ;;
            *)
                if [ -z "$method" ]; then method="$arg"; fi ;;
        esac
    done

    case "$method" in
        PUT)
            if [ "$GH_MOCK_PUT_FAIL" = "1" ]; then
                echo "mock gh: PUT failed" >&2; exit 1
            fi
            echo '{}'; exit 0 ;;
    esac

    echo "mock gh (update-allowlist): unmapped method=$method" >&2
    exit 1
fi

echo "mock gh (update-allowlist): unhandled: $*" >&2
exit 1
EOF

    chmod +x "$BATS_TEST_TMPDIR/bin/gh"
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

run_update_allowlist() {
    unset FORGE_DRY_RUN
    run sh scripts/update-actions-allowlist.sh "$@"
}

# ==============================================================================
# Clean run — PUT selected-actions from committed file
# ==============================================================================

@test "update-actions-allowlist: clean run -> PUT selected-actions" {
    mock_gh_allowlist
    run_update_allowlist
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    grep -q -- "--method PUT repos/dgalbraith/sdmx-rs/actions/permissions/selected-actions" "$GH_CALL_LOG" \
        || { echo "Missing PUT selected-actions in call log"; cat "$GH_CALL_LOG" >&2; false; }
}

# ==============================================================================
# Missing allowlist file — fatal before any mutation
# ==============================================================================

@test "update-actions-allowlist: missing allowlist file -> fatal, no mutations" {
    mock_gh_allowlist
    rm -f forge/github/actions-allowlist.json
    run_update_allowlist
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -ne 0 ]
    [[ "$output" == *"actions-allowlist.json"* ]]

    if grep -qE -- "--method (POST|PATCH|PUT|DELETE) " "$GH_CALL_LOG"; then
        echo "Mutating calls found after missing-file fatal" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# gh PUT failure — exits non-zero
# ==============================================================================

@test "update-actions-allowlist: PUT failure -> exits non-zero" {
    GH_MOCK_PUT_FAIL=1 mock_gh_allowlist
    run_update_allowlist
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -ne 0 ]
}

# ==============================================================================
# --dry-run — logs intended PUT, no mutating calls
# ==============================================================================

@test "update-actions-allowlist: --dry-run logs PUT but makes no real call" {
    mock_gh_allowlist
    run_update_allowlist --dry-run
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"[dry-run] PUT"* ]]
    [[ "$output" == *"selected-actions"* ]]

    if grep -qE -- "--method (POST|PATCH|PUT|DELETE) " "$GH_CALL_LOG"; then
        echo "Mutating calls found in dry-run mode" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# FORGE_DRY_RUN=1 env override
# ==============================================================================

@test "update-actions-allowlist: FORGE_DRY_RUN=1 makes no mutating calls" {
    mock_gh_allowlist
    unset FORGE_DRY_RUN
    FORGE_DRY_RUN=1 run sh scripts/update-actions-allowlist.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    if grep -qE -- "--method (POST|PATCH|PUT|DELETE) " "$GH_CALL_LOG"; then
        echo "Mutating calls found when FORGE_DRY_RUN=1" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}
