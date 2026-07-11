#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/ci/compare-registry-crate.sh
#
# Proves the three behaviours publish.yml relies on before it attests a crate:
#   - MATCH:       served bytes equal the packaged artefact -> exit 0.
#   - MISMATCH:    served bytes differ -> fail hard, name the artefact, no retry.
#   - NOT-SERVED:  the CDN has not published the .crate yet -> bounded retry,
#                  then a distinct failure (not a mismatch).
#
# The registry is stubbed by a self-contained curl shim (mirroring
# wait-for-index.bats): a mock_status file drives the HTTP code and a mock_body
# file drives the served bytes.
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    mkdir -p ci lib pkg
    cp "$BATS_TEST_DIRNAME/../../scripts/ci/compare-registry-crate.sh" ci/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    # The locally packaged .crate: the attestation subject the guard hashes.
    LOCAL_FILE="pkg/sdmx-types-0.1.0.crate"
    printf 'packaged-crate-bytes' > "$LOCAL_FILE"

    # Stub curl: honours -o <file> and -w '%{http_code}', writing the canned
    # served body to the -o target and echoing the canned status for -w.
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
# MATCH path
# ==============================================================================

@test "compare-registry-crate: exits 0 when served bytes match the local artefact" {
    echo "200" > "$BATS_TEST_TMPDIR/mock_status"
    printf 'packaged-crate-bytes' > "$BATS_TEST_TMPDIR/mock_body"

    run_isolated ./ci/compare-registry-crate.sh sdmx-types 0.1.0 "$LOCAL_FILE"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2

    [ "$status" -eq 0 ]
    [[ "$output" == *"matches the registry-served .crate"* ]]
}

# ==============================================================================
# MISMATCH path: fail hard, name the artefact, no retry
# ==============================================================================

@test "compare-registry-crate: exits 1 and names the artefact on a byte mismatch" {
    echo "200" > "$BATS_TEST_TMPDIR/mock_status"
    printf 'tampered-registry-bytes' > "$BATS_TEST_TMPDIR/mock_body"

    run_isolated ./ci/compare-registry-crate.sh sdmx-types 0.1.0 "$LOCAL_FILE"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2

    [ "$status" -eq 1 ]
    [[ "$output" == *"SHA-256 MISMATCH"* ]]
    [[ "$output" == *"pkg/sdmx-types-0.1.0.crate"* ]]
    # A mismatch is not a transient state, so it must not enter the retry loop.
    [[ "$output" != *"Attempt "* ]]
    [[ "$output" != *"was not served"* ]]
}

# ==============================================================================
# NOT-SERVED path: bounded retry, then a failure distinct from a mismatch
# ==============================================================================

@test "compare-registry-crate: retries a not-yet-served .crate then fails distinctly" {
    # Shrink the retry budget so the test does not sleep through the full
    # backoff. Value-agnostic anchor (^MAX_RETRIES=) so a change to the
    # production budget cannot silently no-op this patch.
    sed -i 's/^MAX_RETRIES=.*/MAX_RETRIES=2/' ci/compare-registry-crate.sh
    echo "404" > "$BATS_TEST_TMPDIR/mock_status"

    run_isolated ./ci/compare-registry-crate.sh sdmx-types 0.1.0 "$LOCAL_FILE"
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2

    [ "$status" -eq 1 ]
    [[ "$output" == *"not yet served"* ]]
    [[ "$output" == *"Attempt 1/2"* ]]
    [[ "$output" == *"was not served by the registry after"* ]]
    # Distinct from a mismatch: the two failure modes never share wording.
    [[ "$output" != *"MISMATCH"* ]]
}

# ==============================================================================
# Argument / precondition guards
# ==============================================================================

@test "compare-registry-crate: exits 1 when the local crate file is missing" {
    echo "200" > "$BATS_TEST_TMPDIR/mock_status"
    printf 'packaged-crate-bytes' > "$BATS_TEST_TMPDIR/mock_body"

    run_isolated ./ci/compare-registry-crate.sh sdmx-types 0.1.0 pkg/absent.crate
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2

    [ "$status" -eq 1 ]
    [[ "$output" == *"Local crate file not found"* ]]
}
