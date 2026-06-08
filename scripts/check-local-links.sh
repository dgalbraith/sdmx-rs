#!/bin/sh
# ===================================================================
# check-local-links.sh
#
# Validates that committed sources contain no absolute `file://` links.
# Such links (e.g. file:///home/user/project/crates/x/Cargo.toml) are
# machine-specific: they resolve only on the author's filesystem and are
# non-portable for every other contributor and for CI checkouts.
#
# Scanned file types and why each matters:
#   *.md    — Markdown links; the common leak vector (IDE "copy/drag as
#             link" emits a file:// URL).
#   *.toml  — manifest URL fields (documentation/homepage/repository) are
#             PUBLISHED to crates.io, so a file:// here ships your local
#             path to the world.
#   *.rs    — rustdoc links in doc comments render on docs.rs.
#
# Neither lychee's offline check nor `cargo doc -D warnings` catches these:
# an absolute file:// target that happens to exist on the checking machine
# is treated as valid. This scheme-level ban is the deterministic guard.
#
# Exit codes:
#   0 — no absolute file:// links found
#   1 — one or more absolute file:// links found
#
# ===================================================================

set -u

# Source shared configuration and loggers
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"


tmp_fail=$(mktemp)
trap 'rm -f "$tmp_fail"' EXIT

log_section "Checking sources for absolute file:// links..."

# Recurse the whole tree (excluding generated/vendored dirs) so a newly
# added nested directory cannot silently bypass the check. `find … | while
# read` is the POSIX-portable recursion idiom here — /bin/sh (dash) has no
# `globstar`. Template files are excluded to match the md-link-check recipe.
find . \
    \( -name '*.md' -o -name '*.toml' -o -name '*.rs' \) \
    -not -path '*/templates/*' \
    -not -path '*/target/*' \
    -not -path '*/.direnv/*' \
    -not -path '*/.git/*' \
    -not -path '*/node_modules/*' \
    | while IFS= read -r file; do
    # Match an ABSOLUTE file:// URL — the leak form an IDE emits, i.e.
    # file:/// (scheme + empty host + absolute path). Requiring the third
    # slash deliberately excludes bare prose mentions of "file://" (e.g. this
    # check's own documentation) which are not links and not a hazard.
    # One grep per file; each hit is line-numbered (grep -n) for the report.
    hits=$(grep -nE 'file:///' "$file") || continue
    echo "$hits" | while IFS= read -r hit; do
        log_err_file "$file" "Absolute file:// link (non-portable): $hit"
    done
    echo "1" > "$tmp_fail"
done

if [ -s "$tmp_fail" ]; then
    log_fail "check-local-links: absolute file:// links must be replaced with repo-relative paths"
    exit 1
fi

log_ok "check-local-links: no absolute file:// links in sources"
