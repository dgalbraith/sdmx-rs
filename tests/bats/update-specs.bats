#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/update-specs.sh (the re-pin / mutating surface).
#
# Testing approach: the re-pin behaviour contract with git and nix mocked (via
# PATH shims) so tests are fast, offline, and deterministic. We assert that a
# pin captures the resolved commit, the NAR hash (trust-on-first-use, parsed
# from the placeholder build's hash-mismatch), and a per-file sha256 for every
# fetched schema — and that adding an edition preserves the existing one.
#
# Run with: bats tests/bats/update-specs.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"
    REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"

    cd "$BATS_TEST_TMPDIR" || exit 1

    mkdir -p scripts/lib bin "fake-tree/schemas"
    cp "$REPO_ROOT/scripts/update-specs.sh" scripts/
    cp "$REPO_ROOT/scripts/lib/log.sh" "$REPO_ROOT/scripts/lib/specs-fetch.sh" scripts/lib/

    # The tree the mocked `nix` "fetch" returns: raw CRLF, like upstream.
    printf '<xs:schema>\r\n\t<xs:complexType name="FooType"/>\r\n</xs:schema>\r\n' \
        > "fake-tree/schemas/SDMXCommon.xsd"
    printf '<?xml version="1.0"?>\n' > "fake-tree/schemas/xml.xsd"

    # Mock git: ls-remote resolves a lightweight tag (no ^{} deref) to a commit.
    cat > bin/git <<'EOF'
#!/bin/sh
case "$*" in
  *"^{}"*)     exit 0 ;;
  *ls-remote*) printf '29f1a3d856c4259429f5ec0eae811653adc5cdb5\trefs/tags/x\n' ;;
  *)           exit 0 ;;
esac
EOF
    chmod +x bin/git

    # Mock nix: the placeholder build (fake hash) fails reporting the real hash;
    # the real build prints the fixture tree's path.
    cat > bin/nix <<EOF
#!/bin/sh
for a in "\$@"; do
  case "\$a" in
    *sha256-AAAAAAAA*)
      printf 'error: hash mismatch in fixed-output derivation:\n' >&2
      printf '         specified: sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\n' >&2
      printf '            got:    sha256-TEST0000000000000000000000000000000000000000=\n' >&2
      exit 1 ;;
  esac
done
printf '%s\n' "$BATS_TEST_TMPDIR/fake-tree"
EOF
    chmod +x bin/nix

    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
    unset NIX GIT SHA256SUM
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
}

@test "update-specs: rejects wrong argument count" {
    run sh scripts/update-specs.sh 3.0
    [ "$status" -eq 2 ]
    [[ "$output" == *"usage:"* ]]
}

@test "update-specs: initial pin records rev, NAR hash (TOFU) and per-file shas" {
    run sh scripts/update-specs.sh 3.0 v3.0.0
    [ "$status" -eq 0 ]

    src="$BATS_TEST_TMPDIR/specs/sources.toml"
    [ -f "$src" ]
    grep -q '^\[edition."3.0"\]' "$src"
    grep -q 'rev = "29f1a3d856c4259429f5ec0eae811653adc5cdb5"' "$src"
    grep -q 'narHash = "sha256-TEST0000000000000000000000000000000000000000="' "$src"
    grep -q '^\[files."3.0"\]' "$src"

    want=$(sha256sum fake-tree/schemas/SDMXCommon.xsd | cut -d' ' -f1)
    grep -q "\"SDMXCommon.xsd\" = \"$want\"" "$src"
    grep -q '"xml.xsd" = ' "$src"
}

@test "update-specs: emits a valid [upstream] table with the w3c provenance list" {
    run sh scripts/update-specs.sh 3.0 v3.0.0
    [ "$status" -eq 0 ]
    src="$BATS_TEST_TMPDIR/specs/sources.toml"
    grep -q '^owner = "sdmx-twg"' "$src"
    grep -q '^repo = "sdmx-ml"' "$src"
    grep -q '^w3c = \["xml.xsd"\]' "$src"
}

@test "update-specs: generated sources.toml is taplo-canonical (toml-check)" {
    command -v taplo >/dev/null || skip "taplo not available"
    run sh scripts/update-specs.sh 3.0 v3.0.0
    [ "$status" -eq 0 ]
    run env RUST_LOG=error taplo fmt --check "$BATS_TEST_TMPDIR/specs/sources.toml"
    [ "$status" -eq 0 ]
}

@test "update-specs: adding an edition preserves the existing one" {
    run sh scripts/update-specs.sh 3.0 v3.0.0
    [ "$status" -eq 0 ]
    run sh scripts/update-specs.sh 3.1 v3.1.0
    [ "$status" -eq 0 ]

    src="$BATS_TEST_TMPDIR/specs/sources.toml"
    grep -q '^\[edition."3.0"\]' "$src"
    grep -q '^\[edition."3.1"\]' "$src"
    grep -q '^\[files."3.0"\]' "$src"
    grep -q '^\[files."3.1"\]' "$src"
}

@test "update-specs: re-pinning the same edition is idempotent (no duplicate tables)" {
    run sh scripts/update-specs.sh 3.0 v3.0.0
    [ "$status" -eq 0 ]
    run sh scripts/update-specs.sh 3.0 v3.0.0
    [ "$status" -eq 0 ]
    src="$BATS_TEST_TMPDIR/specs/sources.toml"
    [ "$(grep -c '^\[edition."3.0"\]' "$src")" -eq 1 ]
    [ "$(grep -c '^\[files."3.0"\]' "$src")" -eq 1 ]
}

@test "update-specs: fails clearly when the ref cannot be resolved" {
    cat > bin/git <<'EOF'
#!/bin/sh
exit 0
EOF
    chmod +x bin/git
    run sh scripts/update-specs.sh 3.0 v9.9.9
    [ "$status" -ne 0 ]
    [[ "$output" == *"could not resolve ref"* ]]
}
