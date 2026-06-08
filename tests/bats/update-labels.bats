#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/update-labels.sh
#
# Testing approach: update-labels never touches a live forge — `gh` is replaced
# by mock_gh_labels (a recording shim that logs every call to $GH_CALL_LOG).
#
# Test contracts:
#   - Label absent (PATCH 404) → POST to create.
#   - Label present (PATCH 200) → update in place, no POST.
#   - PATCH non-404 error → report failure, do NOT fall through to POST.
#   - --dry-run → no mutating calls.
#   - FORGE_DRY_RUN=1 → same as --dry-run.
#
# Run with: bats tests/bats/update-labels.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    REPO_ROOT="$BATS_TEST_DIRNAME/../.."

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$REPO_ROOT/scripts/update-labels.sh" scripts/
    cp "$REPO_ROOT/scripts/lib/forge-spec.sh" scripts/lib/
    cp "$REPO_ROOT/scripts/lib/log.sh" scripts/lib/

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
    rm -rf "$TMPDIR"
}

# Inject a recording `gh` shim for update-labels tests.
#
# State control (set before calling mock_gh_labels):
#   GH_MOCK_LABEL_STATE   "present" → PATCH succeeds (200) for all labels
#                         "absent"  → PATCH exits 1 with "404 Not Found" in
#                                     stderr (label does not exist; POST needed)
#                         "error"   → PATCH exits 1 with a non-404 error
#                                     (5xx / permission) — must NOT fall to POST
mock_gh_labels() {
    : "${GH_CALL_LOG:?mock_gh_labels requires GH_CALL_LOG to be set}"
    mkdir -p "$BATS_TEST_TMPDIR/bin"

    cat > "$BATS_TEST_TMPDIR/bin/gh" << EOF
#!/bin/sh
GH_CALL_LOG="$GH_CALL_LOG"
GH_MOCK_LABEL_STATE="${GH_MOCK_LABEL_STATE:-present}"
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
        POST)
            echo '{}'; exit 0 ;;
        PATCH)
            case "$GH_MOCK_LABEL_STATE" in
                present)
                    echo '{}'; exit 0 ;;
                absent)
                    echo "404 Not Found" >&2; exit 1 ;;
                error)
                    echo "500 Internal Server Error" >&2; exit 1 ;;
            esac ;;
        DELETE)
            echo '{}'; exit 0 ;;
    esac

    echo "mock gh (update-labels): unmapped: method=$method endpoint=$endpoint" >&2
    exit 1
fi

echo "mock gh (update-labels): unhandled: $*" >&2
exit 1
EOF

    chmod +x "$BATS_TEST_TMPDIR/bin/gh"
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

run_update_labels() {
    unset FORGE_DRY_RUN
    run sh scripts/update-labels.sh "$@"
}

# ==============================================================================
# All labels present — PATCH only, no POST
# ==============================================================================

@test "update-labels: labels present -> PATCH only, no POST" {
    GH_MOCK_LABEL_STATE=present mock_gh_labels
    run_update_labels
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    patch_count="$(grep -c -- "--method PATCH " "$GH_CALL_LOG" || true)"
    [ "$patch_count" -ge 14 ] \
        || { echo "Expected >=14 PATCHes, got $patch_count"; cat "$GH_CALL_LOG" >&2; false; }

    if grep -q -- "--method POST " "$GH_CALL_LOG"; then
        echo "Unexpected POST found when labels are present" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# All labels absent — PATCH 404 falls through to POST for each
# ==============================================================================

@test "update-labels: labels absent -> POST for each after PATCH 404" {
    GH_MOCK_LABEL_STATE=absent mock_gh_labels
    run_update_labels
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    post_count="$(grep -c -- "--method POST " "$GH_CALL_LOG" || true)"
    [ "$post_count" -ge 14 ] \
        || { echo "Expected >=14 POSTs, got $post_count"; cat "$GH_CALL_LOG" >&2; false; }
}

# ==============================================================================
# Non-404 PATCH error — must NOT fall through to POST
# ==============================================================================

@test "update-labels: non-404 PATCH error -> failure, no POST fallthrough" {
    GH_MOCK_LABEL_STATE=error mock_gh_labels
    run_update_labels
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -ne 0 ]

    # The error path must record failures and not issue any POST.
    if grep -q -- "--method POST " "$GH_CALL_LOG"; then
        echo "POST found after non-404 PATCH error — should not fall through" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}

# ==============================================================================
# --dry-run — no mutating calls
# ==============================================================================

@test "update-labels: --dry-run makes no mutating calls" {
    GH_MOCK_LABEL_STATE=present mock_gh_labels
    run_update_labels --dry-run
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

@test "update-labels: FORGE_DRY_RUN=1 makes no mutating calls" {
    GH_MOCK_LABEL_STATE=present mock_gh_labels
    unset FORGE_DRY_RUN
    FORGE_DRY_RUN=1 run sh scripts/update-labels.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    if grep -qE -- "--method (POST|PATCH|PUT|DELETE) " "$GH_CALL_LOG"; then
        echo "Mutating calls found when FORGE_DRY_RUN=1" >&2
        cat "$GH_CALL_LOG" >&2
        false
    fi
}
