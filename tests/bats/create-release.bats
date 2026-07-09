#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/ci/create-release.sh
#
# Verifies the per-crate release-notes resolution (design-0004 §9) without a real
# GitHub Release. `gh` is stubbed via a PATH shim that records the notes piped to
# it (release create/edit read --notes-file -), and `release view` exit code is
# tuned to fake "release already exists" / "facade release exists". Contract:
#   - FACADE + curated file present  -> body is the curated prose,
#   - FACADE + curated missing, CHANGELOG has content -> backstop to CHANGELOG,
#   - FACADE + curated missing + empty CHANGELOG -> FATAL,
#   - LEAF + CHANGELOG content       -> body is the CHANGELOG section,
#   - LEAF + empty CHANGELOG         -> provenance placeholder (NOT a failure),
#   - LEAF empty + facade release exists -> placeholder links the facade batch.
#
# Run with: bats tests/bats/create-release.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    mkdir -p scripts/ci scripts/lib
    cp "$BATS_TEST_DIRNAME/../../scripts/ci/create-release.sh" scripts/ci/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" scripts/lib/

    # An asset file so the "no assets" guard passes.
    touch asset.crate

    # gh stub: log argv; capture piped notes (release create/edit) to NOTES_OUT;
    # `release view` exit code tuned by STUB_RELEASE_EXISTS / STUB_FACADE_EXISTS.
    mkdir -p bin
    GH_LOG="$BATS_TEST_TMPDIR/gh-calls.log"
    NOTES_OUT="$BATS_TEST_TMPDIR/notes.out"
    export GH_LOG NOTES_OUT
    cat > bin/gh <<'EOF'
#!/bin/sh
echo "$*" >> "$GH_LOG"
case "$1 $2" in
    "release view")
        # $3 is the tag. Facade tag existence is controlled separately so the
        # leaf-placeholder facade-link branch is testable.
        case "$3" in
            sdmx-rs/v*) [ "${STUB_FACADE_EXISTS:-0}" = "1" ] && exit 0 || exit 1 ;;
            *)          [ "${STUB_RELEASE_EXISTS:-0}" = "1" ] && exit 0 || exit 1 ;;
        esac
        ;;
    "release create"|"release edit")
        cat > "$NOTES_OUT"   # the body arrives on stdin via --notes-file -
        exit 0
        ;;
    "release upload")
        exit 0
        ;;
esac
exit 0
EOF
    chmod +x bin/gh
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
    export GH_TOKEN="fake"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# Write crates/<crate>/CHANGELOG.md with a <version> section body ($2 may be empty
# to simulate a no-op lockstep section).
write_changelog() {
    local crate="$1" body="$2"
    mkdir -p "crates/$crate"
    {
        echo "# Changelog"
        echo ""
        echo "## [${VERSION:-0.2.0}]"
        [ -n "$body" ] && echo "$body"
        echo ""
        echo "## [0.0.0]"
        echo "- seed"
    } > "crates/$crate/CHANGELOG.md"
}

@test "create-release: FACADE uses the curated release-notes file as the body" {
    VERSION=0.2.0
    write_changelog sdmx-rs "- machine: feat stuff"
    mkdir -p crates/sdmx-rs/release-notes
    printf 'Curated: streaming parser and clearer errors.\n' \
        > crates/sdmx-rs/release-notes/0.2.0.md

    run_isolated ./scripts/ci/create-release.sh sdmx-rs 0.2.0 asset.crate
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    echo "BODY: $(cat "$NOTES_OUT")" >&2
    [ "$status" -eq 0 ]
    grep -q "Curated: streaming parser" "$NOTES_OUT"
    # The machine line must NOT be the body.
    run ! grep -q "machine: feat stuff" "$NOTES_OUT"
}

@test "create-release: FACADE backstops to CHANGELOG when curated file is absent" {
    VERSION=0.2.0
    write_changelog sdmx-rs "- backstop machine notes"
    # No release-notes/0.2.0.md.

    run_isolated ./scripts/ci/create-release.sh sdmx-rs 0.2.0 asset.crate
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    grep -q "backstop machine notes" "$NOTES_OUT"
    [[ "$output" == *"falling back to CHANGELOG"* ]]
}

@test "create-release: FACADE with no curated notes AND empty CHANGELOG is fatal" {
    VERSION=0.2.0
    write_changelog sdmx-rs ""   # empty section

    run_isolated ./scripts/ci/create-release.sh sdmx-rs 0.2.0 asset.crate
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"No facade release notes"* ]]
}

@test "create-release: LEAF uses its CHANGELOG section as the body" {
    VERSION=0.2.0
    write_changelog sdmx-types "- types: added Foo"

    run_isolated ./scripts/ci/create-release.sh sdmx-types 0.2.0 asset.crate
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    grep -q "types: added Foo" "$NOTES_OUT"
}

@test "create-release: LEAF with empty CHANGELOG emits a provenance placeholder with unconditional facade link" {
    VERSION=0.2.0
    write_changelog sdmx-types ""   # no-op lockstep section

    run_isolated ./scripts/ci/create-release.sh sdmx-types 0.2.0 asset.crate
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    echo "BODY: $(cat "$NOTES_OUT")" >&2
    [ "$status" -eq 0 ]
    grep -q "No user-facing changes" "$NOTES_OUT"
    grep -q "build-provenance and SBOM" "$NOTES_OUT"
    # The facade release link is now emitted unconditionally
    grep -q "sdmx-rs.* v0.2.0 release notes" "$NOTES_OUT"
}

# ---------------------------------------------------------------------------
# Release TITLE is a plain "<crate> v<version>" (facade and leaf alike).
# ---------------------------------------------------------------------------
@test "create-release: title is plain <crate> v<version>" {
    VERSION=0.2.0
    write_changelog sdmx-rs "- machine notes"
    mkdir -p crates/sdmx-rs/release-notes
    printf 'Curated summary.\n' > crates/sdmx-rs/release-notes/0.2.0.md

    run_isolated ./scripts/ci/create-release.sh sdmx-rs 0.2.0 asset.crate
    echo "GH CALLS:" >&2; cat "$GH_LOG" >&2
    [ "$status" -eq 0 ]
    grep -qF -- '--title sdmx-rs v0.2.0 ' "$GH_LOG"
    run ! grep -qF -- 'sdmx-rs v0.2.0:' "$GH_LOG"
}
