#!/bin/sh
# ===================================================================
# check-decision-refs.sh
#
# Validates that every decision-register reference (D-NNNN) appearing in
# crate source resolves to an entry in docs/decisions.md.
#
# Internal documentation (the `design_docs` layer) and ordinary comments
# cite decisions by id, e.g. `Decisions: D-0027`. A reference to a decision
# that does not exist in the register — a typo, or a not-yet-created entry
# such as a pending D-NNNN — is a dangling reference that points the reader
# at nothing. This check fails on any such reference.
#
# Scope: every D-NNNN token in crates/**/*.rs. This is a superset of the
# design_docs blocks (it also catches a stray reference in an ordinary `//`
# comment), and it is simpler and stricter than parsing cfg_attr regions.
# docs/decisions.md is the source of truth: a reference resolves iff its id
# appears there, since every entry registers its id in the index row and its
# heading anchor.
#
# Exit codes:
#   0 — every crate-source D-NNNN reference resolves
#   1 — one or more dangling references (or the register is missing)
#
# ===================================================================

set -u

# Source shared loggers
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

REGISTER="docs/decisions.md"

log_section "Checking crate-source decision references resolve to ${REGISTER}..."

if [ ! -f "$REGISTER" ]; then
    log_fail "check-decision-refs: decision register not found at ${REGISTER}"
    exit 1
fi

registered=$(mktemp)
tmp_fail=$(mktemp)
trap 'rm -f "$registered" "$tmp_fail"' EXIT

# The set of registered decision ids, one D-NNNN per line. Every entry records
# its id in the register (index row and heading anchor), so any occurrence
# there marks the id as defined.
grep -oE 'D-[0-9]{4}' "$REGISTER" | sort -u > "$registered"

# No crate source is a valid state (nothing to validate). `find … | while read`
# is the POSIX-portable recursion idiom; /bin/sh (dash) has no `globstar`.
if [ -d crates ]; then
    find crates -name '*.rs' -not -path '*/target/*' | while IFS= read -r file; do
        # Each matching line is line-numbered and may carry several refs
        # (e.g. "D-0048/D-0049/D-0051"); extract and check every token.
        grep -nE 'D-[0-9]{4}' "$file" | while IFS= read -r hit; do
            lineno=${hit%%:*}
            echo "$hit" | grep -oE 'D-[0-9]{4}' | sort -u | while IFS= read -r ref; do
                if ! grep -qxF "$ref" "$registered"; then
                    log_err_file "$file" "line ${lineno}: dangling decision reference ${ref} (not in ${REGISTER})"
                    echo "1" > "$tmp_fail"
                fi
            done
        done
    done
fi

if [ -s "$tmp_fail" ]; then
    log_fail "check-decision-refs: dangling decision references must resolve to ${REGISTER} (a typo, or a not-yet-created entry)"
    exit 1
fi

log_ok "check-decision-refs: all crate-source decision references resolve"
