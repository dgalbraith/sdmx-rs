#!/bin/sh
# ===================================================================
# gen-xsd-fragments.sh
#
# Generates the vendored XSD contract fragments for sdmx-types from
# crates/sdmx-types/xsd-manifest.toml. For each [[fragment]] entry it
# slices the named complexType/simpleType verbatim out of the pinned
# schema(s), wraps it in a collapsible <details> Markdown block, and
# writes crates/sdmx-types/docs/xsd-fragments/<symbol>.md.
#
# Editions are compared with <xs:documentation> narrative normalised
# out (documentation is narrative, not the contract; the structure is
# the arbiter), so a fragment splits into <symbol>.<edition>.md only on
# a STRUCTURAL divergence (an element/attribute difference), never on a
# documentation-prose difference. xs:appinfo is left in the comparison
# (it can be material). The emitted fragment is always byte-verbatim.
#
# This is the "apply" half of the spec -> doctor -> apply kernel: the
# only writer of the fragment files. Run by hand (just gen-xsd-fragments)
# when adding a manifest entry or re-vendoring a schema; the doctor
# (check-xsd-fragments.sh) verifies the committed result.
# ===================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
MANIFEST="$ROOT/crates/sdmx-types/xsd-manifest.toml"
SPECS="$ROOT/specs"
# Output dir; overridable (XSD_FRAGMENTS_OUT) so the doctor can regenerate into a
# temp dir for a non-mutating freshness diff.
OUT="${XSD_FRAGMENTS_OUT:-$ROOT/crates/sdmx-types/docs/xsd-fragments}"

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

# Print the verbatim definition of a named xs:complexType/xs:simpleType,
# depth-aware so nested anonymous complexTypes do not close it early.
slice() { # file symbol
  awk -v name="$2" '
    function opens(s,  t){ t=s; return gsub("<xs:"TAG"[ />]","X",t) }
    function closes(s,  t){ t=s; return gsub("</xs:"TAG">","X",t) }
    BEGIN{ started=0; depth=0 }
    !started && $0 ~ ("<xs:complexType name=\""name"\"") { TAG="complexType"; started=1 }
    !started && $0 ~ ("<xs:simpleType name=\""name"\"")  { TAG="simpleType";  started=1 }
    started { print; depth += opens($0) - closes($0); if (depth<=0) exit }
  ' "$1"
}

# Blank out <xs:documentation> narrative so divergence tracks structure, not
# prose. Leaves xs:appinfo intact (it can carry material, machine-read hints).
strip_doc() { sed 's#<xs:documentation>.*</xs:documentation>##g'; }

# Wrap verbatim XSD (stdin) in the fragment template.
emit() { # symbol editions-label outfile
  {
    printf '<details>\n'
    printf '<summary>XSD contract: <code>%s</code> (SDMX %s)</summary>\n\n' "$1" "$2"
    printf '```xml\n'
    cat
    printf '```\n\n'
    printf '</details>\n'
  } >"$3"
}

# "3.0 3.1" -> "3.0 and 3.1" for an identical-fragment summary.
# Subshell body ( ) so the loop locals (out, e) stay contained, not leaked.
join_editions() (
  # shellcheck disable=SC2086
  set -- $1
  case $# in
    1) printf '%s' "$1" ;;
    2) printf '%s and %s' "$1" "$2" ;;
    *) out=""; for e do out="${out:+$out, }$e"; done; printf '%s' "$out" ;;
  esac
)

# Parse the manifest into tab-separated rows: symbol  file  "ed ed".
# (rust items are the doctor's concern; the generator only needs symbol/file/editions.)
parse_manifest() {
  awk '
    function val(  s){ s=$0; sub(/^[^=]*=[ \t]*/,"",s); gsub(/"/,"",s); sub(/[ \t]+$/,"",s); return s }
    function arr(  s){ s=$0; sub(/^[^=]*=[ \t]*/,"",s); gsub(/[]["]/,"",s); gsub(/[ \t]/,"",s); gsub(/,/," ",s); return s }
    /^\[\[fragment\]\]/ { if (sym!="") print sym"\t"file"\t"eds; sym="";file="";eds="" }
    /^symbol[ \t]*=/   { sym=val() }
    /^file[ \t]*=/     { file=val() }
    /^editions[ \t]*=/ { eds=arr() }
    END { if (sym!="") print sym"\t"file"\t"eds }
  ' "$MANIFEST"
}

mkdir -p "$OUT"
# Purge generated fragments (a removed manifest entry leaves no orphan), but keep the
# authored README.md.
find "$OUT" -name '*.md' ! -name 'README.md' -exec rm -f {} +

TAB=$(printf '\t')
parse_manifest | while IFS="$TAB" read -r symbol file editions; do
  # Extract each edition's verbatim form and its structural (doc-stripped) form.
  divergent=0; first=""
  for ed in $editions; do
    src="$SPECS/$ed/$file"
    slice "$src" "$symbol" >"$WORK/$ed.xml"
    [ -s "$WORK/$ed.xml" ] || { log_err "gen-xsd-fragments: symbol '$symbol' not found in $src"; exit 1; }
    strip_doc <"$WORK/$ed.xml" >"$WORK/$ed.stripped"
    if [ -z "$first" ]; then first="$ed"
    elif ! cmp -s "$WORK/$first.stripped" "$WORK/$ed.stripped"; then divergent=1; fi
  done

  if [ "$divergent" -eq 0 ]; then
    # Structurally identical: one fragment, latest edition's verbatim.
    for ed in $editions; do last="$ed"; done
    emit "$symbol" "$(join_editions "$editions")" "$OUT/$symbol.md" <"$WORK/$last.xml"
  else
    # Structural divergence: one verbatim fragment per edition.
    for ed in $editions; do
      emit "$symbol" "$ed" "$OUT/$symbol.$ed.md" <"$WORK/$ed.xml"
    done
  fi
done

log_ok "gen-xsd-fragments: wrote $(find "$OUT" -name '*.md' ! -name 'README.md' | wc -l | tr -d ' ') fragment(s) to ${OUT#"$ROOT"/}"
