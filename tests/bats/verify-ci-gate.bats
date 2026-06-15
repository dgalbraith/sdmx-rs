#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/verify-ci-gate.sh
#
# Verifies the CI Quality Gate coverage cross-check: the ci-gate job's `needs:`
# list in .github/workflows/ci.yml must equal the gating set declared in
# forge/github/ci-gating-jobs.json, with no ghost gates (manifest entries that
# do not name a real job).
#
# Failure-first: the drift/ghost cases assert exit 1 so a regression that makes
# the gate pass-open is caught, not just the happy path.
#
# The script reads fixed relative paths, so each test builds a minimal repo tree
# under $BATS_TEST_TMPDIR with a copyable ci.yml + manifest, then mutates copies
# for the failure cases. The REAL committed files are the baseline (so this suite
# also fails if someone edits the live needs:/manifest out of sync).
#
# Run with: bats tests/bats/verify-ci-gate.bats
# ==============================================================================

setup() {
    REPO_ROOT="$BATS_TEST_DIRNAME/../.."
    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    mkdir -p scripts/lib forge/github .github/workflows docs/project
    cp "$REPO_ROOT/scripts/verify-ci-gate.sh" scripts/
    cp "$REPO_ROOT/scripts/lib/log.sh" scripts/lib/
    cp "$REPO_ROOT/forge/github/ci-gating-jobs.json" forge/github/
    cp "$REPO_ROOT/.github/workflows/ci.yml" .github/workflows/
    cp "$REPO_ROOT/docs/project/ci-gating.md" docs/project/
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

run_verify() {
    unset GITHUB_ACTIONS CI
    run sh scripts/verify-ci-gate.sh
}

MANIFEST="forge/github/ci-gating-jobs.json"
CI=".github/workflows/ci.yml"

# ==============================================================================
# Happy path — committed files are in sync
# ==============================================================================

@test "verify-ci-gate: committed manifest matches committed ci-gate needs: -> exit 0" {
    run_verify
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"in sync across"* ]]
}

# ==============================================================================
# Drift — a gating job is dropped from the ci-gate needs: list (UNGATED)
# ==============================================================================

@test "verify-ci-gate: gating job missing from ci-gate needs: -> exit 1" {
    # Remove `- clippy` from the ci-gate needs: block only. It still exists as a
    # job and stays in the manifest, so this is pure drift: an ungated gate.
    # Restrict the deletion to the ci-gate needs: block (after the `ci-gate:`
    # line) so the `- clippy`-style lines elsewhere are untouched.
    awk '
        /^  ci-gate:/ {ingate=1}
        ingate && /^      - clippy$/ {next}
        /^    steps:/ && ingate {ingate=0}
        {print}
    ' "$CI" > "$CI.tmp" && mv "$CI.tmp" "$CI"

    run_verify
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"drift"* ]]
    [[ "$output" == *"MISSING from ci-gate"* ]]
    [[ "$output" == *"clippy"* ]]
}

# ==============================================================================
# Drift — an undeclared job is added to ci-gate needs: (not in manifest)
# ==============================================================================

@test "verify-ci-gate: undeclared job in ci-gate needs: -> exit 1" {
    # Add `- semver-check` to the ci-gate needs: block. semver-check IS a real
    # job in ci.yml (so it is not a ghost), but it is deliberately NOT in the
    # manifest (PR-only). Its presence in needs: is an undeclared gate.
    awk '
        /^  ci-gate:/ {ingate=1}
        ingate && /^      - release-dry-run$/ {print; print "      - semver-check"; next}
        {print}
    ' "$CI" > "$CI.tmp" && mv "$CI.tmp" "$CI"

    run_verify
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"drift"* ]]
    [[ "$output" == *"NOT declared in manifest"* ]]
    [[ "$output" == *"semver-check"* ]]
}

# ==============================================================================
# Ghost gate — manifest names a job that does not exist in ci.yml
# ==============================================================================

@test "verify-ci-gate: manifest names a non-existent job -> exit 1" {
    # Append a phantom job to the manifest's .jobs array. It is not a real job in
    # ci.yml, so it is a ghost gate. (It is also not in needs:, so it would also
    # trip the drift check — but the ghost assertion is the primary signal here.)
    jq '.jobs += [{"job":"phantom-job","why":"does not exist"}]' \
        "$MANIFEST" > "$MANIFEST.tmp" && mv "$MANIFEST.tmp" "$MANIFEST"

    run_verify
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Ghost gate"* ]]
    [[ "$output" == *"phantom-job"* ]]
}

# ==============================================================================
# Doc drift — a gating job is not documented in ci-gating.md
# ==============================================================================

@test "verify-ci-gate: gating job not documented in ci-gating.md -> exit 1" {
    # Remove the `#### clippy` Check Details heading from the prose doc. clippy
    # stays in the manifest AND the ci-gate needs: list, so it is a real gating
    # job that is now undocumented — the drift the prose check must catch and the
    # needs:/manifest check cannot see.
    DOC="docs/project/ci-gating.md"
    grep -v '^#### clippy$' "$DOC" > "$DOC.tmp" && mv "$DOC.tmp" "$DOC"

    run_verify
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"not documented"* ]]
    [[ "$output" == *"clippy"* ]]
}

# ==============================================================================
# Missing inputs fail closed
# ==============================================================================

@test "verify-ci-gate: missing manifest -> exit 1" {
    rm -f "$MANIFEST"
    run_verify
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Manifest not found"* ]]
}

@test "verify-ci-gate: missing ci-gating.md -> exit 1" {
    rm -f docs/project/ci-gating.md
    run_verify
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Gate documentation not found"* ]]
}

@test "verify-ci-gate: empty manifest jobs array -> exit 1" {
    jq '.jobs = []' "$MANIFEST" > "$MANIFEST.tmp" && mv "$MANIFEST.tmp" "$MANIFEST"
    run_verify
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"no non-empty .jobs array"* ]]
}
