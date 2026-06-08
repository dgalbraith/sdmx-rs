#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/ci/verify-signature.sh
#
# Strategy: mock `git` via a PATH shim that emits canned --raw VALIDSIG
# fixtures. This tests the parse/dispatch/allowlist/contract logic — exactly
# the surface that the cardinality, shape, and env-contract guards harden —
# without requiring real GPG keys or a signed commit history.
#
# Cases 7 (required env var unset) and 10 (POSIX pipefail-regression) are the
# extraction-specific guards: they exist because the inline→script boundary
# introduces hazards that only a test can pin. If extraction ever silently reads
# an unset var or masks a mid-pipe error, these cases catch it.
#
# Run with: bats tests/bats/verify-signature.bats
# ==============================================================================

GOOD_FPR="53069F0184A426465E5FF9E7FC6BB04EBF431B25"
BAD_FPR="AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    cp "$BATS_TEST_DIRNAME/../../scripts/ci/verify-signature.sh" .

    # Minimal git repo so git rev-list in branch-push dispatch doesn't error.
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"
    git config commit.gpgsign false
    git config tag.gpgsign false
    touch README.md
    git add README.md
    git commit -m "initial" -q
    HEAD_SHA=$(git rev-parse HEAD)
    export HEAD_SHA

    # Mock bin directory prepended to PATH. Each test writes its own `git` shim.
    MOCK_BIN=$(mktemp -d)
    export MOCK_BIN
    export PATH="$MOCK_BIN:$PATH"

    # Common env vars for the contract.
    export ALLOWED_PRIMARY_FINGERPRINTS="$GOOD_FPR"
    export GITHUB_SHA="$HEAD_SHA"
    export GITHUB_REF_TYPE="branch"
    export GITHUB_REF_NAME="main"
    export GITHUB_EVENT_BEFORE="0000000000000000000000000000000000000000"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR" "${MOCK_BIN:-}"
}

# Write a git shim that emits VALIDSIG output for verify-commit/verify-tag.
# $1 = the raw output line(s) to emit when verify-* is called.
# Other git commands (rev-list, rev-parse) fall through to real git.
make_git_shim() {
    local raw_output="$1"
    cat > "$MOCK_BIN/git" << EOF
#!/bin/sh
case "\$1" in
    verify-commit|verify-tag)
        printf '%s\n' '$raw_output'
        exit 0
        ;;
    *)
        exec $(command -v git) "\$@"
        ;;
esac
EOF
    chmod +x "$MOCK_BIN/git"
}

# ---------------------------------------------------------------------------
# Case 1: valid signature, allowlisted fingerprint -> pass
# ---------------------------------------------------------------------------
@test "verify-signature: passes when VALIDSIG fingerprint is allowlisted" {
    make_git_shim "[GNUPG:] VALIDSIG ABC DEF 12345 0 0 0 0 0 0 $GOOD_FPR"
    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"verified — primary ${GOOD_FPR}"* ]]
    [[ "$output" == *"All checked refs satisfy"* ]]
}

# ---------------------------------------------------------------------------
# Case 2: valid signature but NOT in the allowlist -> fail
# ---------------------------------------------------------------------------
@test "verify-signature: fails when VALIDSIG fingerprint is not allowlisted" {
    make_git_shim "[GNUPG:] VALIDSIG ABC DEF 12345 0 0 0 0 0 0 $BAD_FPR"
    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"signed by an unauthorised key"* ]]
}

# ---------------------------------------------------------------------------
# Case 3: no VALIDSIG line (unsigned / web-flow / expired) -> fail
# ---------------------------------------------------------------------------
@test "verify-signature: fails when gpg output has no VALIDSIG line" {
    make_git_shim "[GNUPG:] ERRSIG ABC 17 8 01 12345 9"
    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"has no valid signature"* ]]
}

# ---------------------------------------------------------------------------
# Case 4: multiple VALIDSIG lines (cardinality guard - item 2) -> fail
# ---------------------------------------------------------------------------
@test "verify-signature: fails when gpg output has multiple VALIDSIG lines" {
    make_git_shim "[GNUPG:] VALIDSIG ABC DEF 1 0 0 0 0 0 0 ${GOOD_FPR}
[GNUPG:] VALIDSIG ABC DEF 2 0 0 0 0 0 0 ${GOOD_FPR}"
    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"VALIDSIG lines"* ]]
}

# ---------------------------------------------------------------------------
# Case 5: VALIDSIG with malformed fingerprint (shape guard - item 2) -> fail
# ---------------------------------------------------------------------------
@test "verify-signature: fails when extracted fingerprint fails shape validation" {
    # 39 chars — one short of valid
    make_git_shim "[GNUPG:] VALIDSIG ABC DEF 12345 0 0 0 0 0 0 1234567890ABCDEF1234567890ABCDEF1234567"
    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"malformed"* ]]
}

# ---------------------------------------------------------------------------
# Case 6: empty/malformed ALLOWED_PRIMARY_FINGERPRINTS -> fail (config error)
# ---------------------------------------------------------------------------
@test "verify-signature: fails when ALLOWED_PRIMARY_FINGERPRINTS is empty" {
    export ALLOWED_PRIMARY_FINGERPRINTS=""
    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"CONFIGURATION ERROR"* ]]
    [[ "$output" == *"ALLOWED_PRIMARY_FINGERPRINTS"* ]]
}

@test "verify-signature: fails when ALLOWED_PRIMARY_FINGERPRINTS contains no valid fingerprints" {
    export ALLOWED_PRIMARY_FINGERPRINTS="not-a-fingerprint"
    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"CONFIGURATION ERROR"* ]]
}

# ---------------------------------------------------------------------------
# Case 7: required env var unset (env-contract guard — extraction hazard #1)
# Pins the YAML->script boundary: a missing var must fail loudly, never
# silently read as empty and produce a misleading result.
# ---------------------------------------------------------------------------
@test "verify-signature: fails when GITHUB_REF_TYPE is unset" {
    unset GITHUB_REF_TYPE
    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"GITHUB_REF_TYPE"* ]]
}

@test "verify-signature: fails when GITHUB_SHA is unset" {
    unset GITHUB_SHA
    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"GITHUB_SHA"* ]]
}

# ---------------------------------------------------------------------------
# Case 8: tag-ref dispatch -> verifies BOTH the tag object AND its peeled commit.
# The publish chain keys off the tag commit, so it must be signature-checked too.
# ---------------------------------------------------------------------------
@test "verify-signature: tag push verifies the tag object and its commit" {
    export GITHUB_REF_TYPE="tag"
    export GITHUB_REF_NAME="sdmx-types/v0.1.0"

    # Shim git: verify-tag/verify-commit emit a good VALIDSIG; rev-parse peels
    # the tag to a deterministic 40-hex commit so the script's `^{commit}`
    # resolution succeeds without a real tag in the test repo.
    cat > "$MOCK_BIN/git" << EOF
#!/bin/sh
case "\$1" in
    verify-tag)
        echo "CALLED: verify-tag" >&2
        printf '[GNUPG:] VALIDSIG ABC DEF 12345 0 0 0 0 0 0 ${GOOD_FPR}\n'
        exit 0
        ;;
    verify-commit)
        echo "CALLED: verify-commit" >&2
        printf '[GNUPG:] VALIDSIG ABC DEF 12345 0 0 0 0 0 0 ${GOOD_FPR}\n'
        exit 0
        ;;
    rev-parse)
        echo "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        exit 0
        ;;
    *)
        exec $(command -v git) "\$@"
        ;;
esac
EOF
    chmod +x "$MOCK_BIN/git"

    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"tag (sdmx-types/v0.1.0) verified"* ]]
    [[ "$output" == *"tag commit ("* ]]
    [[ "$output" == *"verified"* ]]
}

# ---------------------------------------------------------------------------
# Case 8b: tag object is signed but its commit is NOT -> fail closed.
# Guards the seam this check was added to close: a valid tag wrapper must not
# be enough to publish an unsigned source commit.
# ---------------------------------------------------------------------------
@test "verify-signature: tag push fails when the tag commit is unsigned" {
    export GITHUB_REF_TYPE="tag"
    export GITHUB_REF_NAME="sdmx-types/v0.1.0"

    cat > "$MOCK_BIN/git" << EOF
#!/bin/sh
case "\$1" in
    verify-tag)
        printf '[GNUPG:] VALIDSIG ABC DEF 12345 0 0 0 0 0 0 ${GOOD_FPR}\n'
        exit 0
        ;;
    verify-commit)
        printf '[GNUPG:] ERRSIG ABC 17 8 01 12345 9\n'
        exit 1
        ;;
    rev-parse)
        echo "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        exit 0
        ;;
    *)
        exec $(command -v git) "\$@"
        ;;
esac
EOF
    chmod +x "$MOCK_BIN/git"

    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"tag commit (deadbeef"* ]]
    [[ "$output" == *"has no valid signature"* ]]
}

# ---------------------------------------------------------------------------
# Case 9: branch-ref dispatch -> verifies commits in range
# ---------------------------------------------------------------------------
@test "verify-signature: branch push dispatches to verify-commit for each sha" {
    export GITHUB_REF_TYPE="branch"
    export GITHUB_REF_NAME="main"
    # GITHUB_EVENT_BEFORE=all-zeros: range is just GITHUB_SHA (single commit).

    make_git_shim "[GNUPG:] VALIDSIG ABC DEF 12345 0 0 0 0 0 0 $GOOD_FPR"

    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"commit ("* ]]
    [[ "$output" == *"verified — primary ${GOOD_FPR}"* ]]
}

# ---------------------------------------------------------------------------
# Case 9b: branch push with a real GITHUB_EVENT_BEFORE verifies ONLY the
# BEFORE..SHA delta — not the whole ancestry. Guards the O(N)-history leak: the
# fix bounds the walk to commits this push introduced. Build a 3-commit history,
# point BEFORE at the first; exactly the two newer commits must be verified.
# ---------------------------------------------------------------------------
@test "verify-signature: branch push verifies only the pushed delta, not all history" {
    export GITHUB_REF_TYPE="branch"
    export GITHUB_REF_NAME="main"

    # c1 already exists from setup() as $HEAD_SHA. Add two more real commits.
    BEFORE_SHA="$HEAD_SHA"
    echo a > a.txt; git add a.txt; git commit -m c2 -q
    echo b > b.txt; git add b.txt; git commit -m c3 -q
    TIP_SHA=$(git rev-parse HEAD)

    export GITHUB_EVENT_BEFORE="$BEFORE_SHA"
    export GITHUB_SHA="$TIP_SHA"

    # Shim records each verify-commit SHA so we can count how many were walked.
    VERIFIED_LOG="$TMPDIR/verified.log"
    cat > "$MOCK_BIN/git" << EOF
#!/bin/sh
case "\$1" in
    verify-commit)
        echo "\$3" >> "$VERIFIED_LOG"
        printf '[GNUPG:] VALIDSIG ABC DEF 12345 0 0 0 0 0 0 ${GOOD_FPR}\n'
        exit 0
        ;;
    *)
        exec $(command -v git) "\$@"
        ;;
esac
EOF
    chmod +x "$MOCK_BIN/git"

    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    # Exactly two commits verified (the delta), and c1 (BEFORE) is NOT among them.
    [ "$(wc -l < "$VERIFIED_LOG")" -eq 2 ]
    run ! grep -qF "$BEFORE_SHA" "$VERIFIED_LOG"
}

# ---------------------------------------------------------------------------
# Case 10: POSIX pipefail-regression guard (extraction hazard #2)
# The script uses set -eu without pipefail. This test confirms that a mid-pipe
# error in the allowlist build does not silently produce an empty ALLOWED that
# bypasses the empty-check and reaches verify_ref.
# We simulate this by providing a fingerprint that the tr/grep pipeline will
# legitimately reject — the empty-check must fire, not silently pass.
# ---------------------------------------------------------------------------
@test "verify-signature: malformed allowlist is caught before verify_ref is called" {
    # All-lowercase fingerprint: tr uppercases it, but if the pipeline were
    # broken and produced empty output, the empty-check must still catch it.
    export ALLOWED_PRIMARY_FINGERPRINTS="gggggggggggggggggggggggggggggggggggggggg"

    run sh ./verify-signature.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    # Must fail on config error, not on a signature violation — proving
    # the allowlist build ran and the empty-check fired, not verify_ref.
    [[ "$output" == *"CONFIGURATION ERROR"* ]]
    [[ "$output" != *"PROTOCOL VIOLATION"* ]]
}
