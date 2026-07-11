# CI Gates & Quality Checks

This document defines the continuous integration (CI) gates that enforce code quality, compliance, and correctness. All checks must pass before code can be merged to `main`.

## Check Categorisation

Checks are classified into five categories based on when they run and whether they block merge:

### 1. Always-Mandatory (Run on every PR & scheduled)

These checks validate universal code quality and must pass unconditionally:

| Check                | Purpose                                                                                      | Triggers                     | Blocks Merge |
|----------------------|----------------------------------------------------------------------------------------------|:----------------------------:|:------------:|
| **check-commits**    | Conventional commit message validation (local: `check-commits`, CI: `check-commit-messages`) | All PRs                      |    ✅ Yes    |
| **test-matrix**      | Builds and tests on Linux, macOS, Windows with stable Rust                                   | Rust/infra changes, schedule |    ✅ Yes    |
| **clippy**           | Static analysis and linting                                                                  | Rust/infra changes, schedule |    ✅ Yes    |
| **check-formatting** | Code formatting and import sorting                                                           | Rust/infra changes, schedule |    ✅ Yes    |
| **semver-check**     | Semantic versioning compliance (PRs to main only)                                            | PRs targeting main           |    ✅ Yes    |
| **check-security**   | Supply chain audit (deny + machete)                                                          | Rust/infra changes, schedule |    ✅ Yes    |
| **check-secrets**    | Secret leak scan over full git history (`gitleaks`)                                          | All pushes & PRs             |    ✅ Yes    |
| **verify-wasm**      | WASM compile + headless test execution                                                       | Rust/infra changes, schedule |    ✅ Yes    |
| **docs**             | Documentation generation and warnings                                                        | Rust/infra changes, schedule |    ✅ Yes    |
| **docs-internal**    | Internal design_docs layer builds clean (verify-only)                                        | Rust/infra changes, schedule |    ✅ Yes    |
| **coverage**         | Per-crate code coverage floors (`cargo-llvm-cov`)                                            | Rust/infra changes, schedule |    ✅ Yes    |

### 2. Mandatory-When-Triggered (Run only on relevant file changes)

These checks are mandatory but scoped to specific file types. They block merge if triggered:

| Check                    | Purpose                                                        | Triggers                                        | Blocks Merge |
|--------------------------|----------------------------------------------------------------|:-----------------------------------------------:|:------------:|
| **check-docs**           | Markdown/ADR/Design Doc validation                             | Docs/infra changes, schedule                    | ✅ Yes       |
| **nix-check**            | Nix flake integrity                                            | Rust/infra changes, schedule                    | ✅ Yes       |
| **msrv-verify**          | Declared MSRV compiles and passes checks                       | Rust/infra changes, schedule                    | ✅ Yes       |
| **check-scripts**        | Shell script linting and BATS tests                            | Scripts/infra changes, schedule                 | ✅ Yes       |
| **check-workflows**      | GitHub Actions workflow validation                             | Infra changes, schedule                         | ✅ Yes       |
| **validate-scaffolding** | Repository scaffolding conformance                             | Rust/infra changes, schedule                    | ✅ Yes       |
| **check-xsd-fragments**  | XSD contract fragments wired to their Rust types (design_docs) | XSD manifest/fragment/source changes, schedule  | ✅ Yes       |
| **check-decision-refs**  | Crate-source `D-NNNN` references resolve to the register       | Crate source / `decisions.md` changes, schedule | ✅ Yes       |

### 3. Continuous Monitoring & Maintenance

These checks run continuously (on schedule and main pushes) to track maintenance obligations. They are informational — the **scheduled check creates GitHub Issues** for overdue items, which pull work into maintainers' queue:

| Check                  | Purpose                                | Triggers                     | Blocks Merge          |
|------------------------|----------------------------------------|:----------------------------:|:---------------------:|
| **detect-maintenance** | Maintenance obligation tracking        | Schedule, main pushes        | ❌ No (informational) |

### 4. Release Path Gates (Tag-triggered)

These checks run only on tag pushes and are not part of the PR or main-push quality gate. They guard the publish pipeline itself — by the time they run, the release has already been prepared locally and the tag has been pushed. Contributors do not interact with these gates; they are maintainer-only.

| Check                  | Purpose                                                                   | Triggers                     | Blocks Publish |
|------------------------|---------------------------------------------------------------------------|:----------------------------:|:--------------:|
| **validate-changelog** | Verifies `CHANGELOG.md` matches `git-cliff` regeneration byte-for-byte    | Tag push                     | ✅ Yes         |
| **release-dry-run**    | Simulates the release sequence (plan, hooks, tag names); packages nothing | Non-PR pushes/tags, schedule | ✅ Yes         |

`validate-changelog` is gate 2 of the publish chain in `publish.yml` (after signature verification, before setup/tag resolution). See [releasing.md §6](releasing.md#6-ci-publishes-to-github--cratesio) for the full gate sequence.

`release-dry-run` runs on `main` and tag pushes (never on PRs) as an early-warning check that the release sequence would run cleanly if a release were cut from the current tree; it simulates the release plan and packages nothing. It is distinct from the local `just release-dry-run` step in the release workflow, which runs on the release branch as part of release preparation.

### 5. Scheduled & Informational (Weekly check)

These checks run on schedule and report findings but do not block:

| Check                    | Purpose                                  | Triggers      | Blocks Merge          |
|--------------------------|------------------------------------------|:-------------:|:---------------------:|
| **check-msrv**           | MSRV floor detection (vs. declared)      | Schedule only | ❌ No (informational) |
| **msrv-features-check**  | MSRV compatibility across feature combos | Schedule only | ❌ No (informational) |
| **coverage-strict**      | Spillover-free per-crate coverage floors | Schedule only | ❌ No (informational) |

## Trigger Logic

### File Change Detection

The CI pipeline uses path-based filtering to determine which checks run. File change detection uses this matrix:

```
rust:
  - crates/**
  - Cargo.toml
  - Cargo.lock
  - rust-toolchain.toml

scripts:
  - scripts/**
  - tests/bats/**
  - Justfile
  - .pre-commit-config.yaml

docs:
  - **/*.md
  - docs/**

infra:
  - flake.nix
  - flake.lock
  - .github/workflows/ci.yml
  - .github/workflows/publish.yml
  - .github/workflows/verify-signature.yml
  - .github/actions/**
```

### PR vs. Push vs. Schedule

**Pull Requests to main**:
- Always-mandatory checks: REQUIRED
- Mandatory-when-triggered checks: REQUIRED (if file changes detected)
- Scheduled checks: SKIP
- Continuous monitoring checks: INFORMATIONAL (don't block contributions)

**Pushes to `staging-*` (CI verification surface before fast-forward to main)**:
- Same jobs as pushes to `main` — triggered by `on: push: branches: staging-*`
- Status check results are stored against the commit SHA; a green SHA on `staging-*` satisfies the Zero Trust Gate when that SHA is fast-forwarded to `main`
- Cancellation: concurrent `staging-*` runs are cancellable (they satisfy the `cancel-in-progress` expression `ref != refs/heads/main && !startsWith(ref, refs/tags/)`)

**Pushes to main (after fast-forward from staging)**:
- All mandatory checks: REQUIRED (satisfied by the SHA's prior green run on `staging-*`)
- Continuous monitoring checks: INFORMATIONAL (no blocks)

**Scheduled (Weekly, Saturday 00:00 UTC)**:
- All checks run (including informational)
- detect-maintenance detects overdue items and creates GitHub Issues labelled `maintenance` for maintainer action

## Status Check Requirements

### The CI Quality Gate aggregator

The Zero Trust Gate ruleset on `main` requires exactly **one** status-check context: `CI Quality Gate`. This is the `ci-gate` job in `.github/workflows/ci.yml` — an aggregator that depends on every merge-gating job (`needs:`) and passes only when each one is `success` or legitimately `skipped`. A green seal on this single context, earned on a `staging-*` SHA, carries to `main` on fast-forward.

A single aggregate context (rather than enumerating each job) buys two things:

1. **No path-filter deadlock.** Path-filtered jobs that legitimately do not run on a given SHA report `skipped`. If each were an individually required check, GitHub would treat the missing result as *unsatisfied* and block the fast-forward for any SHA that did not touch every path category. The aggregator treats a legitimate `skipped` as acceptable, so a docs-only or config-only change is not blocked by Rust jobs that never ran.
2. **No brittle string list.** The ruleset no longer has to stay byte-identical to a dozen job `name:` fields; only the one aggregate name must match.

The aggregator is **fail-closed** (see the in-workflow comment on `ci-gate`):

- It asserts `changes.result == "success"` explicitly. Every gating job has `needs: [changes]`; if the path-filter job dies, all dependents are `skipped`, and a naive "no failures" check would read all-skips as green and let an unverified SHA onto `main`. Asserting `changes` succeeded closes that hole.
- It allowlists the safe states (`success`/`skipped`) rather than denylisting bad ones, so a new or unknown result string from GitHub fails closed, not open.

### Declared gating set & drift protection

The exact set of jobs the aggregator must cover is declared in [`forge/github/ci-gating-jobs.json`](../../forge/github/ci-gating-jobs.json) — the **intent**. The `ci-gate` job's `needs:` list is the **execution**, and the Check Details below are the **human-readable mirror**. `scripts/verify-ci-gate.sh` (run by the `check-workflows` job, and locally under both `just verify-infra` and `just verify-docs`) asserts all three agree: the `needs:` list equals the manifest, every declared job exists in `ci.yml`, and every gating job has a `#### <job>` Check Details entry below (the doc may document *more* — the excluded jobs — so this is a subset check). A stray edit that drops a gating job from `needs:` (silently ungating it on `main`), or adds a gating job without documenting it here, therefore fails CI rather than reaching production. When a job is added to or removed from the gate, update the manifest, the `needs:` list, **and this document** in the same change.

### What the gate does *not* cover, and why

Seven jobs are deliberately **excluded** from the aggregator. Including any of them would either deadlock the staging fast-forward (they do not run on `push` events) or block a merge for non-code reasons:

| Excluded job              | Class                   | Why excluded |
|---------------------------|-------------------------|--------------|
| **semver-check**          | PR-only (procedural)    | Validates version strings against commit history on PRs. A local `--no-ff` merge does not alter `Cargo.toml` versions, so re-running it on the staging SHA adds noise without signal; it does not run on `push`, so requiring it would deadlock the fast-forward. Remains required on PRs to `main`. |
| **check-commit-messages** | PR-only (procedural)    | Validates commit message format on the feature branch. Does not run on `push` events. |
| **validate-changelog**    | Tag-scoped              | Triggers strictly on `refs/tags/**`. No bearing on a `staging-*` or `main` push. |
| **detect-maintenance**    | Informational           | Non-blocking maintenance tracker; runs on schedule/main-push and creates Issues, never gates a merge. |
| **check-msrv**            | Scheduled/informational | Opportunistic MSRV-floor detection; schedule-only, non-blocking. |
| **msrv-features-check**   | Scheduled/informational | MSRV feature-matrix check; schedule-only, non-blocking. |
| **coverage-strict**       | Scheduled/informational | Spillover-free per-crate coverage audit; schedule-only, non-blocking. |

### Checks enforced locally, not in CI

Not every check in `just verify` has a matching CI job. `check-conventions`
(the greppable source conventions in `crates/*/src`, e.g. typed `None::<T>` and
empty `Vec::new()`) runs only through the `verify-rust` chain: locally in `just
verify`, and on every push through the `run-just-verify-rust` pre-push hook. It
is deliberately not mirrored as a standalone CI job: the convention is cheap to
enforce at the point of change and carries no cross-platform or release risk
that would justify a dedicated runner. The other `verify-rust` steps
(formatting, clippy, docs, doctests, semver, coverage, release dry-run) each do
have a CI job, so this is the one local-only member of that chain.

## Check Details

### Always-Mandatory Checks

#### check-commit-messages
Validates that commit messages follow the Conventional Commits specification. This is enforced as a GitHub Actions job on all PRs.

**Runs on**: All PRs (validates commits not yet on main).
**Purpose**: Ensure commit messages are well-formed and consistent for changelog generation and history readability.

#### test-matrix
Builds and tests the workspace on three operating systems (Ubuntu Linux, macOS, Windows) with the latest stable Rust compiler.

**Runs on**: Rust code changes, infrastructure changes, or scheduled.
**Purpose**: Ensures portability across platforms.

#### clippy
Static analysis and linting with strict warnings-as-errors enforcement.

**Runs on**: Rust code changes, infrastructure changes, or scheduled.
**Purpose**: Catch common mistakes, performance issues, and non-idiomatic patterns.

#### check-formatting
Enforces consistent formatting of Rust code and TOML manifests using nightly rustfmt rules.

**Runs on**: Rust code changes, infrastructure changes, or scheduled.
**Purpose**: Maintain readable, consistent code style.

#### semver-check
Verifies semantic versioning compliance on PRs to `main`. Applicability is decided
from the crates' own versions (read locally), not a network probe: **warn-skipped
while pre-1.0** (no stability promise to diff against), and **mandatory once any
crate is 1.0+, including 1.0 pre-releases like `1.0.0-rc.1`** — the rc cycle is the
1.0 cycle. Post-1.0 the check fails **closed**: if the published baseline cannot be
confirmed (network/registry error), the gate fails rather than silently skipping;
the only post-1.0 skip is the explicit `SEMVER_ALLOW_NO_BASELINE=1` opt-in for the
first `1.0.0` publish.

**Runs on**: PRs targeting `main` only.
**Purpose**: Ensure version bumps follow SemVer conventions; prevent an
undetected breaking change from shipping in a 1.0+ release.

#### check-security
Runs `cargo deny` (advisory/license checks) and `cargo machete` (unused dependency detection).

**Runs on**: Rust code changes, infrastructure changes, or scheduled.
**Purpose**: Prevent vulnerable or dead dependencies.

#### check-secrets
Scans the full git history for committed secrets (keys, tokens, private keys) with `gitleaks` (`just secrets-scan`).

**Runs on**: All pushes and PRs (unconditional; not path-filtered).
**Purpose**: Block committed secrets from reaching or persisting on the remote (see [SECURITY.md § Secret Leak Prevention](../../SECURITY.md#secret-leak-prevention)).

#### verify-wasm
Verifies that the `no_std` crates compile for `wasm32-unknown-unknown` and that a representative test subset executes under Node/V8.

**Runs on**: Rust code changes, infrastructure changes, or scheduled.
**Purpose**: Enforce WASM portability and runtime parity (ADR-0007).

#### docs
Generates workspace documentation and enforces `missing_docs` linting.

**Runs on**: Rust code changes, infrastructure changes, or scheduled.
**Purpose**: Ensure all public API items are documented.

#### docs-internal
Builds the internal `design_docs` rationale layer (private items under `--cfg design_docs`) to confirm it compiles clean under `-D warnings`. Verify-only: the output is never published, and docs.rs remains public-only.

**Runs on**: Rust code changes, infrastructure changes, or scheduled.
**Purpose**: Catch a broken internal-docs build (a bad fragment `include_str!` path, or a `--cfg design_docs` warning) that the public `docs` build does not exercise.

#### coverage
Enforces per-crate code coverage floors with `cargo-llvm-cov` (`just coverage-gate`). The per-crate thresholds are the gate; the Codecov upload is a non-blocking nice-to-have.

**Runs on**: Rust code changes, infrastructure changes, or scheduled.
**Purpose**: Keep each crate above its minimum coverage floor.

#### changes
The path-filter engine (`dorny/paths-filter`): computes which file categories changed (`rust`, `scripts`, `docs`, `infra`, `xsd`, `decisionrefs`, `cigate`, `security`, `formatting`, `coveragecfg`, `releasecfg`) so every path-gated job knows whether to run. It is a load-bearing gate, not background infrastructure — the `ci-gate` aggregator asserts `changes` itself succeeded before trusting any job that legitimately skipped (the fail-closed invariant above).

**Runs on**: Every PR and push — it is the first job, and every other job declares `needs: [changes]`.
**Purpose**: Drive path-based job selection and anchor the aggregator's fail-closed check.

### Mandatory-When-Triggered Checks

#### nix-check
Validates Nix flake schema and evaluation.

**Runs on**:
- Rust or infra changes
- Scheduled weekly

**Purpose**: Ensure Nix environment integrity.

#### msrv-verify
Verifies that code compiles and passes checks on the declared Minimum Supported Rust Version (MSRV).

**Runs on**: Rust code changes, infrastructure changes, or scheduled.
**Purpose**: Ensure MSRV promises to users are kept.

#### check-docs
Validates documentation structure and style for Markdown, ADRs, and Design Docs.

**Runs on**: Documentation/Markdown changes, infrastructure changes, or scheduled.
**Purpose**: Maintain consistent, well-formed documentation.

#### check-scripts
Lints shell scripts with `shellcheck` and runs BATS test suite for maintenance system.

**Runs on**: Script changes, infrastructure changes, or scheduled.
**Purpose**: Ensure shell scripts are safe and maintainable.

#### check-workflows
Validates GitHub Actions workflow syntax and security with `actionlint`.

**Runs on**: Workflow file changes, infrastructure changes, or scheduled.
**Purpose**: Prevent broken or insecure CI workflows.

#### validate-scaffolding
Verifies that repository scaffolding (crate structure, required files, workspace configuration) conforms to the project spec.

**Runs on**: Rust code changes, infrastructure changes, or scheduled.
**Purpose**: Catch scaffolding drift — missing or malformed crate metadata, workspace member mismatches, or required file absences — before they compound.

#### check-xsd-fragments
Verifies that every modelled type's `## Specification` cites its XSD symbol and wires its `include_str!`, that no orphan includes exist, and that every manifest fragment is sliced from a schema pinned in `specs/sources.toml` (the subset invariant) (`just check-xsd-fragments`). Fragment freshness is not diffed: the fragments are a pure function of the pinned schemas, manifest, and generator, regenerated on each build.

**Runs on**: XSD manifest/fragment or crate-source changes, infrastructure changes, or scheduled.
**Purpose**: Keep the design_docs XSD contracts in lockstep with the pinned schemas.

#### check-decision-refs
Verifies that every `D-NNNN` decision reference in the crate sources resolves to an entry in `docs/decisions.md` (`just check-decision-refs`).

**Runs on**: Crate-source or `decisions.md` changes, infrastructure changes, or scheduled.
**Purpose**: Prevent dangling decision references in the code from reaching `main`.

### Release Path Gate Checks

#### validate-changelog
Verifies that each crate's committed `CHANGELOG.md` matches what `git-cliff` regenerates byte-for-byte from the current tag's commit history. A mismatch means the changelog was either hand-edited or not regenerated before the release tag was pushed.

**Runs on**: Tag pushes only (`refs/tags/**`).
**Position in publish chain**: Gate 2 — runs after `verify-signature`, before `setup`/tag-resolution. See [releasing.md §6](releasing.md#6-ci-publishes-to-github--cratesio).
**Purpose**: Enforce the machine-record integrity of `CHANGELOG.md`. Manual edits must never reach a release; fix the underlying commit messages instead and regenerate.

#### release-dry-run
Runs `cargo release` in its default dry-run mode (no `--execute`) across all crates to simulate the release sequence from the current tree: release-config validity, per-crate change detection against the previous release tags, the derived publish order, pre-release hook wiring, and the tag names that would be stamped. Nothing is packaged or compiled in this path. Packaging validation (metadata, licences, package contents, compilation of the packaged sources) lives in the `prepublish-check` dry-run at release time, which resolves intra-workspace pins through its `[patch.crates-io]` workspace-path overlay and so does not depend on the registry.

**Runs on**: Non-PR pushes (main and tag) and scheduled (weekly). Never on PRs: it exercises the release path, not the PR's changes, so it is irrelevant to review feedback.
**Purpose**: Early warning that the release simulation still passes from the current tree. Distinct from the local `just release-dry-run` step in the release workflow (which runs on the release branch as part of release preparation): this gate monitors the state of `main` between releases.

### Continuous Monitoring & Maintenance Checks

#### detect-maintenance
Validates that maintenance obligations (in `maintenance.toml`) are tracked and current. Informational check — does not block merge.

**Runs on**:
- Every scheduled run (Saturday 00:00 UTC)
- All pushes to main (after merge)
- Demoted to informational on PRs (warns but doesn't block external contributions)

**Result**:
- **Scheduled (Saturday)**: Creates GitHub Issues labelled `maintenance` for any overdue items. This pulls work into maintainers' queue.
- **Post-merge**: Informational (doesn't fail). Visibility via scheduled issues is the enforcement mechanism.

**Purpose**: Track periodic maintenance obligations and surface overdue work via automated GitHub Issues rather than CI failures.

### Scheduled & Informational Checks

#### check-msrv
Detects the actual MSRV floor using `cargo-msrv` and reports if it differs from declared.

**Runs on**: Schedule only (weekly).
**Result**: Green checkmark with floor details in logs (non-blocking).

**Purpose**: Identify opportunistic MSRV lowering (feature work, not breaking).

#### msrv-features-check
Verifies MSRV compatibility (see `rust-toolchain.toml`) across feature combinations: `--no-default-features` and `--all-features`.

**Runs on**: Schedule only (weekly).
**Result**: Confirms compilation succeeds with feature matrices (informational).

**Purpose**: Catch regressions where feature flags introduce dependencies newer than MSRV. Developers can run locally (`just msrv-features`) when working on optional features.

#### coverage-strict
Runs each crate's suite in isolation (`just test-coverage-strict`) and enforces the same per-crate floors as the merge-gating `coverage` job, but without the cross-crate spillover the single-run gate allows. Slower than the gate, so it is not wired into `verify`; the weekly run is where a spillover-hidden drop below a floor surfaces.

**Runs on**: Schedule only (weekly).
**Result**: Confirms each crate meets its floor in isolation (informational).

**Purpose**: Catch a crate whose own tests no longer cover its floor but which the replayed single-run gate keeps green via coverage from sibling crates' tests.

## Interpreting CI Failures

### "check-commit-messages" failed
**Probable causes**:
- Commit message doesn't follow Conventional Commits format
- Missing scope, type, or description
- Wrong type (should be `feat`, `fix`, `chore`, `docs`, `test`, `refactor`, `perf`, `style`)

**Fix**: Amend commit message to follow format:
```
<type>(<scope>): <description>

<body (optional)>

<footer (optional, e.g., Closes #123)>
```

Example:
```
feat(sdmx-types): add codelist representation

Implements core structure for SDMX codelists with serialisation traits.

Closes #42
```

For more details, see [CONTRIBUTING.md § Commit Requirements](../../CONTRIBUTING.md#4-commit-requirements).

### "test-matrix" failed
**Probable causes**:
- Code doesn't compile on one or more platforms (Linux/macOS/Windows)
- Tests fail on stable Rust

**Fix**: Reproduce locally on the failing OS or use `cargo build --target <triple>`.

### "clippy" failed
**Probable causes**:
- Code violates clippy linting rules
- Performance or style issues detected

**Fix**: Run `cargo clippy --all-targets -- -D warnings` locally and address warnings.

### "check-formatting" failed
**Probable causes**:
- Code is not formatted with nightly rustfmt rules
- TOML files are not formatted

**Fix**: Run `just fmt` (which uses the nightly rustfmt from Nix) to auto-fix.

### "semver-check" failed
**Probable causes**:
- Public API changed without version bump
- Breaking change detected
- (1.0+ only) the published baseline could not be confirmed — a network/registry
  error, or no `1.0.0` baseline exists yet on crates.io

**Fix**: For an API change, review it and update the version in `Cargo.toml` if the
change is intentional. For a baseline/network failure, retry once crates.io is
reachable — the gate fails closed here by design rather than skipping. If this is
the genuine **first** `1.0.0` publish (no 1.0 baseline can exist yet), set
`SEMVER_ALLOW_NO_BASELINE=1` deliberately for that one release, then remove it.

### "check-security" failed
**Probable causes**:
- Dependency has known vulnerability
- Dependency license is not approved
- Unused dependency added

**Fix**:
- Vulnerabilities: upgrade to patched version or document exception in `deny.toml`
- Licenses: approve in `deny.toml` (rare) or remove dependency
- Unused: remove from Cargo.toml or add to `[package.metadata.cargo-machete]` with comment

### "verify-wasm" failed
**Probable causes**:
- Code uses `std` features in a `no_std` crate (compile failure)
- A wasm-annotated test panics or diverges under Node/V8 (execution failure)

**Fix**: For a compile failure, ensure `#![no_std]` crates use only core/alloc APIs. For an execution failure, reproduce locally with `wasm-pack test --node crates/<crate>`.

### "docs" failed
**Probable causes**:
- Public item lacks doc comment
- Doc comment contains warnings

**Fix**: Add `///` doc comments to public items.

### "nix-check" failed
**Probable causes**:
- Nix flake syntax error
- Missing input or output in `flake.nix`

**Fix**: Review `flake.nix` for syntax errors; run `nix flake check` locally.

### "msrv-verify" failed
**Probable causes**:
- Code uses features only available in Rust version newer than declared MSRV
- Code doesn't compile on declared MSRV

**Fix**: Either lower MSRV if intentional, or refactor to use older APIs.

### "check-docs" failed
**Probable causes**:
- Markdown syntax error
- ADR or Design Doc missing required fields
- Broken links

**Fix**: Review documentation format; run `just verify-docs` locally.

### "check-scripts" failed
**Probable causes**:
- Shell script has syntax error or unsafe pattern
- BATS test failed

**Fix**: Run `just shellcheck` and review output; run `just test-scripts` locally.

### "check-workflows" failed
**Probable causes**:
- GitHub Actions workflow YAML syntax error
- Expression syntax error

**Fix**: Review `.github/workflows/ci.yml` for syntax errors; run `just test-workflows` locally.

### "detect-maintenance" failed
**Probable causes**:
- Maintenance obligation is overdue
- Inline comment doesn't match `maintenance.toml`

**Fix**: Update maintenance items using `scripts/maintenance-bump.sh`; see [maintenance.md](maintenance.md).

### "check-msrv" (informational)
**Meaning**: MSRV floor is lower than declared (not a failure).

**Example**: Declared 1.90.0, but code works on 1.88.0.

**Action**: Opportunistic—decide during maintenance if you want to lower MSRV. See [msrv.md](msrv.md).

## Local Verification

All mandatory checks are available locally via `just verify`:

```bash
just verify        # Run complete verification (CI equivalent)
just verify-rust   # Rust checks only
just verify-docs   # Documentation checks only
just verify-infra  # Infrastructure checks (Nix, workflows)
just test-help     # Testing help menu
just audit-help    # Audit & compliance help menu
just lint-help     # Linting & style help menu
```

**Key principle**: If `just verify` passes locally, CI should pass.

## Related Documentation

- [maintenance.md](maintenance.md) — Maintenance obligation tracking and updates
- [msrv.md](msrv.md) — MSRV policy and upgrade procedures
- [merging.md](merging.md) — Merge protocol and check trigger matrix
- [CONTRIBUTING.md](../../CONTRIBUTING.md) — Contributor requirements and PR workflow
