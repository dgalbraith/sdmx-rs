#!/bin/sh
# ==============================================================================
# scripts/run-fuzz-check.sh
#
# Short (10-second) cargo-fuzz smoke check for one target: verifies the fuzz
# harness still COMPILES and runs without an immediate crash. Not a real fuzzing
# campaign — a fast guard that a target hasn't bit-rotted.
#
# Output policy (preserved from the original recipe): cargo-fuzz is NOISY, so its
# output is buffered to a temp file and printed ONLY on failure. A passing check
# is quiet (just the section header + success line); a failing one dumps the full
# captured log so the crash/compile error is visible. This is the buffer-and-
# flush-on-failure pattern — keep it: it is the difference between a clean smoke
# check and pages of fuzz output on every run.
#
# Extracted from the `fuzz-check` Justfile recipe so the buffering/trap logic is a
# testable unit and its output is framed by log.sh. The recipe now delegates here.
#
# POSIX sh only.
#
# Usage: scripts/run-fuzz-check.sh <target>
#
# Environment:
#   CARGO  cargo invocation to use (default: cargo) — indirection for tests,
#          which stub it to fake pass/fail without a real fuzz run.
#
# Exit codes:
#   0 = target compiled and survived the 10s smoke run
#   1 = no target argument given
#   N = cargo fuzz failed (its exit code); the captured log is flushed first
# ==============================================================================

set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

CARGO="${CARGO:-cargo}"

TARGET="${1:-}"
if [ -z "$TARGET" ]; then
    log_err "run-fuzz-check: no fuzz target given."
    log_err_detail "Usage: run-fuzz-check.sh <target>   (e.g. parse_xml)"
    exit 1
fi

log_section "Fuzz smoke check (10s): $TARGET"

# Buffer cargo-fuzz output; flush only on failure. The trap cleans the temp file
# on any exit path (normal, error, or signal).
TMP_LOG=$(mktemp)
trap 'rm -f "$TMP_LOG"' EXIT INT TERM

status=0
"$CARGO" fuzz run "$TARGET" -- -max_total_time=10 >"$TMP_LOG" 2>&1 || status=$?

if [ "$status" -ne 0 ]; then
    cat "$TMP_LOG"
    log_fail "fuzz-check: target '$TARGET' failed its 10s smoke check (exit ${status}). See output above."
    exit "$status"
fi

log_ok "fuzz-check: '$TARGET' compiled and survived the 10s smoke run"
