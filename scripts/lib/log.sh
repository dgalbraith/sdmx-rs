#!/bin/sh
# ==============================================================================
# scripts/lib/log.sh
#
# Single source of truth for status/log output across all repo shell scripts.
# Callers name a SEMANTIC ROLE; this library owns the glyph, label, spacing,
# colour, output stream, and GitHub Actions annotation form. Nothing else in the
# repo should emit a status glyph directly — routing all output through these
# functions makes format drift structurally impossible ("consistency by
# construction").
#
# DUAL-USE: source it (`. lib/log.sh`) to get the functions in-process, OR run it
# (`./lib/log.sh log_ok "msg"`) to dispatch a single role from a context that
# cannot cheaply source — notably Justfile recipe lines (each its own subshell).
# See the CLI dispatcher at the foot of the file.
#
# POSIX sh only — no bashisms (no `local`, `[[ ]]`, arrays, ${var,,}, $BASHPID).
#
# ------------------------------------------------------------------------------
# Roles (stdout unless noted):
#   log_ok      "msg" [lvl]   ✅ msg                  success
#   log_info    "msg" [lvl]   ℹ️  msg                  informational status/step
#   log_warn    "msg" [lvl]   ⚠️  Warning: msg         non-fatal, draws attention
#   log_section "msg" [lvl]   🔷 msg  yes                 section header
#   log_item    "msg" [lvl]   • msg                   nested sub-item
#   log_hint    "msg" [lvl]   💡 msg                  hint / suggested action
#
#   log_fail    "msg" [lvl]   ❌ msg                   advisory red CHECK RESULT
#                                                      (does not abort), stdout
#   log_err     "msg"         ❌ Error: msg            actionable error, stderr,
#                                                      returns 0 (NO exit)
#   log_fatal   "msg"         ❌ Error: msg            actionable error, stderr,
#                                                      then `exit 1`
#   log_err_ci  "msg"         actionable error w/o a file anchor; emits the
#                             file-less `::error::msg` annotation in CI
#   log_err_detail "msg" [lvl]  indented detail under an error, stderr, no glyph
#                             (lvl DEFAULT 1 — see note at the function)
#
# File-anchored diagnostics (emit GitHub Actions annotations in CI):
#   log_warn_file "file" "msg"   ::warning file=...   / pretty + cyan path local
#   log_err_file  "file" "msg"   ::error  file=...    / pretty, stderr, local
#
# ------------------------------------------------------------------------------
# Label convention: only the ACTIONABLE/SEARCHABLE severities carry a text
# label — `Error:` (log_err/log_fatal/log_*_file/log_err_ci) and `Warning:`
# (log_warn/log_warn_file) — so humans and CI automation can grep for them.
# Success/info/section/item/hint stay bare (the emoji is the signal; a label
# there is noise). NOTE: the ❌ glyph appears in TWO forms by design — bare via
# log_fail (a red check result) and labelled `❌ Error:` via log_err/log_fatal
# (an actionable error). Grepping `Error:` finds only the latter.
#
# Indentation: an optional trailing integer level indents the line by level*2
# spaces (default 0). The indent is applied before any colour codes so width
# math stays correct.
#
# log_ok has TWO distinct uses — keep them apart:
#
#   1. Per-item tick — a log_ok at level >=1, under a log_section, that reports
#      ONE fact among many (e.g. `log_ok "rustc: $version" 1`). Free-form prose;
#      the section header already names the context. Most doctor-* output is
#      this. Leave these as-is; no prefix rule applies.
#
#   2. Gate-passed summary — the FINAL, unindented (level 0) log_ok a gate script
#      emits to mean "the whole gate passed". This is what `verify` surfaces, so
#      it follows a fixed shape for scannability:
#
#          <feature>: <result>
#
#      <feature> is the USER-RECOGNISABLE name — the recipe or capability a
#      reader meets in `verify` output (e.g. `update-msrv`, `secrets-scan`), NOT
#      the internal script filename when the two differ (a reader scanning green
#      lines knows the feature, not `check-test-manifests`). When script and
#      feature coincide (secrets-scan, check-shebangs), they are the same word.
#      <result> is a STATEMENT, never a question (it is the affirmative branch).
#      Examples: `secrets-scan: no secrets found`,
#                `update-msrv: test fixture manifest up to date`.
#
# Subshell contract (log_fatal): `exit` from a function only terminates the
# CURRENT shell. If log_fatal were called inside a `$(...)`, a pipeline, or a
# `( )` subshell, the exit would kill only that subshell and the script would
# continue past a "fatal" error. There is NO portable POSIX-sh way to detect
# subshell context ($$ does not change in a subshell; $BASHPID is a bashism), so
# this cannot be guarded at runtime: DO NOT call log_fatal inside a subshell —
# use log_err there and place the `exit`/failure flag at the top level (see
# check-shebangs.sh for the pattern).
# ==============================================================================

# --- Colour capability, computed once at source time --------------------------
# Colour is enabled only for interactive local terminals: never in CI (the
# annotation form is used there), never when NO_COLOR is set, never for a dumb
# terminal, and only when the relevant stream is a TTY. Two flags because errors
# go to stderr (fd 2) and everything else to stdout (fd 1) — a redirected stdout
# must not strip colour from an still-interactive stderr, and vice versa.
_LOG_COLOR_OUT=0
_LOG_COLOR_ERR=0
if [ "${GITHUB_ACTIONS:-}" != "true" ] && [ -z "${NO_COLOR:-}" ] && [ "${TERM:-}" != "dumb" ]; then
    [ -t 1 ] && _LOG_COLOR_OUT=1
    [ -t 2 ] && _LOG_COLOR_ERR=1
fi

# ANSI codes (used only when the corresponding _LOG_COLOR_* flag is 1).
_LOG_RESET='\033[0m'
_LOG_GREEN='\033[32m'
_LOG_YELLOW='\033[1;33m'
_LOG_RED='\033[1;31m'
_LOG_BLUE='\033[34m'
_LOG_CYAN='\033[36m'
_LOG_BOLD='\033[1m'

# --- Internal helpers ---------------------------------------------------------

# _log_indent <level> — echo level*2 spaces (no newline). Non-numeric/empty → 0.
_log_indent() {
    _lvl="${1:-0}"
    case "$_lvl" in
        ''|*[!0-9]*) _lvl=0 ;;
    esac
    _n=$((_lvl * 2))
    while [ "$_n" -gt 0 ]; do
        printf ' '
        _n=$((_n - 1))
    done
    unset _lvl _n
}

# _log_out <colour-on> <colour> <glyph> <label> <msg> <level>
# Emits one line to stdout. <label> may be empty (bare role). Message is always
# printed via %s (never interpreted as a format string — SC2059 / injection).
_log_out() {
    _con="$1"; _col="$2"; _glyph="$3"; _label="$4"; _msg="$5"; _lvl="$6"
    _log_indent "$_lvl"
    if [ "$_con" = "1" ]; then
        printf "%b%s%b %s%s\n" "$_col" "$_glyph" "$_LOG_RESET" "$_label" "$_msg"
    else
        printf "%s %s%s\n" "$_glyph" "$_label" "$_msg"
    fi
    unset _con _col _glyph _label _msg _lvl
}

# _log_err_line <glyph> <label> <msg> — same as _log_out but to STDERR, no indent.
_log_err_line() {
    _glyph="$1"; _label="$2"; _msg="$3"
    if [ "$_LOG_COLOR_ERR" = "1" ]; then
        printf "%b%s%b %s%s\n" "$_LOG_RED" "$_glyph" "$_LOG_RESET" "$_label" "$_msg" >&2
    else
        printf "%s %s%s\n" "$_glyph" "$_label" "$_msg" >&2
    fi
    unset _glyph _label _msg
}

# --- Public status roles (stdout) ---------------------------------------------

log_ok() {
    _log_out "$_LOG_COLOR_OUT" "$_LOG_GREEN" "✅" "" "$1" "${2:-0}"
}

log_info() {
    # Two spaces after ℹ️ : the variation-selector glyph renders narrow in many
    # terminals; the pad aligns it with single-cell glyphs. Baked in so it
    # cannot drift.
    _log_out "$_LOG_COLOR_OUT" "$_LOG_BLUE" "ℹ️ " "" "$1" "${2:-0}"
}

log_warn() {
    # Two spaces after ⚠️  for the same alignment reason as log_info.
    _log_out "$_LOG_COLOR_OUT" "$_LOG_YELLOW" "⚠️ " "Warning: " "$1" "${2:-0}"
}

log_section() {
    _log_out "$_LOG_COLOR_OUT" "$_LOG_BOLD" "🔷" "" "$1" "${2:-0}"
}

log_item() {
    _log_out "$_LOG_COLOR_OUT" "$_LOG_CYAN" "•" "" "$1" "${2:-0}"
}

log_hint() {
    _log_out "$_LOG_COLOR_OUT" "$_LOG_BLUE" "💡" "" "$1" "${2:-0}"
}

# Advisory red CHECK RESULT — does not abort. Bare ❌, stdout (see header note on
# the two ❌ forms).
log_fail() {
    if [ "$_LOG_COLOR_OUT" = "1" ]; then
        _log_out "1" "$_LOG_RED" "❌" "" "$1" "${2:-0}"
    else
        _log_out "0" "" "❌" "" "$1" "${2:-0}"
    fi
}

# --- Public error roles (stderr) ----------------------------------------------

# Actionable error, stderr, returns 0 (does NOT exit). Safe to call inside a
# subshell/pipeline; pair with your own top-level exit/flag.
log_err() {
    _log_err_line "❌" "Error: " "$1"
    return 0
}

# Actionable error, stderr, then exit 1. DO NOT call inside a subshell (see the
# subshell contract in the header).
log_fatal() {
    _log_err_line "❌" "Error: " "$1"
    exit 1
}

# Actionable error with no file anchor. In CI emit the file-less annotation so it
# surfaces in the Actions panel; locally behave like log_err.
log_err_ci() {
    if [ "${GITHUB_ACTIONS:-}" = "true" ]; then
        printf '::error::%s\n' "$1" >&2
    else
        _log_err_line "❌" "Error: " "$1"
    fi
}

# Continuation / detail line under an actionable error — the explanatory line(s)
# that follow a log_err / log_fatal / log_err_ci. STDERR, so it stays grouped
# with the error above it; no glyph, no label, no colour (it is subordinate
# prose, not a second error); never a CI annotation (the error it follows
# already carries that, and ::…:: forms are one-per-error). Indents level*2
# spaces, DEFAULT 1 — a detail is inherently subordinate, so it indents by
# default. (This differs deliberately from log_item, which defaults to level 0:
# an item is not always nested, but a detail always is.) Override with an
# explicit level, e.g. `log_err_detail "deeper" 2` or `log_err_detail "flush" 0`.
log_err_detail() {
    _log_indent "${2:-1}" >&2
    printf '%s\n' "$1" >&2
}

# --- File-anchored diagnostics (CI annotations) -------------------------------

log_warn_file() {
    _file="$1"; _msg="$2"
    if [ "${GITHUB_ACTIONS:-}" = "true" ]; then
        printf '::warning file=%s::%s\n' "$_file" "$_msg"
    elif [ "$_LOG_COLOR_OUT" = "1" ]; then
        printf "  %b⚠️  Warning:%b %s (%b%s%b)\n" "$_LOG_YELLOW" "$_LOG_RESET" "$_msg" "$_LOG_CYAN" "$_file" "$_LOG_RESET"
    else
        printf "  ⚠️  Warning: %s (%s)\n" "$_msg" "$_file"
    fi
    unset _file _msg
}

log_err_file() {
    _file="$1"; _msg="$2"
    if [ "${GITHUB_ACTIONS:-}" = "true" ]; then
        printf '::error file=%s::%s\n' "$_file" "$_msg" >&2
    elif [ "$_LOG_COLOR_ERR" = "1" ]; then
        printf "  %b❌ Error:%b %s (%b%s%b)\n" "$_LOG_RED" "$_LOG_RESET" "$_msg" "$_LOG_CYAN" "$_file" "$_LOG_RESET" >&2
    else
        printf "  ❌ Error: %s (%s)\n" "$_msg" "$_file" >&2
    fi
    unset _file _msg
}

# --- CLI dispatcher (dual-use: sourced library OR executable) -----------------
# When SOURCED (`. lib/log.sh`), the block below is skipped and only the function
# definitions above take effect — the normal in-script use. When EXECUTED
# directly (`./lib/log.sh log_section "msg"`), it dispatches to the named role,
# so the SAME single source of truth is reachable from Justfile recipe lines —
# each its own `bash -c` subshell that cannot source the library cheaply. This
# keeps ALL status formatting in one place without re-implementing any of it in
# the Justfile.
#
# Detection is by basename of $0: sourcing does not change $0 (it stays the
# CALLING script/shell), so $0 only reads as "log.sh" when this file is the
# executed program. INVARIANT this relies on: no script that is itself invoked
# as `log.sh` sources this file. All in-repo consumers source it by path
# (`. "$(dirname "$0")/lib/log.sh"`), so their $0 is the consumer, never log.sh.
#
# Unknown or missing role → diagnostic on stderr + exit 2 (POSIX usage-error
# convention; never a silent no-op, which would let a status line vanish while a
# gate still "passes"). The allow-list is COMPLETE — every public role — so a
# direct caller can reach any of them and a typo fails loudly rather than running
# the wrong thing.
if [ "$(basename "$0" 2>/dev/null)" = "log.sh" ]; then
    case "${1:-}" in
        log_ok|log_info|log_warn|log_section|log_item|log_hint|\
        log_fail|log_err|log_fatal|log_err_ci|log_err_detail|\
        log_warn_file|log_err_file)
            _cmd="$1"
            shift
            "$_cmd" "$@"
            ;;
        "")
            printf 'Usage: log.sh <role> [args...]\n' >&2
            exit 2
            ;;
        *)
            printf 'log.sh: unknown or unsupported role: %s\n' "$1" >&2
            exit 2
            ;;
    esac
    unset _cmd
fi
