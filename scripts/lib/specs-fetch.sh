#!/bin/sh
# ==============================================================================
# scripts/lib/specs-fetch.sh
#
# Shared kernel for fetching SDMX schemas on demand. The repo tracks PINS, not
# schema files: specs/sources.toml records, per edition, the upstream git commit
# + the Nix fixed-output NAR hash, and per file a sha256. This library is the
# single place that reads those pins and turns them into a materialised,
# integrity-checked schema tree.
#
# It is SOURCED by the two thin drivers (mirroring the gen / check-xsd-fragments
# and forge-spec / doctor-forge split):
#   - scripts/fetch-specs.sh   VERIFY: materialise + sha-check (idempotent)
#   - scripts/update-specs.sh  RE-PIN: fetch a tag, record shas/hash TOFU
#
# REQUIRES scripts/lib/log.sh to be sourced FIRST (uses the log_* roles).
# POSIX sh only — no bashisms. sources.toml is parsed with awk, not a TOML
# library, so the pin file stays the single source of truth for BOTH this shell
# layer AND the Nix FOD (which reads it via builtins.fromTOML); the two must
# agree, so the format stays flat and regular (see specs/sources.toml).
#
# Overridable for tests / portability:
#   NIX       — the nix binary            (default: nix)
#   SHA256SUM — the sha256 hasher         (default: sha256sum)
#   SPECS_FLAKE — flake ref for the FOD   (default: the repo root ".")
# Inputs read from the environment (set by the driver):
#   SPECS_SOURCES  — path to specs/sources.toml (the pin file)
# ==============================================================================

# --- sources.toml parsing (awk; no TOML lib) ----------------------------------
# The schema (see specs/sources.toml):
#   [upstream]            owner/repo (+ a w3c = [..] provenance list)
#   [edition."<ed>"]      ref / rev / narHash, one table per edition
#   [files."<ed>"]        "<name>.xsd" = "<sha256>", one line per file
# Quoted dotted keys keep it valid for builtins.fromTOML while staying trivially
# awk-addressable (the edition name is field 2 when splitting a header on ").

# specs_upstream <key> — print upstream.<key> (owner | repo).
specs_upstream() {
    awk -v key="$1" '
        /^\[/ { inside = ($0 == "[upstream]") }
        inside && $0 ~ ("^" key "[ \t]*=") {
            sub(/^[^=]*=[ \t]*"/, ""); sub(/"[ \t]*$/, ""); print; exit
        }
    ' "$SPECS_SOURCES"
}

# specs_editions — print each pinned edition name (one per line), file order.
specs_editions() {
    awk -F'"' '/^\[edition\."/ { print $2 }' "$SPECS_SOURCES"
}

# specs_field <edition> <key> — print edition.<edition>.<key> (ref | rev | narHash).
specs_field() {
    awk -v ed="$1" -v key="$2" '
        /^\[/ { inside = ($0 == "[edition.\"" ed "\"]") }
        inside && $0 ~ ("^" key "[ \t]*=") {
            sub(/^[^=]*=[ \t]*"/, ""); sub(/"[ \t]*$/, ""); print; exit
        }
    ' "$SPECS_SOURCES"
}

# specs_file_names <edition> — print every file name recorded for the edition.
specs_file_names() {
    awk -v ed="$1" '
        /^\[/ { inside = ($0 == "[files.\"" ed "\"]") }
        inside && match($0, /^"[^"]+"/) { print substr($0, 2, RLENGTH - 2) }
    ' "$SPECS_SOURCES"
}

# specs_file_sha <edition> <name> — print the recorded sha256 for one file.
specs_file_sha() {
    awk -v ed="$1" -v name="$2" '
        /^\[/ { inside = ($0 == "[files.\"" ed "\"]") }
        inside && index($0, "\"" name "\"") == 1 {
            sub(/^[^=]*=[ \t]*"/, ""); sub(/"[ \t]*$/, ""); print; exit
        }
    ' "$SPECS_SOURCES"
}

# specs_blob_url <edition> <name> — derive the canonical upstream blob URL for a
# file (no #L anchor; the per-symbol anchor is appended by the register rewrite).
# Derived from the pinned commit so a re-pin moves every link by editing one rev.
specs_blob_url() {
    _su_owner=$(specs_upstream owner)
    _su_repo=$(specs_upstream repo)
    _su_rev=$(specs_field "$1" rev)
    printf 'https://github.com/%s/%s/blob/%s/schemas/%s' \
        "$_su_owner" "$_su_repo" "$_su_rev" "$2"
    unset _su_owner _su_repo _su_rev
}

# --- symbol line spans (for the register-link #L anchors, B5) ------------------
# specs_symbol_span <file> <symbol> — print "START END": the 1-based line span of
# the named xs:complexType / xs:simpleType, depth-aware so a nested anonymous
# complexType does not close it early (the same boundary logic as the generator's
# slice(), here recording absolute line numbers instead of emitting the body).
# Line numbers count \n-delimited records, matching GitHub's blob line numbering.
#
# Two open-tag subtleties the start-match and depth accounting handle:
#   - Attribute order: the name= may not be the first attribute. SDMXStructure-
#     Dataflow.xsd declares <xs:complexType abstract="true" name="DataflowBaseType">,
#     so the match allows [^>]* (any attributes, but not past the tag's own '>')
#     before name=.
#   - Self-closing tags: selfcloses() nets out a same-name <xs:complexType …/>.
#     opens() counts its "<xs:complexType " as +1 but it has no separate close, so
#     without this a self-closing NAMED type (start == end) never reaches depth 0
#     and a nested self-closing anonymous one would close the span late.
# (The generator's slice() shares this boundary logic; the SDMX schemas carry no
# self-closing named types, so that case is dormant there.)
specs_symbol_span() {
    awk -v name="$2" '
        function opens(s,  t) { t = s; return gsub("<xs:" TAG "[ />]", "X", t) }
        function closes(s,  t) { t = s; return gsub("</xs:" TAG ">", "X", t) }
        function selfcloses(s,  t) { t = s; return gsub("<xs:" TAG "[^>]*/>", "X", t) }
        BEGIN { started = 0; depth = 0; start = 0 }
        !started && $0 ~ ("<xs:complexType[^>]*name=\"" name "\"") { TAG = "complexType"; started = 1; start = NR }
        !started && $0 ~ ("<xs:simpleType[^>]*name=\"" name "\"")  { TAG = "simpleType";  started = 1; start = NR }
        started { depth += opens($0) - closes($0) - selfcloses($0); if (depth <= 0) { print start, NR; exit } }
    ' "$1"
}

# --- direct fetch via the flake's pinned nixpkgs (re-pin, pre-commit) ----------
# update-specs.sh captures a pin BEFORE sources.toml is committed, but a Nix
# flake can only read git-tracked files, so the FOD (.#sdmxSpecs, used by
# fetch-specs.sh) cannot read an as-yet-untracked pin. These helpers build the
# SAME fetchFromGitHub derivation directly through the flake's already-pinned
# nixpkgs, so the re-pin path needs no git staging and works on a fresh checkout.
# The output is content-addressed by its NAR hash, hence byte-identical to what
# .#sdmxSpecs later produces from the committed pin.
# shellcheck disable=SC2016  # ${builtins.currentSystem} is a Nix antiquotation, not shell
_specs_ffgh_expr() { # owner repo rev hash
    printf '(builtins.getFlake "%s").inputs.nixpkgs.legacyPackages.${builtins.currentSystem}.fetchFromGitHub { owner = "%s"; repo = "%s"; rev = "%s"; hash = "%s"; }' \
        "${SPECS_FLAKE:-.}" "$1" "$2" "$3" "$4"
}

# specs_capture_hash <owner> <repo> <rev> — build with a placeholder hash and
# print the real NAR hash parsed from the resulting mismatch (trust-on-first-use).
specs_capture_hash() {
    _ch_log=$(mktemp)
    if "${NIX:-nix}" build --impure --no-link --print-out-paths --no-warn-dirty \
            --expr "$(_specs_ffgh_expr "$1" "$2" "$3" "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")" \
            > "$_ch_log" 2>&1; then
        log_err "specs: expected a hash mismatch but the placeholder build succeeded"
        rm -f "$_ch_log"; return 1
    fi
    _ch_got=$(sed -n 's/.*got:[[:space:]]*\(sha256-[A-Za-z0-9+/=]*\).*/\1/p' "$_ch_log" | head -1)
    if [ -z "$_ch_got" ]; then
        log_err "specs: could not parse the NAR hash from the build output:"
        sed 's/^/    /' "$_ch_log" >&2
        rm -f "$_ch_log"; return 1
    fi
    rm -f "$_ch_log"
    printf '%s\n' "$_ch_got"
    unset _ch_log _ch_got
}

# specs_fetch_path <owner> <repo> <rev> <hash> — build the pinned tree and print
# its /nix/store path (the whole repo tree; schemas/ is a subdirectory).
specs_fetch_path() {
    "${NIX:-nix}" build --impure --no-link --print-out-paths --no-warn-dirty \
        --expr "$(_specs_ffgh_expr "$1" "$2" "$3" "$4")"
}

# --- materialise + verify -----------------------------------------------------
# specs_build <destdir> — build the Nix FOD (.#sdmxSpecs) and copy the schema
# tree into <destdir> (so <destdir>/<ed>/schemas/*.xsd mirrors the specs/ layout).
# The store output is read-only; the copy is made writable so a later re-fetch
# can overwrite it. Hermetic and content-addressed: the per-file sha re-check in
# specs_verify is an independent gate layered on top of the NAR-hash integrity.
specs_build() {
    _sb_dest="$1"
    _sb_path=$("${NIX:-nix}" build "${SPECS_FLAKE:-.}#sdmxSpecs" \
        --no-link --print-out-paths --no-warn-dirty) || return 1
    mkdir -p "$_sb_dest"
    # cp -RL: dereference (the FOD output may symlink into the store).
    cp -RL "$_sb_path"/. "$_sb_dest"/
    chmod -R u+w "$_sb_dest"
    unset _sb_dest _sb_path
}

# specs_verify <dir> — re-hash every pinned file under <dir>/<ed>/schemas and
# compare against sources.toml. Returns 0 if all match, 1 otherwise (reporting
# each mismatch). This is the transport-independent content gate: it holds in
# steady state (no prior tree) exactly as at a first pin.
specs_verify() {
    _sv_dir="$1"
    _sv_fail=0
    for _sv_ed in $(specs_editions); do
        for _sv_name in $(specs_file_names "$_sv_ed"); do
            _sv_want=$(specs_file_sha "$_sv_ed" "$_sv_name")
            _sv_file="$_sv_dir/$_sv_ed/schemas/$_sv_name"
            if [ ! -f "$_sv_file" ]; then
                log_err "specs-verify: missing $_sv_ed/schemas/$_sv_name"
                _sv_fail=1
                continue
            fi
            _sv_got=$("${SHA256SUM:-sha256sum}" "$_sv_file" | cut -d' ' -f1)
            if [ "$_sv_got" != "$_sv_want" ]; then
                log_err "specs-verify: sha256 mismatch $_sv_ed/schemas/$_sv_name"
                log_err_detail "want $_sv_want"
                log_err_detail "got  $_sv_got"
                _sv_fail=1
            fi
        done
    done
    unset _sv_dir _sv_ed _sv_name _sv_want _sv_file _sv_got
    return "$_sv_fail"
}

# specs_present <dir> — true if every pinned file exists under <dir>/<ed>/schemas
# (existence only, no hashing). A cheap guard so fetch-specs.sh's stamp fast path
# never trusts a stamp whose materialised tree was removed. A rebase across the
# untrack commit deletes the once-tracked .xsd while the gitignored .sha256.stamp
# survives; without this check that stale stamp would wedge a false no-op.
specs_present() {
    _sp_dir="$1"
    for _sp_ed in $(specs_editions); do
        for _sp_name in $(specs_file_names "$_sp_ed"); do
            [ -f "$_sp_dir/$_sp_ed/schemas/$_sp_name" ] || {
                unset _sp_dir _sp_ed _sp_name
                return 1
            }
        done
    done
    unset _sp_dir _sp_ed _sp_name
    return 0
}

# specs_stamp_value — a fingerprint of the current pin (all per-file shas), so a
# re-pin invalidates a previously-materialised tree. Written to .sha256.stamp by
# fetch-specs.sh for its idempotent no-op-when-present check.
specs_stamp_value() {
    for _ss_ed in $(specs_editions); do
        for _ss_name in $(specs_file_names "$_ss_ed"); do
            printf '%s/%s %s\n' "$_ss_ed" "$_ss_name" "$(specs_file_sha "$_ss_ed" "$_ss_name")"
        done
    done | "${SHA256SUM:-sha256sum}" | cut -d' ' -f1
    unset _ss_ed _ss_name
}
