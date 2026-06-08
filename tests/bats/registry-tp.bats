#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/registry-tp.sh
#
# registry-tp is PRINT-ONLY: it emits register/enforce commands but NEVER issues
# a mutating crates.io request and never holds a token to do so. These tests use
# the mock_crates `curl` shim (read-only GETs only) and assert that:
#   - the printed register/enforce commands are correct,
#   - it issues NO mutating curl (no -X POST/PATCH) — print-only contract,
#   - already-registered crates are skipped,
#   - --print-enforce refuses without the precondition.
#
# Run with: bats tests/bats/registry-tp.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    REPO_ROOT="$BATS_TEST_DIRNAME/../.."

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$REPO_ROOT/scripts/registry-tp.sh" scripts/
    cp "$REPO_ROOT/scripts/lib/registry-spec.sh" scripts/lib/
    cp "$REPO_ROOT/scripts/lib/forge-spec.sh" scripts/lib/
    cp "$REPO_ROOT/scripts/lib/log.sh" scripts/lib/

    git init --initial-branch=main -q
    git config user.email "dg@lbraith.io"
    git config user.name "David Galbraith"
    git remote add origin "git@github.com:dgalbraith/sdmx-rs.git"

    export FORGE_FIXTURES="$TMPDIR/forge-fixtures"
    mkdir -p "$FORGE_FIXTURES"
    cp -r "$REPO_ROOT/tests/bats/fixtures/crates" "$FORGE_FIXTURES/crates"

    # A recording curl wrapper around mock_crates' shim is unnecessary: the shim
    # rejects any mutating verb implicitly (it only maps GET-shaped URLs), and we
    # assert on the printed output. But to PROVE print-only, also assert the shim
    # is never asked to POST/PATCH by checking the script emits curl commands as
    # TEXT, never invoking them.
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

run_tp() {
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE
    run sh scripts/registry-tp.sh "$@"
}

# ==============================================================================
# --print-register prints the registration curl + reserve commands
# ==============================================================================

@test "registry-tp: --print-register prints POST curl for unregistered crates" {
    mock_crates
    # No token: cannot detect existing configs, so it prints the command for all.
    unset CRATES_IO_TOKEN
    run_tp --print-register
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"trusted_publishing/github_configs"* ]]
    [[ "$output" == *'"crate":"sdmx-types"'* ]]
    [[ "$output" == *'"workflow_filename":"publish.yml"'* ]]
    [[ "$output" == *'"environment":"release"'* ]]
}

# ==============================================================================
# Print-only contract: NO mutating curl is ever invoked
# ==============================================================================

@test "registry-tp: issues no mutating request (print-only)" {
    # Replace curl with a recorder that logs every invocation and fails the test
    # if asked to mutate. Register/enforce should produce ZERO such calls.
    mkdir -p "$BATS_TEST_TMPDIR/bin"
    CURL_LOG="$BATS_TEST_TMPDIR/curl.log"; export CURL_LOG
    cat > "$BATS_TEST_TMPDIR/bin/curl" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$CURL_LOG"
case " $* " in
    *" -X POST "*|*" -X PATCH "*|*" -X PUT "*|*" -X DELETE "*)
        echo "MUTATING CURL INVOKED: $*" >&2; exit 99 ;;
esac
# Read-only GETs: behave like the index/api as needed (empty configs is fine).
case "$*" in
    *index.crates.io*) printf '200' ;;
    *github_configs*)  echo '{"github_configs":[]}' ;;
    *api/v1/crates/*)  echo '{"crate":{"trustpub_only":false}}' ;;
esac
exit 0
EOF
    chmod +x "$BATS_TEST_TMPDIR/bin/curl"
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"

    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE CRATES_IO_TOKEN
    run sh scripts/registry-tp.sh --print-register
    echo "REGISTER OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    run sh scripts/registry-tp.sh --print-enforce
    echo "ENFORCE OUTPUT: $output" >&2
    [ "$status" -eq 0 ]

    # No mutating verb ever hit curl.
    if grep -qE "\-X (POST|PATCH|PUT|DELETE)" "$CURL_LOG"; then
        echo "Mutating curl found:" >&2; cat "$CURL_LOG" >&2; false
    fi
}

# ==============================================================================
# Already-registered crates are skipped (with a token to detect them)
# ==============================================================================

@test "registry-tp: --print-register skips already-registered crates" {
    mock_crates
    CRATES_IO_TOKEN=tok run sh scripts/registry-tp.sh --print-register
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    # All 5 are registered in the fixture -> all should be reported as such, with
    # no POST command emitted.
    [[ "$output" == *"already registered"* ]]
    [[ "$output" != *"trusted_publishing/github_configs\""* ]]
}

# ==============================================================================
# --print-enforce refuses when the precondition is unmet
# ==============================================================================

@test "registry-tp: --print-enforce skips unreserved crates" {
    mock_crates
    # Make sdmx-rs unreserved -> enforce must skip it, not print a PATCH.
    rm -f "$FORGE_FIXTURES/crates/reserved/sdmx-rs"
    CRATES_IO_TOKEN=tok run sh scripts/registry-tp.sh --print-enforce
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"SKIP — not yet published"* ]]
}

@test "registry-tp: --print-enforce prints PATCH for a ready crate" {
    mock_crates
    CRATES_IO_TOKEN=tok run sh scripts/registry-tp.sh --print-enforce
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *'"trustpub_only":true'* ]]
    [[ "$output" == *"PATCH"* ]]
}

# ==============================================================================
# Unknown argument is rejected
# ==============================================================================

@test "registry-tp: unknown argument exits non-zero" {
    mock_crates
    run_tp --bogus
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -ne 0 ]
    [[ "$output" == *"Unknown argument"* ]]
}
