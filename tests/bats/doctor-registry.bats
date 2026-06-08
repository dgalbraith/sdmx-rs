#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/doctor-registry.sh
#
# doctor-registry is READ-ONLY. These tests NEVER touch crates.io — `curl` is
# replaced by a PATH shim (mock_crates) that serves canned index/API responses
# from a per-test, mutable copy of tests/bats/fixtures/crates/. The OFFLINE tier
# checks committed files (publish.yml, release.toml, crate manifests), recreated
# minimally in an isolated tree. The ONLINE tier is gated behind CRATES_IO_TOKEN.
#
# Run with: bats tests/bats/doctor-registry.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    REPO_ROOT="$BATS_TEST_DIRNAME/../.."

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    # Scripts + sourced libs (registry-spec sources forge-spec + log).
    mkdir -p scripts/lib
    cp "$REPO_ROOT/scripts/doctor-registry.sh" scripts/
    cp "$REPO_ROOT/scripts/lib/registry-spec.sh" scripts/lib/
    cp "$REPO_ROOT/scripts/lib/forge-spec.sh" scripts/lib/
    cp "$REPO_ROOT/scripts/lib/log.sh" scripts/lib/

    # OFFLINE prerequisites:
    #  - publish.yml bound to the `release` environment + the tag trigger glob
    mkdir -p .github/workflows
    cat > .github/workflows/publish.yml <<'EOF'
name: Publish
on:
  push:
    tags: [ 'sdmx-*/v*' ]
jobs:
  publish:
    environment: release
    steps:
      - run: echo publish
EOF
    #  - release.toml tag convention
    cat > release.toml <<'EOF'
tag-name = "{{crate_name}}/v{{version}}"
EOF
    #  - per-crate manifests with publish-required metadata (workspace-inherited
    #    license/repository + a direct rust-version, matching the real crates).
    for c in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        mkdir -p "crates/$c"
        cat > "crates/$c/Cargo.toml" <<EOF
[package]
name = "$c"
rust-version = "1.91.0"
license.workspace = true
repository.workspace = true
EOF
    done

    # Git repo + origin so registry_spec_tp_repo resolves to dgalbraith/sdmx-rs.
    git init --initial-branch=main -q
    git config user.email "dg@lbraith.io"
    git config user.name "David Galbraith"
    git remote add origin "git@github.com:dgalbraith/sdmx-rs.git"

    # Per-test MUTABLE fixture copy.
    export FORGE_FIXTURES="$TMPDIR/forge-fixtures"
    mkdir -p "$FORGE_FIXTURES"
    cp -r "$REPO_ROOT/tests/bats/fixtures/crates" "$FORGE_FIXTURES/crates"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# Run with ambient env scrubbed; pass CRATES_IO_TOKEN through when set by the test.
run_doctor() {
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE REGISTRY_ENFORCEMENT_REQUIRED
    run sh scripts/doctor-registry.sh
}

# ==============================================================================
# Token gating
# ==============================================================================

@test "doctor-registry: no token -> online tier skipped, offline ran, exit 0" {
    mock_crates
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE CRATES_IO_TOKEN
    run sh scripts/doctor-registry.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"Offline checks"* ]]
    [[ "$output" == *"Publish workflow present"* ]]
    [[ "$output" == *"CRATES_IO_TOKEN not set"* ]]
    [[ "$output" == *"offline checks passed"* ]]
}

# ==============================================================================
# Full match
# ==============================================================================

@test "doctor-registry: all live state matches spec -> exit 0" {
    mock_crates
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE REGISTRY_ENFORCEMENT_REQUIRED
    CRATES_IO_TOKEN=tok run sh scripts/doctor-registry.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"matches spec"* ]]
}

# ==============================================================================
# Missing TP config -> drift
# ==============================================================================

@test "doctor-registry: a crate with no TP config -> exit 1" {
    mock_crates
    # Drop sdmx-writers' config from the live response.
    jq '.github_configs |= map(select(.crate != "sdmx-writers"))' \
        "$FORGE_FIXTURES/crates/configs.json" > "$FORGE_FIXTURES/crates/configs.json.tmp"
    mv "$FORGE_FIXTURES/crates/configs.json.tmp" "$FORGE_FIXTURES/crates/configs.json"
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE REGISTRY_ENFORCEMENT_REQUIRED
    CRATES_IO_TOKEN=tok run sh scripts/doctor-registry.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"no matching Trusted Publisher"* ]]
    [[ "$output" == *"sdmx-writers"* ]]
}

# ==============================================================================
# Stray/duplicate TP config -> drift (exactly-one invariant)
# ==============================================================================

@test "doctor-registry: an extra TP config for a crate -> exit 1" {
    mock_crates
    # Add a second config for sdmx-types pointing at a DIFFERENT workflow — the
    # spec-matching count stays 1 but the total is 2, so "exactly one" fails.
    jq '.github_configs += [{"id":99,"crate":"sdmx-types","repository_owner":"dgalbraith","repository_owner_id":100,"repository_name":"sdmx-rs","workflow_filename":"evil.yml","environment":"release","created_at":"2026-01-01T00:00:00Z"}]' \
        "$FORGE_FIXTURES/crates/configs.json" > "$FORGE_FIXTURES/crates/configs.json.tmp"
    mv "$FORGE_FIXTURES/crates/configs.json.tmp" "$FORGE_FIXTURES/crates/configs.json"
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE REGISTRY_ENFORCEMENT_REQUIRED
    CRATES_IO_TOKEN=tok run sh scripts/doctor-registry.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"extra Trusted Publishing config"* ]]
}

# ==============================================================================
# Wrong workflow in TP config -> drift
# ==============================================================================

@test "doctor-registry: TP config pointing at wrong workflow -> exit 1" {
    mock_crates
    jq '(.github_configs[] | select(.crate == "sdmx-rs") | .workflow_filename) = "release.yml"' \
        "$FORGE_FIXTURES/crates/configs.json" > "$FORGE_FIXTURES/crates/configs.json.tmp"
    mv "$FORGE_FIXTURES/crates/configs.json.tmp" "$FORGE_FIXTURES/crates/configs.json"
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE REGISTRY_ENFORCEMENT_REQUIRED
    CRATES_IO_TOKEN=tok run sh scripts/doctor-registry.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"no matching Trusted Publisher"* ]]
}

# ==============================================================================
# Enforcement: trustpub_only=false warns by default, fails under opt-in
# ==============================================================================

@test "doctor-registry: trustpub_only=false -> warn, exit 0 (default)" {
    mock_crates
    # Turn enforcement off for one crate.
    printf '{"crate":{"name":"sdmx-rs","trustpub_only":false}}\n' \
        > "$FORGE_FIXTURES/crates/crate-sdmx-rs.json"
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE REGISTRY_ENFORCEMENT_REQUIRED
    CRATES_IO_TOKEN=tok run sh scripts/doctor-registry.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"enforcement deferred"* ]]
}

@test "doctor-registry: trustpub_only=false + REGISTRY_ENFORCEMENT_REQUIRED=1 -> exit 1" {
    mock_crates
    printf '{"crate":{"name":"sdmx-rs","trustpub_only":false}}\n' \
        > "$FORGE_FIXTURES/crates/crate-sdmx-rs.json"
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE
    REGISTRY_ENFORCEMENT_REQUIRED=1 CRATES_IO_TOKEN=tok run sh scripts/doctor-registry.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"trustpub_only=false"* ]]
}

# ==============================================================================
# Crate not yet reserved -> warn, not fail
# ==============================================================================

@test "doctor-registry: an unreserved crate -> warn, exit 0" {
    mock_crates
    # Remove the reservation marker so the index probe 404s for sdmx-rs.
    rm -f "$FORGE_FIXTURES/crates/reserved/sdmx-rs"
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE REGISTRY_ENFORCEMENT_REQUIRED
    CRATES_IO_TOKEN=tok run sh scripts/doctor-registry.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"not yet reserved"* ]]
}

# ==============================================================================
# Offline drift: publish.yml not bound to the release environment
# ==============================================================================

@test "doctor-registry: publish.yml missing release environment -> exit 1" {
    mock_crates
    # Rewrite publish.yml without the environment binding.
    cat > .github/workflows/publish.yml <<'EOF'
name: Publish
on:
  push:
    tags: [ 'sdmx-*/v*' ]
jobs:
  publish:
    steps:
      - run: echo publish
EOF
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE CRATES_IO_TOKEN
    run sh scripts/doctor-registry.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"does not declare 'environment: release'"* ]]
}

# ==============================================================================
# Offline drift: a crate marked publish = false
# ==============================================================================

@test "doctor-registry: a crate with publish=false -> exit 1" {
    mock_crates
    printf '\npublish = false\n' >> crates/sdmx-types/Cargo.toml
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE CRATES_IO_TOKEN
    run sh scripts/doctor-registry.sh
    echo "STATUS: $status" >&2; echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"publish=false but is in the registry spec"* ]]
}
