#!/bin/sh
# ===================================================================
# check-conventions.sh
#
# Enforces the repo's greppable source conventions across crate source.
#
# Some house conventions are cross-file consistency choices that no clippy
# lint covers: a typed None is written `None::<T>`, never `Option::<T>::None`
# or `Option::None`; an empty vector is `Vec::new()`, never empty `vec![]`
# (the `vec!` macro is reserved for element-bearing literals); a string
# literal becomes a `String` through `String::from("x")`, never literal
# `.to_string()`/`.into()` (the crate defines no `From<&str>`, so a literal
# `.into()` can only target `String`; `.to_string()` stays correct on
# non-literal receivers, which the literal-anchored patterns do not match).
# The spellings in each pair compile to the same thing, so the drift is
# silent. This gate greps crates/*/src for each anti-pattern and fails with
# the offending file:line and the canonical form to use.
#
# Scope: crates/*/src. Only greppable, semantically unambiguous conventions
# belong here. Semantic judgements (e.g. whether a test fixture may bypass a
# validating constructor) cannot be grepped and are out of scope; those live
# as documented conventions in docs/dev/practices.md.
#
# Adding a rule: append one `check_rule` call with a human name, an ERE
# anti-pattern, and the canonical form. Each call is self-contained.
#
# Exit codes:
#   0 — no anti-pattern occurs in crate source
#   1 — one or more occurrences (each reported as file:line)
#
# ===================================================================

set -u

# Source shared loggers
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

log_section "Checking crate-source conventions..."

tmp_fail=$(mktemp)
trap 'rm -f "$tmp_fail"' EXIT

# check_rule <name> <ere-anti-pattern> <canonical-form>
# Reports every crates/*/src occurrence of the anti-pattern as file:line and
# flags failure. `find … | while read` is the POSIX-portable recursion idiom
# (/bin/sh has no `globstar`); the fail flag is a file because the pipeline
# body runs in a subshell whose variable writes would not persist.
check_rule() {
    _name="$1"
    _pattern="$2"
    _canonical="$3"
    find crates -type f -name '*.rs' -path '*/src/*' -not -path '*/target/*' \
        | while IFS= read -r file; do
        grep -nE "$_pattern" "$file" | while IFS= read -r hit; do
            lineno=${hit%%:*}
            log_err_file "$file" "line ${lineno}: non-canonical ${_name}; use ${_canonical}"
            echo "1" > "$tmp_fail"
        done
    done
    unset _name _pattern _canonical
}

# No crate source is a valid state (nothing to validate).
if [ -d crates ]; then
    # Typed None: `None` / `None::<T>`, never `Option::None` or `Option::<T>::None`.
    check_rule "typed None" '(^|[^[:alnum:]_])Option::(<[^>]*>::)?None' 'None or None::<T>'

    # Empty vector: `Vec::new()`, never empty `vec![]` (reserve `vec!` for elements).
    check_rule "empty vec![]" '(^|[^[:alnum:]_])vec!\[[[:space:]]*\]' 'Vec::new()'

    # String from a literal: `String::from("x")`, never literal `.to_string()`
    # or `.into()`. The literal-anchored patterns leave non-literal receivers
    # (Display rendering such as `version.to_string()`) unmatched.
    check_rule "string-literal .to_string()" '"([^"\\]|\\.)*"\.to_string\(\)' 'String::from("...")'
    check_rule "string-literal .into()" '"([^"\\]|\\.)*"\.into\(\)' 'String::from("...")'
fi

if [ -s "$tmp_fail" ]; then
    log_fail "check-conventions: source conventions must hold (use the canonical form shown)"
    exit 1
fi

log_ok "check-conventions: all crate-source conventions hold"
