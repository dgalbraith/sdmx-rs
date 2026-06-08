#!/bin/sh
set -eu

# ==============================================================================
# scripts/run-bats.sh
# Run the BATS shell-script test suite and summarize its TAP output:
#   - count passed / failed / skipped
#   - pass failures (and their `#` diagnostic lines) through to the terminal
#   - suppress the per-test noise of passing tests
#   - print a colored summary; exit non-zero if any test failed
#
# Reads BATS' TAP stream and reduces it to a digest — invoked by `just
# test-scripts`. Kept as a script (not inline awk in the Justfile) so the
# summarization logic is unit-testable; see tests/bats/run-bats.bats.
#
# Usage: scripts/run-bats.sh [bats args...]   (defaults to tests/bats/)
#
# Indirection for tests: BATS may be overridden to point at a mock that emits a
# canned TAP stream, so the summarizer can be exercised without a real suite.
# ==============================================================================

BATS="${BATS:-bats}"

# Default the target to the suite directory when no args are given.
if [ "$#" -eq 0 ]; then
    set -- tests/bats/
fi

# Capture the TAP stream to a temp file rather than piping directly: this repo's
# /bin/sh (dash) does not portably support `set -o pipefail` (shellcheck SC3040),
# so the temp-file pattern lets the awk summarizer be the authoritative exit
# signal — its END block exits 1 whenever a `not ok` was seen, which is exactly
# bats' own failure condition. (Mirrors the no-pipefail approach in
# release-merge.sh.) `|| true` keeps `set -e` from aborting before awk runs.
tap=$(mktemp)
trap 'rm -f "$tap"' EXIT
"$BATS" "$@" > "$tap" 2>&1 || true

awk '
    BEGIN { passed = 0; failed = 0; skipped = 0; in_failure = 0 }
    /^ok / {
        if ($0 ~ /#[[:space:]]*(skip|SKIP)/) {
            skipped++;
        } else {
            passed++;
        }
        in_failure = 0;
        next;
    }
    /^not ok / { failed++; in_failure = 1; print $0; next; }
    /^[[:space:]]*#/ { if (in_failure) print $0; next; }
    { print $0; }
    END {
        # NB: these ✅/❌ summary lines deliberately do NOT use scripts/lib/log.sh —
        # awk is a separate process and cannot call the shell logger functions.
        if (failed > 0) {
            printf "\n❌ BATS tests: %d failed, %d passed", failed, passed > "/dev/stderr";
            if (skipped > 0) {
                printf ", %d skipped", skipped > "/dev/stderr";
            }
            printf "\n" > "/dev/stderr";
            exit 1;
        } else {
            if (skipped > 0) {
                printf "✅ BATS tests: %d passed (%d skipped)\n", passed, skipped;
            } else {
                printf "✅ BATS tests: %d passed\n", passed;
            }
        }
    }
' "$tap"
