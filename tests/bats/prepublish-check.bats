#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/prepublish-check.sh
#
# Lightweight: `cargo` is stubbed via a PATH shim (the script calls `cargo`
# directly, so there is no CARGO env seam to use). The shim records its argv and
# returns a configurable exit code, so these tests assert the script's CONTRACT
# without a real, networked `cargo publish --dry-run`:
#
#   - every crate is dry-run published with BOTH --dry-run AND --allow-dirty
#     (the regression: the gate runs on a deliberately-dirty tree per
#      releasing.md §0 — generated-but-uncommitted changelogs — so without
#      --allow-dirty cargo aborts on its dirty guard before validating anything),
#   - cargo's real exit code is propagated, not flattened to 1,
#   - a crate-name argument scopes the run.
#
# Run with: bats tests/bats/prepublish-check.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    # Mirror the script's on-disk layout: it sources scripts/common.sh, which in
    # turn sources scripts/lib/log.sh, and it shells out to check-release-notes.sh.
    mkdir -p scripts/lib
    cp "$BATS_TEST_DIRNAME/../../scripts/prepublish-check.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/check-release-notes.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/common.sh" scripts/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" scripts/lib/

    # prepublish-check folds in the facade release-notes gate (design-0004 §9):
    # when the facade (sdmx-rs) is in scope it reads the facade version from its
    # manifest and requires a FULLY CURATED release-notes/<version>.md (all required
    # sections, no template guidance). Provide both so the default (all-crates) runs
    # exercise the cargo path, not a notes failure.
    mkdir -p crates/sdmx-rs/release-notes
    printf 'version = "0.2.0"\nrust-version = "1.91.0"\n' > crates/sdmx-rs/Cargo.toml
    cat > crates/sdmx-rs/release-notes/0.2.0.md <<'NOTES'
Curated facade notes for 0.2.0.

## Breaking Changes & Migration
This release contains no breaking changes.

## Bug Fixes
No bug fixes in this release.

## New Features & Enhancements
No new features in this release.

## Deprecations
No deprecations in this release.

## Minimum Supported Rust Version (MSRV)
* **Current MSRV**: `1.91.0`

## Feature Flags
No changes to Cargo feature flags.

## Security
No security advisories addressed in this release.

## Dependency Updates
No notable dependency updates in this release.

## Verifying Release Provenance
See SECURITY.md.
NOTES

    # PATH-shim cargo: log full argv (one line per call) and exit configurably.
    mkdir -p bin
    CARGO_LOG="$TMPDIR/cargo-calls.log"
    export CARGO_LOG
    cat > bin/cargo <<'EOF'
#!/bin/sh
echo "$*" >> "$CARGO_LOG"
exit "${STUB_CARGO_EXIT:-0}"
EOF
    chmod +x bin/cargo
    export PATH="$TMPDIR/bin:$PATH"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# ---------------------------------------------------------------------------
# THE REGRESSION: every dry-run must carry --allow-dirty. Without it the gate
# aborts on cargo's dirty-tree guard at this step (uncommitted changelogs) and
# never validates packaging.
# ---------------------------------------------------------------------------
@test "prepublish-check: passes --dry-run AND --allow-dirty for a crate" {
    run_isolated ./scripts/prepublish-check.sh sdmx-types
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    echo "CARGO CALLS:" >&2; cat "$CARGO_LOG" >&2
    [ "$status" -eq 0 ]
    grep -q -- "--dry-run" "$CARGO_LOG"
    grep -q -- "--allow-dirty" "$CARGO_LOG"
    # Both flags on the SAME invocation, scoped to the named crate.
    grep -qE 'publish -p sdmx-types .*--dry-run.*--allow-dirty|publish -p sdmx-types .*--allow-dirty.*--dry-run' "$CARGO_LOG"
}

# ---------------------------------------------------------------------------
# Default (no args) runs every workspace crate in topological order.
# ---------------------------------------------------------------------------
@test "prepublish-check: dry-runs all crates in topological order by default" {
    run_isolated ./scripts/prepublish-check.sh
    echo "STATUS: $status" >&2
    echo "CARGO CALLS:" >&2; cat "$CARGO_LOG" >&2
    [ "$status" -eq 0 ]
    # One call per crate, all carrying --allow-dirty.
    [ "$(grep -c -- '--allow-dirty' "$CARGO_LOG")" -eq 5 ]
    # Topological order preserved: types before parsers before ... before rs.
    expected="sdmx-types
sdmx-parsers
sdmx-writers
sdmx-client
sdmx-rs"
    actual=$(sed -E 's/.*publish -p ([^ ]+).*/\1/' "$CARGO_LOG")
    [ "$actual" = "$expected" ]
}

# ---------------------------------------------------------------------------
# cargo's real exit code is propagated (not flattened to 1), and the loop
# fails fast on the first bad crate.
# ---------------------------------------------------------------------------
@test "prepublish-check: propagates cargo's exit code and fails fast" {
    export STUB_CARGO_EXIT=101
    run_isolated ./scripts/prepublish-check.sh
    echo "STATUS: $status" >&2
    echo "CARGO CALLS:" >&2; cat "$CARGO_LOG" >&2
    [ "$status" -eq 101 ]
    # Fail-fast: aborted on the FIRST crate, did not march through all five.
    [ "$(grep -c -- 'publish -p' "$CARGO_LOG")" -eq 1 ]
}

# ---------------------------------------------------------------------------
# The folded-in facade release-notes gate: a default (facade-in-scope) run must
# FAIL when the curated notes file is missing, before any publish dry-run.
# ---------------------------------------------------------------------------
@test "prepublish-check: fails when facade is in scope but curated notes are missing" {
    rm -f crates/sdmx-rs/release-notes/0.2.0.md
    run_isolated ./scripts/prepublish-check.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"release-notes/0.2.0.md"* ]]
    # Gate ran before the publish loop: no cargo publish call was logged.
    run ! grep -q -- 'publish -p' "$CARGO_LOG"
}

# ---------------------------------------------------------------------------
# Leaf-only invocation must NOT trigger the facade notes gate.
# ---------------------------------------------------------------------------
@test "prepublish-check: leaf-only run skips the facade notes gate" {
    rm -f crates/sdmx-rs/release-notes/0.2.0.md   # facade notes absent...
    run_isolated ./scripts/prepublish-check.sh sdmx-types   # ...but facade not in scope
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    grep -q -- 'publish -p sdmx-types' "$CARGO_LOG"
}
