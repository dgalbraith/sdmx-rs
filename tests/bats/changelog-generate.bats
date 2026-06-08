#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/changelog-generate.sh
#
# Drives the per-crate changelog generator WITHOUT a real git-cliff run: a stub
# git-cliff (via GIT_CLIFF) records its argv and fakes the outcome, so we can
# assert the script's contract:
#   - one invocation per crate, in topological order,
#   - the gate-parity flags (--include-path "crates/<crate>/**", the per-crate
#     --tag-pattern, --output "crates/<crate>/CHANGELOG.md") passed for each,
#   - a subset run touches ONLY the named crates,
#   - a crate's non-zero git-cliff exit is WARNED but does not abort the batch.
#
# The include-path /** suffix and the per-crate --tag-pattern are load-bearing
# for gate parity with check-changelog.sh; these tests pin them so a future edit
# that breaks the matched pair fails loudly here.
#
# Run with: bats tests/bats/changelog-generate.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    mkdir -p scripts/lib
    cp "$BATS_TEST_DIRNAME/../../scripts/changelog-generate.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/common.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" scripts/lib/

    # The real cliff.toml is referenced via --config; the stub ignores it, but
    # the script's argv still names it, so a placeholder keeps the layout honest.
    touch cliff.toml
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        mkdir -p "crates/$crate"
    done

    mkdir -p bin
    LOG="$TMPDIR/cliff-calls.log"
    export LOG
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# Stub `git-cliff`. Appends its full argv as one line to $LOG, creates/updates the
# --output file so a real generation is simulated, and exits STUB_CLIFF_EXIT
# (default 0). Because the script invokes it as "$GIT_CLIFF", pointing GIT_CLIFF
# at this stub fully intercepts generation.
make_cliff_stub() {
    cat > bin/git-cliff <<'EOF'
#!/bin/sh
echo "$*" >> "$LOG"
# Find the --output path and touch it, mirroring a real generation writing a file.
out=""
prev=""
for arg in "$@"; do
    if [ "$prev" = "--output" ]; then out="$arg"; fi
    prev="$arg"
done
[ -n "$out" ] && echo "generated" > "$out"
exit "${STUB_CLIFF_EXIT:-0}"
EOF
    chmod +x bin/git-cliff
    export GIT_CLIFF="$TMPDIR/bin/git-cliff"
}

@test "changelog-generate: runs once per crate in topological order" {
    make_cliff_stub
    run_isolated ./scripts/changelog-generate.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    # One git-cliff call per crate.
    [ "$(wc -l < "$LOG")" -eq 5 ]
    # Topological order: types first, rs last.
    [ "$(sed -n '1p' "$LOG" | grep -c 'crates/sdmx-types/\*\*')" -eq 1 ]
    [ "$(sed -n '5p' "$LOG" | grep -c 'crates/sdmx-rs/\*\*')" -eq 1 ]
    [[ "$output" == *"changelogs written"* ]]
}

@test "changelog-generate: passes gate-parity flags for each crate" {
    make_cliff_stub
    run_isolated ./scripts/changelog-generate.sh
    [ "$status" -eq 0 ]
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        # include-path carries the load-bearing /** suffix.
        grep -q -- "--include-path crates/${crate}/\*\*" "$LOG"
        # per-crate tag-pattern is anchored to this crate's own tags.
        grep -q -- "--tag-pattern \^${crate}/v" "$LOG"
        # output target is the crate's own CHANGELOG.md, and it was written.
        grep -q -- "--output crates/${crate}/CHANGELOG.md" "$LOG"
        [ -f "crates/${crate}/CHANGELOG.md" ]
    done
    # No --tag (only --tag-pattern): no concrete version is being cut here.
    run ! grep -E -- '(^| )--tag ' "$LOG"
}

@test "changelog-generate: subset run touches only the named crates" {
    make_cliff_stub
    run_isolated ./scripts/changelog-generate.sh sdmx-types sdmx-client
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [ "$(wc -l < "$LOG")" -eq 2 ]
    grep -q -- "--include-path crates/sdmx-types/\*\*" "$LOG"
    grep -q -- "--include-path crates/sdmx-client/\*\*" "$LOG"
    run ! grep -- "--include-path crates/sdmx-parsers/\*\*" "$LOG"
}

@test "changelog-generate: a crate's git-cliff failure is warned but does not abort" {
    export STUB_CLIFF_EXIT=1
    make_cliff_stub
    run_isolated ./scripts/changelog-generate.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    # Batch completes (exit 0) despite every crate's git-cliff returning non-zero.
    [ "$status" -eq 0 ]
    # All five crates were still attempted.
    [ "$(wc -l < "$LOG")" -eq 5 ]
    # The failure is surfaced, not swallowed.
    [[ "$output" == *"git-cliff exited 1"* ]]
}
