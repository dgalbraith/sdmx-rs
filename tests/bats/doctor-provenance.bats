#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/doctor-provenance.sh
#
# The tool verifies real GPG signatures, so setup builds a throwaway maintainer
# key in an isolated GNUPGHOME and a signed history (empty root -> ledger
# introduction -> work), plus a second non-allowlisted key. Tier 2 (GitHub) is
# exercised with a `gh` shim; Tier-1 tests pass --no-ci to skip it.
#
# Run with: bats tests/bats/doctor-provenance.bats
# ==============================================================================

genkey() {  # $1 = uid ; echoes the primary fingerprint
    gpg --batch --pinentry-mode loopback --passphrase '' \
        --quick-generate-key "$1" ed25519 sign 0 >/dev/null 2>&1
    gpg --batch --with-colons --list-keys "$1" 2>/dev/null | awk -F: '/^fpr:/{print $10; exit}'
}

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"
    TMPDIR=$(mktemp -d); cd "$TMPDIR" || exit 1

    cp "$BATS_TEST_DIRNAME/../../scripts/doctor-provenance.sh" .
    mkdir -p lib; cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    export GNUPGHOME="$TMPDIR/gnupg"; mkdir -m 700 "$GNUPGHOME"
    echo "allow-loopback-pinentry" > "$GNUPGHOME/gpg-agent.conf"
    echo "pinentry-mode loopback" > "$GNUPGHOME/gpg.conf"
    MAINT_FPR=$(genkey "Maintainer <m@test>")
    STRANGER_FPR=$(genkey "Stranger <s@test>")

    git init -q --initial-branch=main
    git config user.email m@test; git config user.name Maintainer
    git config commit.gpgsign true
    git config gpg.program gpg
    git config user.signingkey "$MAINT_FPR"

    # Empty signed root, then ledger introduction, then a work commit.
    # -f the key: a global core.excludesFile may ignore *.asc.
    git commit -q --allow-empty -m "chore: empty root"
    mkdir -p .github/maintainer-keys
    gpg --batch --armor --export "$MAINT_FPR" > .github/maintainer-keys/maintainer.asc
    git add -f .github/maintainer-keys/maintainer.asc
    git commit -q -m "chore: introduce maintainer ledger"
    echo one > f1; git add f1; git commit -q -m "feat: work one"
}

teardown() {
    gpgconf --kill gpg-agent 2>/dev/null || true
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

@test "doctor-provenance: clean signed history passes (self-consistency)" {
    run_isolated ./doctor-provenance.sh --no-ci
    echo "STATUS:$status"; echo "OUT:$output"
    [ "$status" -eq 0 ]
    [[ "$output" == *"as-of-allowlisted primary key"* ]]
    [[ "$output" == *"self-consistency"* ]]
}

@test "doctor-provenance: an unsigned commit is a violation" {
    echo two > f2; git add f2; git commit -q --no-gpg-sign -m "feat: unsigned"
    run_isolated ./doctor-provenance.sh --no-ci
    echo "STATUS:$status"; echo "OUT:$output"
    [ "$status" -eq 1 ]
    [[ "$output" == *"UNVERIFIED"* ]]
}

@test "doctor-provenance: a commit signed by a key absent from the ledger is a violation" {
    git config user.signingkey "$STRANGER_FPR"
    echo three > f3; git add f3; git commit -q -m "feat: stranger-signed"
    run_isolated ./doctor-provenance.sh --no-ci
    echo "STATUS:$status"; echo "OUT:$output"
    [ "$status" -eq 1 ]
    [[ "$output" == *"UNVERIFIED"* ]]
}

@test "doctor-provenance: a key valid only in a later epoch is unauthorised as-of an earlier commit" {
    # Sign a commit with stranger BEFORE stranger is allowlisted, then allowlist
    # stranger in a later ledger epoch. The earlier commit must fail the as-of
    # check even though the key later becomes valid (gpg can verify it via the
    # union keyring, but it was not in the roster as of that commit).
    git config user.signingkey "$STRANGER_FPR"
    echo early > early; git add early; git commit -q -m "feat: stranger before allowlisting"
    git config user.signingkey "$MAINT_FPR"
    gpg --batch --armor --export "$STRANGER_FPR" > .github/maintainer-keys/stranger.asc
    git add -f .github/maintainer-keys/stranger.asc
    git commit -q -m "chore: allowlist stranger (epoch 2)"
    run_isolated ./doctor-provenance.sh --no-ci
    echo "STATUS:$status"; echo "OUT:$output"
    [ "$status" -eq 1 ]
    [[ "$output" == *"UNAUTHORISED"* ]]
    [[ "$output" == *"2 roster epoch"* ]]
}

# Promote the stranger key to a second allowlisted maintainer (signed by the
# still-valid primary maintainer), so it can commit the ledger update that
# records the primary maintainer's expiry/revocation — a dead key can't sign.
add_second_maintainer() {
    gpg --batch --armor --export "$STRANGER_FPR" > .github/maintainer-keys/second.asc
    git add -f .github/maintainer-keys/second.asc
    git commit -q -m "chore: add a second maintainer"
    git config user.signingkey "$STRANGER_FPR"
}

@test "doctor-provenance: a key that expired after signing is still accepted (as-of)" {
    add_second_maintainer
    gpg --batch --pinentry-mode loopback --passphrase '' --quick-set-expire "$MAINT_FPR" seconds=1 >/dev/null 2>&1
    gpg --batch --armor --export "$MAINT_FPR" > .github/maintainer-keys/maintainer.asc
    git add -f .github/maintainer-keys/maintainer.asc; git commit -q -m "chore: refresh maintainer key"
    sleep 3
    run_isolated ./doctor-provenance.sh --no-ci
    echo "STATUS:$status"; echo "OUT:$output"
    [ "$status" -eq 0 ]
    [[ "$output" == *"as-of-allowlisted primary key"* ]]
}

@test "doctor-provenance: a revoked key's commits are surfaced, not silently passed" {
    add_second_maintainer
    # gpg guards the auto-generated revocation cert with a ':' prefix on each
    # armor line so it can't be imported by accident; strip it to apply.
    sed 's/^://' "$GNUPGHOME/openpgp-revocs.d/${MAINT_FPR}.rev" | gpg --batch --yes --import >/dev/null 2>&1
    gpg --batch --armor --export "$MAINT_FPR" > .github/maintainer-keys/maintainer.asc
    git add -f .github/maintainer-keys/maintainer.asc; git commit -q -m "chore: record maintainer revocation"
    run_isolated ./doctor-provenance.sh --no-ci
    echo "STATUS:$status"; echo "OUT:$output"
    [[ "$output" == *"REVOKED"* ]]
    [[ "$output" == *"revoked-key commit"* ]]
}

@test "doctor-provenance: a superseded-key revocation is historic, not flagged (as-of)" {
    add_second_maintainer
    # gpg menu 2 = superseded -> RFC packet reason 0x01 (accepted as-of), unlike
    # the auto-cert's 0x00. --no-tty for the ttyless bats/CI env; --command-fd
    # feeds gen_revoke.okay / reason.code / reason.text / reason.okay.
    printf 'y\n2\n\ny\n' | gpg --no-tty --command-fd 0 --pinentry-mode loopback --passphrase '' \
        --gen-revoke "$MAINT_FPR" > "$TMPDIR/sup.rev" 2>/dev/null
    gpg --batch --yes --import "$TMPDIR/sup.rev" >/dev/null 2>&1
    gpg --batch --armor --export "$MAINT_FPR" > .github/maintainer-keys/maintainer.asc
    git add -f .github/maintainer-keys/maintainer.asc; git commit -q -m "chore: supersede maintainer key"
    run_isolated ./doctor-provenance.sh --no-ci
    echo "STATUS:$status"; echo "OUT:$output"
    [ "$status" -eq 0 ]
    [[ "$output" != *"REVOKED"* ]]
    [[ "$output" == *"as-of-allowlisted primary key"* ]]
}

@test "doctor-provenance: independent audit with the correct root fpr passes" {
    run_isolated ./doctor-provenance.sh --no-ci --root-fpr "$MAINT_FPR"
    echo "STATUS:$status"; echo "OUT:$output"
    [ "$status" -eq 0 ]
    [[ "$output" == *"independent audit"* ]]
}

@test "doctor-provenance: independent audit with a wrong root fpr breaks continuity" {
    run_isolated ./doctor-provenance.sh --no-ci --root-fpr DEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEF
    echo "STATUS:$status"; echo "OUT:$output"
    [ "$status" -eq 1 ]
    [[ "$output" == *"Continuity broken"* ]]
}

@test "doctor-provenance: an allowlisted-signed tag passes; a rogue-signed tag fails" {
    git tag -s -m "release" v0.0.1                     # signed by the maintainer
    run_isolated ./doctor-provenance.sh --no-ci
    echo "STATUS:$status"; echo "OUT:$output"
    [ "$status" -eq 0 ]
    [[ "$output" == *"1 tag(s)"* ]]

    git config user.signingkey "$STRANGER_FPR"
    git tag -s -m "rogue" v0.0.2                        # signed by a non-allowlisted key
    run_isolated ./doctor-provenance.sh --no-ci
    echo "STATUS:$status"; echo "OUT:$output"
    [ "$status" -eq 1 ]
    [[ "$output" == *"tag v0.0.2"* ]]
}

@test "doctor-provenance: Tier 2 flags a direct-push merge" {
    # Two merges: one 'staged' (in STAGED_SHAS), one direct-push.
    git checkout -q -b feat-a; echo a > fa; git add fa; git commit -q -m "feat: a"
    git checkout -q main; git merge -q --no-ff -m "merge: a (staged)" feat-a
    STAGED_MERGE=$(git rev-parse HEAD)
    git checkout -q -b feat-b; echo b > fb; git add fb; git commit -q -m "feat: b"
    git checkout -q main; git merge -q --no-ff -m "merge: b (direct)" feat-b

    export FORGE_FIXTURES="$TMPDIR/fx"; mkdir -p "$FORGE_FIXTURES"
    export GH_MOCK_STAGED_SHAS="$STAGED_MERGE"
    mock_gh
    run_isolated ./doctor-provenance.sh
    echo "STATUS:$status"; echo "OUT:$output"
    [ "$status" -eq 0 ]
    [[ "$output" == *"1 of 2"* ]]
    [[ "$output" == *"direct push"* ]]
    [[ "$output" == *"merge: b (direct)"* ]]
}
