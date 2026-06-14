#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/gen-xsd-fragments.sh (apply) and
# scripts/check-xsd-fragments.sh (doctor).
#
# Exercises the spec -> doctor -> apply kernel on a minimal self-contained
# fixture mirroring the real layout (specs/, crates/sdmx-types/). The generator
# half covers verbatim slicing (complexType and simpleType, depth-aware over
# nested anonymous types), the structural-divergence split versus the
# documentation-only single-fragment case, the symbol-not-found error, and
# README preservation. The doctor half must pass when fresh+wired (ignoring the
# directory's authored README.md and scoped .markdownlint.yaml) and fail on a
# stale fragment, a missing include, or an orphan include.
#
# Run with: bats tests/bats/check-xsd-fragments.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    REPO="$BATS_TEST_DIRNAME/../.."
    FIX=$(mktemp -d)
    OUTDIR="$FIX/crates/sdmx-types/docs/xsd-fragments"

    mkdir -p "$FIX/scripts" "$FIX/specs/3.0/schemas" "$FIX/specs/3.1/schemas" \
        "$FIX/crates/sdmx-types/src" "$OUTDIR"
    cp "$REPO/scripts/gen-xsd-fragments.sh" "$REPO/scripts/check-xsd-fragments.sh" "$FIX/scripts/"
    mkdir -p "$FIX/scripts/lib"
    cp "$REPO/scripts/lib/log.sh" "$FIX/scripts/lib/"

    # A minimal schema, identical across both editions.
    schema='<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="FooType">
    <xs:attribute name="bar" type="xs:string"/>
  </xs:complexType>
</xs:schema>'
    printf '%s\n' "$schema" >"$FIX/specs/3.0/schemas/Test.xsd"
    printf '%s\n' "$schema" >"$FIX/specs/3.1/schemas/Test.xsd"

    cat >"$FIX/crates/sdmx-types/xsd-manifest.toml" <<'EOF'
[[fragment]]
symbol   = "FooType"
file     = "schemas/Test.xsd"
editions = ["3.0", "3.1"]
rust     = ["Foo"]
EOF

    write_src   # without the include, by default
}

teardown() {
    rm -rf "$FIX"
}

# Rewrite the fixture source; pass "wired" to include the fragment.
write_src() {
    if [ "${1:-}" = "wired" ]; then inc='#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/FooType.md"))]'; else inc=""; fi
    {
        printf '/// A foo.\n///\n/// ## Specification\n/// - **Type**: `FooType`\n'
        [ -n "$inc" ] && printf '%s\n' "$inc"
        printf 'pub struct Foo;\n'
    } >"$FIX/crates/sdmx-types/src/lib.rs"
}

# Overwrite one edition's schema, wrapping the given type body in a schema envelope.
write_schema() { # edition type-body
    cat >"$FIX/specs/$1/schemas/Test.xsd" <<EOF
<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
$2
</xs:schema>
EOF
}

# Write the same type body to both editions (the structurally-identical case).
write_both() { write_schema 3.0 "$1"; write_schema 3.1 "$1"; }

@test "gen writes a verbatim fragment for the manifest symbol" {
    run sh "$FIX/scripts/gen-xsd-fragments.sh"
    [ "$status" -eq 0 ]
    [ -f "$OUTDIR/FooType.md" ]
    grep -q '<xs:complexType name="FooType">' "$OUTDIR/FooType.md"
    grep -q '<summary>XSD contract: <code>FooType</code> (SDMX 3.0 and 3.1)</summary>' "$OUTDIR/FooType.md"
}

@test "gen splits into per-edition fragments on a structural divergence" {
    write_schema 3.0 '  <xs:complexType name="FooType">
    <xs:attribute name="bar" type="xs:string"/>
  </xs:complexType>'
    write_schema 3.1 '  <xs:complexType name="FooType">
    <xs:attribute name="bar" type="xs:string"/>
    <xs:attribute name="baz" type="xs:string"/>
  </xs:complexType>'
    run sh "$FIX/scripts/gen-xsd-fragments.sh"
    [ "$status" -eq 0 ]
    [ -f "$OUTDIR/FooType.3.0.md" ]
    [ -f "$OUTDIR/FooType.3.1.md" ]
    [ ! -f "$OUTDIR/FooType.md" ]
    grep -qF '(SDMX 3.0)' "$OUTDIR/FooType.3.0.md"
    grep -qF '(SDMX 3.1)' "$OUTDIR/FooType.3.1.md"
    grep -qF 'name="baz"' "$OUTDIR/FooType.3.1.md"
    ! grep -qF 'name="baz"' "$OUTDIR/FooType.3.0.md"
}

@test "gen keeps a single fragment when editions differ only in xs:documentation" {
    write_schema 3.0 '  <xs:complexType name="FooType">
    <xs:annotation><xs:documentation>Original 3.0 wording.</xs:documentation></xs:annotation>
    <xs:attribute name="bar" type="xs:string"/>
  </xs:complexType>'
    write_schema 3.1 '  <xs:complexType name="FooType">
    <xs:annotation><xs:documentation>Completely reworded for 3.1.</xs:documentation></xs:annotation>
    <xs:attribute name="bar" type="xs:string"/>
  </xs:complexType>'
    run sh "$FIX/scripts/gen-xsd-fragments.sh"
    [ "$status" -eq 0 ]
    [ -f "$OUTDIR/FooType.md" ]
    [ ! -f "$OUTDIR/FooType.3.0.md" ]
    [ ! -f "$OUTDIR/FooType.3.1.md" ]
    # emitted verbatim is the latest edition, documentation intact
    grep -qF 'reworded for 3.1' "$OUTDIR/FooType.md"
}

@test "gen exits non-zero when the manifest symbol is absent from the schema" {
    write_both '  <xs:complexType name="OtherType">
    <xs:attribute name="x" type="xs:string"/>
  </xs:complexType>'
    run sh "$FIX/scripts/gen-xsd-fragments.sh"
    [ "$status" -ne 0 ]
    echo "$output" | grep -qF "not found"
}

@test "gen extracts a simpleType verbatim" {
    write_both '  <xs:simpleType name="FooType">
    <xs:restriction base="xs:string"><xs:pattern value="[A-Z]+"/></xs:restriction>
  </xs:simpleType>'
    run sh "$FIX/scripts/gen-xsd-fragments.sh"
    [ "$status" -eq 0 ]
    [ -f "$OUTDIR/FooType.md" ]
    grep -qF '<xs:simpleType name="FooType">' "$OUTDIR/FooType.md"
    grep -qF '<xs:pattern value="[A-Z]+"/>' "$OUTDIR/FooType.md"
}

@test "gen slices a nested anonymous complexType without closing early" {
    write_both '  <xs:complexType name="FooType">
    <xs:sequence>
      <xs:element name="inner">
        <xs:complexType>
          <xs:attribute name="deep" type="xs:string"/>
        </xs:complexType>
      </xs:element>
    </xs:sequence>
    <xs:attribute name="outer" type="xs:string"/>
  </xs:complexType>'
    run sh "$FIX/scripts/gen-xsd-fragments.sh"
    [ "$status" -eq 0 ]
    [ -f "$OUTDIR/FooType.md" ]
    # outer appears after the nested type: proof the slice did not stop at the inner close
    grep -qF 'name="deep"' "$OUTDIR/FooType.md"
    grep -qF 'name="outer"' "$OUTDIR/FooType.md"
}

@test "gen preserves an authored README.md across regeneration" {
    printf '# Authored\n' >"$OUTDIR/README.md"
    run sh "$FIX/scripts/gen-xsd-fragments.sh"
    [ "$status" -eq 0 ]
    [ -f "$OUTDIR/README.md" ]
    grep -qF 'Authored' "$OUTDIR/README.md"
    [ -f "$OUTDIR/FooType.md" ]
}

@test "doctor passes when fragments are fresh and wired" {
    sh "$FIX/scripts/gen-xsd-fragments.sh"
    write_src wired
    run sh "$FIX/scripts/check-xsd-fragments.sh"
    [ "$status" -eq 0 ]
}

@test "doctor ignores authored files (README.md, .markdownlint.yaml) in the dir" {
    sh "$FIX/scripts/gen-xsd-fragments.sh"
    write_src wired
    # Both are authored, not generated, so the freshness diff must skip them;
    # without the -x exclusions, diff -r would flag each as "only in OUT".
    printf '# XSD Contract Fragments\n' >"$OUTDIR/README.md"
    printf 'extends: ../../../../.markdownlint.yaml\nMD033: false\n' >"$OUTDIR/.markdownlint.yaml"
    run sh "$FIX/scripts/check-xsd-fragments.sh"
    [ "$status" -eq 0 ]
}

@test "doctor fails on a stale (hand-edited) fragment" {
    sh "$FIX/scripts/gen-xsd-fragments.sh"
    write_src wired
    echo "tampered" >>"$OUTDIR/FooType.md"
    run sh "$FIX/scripts/check-xsd-fragments.sh"
    [ "$status" -eq 1 ]
    echo "$output" | grep -q "STALE"
}

@test "doctor fails when an include is missing" {
    sh "$FIX/scripts/gen-xsd-fragments.sh"
    run sh "$FIX/scripts/check-xsd-fragments.sh"
    [ "$status" -eq 1 ]
    echo "$output" | grep -q "missing include"
}

@test "doctor fails on an orphan include" {
    sh "$FIX/scripts/gen-xsd-fragments.sh"
    write_src wired
    printf '#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/Ghost.md"))]\npub struct Bar;\n' >>"$FIX/crates/sdmx-types/src/lib.rs"
    run sh "$FIX/scripts/check-xsd-fragments.sh"
    [ "$status" -eq 1 ]
    echo "$output" | grep -q "orphan"
}
