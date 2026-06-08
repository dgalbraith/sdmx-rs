# 20. Shell Script Test Environment Isolation

Date: 2026-05-30

## Status

Accepted

---

## Context

Shell scripts in `scripts/` are tested using BATS (Bash Automated Testing System) in `tests/bats/`. Tests must run in CI (GitHub Actions) and locally.

During initial BATS test runs, 4 tests consistently failed in CI but passed locally:
- "overdue maintenance item fails check"
- "one failing item fails overall check"
- "--dry-run mode exits 0 even if items are overdue"
- "overdue item fails by default but warns and exits 0 with --warn-overdue"

Root cause: GitHub Actions runner sets `GITHUB_EVENT_NAME=pull_request` in the CI environment. Script logic checked this variable and auto-enabled behavior (`--warn-overdue`) that tests didn't expect. Locally, `GITHUB_EVENT_NAME` isn't set, so tests passed.

The symptom was subtle: tests reported "OVERDUE WARNING" instead of "OVERDUE", causing assertions to fail silently without obvious cause.

## Decision Drivers

* **Test Reproducibility**: Tests must pass identically in CI and locally; environment pollution breaks this assumption
* **Debugging Cost**: Silent failures are expensive to debug; isolation prevents confusing CI-only failures
* **Future-Proofing**: CI environments add new variables over time; tests should not be fragile to these changes
* **Best Practice**: Established testing practice is to provide clean environments for tests

---

## Options Considered

### Option A — Inherit Parent Environment
Run tests with no environment control; inherit `GITHUB_EVENT_NAME`, `PATH`, and all other variables from the CI runner.

```bash
run bash script.sh --force
```

**Pros**:
* Minimal boilerplate; tests are simple
* Naturally uses runner-provided tools (if PATH is inherited)

**Cons**:
* CI variables leak into tests unexpectedly, breaking assumptions
* Tests pass locally (no `GITHUB_EVENT_NAME`) but fail in CI (has it)
* Root cause is hard to diagnose (silent assertion failures)
* Fragile to future CI changes (new variables break tests)
* No explicit control over test environment

**Verdict**: Rejected — Too fragile; cost of debugging CI-only failures outweighs simplicity

### Option B — Use `env` Without `-i` Flag
Run tests with `env`, passing only needed variables.

```bash
run env PATH="..." MAINTENANCE_TODAY="..." bash script.sh --force
```

**Pros**:
* Slightly more explicit than Option A
* Passes through needed variables

**Cons**:
* Does NOT clear the environment; runner variables still leak through
* Still exhibits the same CI-only failure mode as Option A
* False sense of isolation; developers assume it's clean but it isn't

**Verdict**: Rejected — Does not solve the problem

### Option C — Use `env -i` to Clear Environment
Run tests with `env -i` to clear all inherited variables, then explicitly pass only those needed.

```bash
run env -i PATH="$BATS_TEST_TMPDIR/bin:$PATH" \
         MAINTENANCE_TODAY="$MAINTENANCE_TODAY" \
         bash script.sh --force
```

**Pros**:
* Complete environment isolation; no leakage from CI runner
* Tests are reproducible locally and in CI
* Explicit control: only named variables are available to the script
* Future-proof: new CI variables won't affect tests
* Clear intent to readers

**Cons**:
* Slightly more boilerplate per test invocation
* Requires careful attention to which variables to pass through

**Verdict**: Accepted — Solves the root problem with minimal cost

---

## Decision

**Selectively unset problematic CI variables in all BATS test invocations via the `run_isolated()` helper.**

All BATS tests must invoke commands using the `run_isolated()` helper, which unsets `GITHUB_EVENT_NAME`, `GITHUB_ACTIONS`, and `CI`:

```bash
# All BATS tests use run_isolated()
run_isolated "scripts/check-maintenance.sh" --force
run_isolated "./doc-engine.sh" add adr "My ADR"
run_isolated "scripts/update-msrv.sh" --dry-run 1.91.0 1.92.0
```

This ensures:
1. No inherited CI variables (e.g., `GITHUB_EVENT_NAME`) affect test behavior
2. Tests preserve their own environment variables (e.g., `RELEASE_MERGE_NO_SIGN=1`)
3. Tests are reproducible locally and in CI
4. System paths and git configuration remain accessible
5. Consistent test isolation policy across all BATS tests

Rationale: The root cause of CI-only test failures was `GITHUB_EVENT_NAME` leakage. Rather than aggressive environment clearing (`env -i`), we selectively unset only the problematic CI variables. This eliminates silent failures while preserving test flexibility and system functionality. Scripts are tested with `sh` (POSIX shell) to match their production behavior.

---

## Consequences

* **Positive**: Reproducibility — tests now have identical behavior locally and in CI; environment is predictable and isolated
* **Positive**: Debuggability — CI-only failures are eliminated; root causes are visible in logs (not silent)
* **Positive**: Maintainability — future CI environment changes don't break tests; new variables won't leak through
* **Negative**: Boilerplate — each test invocation requires explicit environment setup (minimal; ~2 lines per test)
* **Positive**: Documentation — explicit environment variables in tests document what the script actually requires to run

### Implementation

A helper function `run_isolated()` in `tests/bats/common.sh` encapsulates the pattern by unsetting problematic CI variables:

```bash
run_isolated() {
    local script="$1"
    shift

    # Unset specific CI variables that cause test pollution
    unset GITHUB_EVENT_NAME
    unset GITHUB_ACTIONS
    unset CI

    # Handle bash -c commands (from BATS tests) and normal scripts (sh)
    if [ "$script" = "bash" ]; then
        run bash "$@"
    else
        # Scripts are invoked with sh (POSIX shell) for portability
        run sh "$script" "$@"
    fi
}
```

Usage in tests:
```bash
# Normal script invocation
run_isolated "$BATS_TEST_DIRNAME/../../scripts/check-maintenance.sh" --force

# Complex commands with pipes
run_isolated bash -c 'echo "no" | ./doc-engine.sh remove design "0001-test.md"'
```

**Design rationale**: Rather than clearing the entire environment with `env -i` (which breaks legitimate test variables and system paths), we selectively unset only the problematic CI variables that caused the original issue (`GITHUB_EVENT_NAME`). This allows tests to:
- Set their own environment variables (e.g., `RELEASE_MERGE_NO_SIGN=1`)
- Access system paths and git configuration
- Work identically in both CI and local environments

**Shell choice**: Scripts are invoked with `sh` (POSIX shell), not `bash`, because all shell scripts in the codebase are written for `sh` portability. BATS itself requires `bash`, but the tested scripts are `#!/bin/sh` and should be tested with `sh` to match production behavior.

### Implementation Impact

- **Tests affected**: All BATS tests in `tests/bats/` (19 test files, 278 tests total)
- **Scope**: Universal policy — all test invocations use `run_isolated()`
- **Effort**: Low; helper function reduces boilerplate to a single call
- **Breaking change**: No; all tests pass with the policy applied
- **Benefit**: Consistency enforced across entire test suite; prevents future environment variable leakage bugs

---

## References

* [docs/dev/testing.md](../dev/testing.md) — Testing guide; policy requirement for all BATS tests
* [tests/bats/common.sh](../../tests/bats/common.sh) — `run_isolated()` helper implementation
* [tests/bats/](../../tests/bats/) — All 19 BATS test files apply the policy universally
