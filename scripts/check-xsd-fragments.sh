#!/bin/sh
# ===================================================================
# check-xsd-fragments.sh
#
# The read-only doctor for the XSD contract fragments (the "verify" half
# of the spec -> doctor -> apply kernel). Runs two checks and never
# mutates the working tree:
#
#   1. Freshness: regenerate the fragments into a temp dir and diff
#      against the committed crates/sdmx-types/docs/xsd-fragments/. A
#      difference means the committed fragments are stale (a re-vendored
#      schema, a manifest edit, or a hand-edit). Run gen-xsd-fragments.
#
#   2. Cross-check: for every [[fragment]] in xsd-manifest.toml, each
#      listed rust item's ## Specification must cite the symbol, and the
#      matching include_str!(...) line(s) must be wired into that item.
#      Also flags orphan includes pointing at no generated fragment.
#
# Safe to run anytime; CI runs the same command as a change-gated gate.
# ===================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
MANIFEST="$ROOT/crates/sdmx-types/xsd-manifest.toml"
SRC="$ROOT/crates/sdmx-types/src"
OUT="$ROOT/crates/sdmx-types/docs/xsd-fragments"
GEN="$ROOT/scripts/gen-xsd-fragments.sh"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
MARK="$TMP/failed"   # touched by err(); survives the pipe-to-while subshells

err() { log_err "$*"; : >"$MARK"; }

log_section "Checking XSD contract fragments are fresh and correctly wired..."

# --- 1. Freshness: non-mutating regenerate-and-diff ----------------------------
XSD_FRAGMENTS_OUT="$TMP/gen" sh "$GEN" >/dev/null
# -x: README.md and .markdownlint.yaml are authored, not generated, so they are
# excluded from the freshness diff.
if ! diff -r -x README.md -x .markdownlint.yaml "$OUT" "$TMP/gen" >"$TMP/freshness.diff" 2>&1; then
  err "committed fragments are STALE; run 'just gen-xsd-fragments'. Diff:"
  sed 's/^/    /' "$TMP/freshness.diff" >&2
fi

# --- manifest parse (symbol  "rust rust") --------------------------------------
parse_manifest() {
  awk '
    function val(  s){ s=$0; sub(/^[^=]*=[ \t]*/,"",s); gsub(/"/,"",s); sub(/[ \t]+$/,"",s); return s }
    function arr(  s){ s=$0; sub(/^[^=]*=[ \t]*/,"",s); gsub(/[]["]/,"",s); gsub(/[ \t]/,"",s); gsub(/,/," ",s); return s }
    /^\[\[fragment\]\]/ { if (sym!="") print sym"\t"rust; sym="";rust="" }
    /^symbol[ \t]*=/ { sym=val() }
    /^rust[ \t]*=/   { rust=arr() }
    END { if (sym!="") print sym"\t"rust }
  ' "$MANIFEST"
}

# Item-scoped facts: print "TYPE<tab>symbol" / "INC<tab>filename" found in the doc
# block of `target` (lines since the previous pub item belong to the item they
# precede, so a flush on the `pub ... target` line carries that item's citations).
item_facts() { # file target
  awk -v target="$2" '
    /^pub (struct|trait|enum) / {
      name=$3; sub(/[^A-Za-z0-9_].*/,"",name)
      if (name==target) { for (t in types) print "TYPE\t"t; for (i in incs) print "INC\t"i }
      delete types; delete incs; next
    }
    /\*\*Type\*\*:/ { l=$0; while (match(l,/`[^`]+`/)) { types[substr(l,RSTART+1,RLENGTH-2)]=1; l=substr(l,RSTART+RLENGTH) } }
    /include_str!\("\.\.\/docs\/xsd-fragments\// {
      if (match($0,/xsd-fragments\/[^"]+\.md/)) incs[substr($0,RSTART+14,RLENGTH-14)]=1
    }
  ' "$1"
}

# --- 2. Cross-check ------------------------------------------------------------
TAB=$(printf '\t')
parse_manifest | while IFS="$TAB" read -r symbol rust; do
  for item in $rust; do
    f=$(grep -lE "^pub (struct|trait|enum) ${item}([^A-Za-z0-9_]|\$)" "$SRC"/*.rs 2>/dev/null | head -1 || true)
    if [ -z "$f" ]; then err "manifest item '$item' (symbol $symbol) not found in src"; continue; fi

    facts=$(item_facts "$f" "$item")
    cites=$(printf '%s\n' "$facts" | awk -F"$TAB" '$1=="TYPE"{print $2}')
    incs=$(printf '%s\n' "$facts"  | awk -F"$TAB" '$1=="INC"{print $2}')

    # (a) Specification cites the symbol.
    printf '%s\n' "$cites" | grep -qx "$symbol" \
      || err "$item: ## Specification does not cite '$symbol' (manifest/Specification drift)"

    # (b) Every generated fragment file for this symbol is included by the item.
    for frag in "$OUT/$symbol".*; do
      base=${frag##*/}
      [ "$base" = "$symbol.*" ] && continue   # literal: no glob match (freshness covers absence)
      printf '%s\n' "$incs" | grep -qx "$base" \
        || err "$item: missing include_str! for fragment '$base'"
    done
  done
done

# --- orphan includes: every include in src must point at a real fragment -------
grep -rhoE 'xsd-fragments/[^"]+\.md' "$SRC"/*.rs 2>/dev/null | sed 's#xsd-fragments/##' | sort -u | while read -r base; do
  [ -f "$OUT/$base" ] || err "orphan include_str! 'xsd-fragments/$base': no such generated fragment"
done

if [ -e "$MARK" ]; then
    log_fail "check-xsd-fragments: fragments are stale or incorrectly wired"
    exit 1
fi
log_ok "check-xsd-fragments: fragments are fresh and correctly wired"
