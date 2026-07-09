#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/ci/check-published.sh and wait-for-index.sh
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    mkdir -p ci lib
    cp "$BATS_TEST_DIRNAME/../../scripts/ci/check-published.sh" ci/
    cp "$BATS_TEST_DIRNAME/../../scripts/ci/wait-for-index.sh" ci/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    # Setup mock curl wrapper
    mkdir -p bin
    cat > bin/curl << 'EOF'
#!/bin/sh
status_code=$(cat "$BATS_TEST_TMPDIR/mock_status" 2>/dev/null || echo "200")
body_file="$BATS_TEST_TMPDIR/mock_body"
curl_exit=$(cat "$BATS_TEST_TMPDIR/mock_curl_exit" 2>/dev/null || echo "0")

out_target=""
want_status=0
prev=""
for arg in "$@"; do
    case "$prev" in
        -o) out_target="$arg"; prev=""; continue ;;
        -w) prev=""; continue ;;
    esac
    case "$arg" in
        -o) prev="-o" ;;
        -w) want_status=1 ;;
    esac
done

if [ -n "$out_target" ]; then
    if [ -f "$body_file" ]; then
        cat "$body_file" > "$out_target"
    else
        : > "$out_target"
    fi
fi

if [ "$want_status" -eq 1 ]; then
    printf '%s' "$status_code"
fi

exit "$curl_exit"
EOF
    chmod +x bin/curl
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# ==============================================================================
# check-published.sh Tests
# ==============================================================================

@test "check-published: exits 0 and returns exists=true when version is indexed" {
    echo "200" > "$BATS_TEST_TMPDIR/mock_status"
    echo '{"vers":"0.1.0"}' > "$BATS_TEST_TMPDIR/mock_body"

    run_isolated ./ci/check-published.sh sdmx-types 0.1.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2

    [ "$status" -eq 0 ]
    [[ "$output" == *"exists=true"* ]]
}

@test "check-published: exits 0 and returns exists=false on 404 not found" {
    echo "404" > "$BATS_TEST_TMPDIR/mock_status"

    run_isolated ./ci/check-published.sh sdmx-types 0.1.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2

    [ "$status" -eq 0 ]
    [[ "$output" == *"exists=false"* ]]
}

@test "check-published: exits 2 on permanent client error (403)" {
    echo "403" > "$BATS_TEST_TMPDIR/mock_status"

    run_isolated ./ci/check-published.sh sdmx-types 0.1.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2

    [ "$status" -eq 2 ]
    [[ "$output" == *"Permanent client error"* ]]
}

@test "check-published: exits 3 on transient server error (502)" {
    echo "502" > "$BATS_TEST_TMPDIR/mock_status"

    run_isolated ./ci/check-published.sh sdmx-types 0.1.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2

    [ "$status" -eq 3 ]
    [[ "$output" == *"Transient server error"* ]]
}

@test "check-published: exits 3 on curl network/connection failure" {
    echo "7" > "$BATS_TEST_TMPDIR/mock_curl_exit"

    run_isolated ./ci/check-published.sh sdmx-types 0.1.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2

    [ "$status" -eq 3 ]
    [[ "$output" == *"Curl request failed"* ]]
}

# ==============================================================================
# wait-for-index.sh Tests
# ==============================================================================

@test "wait-for-index: exits 0 immediately when version is found" {
    echo "200" > "$BATS_TEST_TMPDIR/mock_status"
    echo '{"vers":"0.1.0"}' > "$BATS_TEST_TMPDIR/mock_body"

    run_isolated ./ci/wait-for-index.sh sdmx-types 0.1.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2

    [ "$status" -eq 0 ]
    [[ "$output" == *"sdmx-types 0.1.0 is indexed"* ]]
}

@test "wait-for-index: fails fast (exits 1) on permanent 403 error" {
    echo "403" > "$BATS_TEST_TMPDIR/mock_status"

    run_isolated ./ci/wait-for-index.sh sdmx-types 0.1.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2

    [ "$status" -eq 1 ]
    [[ "$output" == *"Permanent error (exit code 2)"* ]]
}

@test "wait-for-index: retries on transient errors and eventually fails" {
    # Simulate a 503 error, but modify MAX_RETRIES to a small number (e.g. 2)
    # so the test completes quickly. Value-agnostic anchor (^MAX_RETRIES=) so a
    # change to the production retry budget cannot silently no-op this patch and
    # leave the test sleeping through the full backoff against a 503 stub.
    sed -i 's/^MAX_RETRIES=.*/MAX_RETRIES=2/' ci/wait-for-index.sh
    echo "503" > "$BATS_TEST_TMPDIR/mock_status"

    run_isolated ./ci/wait-for-index.sh sdmx-types 0.1.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2

    [ "$status" -eq 1 ]
    # check-published.sh emits the transient error directly to stderr; wait-for-index
    # adds only the retry progress line and the final timeout message.
    [[ "$output" == *"Attempt 1/2"* ]]
    [[ "$output" == *"did not appear in the index after"* ]]
}
