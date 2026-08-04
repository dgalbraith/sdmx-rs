#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/ci/wait-for-deps.sh
#
# Validates the pre-publish wait gate of publish.yml: before `cargo publish
# --verify` resolves a crate's "=version" pins against the registry, every
# intra-workspace dependency must already be present on the index at the exact
# version cargo resolved. The script discovers those deps from the `cargo
# metadata` RESOLVE graph and probes each one via its sibling wait-for-index.sh.
#
# Lightweight: `cargo` is stubbed via a PATH shim that records its argv and
# prints a canned `cargo metadata` JSON document, and the sibling probe is
# replaced by a recording stub at ci/wait-for-index.sh (where the script's own
# SCRIPT_DIR resolution looks for it). The suite therefore asserts WHICH
# name+version pairs are probed, with no network round-trip. `jq` is real.
#
# Fixture hyphenation is load-bearing: `.packages[].name` carries the real
# HYPHENATED crate name (sdmx-types), while `.resolve.nodes[].deps[].name`
# carries cargo's UNDERSCORED form (sdmx_types), exactly as real metadata does.
# The script reads names through the package map precisely to get the hyphenated
# form, so a "simplification" to `deps[].name` would emit sdmx_types — an
# unpublishable crate name — and fail these tests rather than passing silently.
#
# Run with: bats tests/bats/wait-for-deps.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    cd "$BATS_TEST_TMPDIR" || exit 1

    # Mirror the real scripts/ci + scripts/lib layout so the script's
    # `$(dirname "$0")/../lib/log.sh` source path resolves inside the fixture,
    # and so SCRIPT_DIR/wait-for-index.sh finds the probe stub below.
    mkdir -p ci lib
    cp "$BATS_TEST_DIRNAME/../../scripts/ci/wait-for-deps.sh" ci/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/

    # Recording probe stub in place of the real wait-for-index.sh: append the
    # "<name> <version>" argument vector it was called with, exit configurably.
    PROBE_LOG="$BATS_TEST_TMPDIR/probe-calls.log"
    export PROBE_LOG
    : > "$PROBE_LOG"
    cat > ci/wait-for-index.sh <<'EOF'
#!/bin/sh
echo "$*" >> "$PROBE_LOG"
exit "${STUB_PROBE_EXIT:-0}"
EOF
    chmod +x ci/wait-for-index.sh

    # Canned `cargo metadata --format-version 1` graph. Three workspace crates at
    # DIFFERENT versions (so a probe asserting an exact version cannot pass by
    # coincidence) plus a third-party package that appears both in .packages[]
    # and in a resolve node's .dependencies[], so the startswith("sdmx-") filter
    # is genuinely exercised rather than trivially true.
    #
    #   sdmx-rs 0.3.0      -> sdmx-types 0.1.0, sdmx-parsers 0.2.0, serde 1.0.219
    #   sdmx-types 0.1.0   -> serde 1.0.219 only (no intra-workspace deps)
    METADATA_JSON="$BATS_TEST_TMPDIR/metadata.json"
    export METADATA_JSON
    cat > "$METADATA_JSON" <<'JSON'
{
  "packages": [
    { "id": "path+file:///w/crates/sdmx-types#0.1.0", "name": "sdmx-types", "version": "0.1.0" },
    { "id": "path+file:///w/crates/sdmx-parsers#0.2.0", "name": "sdmx-parsers", "version": "0.2.0" },
    { "id": "path+file:///w/crates/sdmx-rs#0.3.0", "name": "sdmx-rs", "version": "0.3.0" },
    { "id": "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.219", "name": "serde", "version": "1.0.219" }
  ],
  "workspace_members": [
    "path+file:///w/crates/sdmx-types#0.1.0",
    "path+file:///w/crates/sdmx-parsers#0.2.0",
    "path+file:///w/crates/sdmx-rs#0.3.0"
  ],
  "resolve": {
    "nodes": [
      {
        "id": "path+file:///w/crates/sdmx-rs#0.3.0",
        "dependencies": [
          "path+file:///w/crates/sdmx-types#0.1.0",
          "path+file:///w/crates/sdmx-parsers#0.2.0",
          "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.219"
        ],
        "deps": [
          { "name": "sdmx_types", "pkg": "path+file:///w/crates/sdmx-types#0.1.0", "dep_kinds": [{ "kind": null, "target": null }] },
          { "name": "sdmx_parsers", "pkg": "path+file:///w/crates/sdmx-parsers#0.2.0", "dep_kinds": [{ "kind": null, "target": null }] },
          { "name": "serde", "pkg": "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.219", "dep_kinds": [{ "kind": null, "target": null }] }
        ]
      },
      {
        "id": "path+file:///w/crates/sdmx-parsers#0.2.0",
        "dependencies": ["path+file:///w/crates/sdmx-types#0.1.0"],
        "deps": [
          { "name": "sdmx_types", "pkg": "path+file:///w/crates/sdmx-types#0.1.0", "dep_kinds": [{ "kind": null, "target": null }] }
        ]
      },
      {
        "id": "path+file:///w/crates/sdmx-types#0.1.0",
        "dependencies": ["registry+https://github.com/rust-lang/crates.io-index#serde@1.0.219"],
        "deps": [
          { "name": "serde", "pkg": "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.219", "dep_kinds": [{ "kind": null, "target": null }] }
        ]
      },
      {
        "id": "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.219",
        "dependencies": [],
        "deps": []
      }
    ],
    "root": null
  },
  "version": 1
}
JSON

    # PATH-shim cargo: log argv (one line per call) and print canned metadata.
    CARGO_LOG="$BATS_TEST_TMPDIR/cargo-calls.log"
    export CARGO_LOG
    mkdir -p bin
    cat > bin/cargo <<'EOF'
#!/bin/sh
echo "$*" >> "$CARGO_LOG"
cat "$METADATA_JSON"
EOF
    chmod +x bin/cargo
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

# ==============================================================================
# wait-for-deps.sh Tests
# ==============================================================================

@test "wait-for-deps: probes each intra-workspace dependency at its exact resolved version" {
    run_isolated ./ci/wait-for-deps.sh sdmx-rs
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    echo "PROBE CALLS:" >&2; cat "$PROBE_LOG" >&2

    [ "$status" -eq 0 ]
    # Exact resolved versions, taken from the resolve graph rather than from a
    # requirement string that would still carry its comparator.
    grep -qx 'sdmx-types 0.1.0' "$PROBE_LOG"
    grep -qx 'sdmx-parsers 0.2.0' "$PROBE_LOG"
    # Those two and nothing else: the third-party dependency in the same resolve
    # node is filtered out, not probed against an index that would never list it.
    [ "$(wc -l < "$PROBE_LOG")" -eq 2 ]
    run ! grep -q 'serde' "$PROBE_LOG"
}

# ---------------------------------------------------------------------------
# THE FAIL-OPEN PATH: an empty query result and a genuinely dependency-free
# crate both exit 0 without probing, and the script cannot tell them apart. Pin
# it here so the exit-0 branch stays attached to a fixture that really has no
# intra-workspace deps, and a query that silently stops matching is caught by
# the probing tests above rather than passing as "nothing to wait for".
# ---------------------------------------------------------------------------
@test "wait-for-deps: exits 0 without probing when the crate has no intra-workspace dependencies" {
    run_isolated ./ci/wait-for-deps.sh sdmx-types
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    echo "PROBE CALLS:" >&2; cat "$PROBE_LOG" >&2

    [ "$status" -eq 0 ]
    [[ "$output" == *"has no intra-workspace dependencies"* ]]
    [[ "$output" == *"nothing to wait for"* ]]
    [ ! -s "$PROBE_LOG" ]
}

# ---------------------------------------------------------------------------
# A failing probe must abort the run. Were the failure swallowed, the script
# would report success and publishing would proceed against a dependency that is
# not on the index yet, which is the failure this gate exists to prevent. The
# loop also stops at the first failure rather than probing the remainder.
# ---------------------------------------------------------------------------
@test "wait-for-deps: propagates a non-zero exit when a dependency probe fails" {
    export STUB_PROBE_EXIT=1

    run_isolated ./ci/wait-for-deps.sh sdmx-rs
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    echo "PROBE CALLS:" >&2; cat "$PROBE_LOG" >&2

    [ "$status" -ne 0 ]
    # Aborted on the first failing probe rather than marching through both.
    [ "$(wc -l < "$PROBE_LOG")" -eq 1 ]
    run ! grep -q 'All workspace dependencies' <<< "$output"
}

@test "wait-for-deps: passes the hyphenated crate name to the probe, not cargo's underscored form" {
    run_isolated ./ci/wait-for-deps.sh sdmx-parsers
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    echo "PROBE CALLS:" >&2; cat "$PROBE_LOG" >&2

    [ "$status" -eq 0 ]
    # The package map supplies "sdmx-types"; the resolve node's own deps[].name
    # would supply "sdmx_types", which no registry index would ever serve.
    grep -qx 'sdmx-types 0.1.0' "$PROBE_LOG"
    run ! grep -q 'sdmx_types' "$PROBE_LOG"
}
