#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/lib/log.sh
#
# The logging library is the single source of truth for status output across all
# repo scripts. These tests pin the per-role contract: glyph + label, output
# stream (stdout vs stderr), exit/return behaviour, indentation, and the
# GitHub Actions annotation forms emitted in CI. If any of these drift, a
# migrated script's output (and the bats assertions that depend on it) breaks.
#
# Colour: _LOG_COLOR_{OUT,ERR} are computed at SOURCE time from `[ -t N ]`. Under
# bats, fds 1/2 are not TTYs, so colour is off and output is plain — which is what
# we assert against. Env-dependent paths (GITHUB_ACTIONS, NO_COLOR) are exercised
# by exporting before sourcing inside a subshell via `run sh -c`.
#
# Run with: bats tests/bats/log-lib.bats
# ==============================================================================

LIB="scripts/lib/log.sh"

setup() {
    # shellcheck disable=SC1090
    source "$LIB"
}

# --- Status roles: glyph, label, and stdout stream ----------------------------

@test "log_ok: bare ✅, no label, stdout" {
    run log_ok "all good"
    [ "$status" -eq 0 ]
    [ "$output" = "✅ all good" ]
}

@test "log_info: bare ℹ️ with two-space pad, no label" {
    run log_info "a step"
    [ "$output" = "ℹ️  a step" ]
}

@test "log_warn: ⚠️ carries 'Warning:' label with two-space pad" {
    run log_warn "heads up"
    [ "$output" = "⚠️  Warning: heads up" ]
}

@test "log_section: bare 🔷 header" {
    run log_section "My Section"
    [ "$output" = "🔷 My Section" ]
}

@test "log_item: bare • sub-item" {
    run log_item "child"
    [ "$output" = "• child" ]
}

@test "log_hint: bare 💡" {
    run log_hint "try this"
    [ "$output" = "💡 try this" ]
}

# --- Indentation --------------------------------------------------------------

@test "indent level 1 prepends two spaces" {
    run log_ok "nested" 1
    [ "$output" = "  ✅ nested" ]
}

@test "indent level 2 prepends four spaces" {
    run log_item "deep" 2
    [ "$output" = "    • deep" ]
}

@test "indent: non-numeric level is treated as 0 (no crash)" {
    run log_ok "msg" "bogus"
    [ "$output" = "✅ msg" ]
}

# --- The two ❌ forms ---------------------------------------------------------

@test "log_fail: bare ❌, NO label, on STDOUT, returns 0" {
    run log_fail "check failed"
    [ "$status" -eq 0 ]
    [ "$output" = "❌ check failed" ]
}

@test "log_fail writes to stdout (not stderr)" {
    # Capture only stdout; stderr discarded. Output must survive.
    result=$(log_fail "on stdout" 2>/dev/null)
    [ "$result" = "❌ on stdout" ]
}

@test "log_err: ❌ Error: label, on STDERR, returns 0 (no exit)" {
    run log_err "boom"
    [ "$status" -eq 0 ]
    [ "$output" = "❌ Error: boom" ]
}

@test "log_err writes to stderr (not stdout)" {
    # Capture only stdout; the error must NOT appear there.
    result=$(log_err "to stderr" 2>/dev/null)
    [ -z "$result" ]
}

@test "log_err returns 0 so the next statement still runs" {
    run sh -c '. '"$LIB"'; log_err "x"; echo reached'
    [ "$status" -eq 0 ]
    [[ "$output" == *"reached"* ]]
}

# --- log_fatal ----------------------------------------------------------------

@test "log_fatal: ❌ Error: on stderr, then exits 1" {
    run sh -c '. '"$LIB"'; log_fatal "fatal boom"; echo NOT_REACHED'
    [ "$status" -eq 1 ]
    [[ "$output" == *"❌ Error: fatal boom"* ]]
    [[ "$output" != *"NOT_REACHED"* ]]
}

@test "log_fatal error text goes to stderr, not stdout" {
    run sh -c '. '"$LIB"'; log_fatal "err only" 2>/dev/null'
    [ "$status" -eq 1 ]
    [ -z "$output" ]
}

# --- CI annotation forms (GITHUB_ACTIONS=true) --------------------------------

@test "log_err_ci: emits ::error:: in CI" {
    run sh -c 'GITHUB_ACTIONS=true; export GITHUB_ACTIONS; . '"$LIB"'; log_err_ci "ci err" 2>&1'
    [[ "$output" == *"::error::ci err"* ]]
}

@test "log_err_ci: falls back to ❌ Error: locally" {
    run sh -c 'unset GITHUB_ACTIONS; . '"$LIB"'; log_err_ci "local err" 2>&1'
    [[ "$output" == *"❌ Error: local err"* ]]
    [[ "$output" != *"::error::"* ]]
}

@test "log_warn_file: ::warning file= in CI" {
    run sh -c 'GITHUB_ACTIONS=true; export GITHUB_ACTIONS; . '"$LIB"'; log_warn_file "a/b.toml" "watch out"'
    [ "$output" = "::warning file=a/b.toml::watch out" ]
}

@test "log_err_file: ::error file= in CI, on stderr" {
    run sh -c 'GITHUB_ACTIONS=true; export GITHUB_ACTIONS; . '"$LIB"'; log_err_file "a/b.toml" "bad" 2>&1'
    [ "$output" = "::error file=a/b.toml::bad" ]
}

@test "log_warn_file: pretty local form includes path and message" {
    run sh -c 'unset GITHUB_ACTIONS; . '"$LIB"'; log_warn_file "a/b.toml" "watch out"'
    [[ "$output" == *"Warning:"* ]]
    [[ "$output" == *"watch out"* ]]
    [[ "$output" == *"a/b.toml"* ]]
    [[ "$output" != *"::warning"* ]]
}

# --- log_err_detail: continuation line under an error -------------------------
# Output is on stderr, so the indent assertions merge 2>&1 to capture it.

@test "log_err_detail: plain text, no glyph, default indent (level 1 = 2 spaces)" {
    run sh -c '. '"$LIB"'; log_err_detail "more context" 2>&1'
    [ "$output" = "  more context" ]
}

@test "log_err_detail: default is level 1, NOT 0 (pins the log_item asymmetry)" {
    # Omitting the level must indent (2 spaces), unlike log_item which is flush-left.
    run sh -c '. '"$LIB"'; log_err_detail "x" 2>&1'
    [ "$output" = "  x" ]
    [ "$output" != "x" ]
}

@test "log_err_detail: explicit level 2 = four spaces" {
    run sh -c '. '"$LIB"'; log_err_detail "deeper" 2 2>&1'
    [ "$output" = "    deeper" ]
}

@test "log_err_detail: explicit level 0 = flush-left" {
    run sh -c '. '"$LIB"'; log_err_detail "flush" 0 2>&1'
    [ "$output" = "flush" ]
}

@test "log_err_detail writes to stderr (not stdout)" {
    # Capture only stdout; the detail must NOT appear there.
    result=$(log_err_detail "to stderr" 2>/dev/null)
    [ -z "$result" ]
}

@test "log_err_detail returns 0 so the next statement still runs" {
    run sh -c '. '"$LIB"'; log_err_detail "x"; echo reached'
    [ "$status" -eq 0 ]
    [[ "$output" == *"reached"* ]]
}

@test "log_err_detail: no CI annotation in GITHUB_ACTIONS (plain text only)" {
    run sh -c 'GITHUB_ACTIONS=true; export GITHUB_ACTIONS; . '"$LIB"'; log_err_detail "ctx" 2>&1'
    [ "$output" = "  ctx" ]
    [[ "$output" != *"::"* ]]
}

@test "log_err_detail: message containing % is printed literally" {
    run sh -c '. '"$LIB"'; log_err_detail "100% %s %d" 2>&1'
    [ "$output" = "  100% %s %d" ]
}

# --- NO_COLOR / non-TTY: output is plain (no ANSI escapes) --------------------

@test "no ANSI escape sequences when stdout is not a TTY" {
    # bats already runs non-TTY; assert the byte stream carries no ESC (\033).
    run log_ok "plain"
    printf '%s' "$output" | grep -q "$(printf '\033')" && false
    [ "$output" = "✅ plain" ]
}

@test "NO_COLOR forces plain even if a TTY were present" {
    run sh -c 'NO_COLOR=1; export NO_COLOR; . '"$LIB"'; log_warn "x"'
    [ "$output" = "⚠️  Warning: x" ]
}

# --- Message safety: % and format-like content is literal ---------------------

@test "message containing % is printed literally (no printf interpretation)" {
    run log_ok "100% done %s %d"
    [ "$output" = "✅ 100% done %s %d" ]
}

# --- CLI dispatcher (log.sh executed directly, not sourced) -------------------
# These run the library as a program so the foot-of-file dispatcher fires. The
# Justfile uses exactly this path: `./scripts/lib/log.sh <role> "msg"`.

@test "dispatch: valid role emits the same output as the sourced function" {
    run "$LIB" log_section "My Phase"
    [ "$status" -eq 0 ]
    [ "$output" = "🔷 My Phase" ]
}

@test "dispatch: log_ok via CLI matches the in-process form" {
    run "$LIB" log_ok "done"
    [ "$status" -eq 0 ]
    [ "$output" = "✅ done" ]
}

@test "dispatch: a role taking two args (log_warn_file) works via CLI" {
    # This test pins dispatch plumbing (two args forwarded to a role), not the
    # env branch — so force the local human-readable form. Without this, an
    # ambient GITHUB_ACTIONS=true (as in CI) routes to the ::warning file=…::
    # annotation, which carries no "Warning:" label and fails the assertion.
    # The CI annotation form itself is covered separately above (line ~136).
    # `env -u` unsets the var for the dispatched program (run is a bats
    # function, so a prefix assignment would not scope to the subprocess).
    run env -u GITHUB_ACTIONS "$LIB" log_warn_file "a/b.toml" "watch out"
    [ "$status" -eq 0 ]
    [[ "$output" == *"Warning: watch out"* ]]
    [[ "$output" == *"a/b.toml"* ]]
}

@test "dispatch: unknown role fails LOUD on stderr with exit 2 (no silent no-op)" {
    run "$LIB" log_okk "typo"
    [ "$status" -eq 2 ]
    [[ "$output" == *"unknown or unsupported role: log_okk"* ]]
}

@test "dispatch: an internal helper name is NOT dispatchable (not in allow-list)" {
    # _log_out is private; calling it must be rejected, not executed.
    run "$LIB" _log_out 1 "" "x" "" "msg" 0
    [ "$status" -eq 2 ]
    [[ "$output" == *"unknown or unsupported role: _log_out"* ]]
}

@test "dispatch: no role prints usage with exit 2" {
    run "$LIB"
    [ "$status" -eq 2 ]
    [[ "$output" == *"Usage: log.sh <role>"* ]]
}

@test "dispatch: unknown-role diagnostic goes to STDERR, not stdout" {
    run sh -c './'"$LIB"' bogus 2>/dev/null'
    [ "$status" -eq 2 ]
    [ -z "$output" ]
}

@test "sourcing does NOT trigger the dispatcher even when \$1 looks like a role" {
    # Set positional params, then source: the basename guard must keep $0 != log.sh
    # so the dispatch block is skipped and nothing is emitted by the source itself.
    run sh -c 'set -- log_ok "MUST_NOT_PRINT"; . '"$LIB"'; echo SOURCED_OK'
    [ "$status" -eq 0 ]
    [[ "$output" == *"SOURCED_OK"* ]]
    [[ "$output" != *"MUST_NOT_PRINT"* ]]
}
