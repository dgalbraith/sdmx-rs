# Developer Tooling & Justfile Reference

This guide documents the development environment and provides a comprehensive, tabulated reference for all commands exposed via `just`.

---

## Toolchain Philosophy

To guarantee deterministic, reproducible builds across different development machines (Linux and macOS) and CI environments, `sdmx-rs` relies on **Nix Flakes** and **`direnv`**.

1. **Nix Flake (`flake.nix` / `flake.lock`)**
   Pins the exact compiler version (stable Rust + nightly `rustfmt`), system libraries, linters, and helper tools.
2. **`direnv` (`.envrc`)**
   Automatically loads the Nix devShell environment whenever you enter the project directory.
3. **`just` (`Justfile`)**
   Serves as our unified command runner, coordinating quality gates, code formatting, documentation generators, testing harnesses, and diagnostic suites.

> [!TIP]
   > For local environment setup, consult [CONTRIBUTING.md](../../CONTRIBUTING.md#onboarding-quickstart-clone--setup--build).

### Refreshing Pinned Dependencies

The pins above are the reproducibility contract: every contributor, the Nix
sandbox, and CI all build from the same locked versions. The project tracks
**two independent supply chains**, refreshed by two separate maintainer targets:

| Lockfile     | Governs                                  | Refresh with       | Validated by   |
|--------------|------------------------------------------|--------------------|----------------|
| `Cargo.lock` | The library's crate dependency graph     | `just update-deps` | `verify-rust`  |
| `flake.lock` | The toolchain & dev tools (Nix inputs)   | `just update-flake`| `verify-infra` |

Both targets follow one rule: **refresh → validate → review → commit, in that
order, with the human in the loop.** They mutate the lockfile and run the
relevant gate, but they deliberately do **not** commit. A lockfile bump changes
what every build resolves to, so it is treated as a reviewed event — inspect the
diff, then record it as a GPG-signed checkpoint:

```bash
just update-deps              # all crates  (or: just update-deps serde tokio)
git diff Cargo.lock           # review the resolved changes
git commit -S Cargo.lock -m "chore(deps): update Cargo.lock"
```

> [!NOTE]
> `update-deps` is **lockfile-only** — it applies semver-compatible updates and
> never edits the version ranges declared in `Cargo.toml`. Bumping a range to a
> new major (a breaking upgrade) remains a deliberate manual edit.
>
> Both targets leave the lock **unstaged** for review (`git diff <lock>`).
> `update-flake` stages `flake.lock` *transiently* during validation only —
> the Nix sandbox copies just git-tracked files, so an unstaged lock would leave
> `nix flake check` validating the previous inputs — then unstages it again
> (even if validation fails) so you always start your review from a clean index.

The read-only counterpart to these mutating targets is `just outdated`, which
*reports* stale crates without changing anything — detection (`outdated`) and
remediation (`update-deps`) are intentionally separate commands.

---

## Quick Start Workflows

Common developer patterns and their corresponding commands:

### First-Time Setup
```bash
# After the Nix devShell is active (via direnv or `nix develop`), wire up the
# repository in one step: installs git hooks, then verifies the environment.
# Idempotent — safe to re-run after pulling tooling changes.
just setup
```

> [!NOTE]
> The Nix devShell already provisions the compiler, linters, and helper tools, so
> `just setup` does not install a toolchain or warm build caches — it only
> registers the git hooks and reports any remaining gaps. For the full clone →
> setup → build walkthrough, see [CONTRIBUTING.md](../../CONTRIBUTING.md#onboarding-quickstart-clone--setup--build).

### Local Development Iteration
```bash
# Fast feedback loop while coding (compile + clippy + test)
just verify-minimal

# Full verification before committing (aggregate of all gates below)
just verify

# Modular sub-gates (run automatically locally via pre-push hooks based on changed paths)
just verify-rust            # Rust codebase checks: clippy, nextest, wasm, and semver
just verify-scripts         # Repository scripts/CI: shellcheck, actionlint, and BATS tests
just verify-docs            # Markdown documents: ledger structures and active link check
just verify-security        # Security & supply chain: secret scan, advisories/licenses, unused deps
just verify-infra           # Nix/Devshell config: flake check sandbox validation
just verify-maintenance     # Maintenance obligations: scaffolding and deadline checks

# Check code formatting without fixing
just check-format

# Fix formatting and TOML issues
just fmt
```

### Debugging Environment Issues
```bash
# Start here: shows all available diagnostic checks
just doctor

# Quick health check (compiler, toolchain, git setup)
just doctor-quick

# Compare local environment against CI constraints
just doctor-ci
```

### Before Submitting a Pull Request
```bash
# Run full quality gates locally
just verify

# Check code coverage
just coverage

# Validate commit messages
just check-commits
```

### Documentation & Architecture Decisions
```bash
# Create a new ADR
just adr "Your Decision Title"

# Validate all ADR formatting
just verify-adr

# Create a new Design Document
just design "Your Design Title"
```

---

## Categorised Justfile Recipe Table

All recipes must be executed from within the active Nix devShell (automatic under `direnv`).

**Scope definitions**:
- **Local** — Runs on developer machines; not part of the CI pipeline.
- **Local & CI** — Runs both locally and in the CI pipeline (e.g., on pull requests and main branch).
- **CI** — Runs only in the CI pipeline; not designed for local execution.
- **Maintainer** — Maintainer-only release and versioning operations; part of the release workflow on `main` after approval.

### 1. Environment Setup

| Target              | Scope | Description                                                                           | Tool(s)                         |
|---------------------|:-----:|---------------------------------------------------------------------------------------|---------------------------------|
| `just setup`        | Local | One-shot onboarding: registers Git hooks then verifies the environment. Idempotent.   | `pre-commit`, `just doctor-env` |
| `just hook-install` | Local | Registers pinned local Git hooks (`pre-commit`, `commit-msg`, `pre-push`).            | `pre-commit`                    |

### 2. Unified Verification & Quality Gates

| Target                       | Scope      | Description                                                                                                              | Tool(s)                                        |
|------------------------------|:----------:|--------------------------------------------------------------------------------------------------------------------------|------------------------------------------------|
| `just verify`                | Local & CI | **Primary CI quality gate.** Runs all quality checks in parallel.                                                        | Multiple [1]                                   |
| `just verify-linear`         | Local      | Runs all quality checks sequentially; useful for detailed tracing/debugging.                                             | Multiple [1]                                   |
| `just verify-rust`           | Local & CI | Checks formatting, runs clippy, builds docs, checks WASM, checks semver, enforces coverage floors, and dry-runs release. | Multiple [2]                                   |
| `just verify-scripts`        | Local & CI | Lints shell scripts, validates shebangs, runs BATS tests, and checks test manifests.                                     | `shellcheck`, `bats`, scripts                  |
| `just verify-docs`           | Local & CI | Validates Markdown style, verifies document ledgers, and checks links.                                                   | `markdownlint`, `lychee`, `doc-engine.sh`      |
| `just verify-security`       | Local & CI | Scans git history for secrets and audits dependency advisories/licenses.                                                 | `gitleaks`, `cargo-deny`, `cargo-machete`      |
| `just verify-infra`          | Local & CI | Validates Nix Flake evaluation and GitHub Actions workflow syntax.                                                       | `nix flake check`, `actionlint`                |
| `just verify-maintenance`    | Local & CI | Validates scaffolding configurations and active maintenance obligations.                                                 | `check-scaffolding.sh`, `check-maintenance.sh` |
| `just verify-minimal`        | Local      | Fast validation gate for local iteration. Compiles and runs standard tests.                                              | `cargo check`, `cargo clippy`, `cargo nextest` |
| `just check-commits`         | Local & CI | Validates commit messages in the active branch against Conventional Commit rules.                                        | `commitlint`                                   |
| `just link-check`            | Local & CI | Validates Markdown link reachability and bans absolute `file://` links in md/toml/rs.                                    | `lychee`, `check-local-links.sh`               |
| `just check-decision-refs`   | Local & CI | (Supports `verify-docs`) Validates that crate-source decision references (`D-NNNN`) resolve to `docs/decisions.md`.      | `check-decision-refs.sh`                       |
| `just check-conventions`     | Local & CI | (Supports `verify-rust`) Enforces greppable conventions in `crates/*/src` (typed `None::<T>`, empty `Vec::new()`).       | `check-conventions.sh`                         |
| `just check-shebangs`        | Local & CI | (Supports `verify-scripts`) Validates that all scripts in `scripts/` declare the POSIX-portable `#!/bin/sh` shebang.     | `check-shebangs.sh`                            |
| `just verify-test-manifests` | Local & CI | (Supports `verify-scripts`) Validates that the update-msrv test manifest lists all workspace crate `Cargo.toml` files.   | `check-test-manifests.sh`                      |
| `just test-workflows`        | Local & CI | (Supports `verify-infra`) Validates GitHub Actions workflows for syntax errors and SHA-pinning compliance.               | `actionlint`                                   |
| `just check-maintenance`     | Local & CI | (Supports `verify-maintenance`) Validates that project maintenance tasks are up-to-date and not past their deadlines.    | `scripts/check-maintenance.sh`                 |
| `just check-scaffolding`     | Local & CI | (Supports `verify-maintenance`) Validates that ignored dependencies are properly scaffolded and documented.              | `scripts/check-scaffolding.sh`                 |

[1] invokes `rustfmt`, `clippy`, `rustdoc`, `gitleaks`, `cargo-deny`, `cargo-machete`, `cargo-semver-checks`, `cargo-llvm-cov`, `cargo-release`, `taplo`, `markdownlint`, `shellcheck` and `bats` along with the custom scripts `check-scaffolding.sh`, `check-maintenance.sh`, `check-shebangs.sh` and `doc-engine.sh`.
[2] invokes `cargo check`, `clippy`, `nextest`, `llvm-cov` and`cargo-release`.

### 3. Code Quality, Formatting & Linting

| Target                   | Scope      | Description                                                                                                        | Tool(s)                                                    |
|--------------------------|:----------:|--------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------|
| `just lint-help`         | Local      | **Formatting, style, and static analysis diagnostics guide.** Displays style commands.                             | `just`                                                     |
| `just lint`              | Local      | Runs all non-modifying style, formatting, and clippy checks sequentially.                                          | `rustfmt`, `taplo`, `clippy`, `markdownlint`, `shellcheck` |
| `just fmt`               | Local      | Formats all Rust files and TOML manifests using project-pinned nightly styling rules.                              | Nightly `rustfmt`, `taplo`                                 |
| `just check-format`      | Local & CI | Validates formatting standards without modifying files.                                                            | Nightly `rustfmt`, `taplo`                                 |
| `just check`             | Local      | Type-checks all workspace packages and targets without producing binaries.                                         | `cargo check`                                              |
| `just clippy`            | Local & CI | Strict static analysis under pedantic and nursery lints.                                                           | `cargo-clippy`                                             |
| `just docs`              | Local & CI | Generates workspace documentation and validates that public API comments are warn-free.                            | `cargo-doc`                                                |
| `just docs-internal`     | Local      | Renders the internal documentation layer (`design_docs` rationale plus private items); never published to docs.rs. | `cargo-doc`                                                |
| `just toml-fmt`          | Local      | Formats all TOML files (`Cargo.toml`, `deny.toml`, etc.) in the workspace.                                         | `taplo`                                                    |
| `just toml-check`        | Local & CI | Validates that all workspace TOML files are formatted correctly.                                                   | `taplo`                                                    |
| `just md-fmt`            | Local      | Formats all Markdown documentation files for structure and style.                                                  | `markdownlint`                                             |
| `just md-check`          | Local & CI | Lints all Markdown documentation files for structure and syntax style.                                             | `markdownlint`                                             |
| `just shellcheck`        | Local & CI | Lints all repository shell scripts (`scripts/*.sh`) for errors and bad practices.                                  | `shellcheck`                                               |

### 4. Testing & Coverage

| Target                        | Scope      | Description                                                                                                                                  | Tool(s)          |
|-------------------------------|:----------:|----------------------------------------------------------------------------------------------------------------------------------------------|------------------|
| `just test-help`              | Local      | **Testing and coverage guide.** Displays testing and coverage commands.                                                                      | `just`           |
| `just test`                   | Local & CI | Executes all unit and documentation tests in the workspace.                                                                                  | `cargo-nextest`  |
| `just test-scripts`           | Local & CI | Runs the BATS integration test suite validating shell scripts and the doc engine.                                                            | `bats`           |
| `just coverage`               | Local      | Evaluates test coverage and opens an interactive HTML report in your browser.                                                                | `cargo-llvm-cov` |
| `just test-coverage-headless` | Local      | Standard coverage gate: one workspace run, replayed per crate; enforces Codecov-matching per-crate floors and emits `lcov.info`.             | `cargo-llvm-cov` |
| `just coverage-gate`          | Local & CI | The coverage gate as wired into `verify`: identical to `test-coverage-headless` but suppresses the per-file table (noise in a full run).     | `cargo-llvm-cov` |
| `just test-coverage-strict`   | Local      | Strict audit: runs each crate's suite in isolation (no cross-crate spillover) and enforces the same floors. Slower; not wired into `verify`. | `cargo-llvm-cov` |

### 5. Compliance, Audit & Portability

| Target               | Scope      | Description                                                                                                             | Tool(s)                                         |
|----------------------|:----------:|-------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------|
| `just audit-help`    | Local      | **Compliance diagnostic guide.** Displays audit commands.                                                               | `just`                                          |
| `just audit-all`     | Local      | Runs all local compliance and security audits (vulnerabilities, licenses, and outdated crates).                         | `cargo-deny`, `cargo-machete`, `cargo-outdated` |
| `just secrets-scan`  | Local & CI | Scans the full git history for committed secrets (keys, tokens, private keys). Part of `verify-security`.               | `gitleaks`                                      |
| `just deny`          | Local & CI | Audits licenses, sources, banned crates (e.g. `openssl`), and active RustSec advisories.                                | `cargo-deny`                                    |
| `just machete`       | Local & CI | Detects dead/unused dependencies in workspace manifests.                                                                | `cargo-machete`                                 |
| `just outdated`      | Local      | Lists outdated crate versions in the workspace. Part of local `audit-all` but excluded from CI's `verify` pipeline [3]. | `cargo-outdated`                                |
| `just audit-safety`  | Local      | Scans the facade and dependency tree for `unsafe` code blocks [4].                                                      | `cargo-geiger`                                  |
| `just semver-check`  | Local & CI | Verifies semantic versioning compliance across workspace.                                                               | `cargo-semver-checks`                           |
| `just check-wasm`    | Local & CI | Verifies all `no_std` workspace crates compile cleanly for the `wasm32-unknown-unknown` target.                         | `cargo check`                                   |
| `just msrv-features` | Local      | Verifies MSRV compatibility with no-default and all-features combinations (manual/scheduled check).                     | `cargo check`                                   |
| `just nix-check`     | Local & CI | Validates that the Nix Flake schema, inputs, and outputs evaluate cleanly.                                              | `nix flake check`                               |
| `just bloat [TARGET]` | Local     | Profiles binary/library compile size and lists the largest functions. Defaults to `wasm32-unknown-unknown` if omitted.  | `cargo-bloat`                                   |

[3] Local only non-failing check since outdated dependencies are informational, not a build failure.
[4] Local only non-failing check since `unsafe` code is expected in dependencies and reviewed separately, not a build failure.

### 6. Fuzzing & Performance Benchmarking

| Target                   | Scope      | Description                                                             | Tool(s)       |
|--------------------------|:----------:|-------------------------------------------------------------------------|---------------|
| `just fuzz TARGET`       | Local      | Runs a specific libFuzzer fuzzing harness interactively.                | `cargo-fuzz`  |
| `just fuzz-check TARGET` | Local & CI | Runs a short (10-second) smoke-test compilation check on a fuzz target. | `cargo-fuzz`  |
| `just bench`             | Local      | Runs performance benchmarks for the workspace.                          | `cargo bench` |

### 7. Document Management

#### ADRs, Design Docs, and Guides

| Target                                        | Scope      | Description                                                                 | Tool(s)                 |
|-----------------------------------------------|:----------:|-----------------------------------------------------------------------------|-------------------------|
| `just docs-help`                              | Local      | **Document management guide.** Displays the list of documentation commands. | `just`                  |
| `just adr <title>`                            | Local      | Creates a new Architecture Decision Record using our custom template.       | `scripts/doc-engine.sh` |
| `just adr-rename <old_target> <new_title>`    | Local      | Safely renames an ADR and updates index references.                         | `scripts/doc-engine.sh` |
| `just adr-remove <target>`                    | Local      | Safely removes an ADR interactively.                                        | `scripts/doc-engine.sh` |
| `just verify-adr`                             | Local & CI | Validates formatting and integrity of the ADR ledger.                       | `scripts/doc-engine.sh` |
| `just design <title>`                         | Local      | Creates a new Design Document.                                              | `scripts/doc-engine.sh` |
| `just design-rename <old_target> <new_title>` | Local      | Safely renames a Design Document and updates references.                    | `scripts/doc-engine.sh` |
| `just design-remove <target>`                 | Local      | Safely removes a Design Document interactively.                             | `scripts/doc-engine.sh` |
| `just verify-design`                          | Local & CI | Validates formatting and integrity of the Design Document ledger.           | `scripts/doc-engine.sh` |
| `just guide <title>`                          | Local      | Creates a new User Guide.                                                   | `scripts/doc-engine.sh` |
| `just guide-rename <old_target> <new_title>`  | Local      | Safely renames a User Guide and updates references.                         | `scripts/doc-engine.sh` |
| `just guide-remove <target>`                  | Local      | Safely removes a User Guide interactively.                                  | `scripts/doc-engine.sh` |
| `just verify-guide`                           | Local & CI | Validates formatting and integrity of the User Guides ledger.               | `scripts/doc-engine.sh` |

#### XSD Contract Fragments (`design_docs`)

The `sdmx-types` contract-fragment pipeline keeps each type's verbatim XSD excerpt (rendered under its `## Specification` in the internal docs) in lockstep with the pinned schemas: **materialise → generate → verify**. Only `check-xsd-fragments` is a `verify-docs` gate; `fetch-specs` and `gen-xsd-fragments` are the local materialise and apply steps.

| Target                     | Scope      | Description                                                                                                                      | Tool(s)                  |
|----------------------------|:----------:|----------------------------------------------------------------------------------------------------------------------------------|--------------------------|
| `just fetch-specs`         | Local      | Materialises the pinned SDMX schemas on demand via the Nix FOD, then sha-verifies every file against `sources.toml`; idempotent. | `fetch-specs.sh`         |
| `just gen-xsd-fragments`   | Local      | (Re)generates the sdmx-types XSD contract fragments from `xsd-manifest.toml` for the `design_docs` layer (apply).                | `gen-xsd-fragments.sh`   |
| `just check-xsd-fragments` | Local & CI | (Supports `verify-docs`) Validates the XSD contract fragments are correctly wired into `design_docs`.                            | `check-xsd-fragments.sh` |

### 8. Diagnostics (`just doctor` System)

| Target                  | Scope | Description                                                                                | Tool(s)                       |
|-------------------------|:-----:|--------------------------------------------------------------------------------------------|-------------------------------|
| `just doctor`           | Local | **Developer diagnostic guide.** Displays the list of dedicated health checks.              | `just`                        |
| `just doctor-env`       | Local | Lightweight: quick sanity check of Nix, direnv, compiler toolchain, and Git hooks.         | Custom shell                  |
| `just doctor-devshell`  | Local | Comprehensive: validates that all required flake packages are available in PATH.           | `scripts/doctor-devshell.sh`  |
| `just doctor-nix`       | Local | Validates Nix flake integrity, lockfile age, and features.                                 | `scripts/doctor-nix.sh`       |
| `just doctor-direnv`    | Local | Verifies direnv trust status, environmental exports, and shell loading.                    | `scripts/doctor-direnv.sh`    |
| `just doctor-git`       | Local | Validates local Git configuration, commit signing status (GPG keys), and hooks.            | `scripts/doctor-git.sh`       |
| `just doctor-workspace` | Local | Diagnoses Cargo monorepo structure, member crates, and cyclic deps.                        | `scripts/doctor-workspace.sh` |
| `just doctor-hooks`     | Local | Verifies pre-commit hook setup and runs a validation test.                                 | `scripts/doctor-hooks.sh`     |
| `just doctor-toolchain` | Local | Confirms active compiler versions (MSRV check) and cargo tool availability.                | `scripts/doctor-toolchain.sh` |
| `just doctor-quick`     | Local | Runs a fast compiler sanity check, clippy run, and test suite execution.                   | `scripts/doctor-quick.sh`     |
| `just doctor-ci`        | Local | Compares active local environmental checks with CI pipeline constraints.                   | `scripts/doctor-ci.sh`        |
| `just doctor-docs`      | Local | Verifies document ledger structure, link health, and cross-references.                     | `scripts/doctor-docs.sh`      |
| `just doctor-monorepo`  | Local | Audits version string parity and package consistency across crates.                        | `scripts/doctor-monorepo.sh`  |
| `just doctor-forge`     | Local | Validates forge (GitHub) governance configuration against the declared forge spec.         | `scripts/doctor-forge.sh`     |
| `just doctor-registry`  | Local | Validates registry (crates.io) Trusted Publishing configuration against the registry spec. | `scripts/doctor-registry.sh`  |

> [!NOTE]
> **Bootstrap tools are runbook-only, by policy.** Routine maintainer operations
> (`update-deps`, `update-flake`) are `just` recipes and appear in this table.
> The guarded *one-shot* bootstrap — `scripts/forge-apply.sh` (writes live forge
> config) — and `scripts/registry-tp.sh` (prints crates.io Trusted Publishing
> commands) are deliberately **not** recipes and are documented in their runbooks
> ([forge-setup.md](../project/forge-setup.md),
> [registry-setup.md](../project/registry-setup.md)) instead, so an irreversible
> setup act is never one tab-completion away.
>
> Post-bootstrap forge updates (`just update-rulesets`, `just update-labels`,
> `just update-actions-allowlist`) **are** recipes — they are idempotent, safe to
> re-run, and apply only to the three surfaces that change over the repo's
> lifetime. The read-only `doctor-*` diagnostics stay on the discoverable recipe
> surface alongside them.

### Adding a GitHub Action

When a workflow needs a new third-party action (`uses: org/name@<sha>`), two
obligations must be met **in the same commit**:

1. **SHA-pin it.** Every `uses:` reference must pin to a full 40-hex commit SHA,
   not a tag or branch. `actionlint` (run by `just test-workflows`) enforces this
   and will fail CI if the pin is absent.

2. **Add it to the actions allowlist.** The repository uses
   `allowed_actions=selected` as a supply-chain control (see
   [forge-setup.md](../project/forge-setup.md#security-settings)). Any `uses:`
   reference not covered by `forge/github/actions-allowlist.json` will cause
   `just doctor-forge` to FAIL with a prescriptive message. Add the pattern:

   ```json
   "org/name@*"
   ```

   to the `patterns_allowed` array in `forge/github/actions-allowlist.json` in
   the same commit as the workflow change. `@*` is correct — the *name axis*
   (which-action) is this file's job; the *SHA axis* (which-version) is already
   enforced independently by the pin in the `uses:` line itself.

   See [forge/README.md](../../forge/README.md#actions-allowlist-forgegithubactions-allowlistjson)
   for the full body shape and rationale.

The two controls are orthogonal defence-in-depth:

| Control                                | Axis                         | Enforced by                        |
|----------------------------------------|------------------------------|------------------------------------|
| `sha_pinning_required`                 | Which *version* of an action | `actionlint` + forge spec          |
| `allowed_actions=selected` + allowlist | Which *action* at all        | `doctor-forge` crosscheck + GitHub |

> [!WARNING]
> Skipping step 2 will not break CI immediately (the allowlist is checked by
> `doctor-forge`, not by CI itself). However, when `forge-apply` has been run
> and `allowed_actions=selected` is live on the repo, any unlisted `uses:` makes
> **CI refuse to run the workflow entirely** — a silent, hard-to-diagnose failure.
> Add the allowlist entry in the same PR as the action.

### 9. Maintenance

> [!NOTE]
> The recipes in this section are **maintainer-only** operations: they require forge write access, mutate lockfiles or live configuration, or govern the repository's ongoing health obligations. They are not part of the normal contributor workflow and are intentionally excluded from `just verify`.

| Target                                  | Scope      | Description                                                                                                                                     | Tool(s)                               |
|-----------------------------------------|:----------:|-------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------|
| `just maintain-help`                    | Local      | **Maintenance diagnostics guide.** Displays maintenance and dependency management commands.                                                     | `just`                                |
| `just update-deps *CRATES`              | Maintainer | Refreshes `Cargo.lock` (semver-compatible), validates via `verify-rust`. Updates, no commit [5].                                                | `cargo update`                        |
| `just update-flake`                     | Maintainer | Refreshes `flake.lock` Nix inputs, validates via `verify-infra`. Updates, leaves unstaged, no commit [5].                                       | `nix flake update`                    |
| `just update-msrv <old_ver> <new_ver>`  | Maintainer | Raises or lowers the Minimum Supported Rust Version across files and manifests.                                                                 | `scripts/update-msrv.sh`              |
| `just update-specs <edition> <ref>`     | Maintainer | Re-pins an SDMX schema edition into `specs/sources.toml` (resolves the tag → commit, captures the NAR hash and per-file shas; TOFU). No commit. | `scripts/update-specs.sh`             |
| `just update-rulesets`                  | Maintainer | Applies spec rulesets to the live forge (POST to create, PUT to update by name). Idempotent.                                                    | `scripts/update-rulesets.sh`          |
| `just update-labels`                    | Maintainer | Applies spec labels to the live forge (PATCH to update, POST to create). Does not delete or rename.                                             | `scripts/update-labels.sh`            |
| `just update-actions-allowlist`         | Maintainer | Pushes the committed actions allowlist to the live forge (single PUT).                                                                          | `scripts/update-actions-allowlist.sh` |

[5] **Updates a lockfile but does not commit.** The bump lands in the working tree, left **unstaged**, so you can review the diff and commit a GPG-signed checkpoint manually — mirroring `changelog-generate`. Both targets also refuse to run if the lock is already dirty, so the resulting diff is solely that run's. This preserves the pinned-baseline contract: a lockfile change is a reviewed event, not an automatic side effect. See [Refreshing Pinned Dependencies](#refreshing-pinned-dependencies).

### 10. Release Pipeline

| Target                                     | Scope      | Description                                                                                                                                          | Tool(s)                          |
|--------------------------------------------|:----------:|------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------|
| `just release-help`                        | Local      | **Commit and release pipeline guide.** Displays release workflow commands.                                                                           | `just`                           |
| `just check-changelog`                     | Local & CI | Verifies `CHANGELOG.md` is in sync with git history (used in pre-release checklist).                                                                 | `scripts/check-changelog.sh`     |
| `just prep-release <version>`              | Maintainer | Pre-1.0: bumps every crate to `<version>` and records one signed batch commit (run before cargo release).                                            | `scripts/prep-release.sh`        |
| `just changelog-generate`                  | Maintainer | Generates `CHANGELOG.md` for all crates without committing (review before committing).                                                               | `git-cliff`                      |
| `just release-dry-run *CRATES`             | Local & CI | Simulates a crate release without publishing [6].                                                                                                    | `cargo-release`                  |
| `just release-commit-changelogs`           | Maintainer | Commits generated changelogs as a signed checkpoint before cargo release.                                                                            | `git`                            |
| `just new-release-notes <version>`         | Maintainer | Scaffolds `crates/sdmx-rs/release-notes/<version>.md` from the template; refuses to overwrite a curated file [7].                                    | `scripts/new-release-notes.sh`   |
| `just check-release-notes <version>`       | Maintainer | Gate: the curated `release-notes/<version>.md` exists, has every required section, and retains no template guidance [7].                             | `scripts/check-release-notes.sh` |
| `just prepublish-check`                    | Maintainer | Validates all crates will publish successfully in topological order (dry-run); also runs check-release-notes.                                        | `cargo publish --dry-run`        |
| `just release-merge`                       | Maintainer | Merges release branch to main with auto-generated commit message listing released crates.                                                            | `scripts/release-merge.sh`       |
| `just stage-merge <version>`               | Maintainer | Pushes merge commit to a `staging-release-sdmx-rs-<version>` branch, then polls the GitHub CI Quality Gate until green or failed/timed out [8].      | `scripts/release-stage.sh`, `gh` |
| `just release-push <version>`              | Maintainer | Fast-forwards main, pushes per-crate release tags (triggering publish.yml), and cleans up the staging branch. Run after `stage-merge` reports green. | `scripts/release-push.sh`        |

[6] Runs automatically in CI as part of `verify`; also manually invoked by maintainers in the release workflow (see releasing.md). Accepts zero or more crate names (defaults to all crates if omitted).
[7] **Curated facade release notes (mandatory pre-tag).** The five `CHANGELOG.md` files are the machine record (strict `git-cliff`, never hand-edited); the facade's user-facing prose lives in a separate curated `crates/sdmx-rs/release-notes/<version>.md` that drives its GitHub Release body (the Release title is a plain `sdmx-rs v<version>`). Scaffold it with `new-release-notes`, curate every section, then `check-release-notes` enforces — before a facade release is cut — that the file exists, carries all required sections, and retains no unedited template guidance. The gate is folded into `prepublish-check` and must pass before `cargo release --execute` (the tag push is irreversible). See [releasing.md](../project/releasing.md) §1 and [design 0004](../design/0004-release-publish-pipeline-and-supply-chain-provenance.md) §9.
[8] **Staged CI gate (GitHub-scoped).** The poll uses `gh api repos/<owner>/<repo>/commits/<sha>/check-runs` and requires `gh auth login`. If `gh` is unauthenticated or the origin remote is not a `github.com` URL (e.g. a Codeberg-only clone), the poll is skipped with a warning and the manual hint `just release-push <version>` is emitted instead — the staging branch is still pushed. Exits 1 immediately on any check conclusion of `failure`, `cancelled`, or `timed_out`; exits 1 after 24 attempts (12-minute ceiling) if no terminal state is reached. `RELEASE_STAGE_MAX_ATTEMPTS` and `RELEASE_STAGE_POLL_INTERVAL` override the defaults for testing.
