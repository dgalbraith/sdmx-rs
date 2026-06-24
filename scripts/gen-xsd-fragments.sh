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
# the arbiter) and with each line's leading/trailing whitespace stripped
# (indentation is formatting, not structure), so a fragment splits into
# <symbol>.<edition>.md only on a STRUCTURAL divergence (an element/
# attribute difference), never on a documentation-prose or indentation
# difference. xs:appinfo is left in the comparison (it can be material).
# The emitted fragment is always byte-verbatim.
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
# Schema tree root; overridable (SDMX_SPECS_DIR, shared with fetch-specs.sh) so a
# materialised fetch-on-demand tree can be sliced instead of the in-tree specs/.
SPECS="${SDMX_SPECS_DIR:-$ROOT/specs}"
OUT="$ROOT/crates/sdmx-types/docs/xsd-fragments"

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

# Reduce stdin to the structural comparison form: blank out <xs:documentation>
# narrative (prose, not contract; xs:appinfo is left intact, it can carry
# machine-read hints) and strip each line's leading/trailing whitespace, so
# divergence tracks element/attribute structure, not the editions' indentation
# (issue #70). Applied only to the comparison; the emitted fragment stays
# byte-verbatim.
structural_form() {
  sed 's#<xs:documentation>.*</xs:documentation>##g; s/^[[:space:]]*//; s/[[:space:]]*$//'
}

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
    structural_form <"$WORK/$ed.xml" >"$WORK/$ed.cmp"
    if [ -z "$first" ]; then first="$ed"
    elif ! cmp -s "$WORK/$first.cmp" "$WORK/$ed.cmp"; then divergent=1; fi
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
