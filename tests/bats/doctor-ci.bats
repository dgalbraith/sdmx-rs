#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/doctor-ci.sh
#
# Exercises the CI/local alignment check against a KNOWN `verify` recipe graph
# (a fixture Justfile + minimal ci.yml), asserting the report is accurate:
#   - every key check reachable from `verify` -> exit 0, all "in local verify",
#   - a key check NOT reachable            -> flagged "not in local verify", exit 1,
#   - matching is EXACT: a key check that is a substring of a real recipe
#     (`gam` vs `gamma`) must NOT be counted as covered (the original defect).
#
# DOCTOR_CI_KEY_CHECKS overrides the built-in key-check list so the fixture graph
# can stay small and decoupled from the real recipe names.
#
# Run with: bats tests/bats/doctor-ci.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$BATS_TEST_DIRNAME/../../scripts/doctor-ci.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" scripts/lib/

    # A small verify graph: verify -> {group-a -> {alpha, beta}, group-b -> gamma}.
    cat > Justfile <<'EOF'
[parallel]
verify: group-a group-b

group-a: alpha beta

group-b: gamma

alpha:
    @true
beta:
    @true
gamma:
    @true
EOF

    # Minimal workflow so Checks 1-3 run; the alignment check reads the Justfile.
    mkdir -p .github/workflows
    cat > .github/workflows/ci.yml <<'EOF'
name: CI
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: just verify
EOF
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

@test "doctor-ci: all key checks reachable from verify -> all covered, exit 0" {
    export DOCTOR_CI_KEY_CHECKS="alpha beta gamma"
    run_isolated ./scripts/doctor-ci.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"alpha (in local verify)"* ]]
    [[ "$output" == *"gamma (in local verify)"* ]]
    [[ "$output" == *"covers all key CI checks"* ]]
    # No spurious warnings.
    [[ "$output" != *"not in local verify"* ]]
}

@test "doctor-ci: a key check not reachable is flagged, exit 1" {
    export DOCTOR_CI_KEY_CHECKS="alpha delta"
    run_isolated ./scripts/doctor-ci.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"alpha (in local verify)"* ]]
    [[ "$output" == *"delta (not in local verify)"* ]]
    [[ "$output" == *"1 key check(s) not covered"* ]]
}

@test "doctor-ci: matching is exact (a substring of a recipe is not counted)" {
    # `gam` is a substring of the `gamma` recipe; it must NOT match (the bug).
    export DOCTOR_CI_KEY_CHECKS="gam"
    run_isolated ./scripts/doctor-ci.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"gam (not in local verify)"* ]]
}
