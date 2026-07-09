#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/fetch-specs.sh (the read-only verify driver).
#
# Testing approach: the materialise-and-verify contract with nix mocked (a PATH
# shim that prints a fixture "FOD output" tree) so tests are fast and offline.
# We assert the happy path (materialise + per-file sha-verify + stamp), the
# idempotent no-op on a current stamp, and a hard failure on a sha mismatch.
#
# Run with: bats tests/bats/fetch-specs.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"
    REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"

    cd "$BATS_TEST_TMPDIR" || exit 1

    mkdir -p scripts/lib bin specs "store/3.0/schemas" "store/3.1/schemas"
    cp "$REPO_ROOT/scripts/fetch-specs.sh" scripts/
    cp "$REPO_ROOT/scripts/lib/log.sh" "$REPO_ROOT/scripts/lib/specs-fetch.sh" scripts/lib/

    printf '<xs:schema>\r\n\t<xs:complexType name="A"/>\r\n</xs:schema>\r\n' > "store/3.0/schemas/A.xsd"
    printf '<xs:schema>\r\n\t<xs:complexType name="B"/>\r\n</xs:schema>\r\n' > "store/3.1/schemas/B.xsd"
    sha_a=$(sha256sum "store/3.0/schemas/A.xsd" | cut -d' ' -f1)
    sha_b=$(sha256sum "store/3.1/schemas/B.xsd" | cut -d' ' -f1)

    cat > specs/sources.toml <<EOF
[upstream]
owner = "sdmx-twg"
repo = "sdmx-ml"
w3c = ["xml.xsd"]

[edition."3.0"]
ref = "v3.0.0"
rev = "29f1a3d856c4259429f5ec0eae811653adc5cdb5"
narHash = "sha256-aaa="

[edition."3.1"]
ref = "v3.1.0"
rev = "182248b3c8030b595187dca51ca341d5ff839c24"
narHash = "sha256-bbb="

[files."3.0"]
"A.xsd" = "$sha_a"

[files."3.1"]
"B.xsd" = "$sha_b"
EOF

    # Mock nix: any `build ... .#sdmxSpecs ... --print-out-paths` prints the tree.
    cat > bin/nix <<EOF
#!/bin/sh
printf '%s\n' "$BATS_TEST_TMPDIR/store"
EOF
    chmod +x bin/nix

    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
    unset NIX SHA256SUM
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

run_fetch() {
    run env SDMX_SPECS_DIR="$BATS_TEST_TMPDIR/out" SPECS_SOURCES="$BATS_TEST_TMPDIR/specs/sources.toml" \
        SPECS_FLAKE="$BATS_TEST_TMPDIR" sh scripts/fetch-specs.sh
}

@test "fetch-specs: materialises the tree, verifies shas, writes the stamp" {
    run_fetch
    [ "$status" -eq 0 ]
    [[ "$output" == *"materialised and verified"* ]]
    [ -f "$BATS_TEST_TMPDIR/out/3.0/schemas/A.xsd" ]
    [ -f "$BATS_TEST_TMPDIR/out/3.1/schemas/B.xsd" ]
    [ -f "$BATS_TEST_TMPDIR/out/.sha256.stamp" ]
}

@test "fetch-specs: second run is an idempotent no-op (stamp current)" {
    run_fetch
    [ "$status" -eq 0 ]
    run_fetch
    [ "$status" -eq 0 ]
    [[ "$output" == *"idempotent no-op"* ]]
}

@test "fetch-specs: a current stamp over a deleted tree re-materialises (not a stale no-op)" {
    run_fetch
    [ "$status" -eq 0 ]
    [ -f "$BATS_TEST_TMPDIR/out/.sha256.stamp" ]
    # Drop the materialised tree but keep the stamp: this is what a rebase across
    # the untrack commit does (it deletes the once-tracked .xsd; the gitignored
    # stamp survives). The fast path must re-materialise, not trust the stamp.
    rm -rf "$BATS_TEST_TMPDIR/out/3.0" "$BATS_TEST_TMPDIR/out/3.1"
    run_fetch
    [ "$status" -eq 0 ]
    [[ "$output" == *"materialised and verified"* ]]
    [[ "$output" != *"idempotent no-op"* ]]
    [ -f "$BATS_TEST_TMPDIR/out/3.0/schemas/A.xsd" ]
    [ -f "$BATS_TEST_TMPDIR/out/3.1/schemas/B.xsd" ]
}

@test "fetch-specs: a present, unstamped, sha-valid tree is accepted without a fetch" {
    run_fetch
    [ "$status" -eq 0 ]
    rm -f "$BATS_TEST_TMPDIR/out/.sha256.stamp"
    # Break the mock so any fetch attempt would fail; a valid present tree must not fetch.
    printf '#!/bin/sh\nexit 1\n' > bin/nix
    run_fetch
    [ "$status" -eq 0 ]
    [[ "$output" == *"present and verified"* ]]
}

@test "fetch-specs: fails on a sha mismatch against sources.toml" {
    sed -i 's/^"A.xsd" = .*/"A.xsd" = "deadbeef"/' specs/sources.toml
    run_fetch
    [ "$status" -ne 0 ]
    [[ "$output" == *"sha256"* ]]
}

@test "fetch-specs: fails clearly when the pin file is missing" {
    rm -f specs/sources.toml
    run_fetch
    [ "$status" -ne 0 ]
    [[ "$output" == *"pin file not found"* ]]
}
