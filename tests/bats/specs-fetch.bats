#!/usr/bin/env bats
bats_require_minimum_version 1.5.0
# ==============================================================================
# Test suite for scripts/lib/specs-fetch.sh (the shared fetch-on-demand kernel).
#
# Pure-function coverage of the sources.toml parsers and the symbol line-span
# computation — no network, no nix. The two thin drivers (fetch-specs.sh,
# update-specs.sh) are covered in their own suites; here we pin down the awk
# parsing the whole overlay relies on.
#
# Run with: bats tests/bats/specs-fetch.bats
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"
    REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"

    # shellcheck source=/dev/null
    . "$REPO_ROOT/scripts/lib/log.sh"
    # shellcheck source=/dev/null
    . "$REPO_ROOT/scripts/lib/specs-fetch.sh"

    SPECS_SOURCES="$BATS_TEST_TMPDIR/sources.toml"
    export SPECS_SOURCES
    cat > "$SPECS_SOURCES" <<'EOF'
[upstream]
owner = "sdmx-twg"
repo = "sdmx-ml"
w3c = ["xml.xsd"]

[edition."3.0"]
ref = "v3.0.0"
rev = "29f1a3d856c4259429f5ec0eae811653adc5cdb5"
narHash = "sha256-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa="

[edition."3.1"]
ref = "v3.1.0"
rev = "182248b3c8030b595187dca51ca341d5ff839c24"
narHash = "sha256-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb="

[files."3.0"]
"SDMXCommon.xsd" = "cce51000aa"
"xml.xsd" = "ad5f82bf"

[files."3.1"]
"SDMXCommon.xsd" = "e95ec300bb"
"xml.xsd" = "ad5f82bf"
EOF
}

@test "specs_editions lists every pinned edition in file order" {
    run specs_editions
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "3.0" ]
    [ "${lines[1]}" = "3.1" ]
    [ "${#lines[@]}" -eq 2 ]
}

@test "specs_upstream reads owner and repo" {
    [ "$(specs_upstream owner)" = "sdmx-twg" ]
    [ "$(specs_upstream repo)" = "sdmx-ml" ]
}

@test "specs_field reads ref/rev/narHash scoped to the right edition" {
    [ "$(specs_field 3.0 rev)" = "29f1a3d856c4259429f5ec0eae811653adc5cdb5" ]
    [ "$(specs_field 3.1 rev)" = "182248b3c8030b595187dca51ca341d5ff839c24" ]
    [ "$(specs_field 3.1 ref)" = "v3.1.0" ]
    [ "$(specs_field 3.0 narHash)" = "sha256-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa=" ]
}

@test "specs_file_names lists an edition's files" {
    run specs_file_names 3.0
    [ "${lines[0]}" = "SDMXCommon.xsd" ]
    [ "${lines[1]}" = "xml.xsd" ]
    [ "${#lines[@]}" -eq 2 ]
}

@test "specs_file_sha reads a file's sha and distinguishes editions" {
    [ "$(specs_file_sha 3.0 SDMXCommon.xsd)" = "cce51000aa" ]
    [ "$(specs_file_sha 3.1 SDMXCommon.xsd)" = "e95ec300bb" ]
    [ "$(specs_file_sha 3.0 xml.xsd)" = "ad5f82bf" ]
}

@test "specs_blob_url derives the canonical pinned blob URL" {
    [ "$(specs_blob_url 3.0 SDMXCommon.xsd)" = \
      "https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d856c4259429f5ec0eae811653adc5cdb5/schemas/SDMXCommon.xsd" ]
}

@test "specs_stamp_value is stable and changes when a sha changes" {
    a=$(specs_stamp_value)
    b=$(specs_stamp_value)
    [ "$a" = "$b" ]
    sed -i 's/cce51000aa/dddddddddd/' "$SPECS_SOURCES"
    c=$(specs_stamp_value)
    [ "$a" != "$c" ]
}

@test "specs_symbol_span returns the 1-based line span of a named type" {
    cat > "$BATS_TEST_TMPDIR/x.xsd" <<'EOF'
<xs:schema>
  <xs:complexType name="FooType">
    <xs:sequence>
      <xs:element name="a"/>
    </xs:sequence>
  </xs:complexType>
  <xs:complexType name="BarType">
    <xs:attribute name="b"/>
  </xs:complexType>
</xs:schema>
EOF
    [ "$(specs_symbol_span "$BATS_TEST_TMPDIR/x.xsd" FooType)" = "2 6" ]
    [ "$(specs_symbol_span "$BATS_TEST_TMPDIR/x.xsd" BarType)" = "7 9" ]
}

@test "specs_symbol_span is depth-aware (nested anonymous complexType)" {
    cat > "$BATS_TEST_TMPDIR/y.xsd" <<'EOF'
<xs:schema>
  <xs:complexType name="OuterType">
    <xs:sequence>
      <xs:element name="inner">
        <xs:complexType>
          <xs:attribute name="z"/>
        </xs:complexType>
      </xs:element>
    </xs:sequence>
  </xs:complexType>
</xs:schema>
EOF
    [ "$(specs_symbol_span "$BATS_TEST_TMPDIR/y.xsd" OuterType)" = "2 10" ]
}

@test "specs_symbol_span finds simpleType too" {
    cat > "$BATS_TEST_TMPDIR/z.xsd" <<'EOF'
<xs:schema>
  <xs:simpleType name="VersionType">
    <xs:restriction base="xs:string"/>
  </xs:simpleType>
</xs:schema>
EOF
    [ "$(specs_symbol_span "$BATS_TEST_TMPDIR/z.xsd" VersionType)" = "2 4" ]
}
