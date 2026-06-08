#!/bin/sh
set -eu

# ==============================================================================
# scripts/ci/run-coverage.sh
# Standard coverage gate: runs the workspace test suite under instrumentation,
# writes lcov.info, then enforces per-crate line-coverage floors.
#
# Wired into `just test-coverage-headless` (itself part of `verify-rust`), and
# run by the CI `coverage` job. The numbers MATCH the Codecov dashboard because
# both are workspace-run and path-filtered (a downstream crate's tests count
# toward upstream code); the floors mirror codecov.yaml targets. For isolated,
# spillover-free per-crate numbers use `just test-coverage-strict` instead.
#
# The defining behaviour — and the reason this is a script, not five Justfile
# lines — is that lcov.info is generated UNCONDITIONALLY, before any gate can
# abort, so a partial report exists even when the test suite or a floor check
# fails. CI uploads that report on an otherwise-failing build, which is exactly
# when the Codecov diff is most useful (it shows which lines dropped a crate
# below its floor). cargo llvm-cov still writes profile data for the tests that
# DID run when some fail, so a partial report is meaningful: we capture the test
# exit code, generate the report, then re-raise so the gate still fails.
#
# A genuine inability to PRODUCE the report (as opposed to a test/floor failure)
# still aborts loudly via `set -e` on the `report --lcov` step — that case must
# never pass silently, since this is the hermetic local coverage gate.
#
# Usage: scripts/ci/run-coverage.sh
#
# Environment:
#   CARGO  cargo invocation to use (default: cargo) — indirection for tests,
#          which point it at a stub that fakes llvm-cov without a real run.
#   COVERAGE_REPORT  when set to "1", print the unified human-readable workspace
#          table after writing lcov.info. Unset (the default) keeps the run
#          gate-only and quiet — which is what `verify` wants, where the table is
#          just noise. The standalone `just test-coverage-headless` sets it so a
#          developer asking for coverage directly still sees the numbers.
#          Either way lcov.info is written and the per-crate floors are enforced.
#
# Exit codes:
#   0 = tests passed AND every crate met its line floor
#   N = the test suite failed (N = cargo's exit code); lcov.info still written
#   1 = a crate fell below its line floor
# ==============================================================================

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/../lib/log.sh"

CARGO="${CARGO:-cargo}"

# Per-crate line-coverage floors (crate:floor). Kept inline here, next to the
# logic that enforces them, so the whole gate reads in one place. These mirror
# codecov.yaml's per-crate targets but are an independent, hermetic check (no
# SaaS on the merge path); keep the two in step when thresholds move.
FLOORS="sdmx-types:85 sdmx-writers:80 sdmx-client:80 sdmx-parsers:75 sdmx-rs:70"

# --- Generate coverage (always), capturing the test outcome ------------------
# `|| test_status=$?` handles a non-zero exit so `set -e` does not abort here —
# we deliberately proceed to write lcov.info regardless of test pass/fail.
# Run under nextest (same runner the rest of the workspace uses — `just test`,
# verify-minimal), whose `--status-level fail` collapses the per-test output to a
# run header + one-line summary, instead of libtest's ~25-line "running N tests /
# test result: ok" dump across five crates. FAILING tests still print (that is
# what `fail` selects), so nothing needing attention is hidden, and nextest still
# writes profile data for `report` to consume below.
#
# NB: do NOT reach for a cargo-level `--quiet` plus a `-- --quiet` harness flag —
# cargo forwards its own `--quiet` to the libtest harness, so the harness sees it
# twice and aborts with "Option 'quiet' given more than once". nextest's status
# levels are the clean knob and sidestep that collision entirely.
#
# `--no-cfg-coverage` suppresses llvm-cov's "currently setting cfg(coverage)"
# notice — the flag the notice itself names. Safe here because no crate gates on
# cfg(coverage) (grep crates/ to confirm before relying on it); it only stops
# that cfg being set, which nothing reads.
test_status=0
"$CARGO" llvm-cov nextest --workspace --locked --no-report --no-cfg-coverage \
    --status-level fail --final-status-level fail || test_status=$?

# This step is the one that MUST succeed: if the report cannot be produced there
# is no coverage to gate or upload, so let `set -e` abort loudly on its failure.
"$CARGO" llvm-cov report --lcov --output-path lcov.info

# --- Human-readable summary (opt-in) -----------------------------------------
# Print ONE unified workspace report — a single header/total table for the whole
# workspace, rather than the five per-crate tables the floor loop below would
# otherwise emit. Gated on COVERAGE_REPORT so it shows for a developer running
# coverage directly but stays silent inside `verify`, where it is pure noise.
# Done here (after lcov.info, before the test re-raise) so that when it IS shown
# it appears on BOTH paths: on a passing run before the floor gate, and on a
# failing run above the "tests failed" message — the collected coverage is just
# as useful for diagnosing why the suite broke. Replays collected data (no re-run).
if [ "${COVERAGE_REPORT:-}" = "1" ]; then
    "$CARGO" llvm-cov report
fi

# Re-raise a test failure now — AFTER lcov.info and the summary — so CI still has
# a report to upload (and an on-screen table) while the gate correctly fails.
if [ "$test_status" -ne 0 ]; then
    log_err_ci "Test run failed (exit ${test_status}); lcov.info was still generated from collected coverage. Failing the gate."
    exit "$test_status"
fi

# --- Enforce per-crate line floors -------------------------------------------
# Replay the collected data per crate (no re-run) purely as a GATE: stdout is
# discarded (the unified report above already showed the numbers) so each call
# stays silent unless it trips its floor. The failure reason goes to stderr and
# survives the redirect; we name the offending crate and exit so the abort is
# actionable. The first below-floor crate stops the loop (later crates unchecked),
# which is sufficient signal — lcov.info already exists, so upload is unaffected.
for entry in $FLOORS; do
    crate="${entry%%:*}"
    floor="${entry##*:}"
    "$CARGO" llvm-cov report --package "$crate" --fail-under-lines "$floor" >/dev/null \
        || { log_err_ci "Crate ${crate} fell below its ${floor}% line-coverage floor."; exit 1; }
done
