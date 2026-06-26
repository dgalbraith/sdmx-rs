#!/bin/sh
# ==============================================================================
# update-specs.sh
#
# RE-PIN driver for the fetch-on-demand SDMX schema (the mutating sibling of
# the read-only fetch-specs.sh, mirroring the gen / check-xsd-fragments and
# forge-spec / doctor-forge split — a verify run can never accidentally re-pin).
#
# Pins a new or changed SDMX edition into specs/sources.toml, trust-on-first-use,
# from the upstream commit:
#   1. resolve the ref (tag) to its full 40-char commit SHA;
#   2. capture the Nix FOD NAR hash for that commit (TOFU, via a placeholder
#      build whose hash-mismatch reports the real one);
#   3. materialise the schema tree and record each file's sha256 (the durable,
#      transport-independent content gate).
#
# Usage:
#   update-specs.sh <edition> <ref>     e.g.  update-specs.sh 3.0 v3.0.0
#
# This is also the initial-pin path (a first edition creates sources.toml). The
# whole file is re-emitted deterministically each run so the pin file stays
# canonical regardless of edit history.
#
# Re-pinning does NOT touch the decision register: anchor (#L) maintenance is a
# separate, reviewed step — see the post-run checklist and specs/README.md.
#
# Overridable for tests: GIT, NIX, SHA256SUM (see lib/specs-fetch.sh).
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/specs-fetch.sh"

ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
SPECS_SOURCES="${SPECS_SOURCES:-$ROOT/specs/sources.toml}"
SPECS_FLAKE="${SPECS_FLAKE:-$ROOT}"
export SPECS_SOURCES SPECS_FLAKE

# Defaults used only when bootstrapping a brand-new sources.toml.
DEFAULT_OWNER="sdmx-twg"
DEFAULT_REPO="sdmx-ml"
# xml.xsd is the W3C XML-namespace schema bundled verbatim in the SDMX tree.
W3C_FILES='["xml.xsd"]'
# Standard "I don't know the hash yet" placeholder; its mismatch reveals the real
# NAR hash on the first build (Nix's lib.fakeHash).
FAKE_HASH="sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="

if [ "$#" -ne 2 ]; then
    log_err "usage: update-specs.sh <edition> <ref>   (e.g. update-specs.sh 3.0 v3.0.0)"
    exit 2
fi
TARGET_ED="$1"
TARGET_REF="$2"

WORK=$(mktemp -d)
# shellcheck disable=SC2064
trap "rm -rf '$WORK'" EXIT

log_section "Re-pinning SDMX schemas: edition $TARGET_ED -> $TARGET_REF"

# --- 1. gather existing pin state (everything except the target edition) -------
OWNER="$DEFAULT_OWNER"
REPO="$DEFAULT_REPO"
EDITIONS=""
if [ -f "$SPECS_SOURCES" ]; then
    OWNER=$(specs_upstream owner); OWNER="${OWNER:-$DEFAULT_OWNER}"
    REPO=$(specs_upstream repo);   REPO="${REPO:-$DEFAULT_REPO}"
    for e in $(specs_editions); do
        [ "$e" = "$TARGET_ED" ] && continue
        EDITIONS="$EDITIONS $e"
        specs_field "$e" ref     > "$WORK/$e.ref"
        specs_field "$e" rev     > "$WORK/$e.rev"
        specs_field "$e" narHash > "$WORK/$e.narHash"
        : > "$WORK/$e.files"
        for n in $(specs_file_names "$e"); do
            printf '%s %s\n' "$n" "$(specs_file_sha "$e" "$n")" >> "$WORK/$e.files"
        done
    done
fi
REPO_URL="https://github.com/$OWNER/$REPO"

# --- 2. resolve the ref to a full 40-char commit SHA ---------------------------
# Dereference an annotated tag (^{}) to its commit; fall back to a lightweight
# tag (which already points straight at the commit).
log_info "Resolving $TARGET_REF in $REPO_URL ..."
REV=$("${GIT:-git}" ls-remote "$REPO_URL" "refs/tags/$TARGET_REF^{}" | cut -f1)
[ -n "$REV" ] || REV=$("${GIT:-git}" ls-remote "$REPO_URL" "refs/tags/$TARGET_REF" | cut -f1)
if [ -z "$REV" ]; then
    log_err "could not resolve ref '$TARGET_REF' to a commit in $REPO_URL"
    exit 1
fi
case "$REV" in
    [0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]) ;;
    *) log_err "resolved rev is not a 40-char commit SHA: $REV"; exit 1 ;;
esac
log_ok "$TARGET_REF -> $REV" 1

# Record the target edition's ref/rev; hash + files filled in below.
printf '%s\n' "$TARGET_REF" > "$WORK/$TARGET_ED.ref"
printf '%s\n' "$REV"        > "$WORK/$TARGET_ED.rev"
printf '%s\n' "$FAKE_HASH"  > "$WORK/$TARGET_ED.narHash"
: > "$WORK/$TARGET_ED.files"
# Word-splitting of the space-separated edition list is intentional here.
# shellcheck disable=SC2086
ALL_EDITIONS=$(printf '%s\n' $EDITIONS "$TARGET_ED" | sort -u)

# Drop trailing blank line(s) so the emitted file is taplo-canonical: toml-check
# rejects a blank line at EOF. Internal blank lines between tables are preserved.
_drop_trailing_blank_lines() {
    awk 'NF { last = NR } { line[NR] = $0 } END { for (i = 1; i <= last; i++) print line[i] }'
}

# --- emit the whole pin file from $WORK state ----------------------------------
emit_sources() {
    {
        printf '# specs/sources.toml: pinned SDMX schema sources.\n'
        printf '#\n'
        printf '# The repo tracks PINS, never the xsd schema files. This file is the single\n'
        printf '# source of truth for BOTH:\n'
        printf '#   - the Nix fixed-output derivation in flake.nix (per-edition commit + NAR\n'
        printf '#     hash), read via builtins.fromTOML; and\n'
        printf '#   - the shell verify gate (scripts/fetch-specs.sh), which re-checks each\n'
        printf '#     file sha256 after materialising (a transport-independent content gate).\n'
        printf '#\n'
        printf '# Records are trust-on-first-use, captured by scripts/update-specs.sh at the\n'
        printf '# pinned commit. Blob URLs are derived as\n'
        printf '#   https://github.com/<owner>/<repo>/blob/<rev>/schemas/<file>\n'
        printf '# (+ a per-symbol #L anchor in docs/decisions.md). GENERATED, re-pin with\n'
        printf "# 'just update-specs <edition> <ref>'; see specs/README.md for the procedure.\n"
        printf '\n'
        printf '[upstream]\n'
        printf 'owner = "%s"\n' "$OWNER"
        printf 'repo = "%s"\n' "$REPO"
        printf 'w3c = %s\n' "$W3C_FILES"
        printf '\n'
        for e in $ALL_EDITIONS; do
            printf '[edition."%s"]\n' "$e"
            printf 'ref = "%s"\n' "$(cat "$WORK/$e.ref")"
            printf 'rev = "%s"\n' "$(cat "$WORK/$e.rev")"
            printf 'narHash = "%s"\n' "$(cat "$WORK/$e.narHash")"
            printf '\n'
        done
        for e in $ALL_EDITIONS; do
            printf '[files."%s"]\n' "$e"
            while read -r _name _sha; do
                printf '"%s" = "%s"\n' "$_name" "$_sha"
            done < "$WORK/$e.files"
            printf '\n'
        done
    } | _drop_trailing_blank_lines > "$SPECS_SOURCES"
}

# --- 3. capture the NAR hash TOFU (placeholder build -> hash mismatch) ----------
# Fetched directly through the flake's pinned nixpkgs (not .#sdmxSpecs), so the
# capture needs no git-tracked sources.toml (a flake reads only tracked files).
log_info "Capturing NAR hash for $TARGET_ED (trust-on-first-use) ..."
NAR=$(specs_capture_hash "$OWNER" "$REPO" "$REV") || exit 1
printf '%s\n' "$NAR" > "$WORK/$TARGET_ED.narHash"
log_ok "NAR hash: $NAR" 1

# --- 4. materialise + record per-file sha256 -----------------------------------
log_info "Materialising and recording per-file sha256 ..."
TREE=$(specs_fetch_path "$OWNER" "$REPO" "$REV" "$NAR") \
    || { log_err "failed to materialise $TARGET_ED from $REV"; exit 1; }
: > "$WORK/$TARGET_ED.files"
for f in "$TREE/schemas/"*.xsd; do
    _n=$(basename "$f")
    _s=$("${SHA256SUM:-sha256sum}" "$f" | cut -d' ' -f1)
    printf '%s %s\n' "$_n" "$_s" >> "$WORK/$TARGET_ED.files"
done
sort -o "$WORK/$TARGET_ED.files" "$WORK/$TARGET_ED.files"

# --- 5. emit the complete pin file --------------------------------------------
mkdir -p "$(dirname "$SPECS_SOURCES")"
emit_sources

_count=$(wc -l < "$WORK/$TARGET_ED.files" | tr -d ' ')
log_ok "update-specs: pinned $TARGET_ED to $REV ($_count files) in ${SPECS_SOURCES#"$ROOT"/}"
log_hint "Next steps (reviewed manually, not automated):"
log_hint "  - re-materialise: just fetch-specs"
log_hint "  - regenerate fragments: just gen-xsd-fragments"
log_hint "  - recompute docs/decisions.md #L anchors (they rot if upstream reflows a file) — see specs/README.md"
