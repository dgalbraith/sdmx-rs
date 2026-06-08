#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/new-release-notes.sh
#
# Scaffolds a curated facade release-notes file from the template. Contract:
#   - missing version arg          -> exit 1,
#   - scaffolds the file (copy of the template, with its required sections),
#   - existing target              -> exit 1 (no clobber),
#   - a fresh scaffold does NOT satisfy check-release-notes (interlock).
#
# Run with: bats tests/bats/new-release-notes.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$BATS_TEST_DIRNAME/../../scripts/new-release-notes.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/check-release-notes.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" scripts/lib/

    # Real template fixture so the copy + H1 substitution are exercised for real.
    mkdir -p crates/sdmx-rs/release-notes/templates
    cp "$BATS_TEST_DIRNAME/../../crates/sdmx-rs/release-notes/templates/template.md" \
        crates/sdmx-rs/release-notes/templates/template.md
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

@test "new-release-notes: missing version arg fails" {
    run_isolated ./scripts/new-release-notes.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"no version given"* ]]
}

@test "new-release-notes: scaffolds the file from the template" {
    run_isolated ./scripts/new-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [ -f crates/sdmx-rs/release-notes/0.2.0.md ]
    # Carries the template's required sections (it IS the template).
    grep -q '^## Breaking Changes & Migration$' crates/sdmx-rs/release-notes/0.2.0.md
    grep -q '^## Verifying Release Provenance$' crates/sdmx-rs/release-notes/0.2.0.md
}

@test "new-release-notes: refuses to overwrite an existing file" {
    run_isolated ./scripts/new-release-notes.sh 0.2.0
    [ "$status" -eq 0 ]
    run_isolated ./scripts/new-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"already exists"* ]]
}

@test "new-release-notes: a fresh scaffold does NOT satisfy check-release-notes (interlock)" {
    run_isolated ./scripts/new-release-notes.sh 0.2.0
    [ "$status" -eq 0 ]
    # The scaffold carries template guidance, so the gate must reject it.
    run_isolated ./scripts/check-release-notes.sh 0.2.0
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"template guidance remains"* ]]
}
