# Contributing to sdmx-rs

Contributions are welcome! This project is currently in active early development under a single maintainer, so please open an issue before starting any significant work to avoid duplication and ensure alignment with the roadmap.

Please note that this project is governed by our [Code of Conduct](CODE_OF_CONDUCT.md).
By participating you agree to abide by its terms.

## Table of Contents

- [Onboarding Quickstart](#onboarding-quickstart-clone--setup--build)
- [Workflow](#workflow)
  - [1. Issue First](#1-issue-first)
  - [2. Architecture Decision Records (ADRs)](#2-architecture-decision-records-adrs)
  - [3. Branch Strategy](#3-branch-strategy)
  - [4. Commit Requirements](#4-commit-requirements)
  - [5. Scaffolding Dependency Policy](#5-scaffolding-dependency-policy)
  - [6. Pull Requests](#6-pull-requests)
- [Local Quality Gates](#local-quality-gates)
  - [Dependency Audit Checks](#dependency-audit-checks)
  - [Coverage Validation](#coverage-validation)
- [Testing Conventions](#testing-conventions)
- [Documentation Standards](#documentation-standards)
- [Deprecation & Breaking Changes](#deprecation--breaking-changes)
- [Merge Policy](#merge-policy)
- [Maintainer Merge Protocol](#maintainer-merge-protocol)
- [Release Workflow](#release-workflow)
- [MSRV Policy](#msrv-policy)
- [License](#license)

## Onboarding Quickstart (Clone → Setup → Build)

Follow this step-by-step walkthrough to set up your local development environment and run your first build:

### 0. Install Prerequisites

This repository uses a **Nix Flake** and **`direnv`** workflow to provide a fully deterministic toolchain across Linux and macOS. Before cloning, ensure you have:

1. **Nix** with Flakes enabled (`experimental-features = nix-command flakes` in your Nix configuration)
2. **`direnv`** hooked into your active shell

This activates the toolchain defined in `flake.nix`, pinned by `flake.lock`. No manual `rustup` management is required. This environment provides the stable Rust compiler alongside a nightly `rustfmt` to enforce strict formatting rules.

> **Alternative**: If you prefer not to use `direnv`, you can activate the Nix environment directly with `nix develop`. You'll need to re-run this when entering the repo directory — `direnv` automates this.

If you hit problems at any point during setup, see [Common Setup Issues](#common-setup-issues) at the end of this quickstart.

### 1. Clone and Authorize Environment
Clone the repository and enter the directory. `direnv` will automatically detect the configuration and prompt you to authorize the Nix Flake:
```bash
git clone git@github.com:dgalbraith/sdmx-rs.git
cd sdmx-rs
direnv allow
```
*(Entering the directory triggers Nix to evaluate the flake. The first time you run this, Nix will download and provision the pinned compilers. Subsequent entries are instantaneous).*

### 2. Verify Your Active Environment
Verify that Nix has successfully provisioned the exact workspace toolchain:

> [!WARNING]
> Do **not** run `rustup override` or try to manage compilers manually. The Nix flake automatically provisions and overrides toolchains natively to guarantee environment parity.

Run the following commands to check that the Nix-provisioned stable compiler is active:
```bash
cargo --version
rustc --version
```
Both should report the version declared in `rust-toolchain.toml`. If they don't, re-run `direnv allow` to trigger the Nix environment reload.

> [!NOTE]
> **Nix Sandbox Requirement**: Flake evaluations require all workspace files to be Git-staged (`git add`). Newly added crates or configuration changes must be staged before running `nix develop` or `nix flake check`. The Nix sandbox copies only Git-tracked files to guarantee reproducibility; untracked workspace members (such as new `Cargo.toml` files) will cause build failures inside the sandbox.

### 3. Install Pre-Commit Hooks
Register the cryptographically pinned pre-commit quality gates natively with your local Git repository:
```bash
just hook-install
```

> [!TIP]
> Prefer a single command? `just setup` runs `hook-install` and then `just doctor-env` to confirm your environment in one idempotent step. It is safe to re-run after pulling tooling changes.

This installs the following security and quality check hooks to run automatically within your local Git hooks pipeline:

| Hook Suite            | Tool / Checks                                                                                                                                                   | Stage        | Objective                                                                                                                                                           |
|-----------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|:------------:|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Code Sanitisation** | `check-yaml`, `check-json`, `check-added-large-files`, `check-case-conflict`, `check-symlinks`, `end-of-file-fixer`, `trailing-whitespace`, `mixed-line-ending` | `pre-commit` | Enforces uniform whitespaces, validates YAML/JSON syntax, blocks bloated binary commits, prevents cross-platform filesystem case conflicts, and ensures LF endings. |
| **Security Guard**    | `detect-private-key` *(via core hooks)*, `gitleaks`                                                                                                             | `pre-commit` | Scans all staging changes to block GPG/SSH private keys or secrets from leaking into the commit.                                                                    |
| **Commit Linting**    | `commitlint`                                                                                                                                                    | `commit-msg` | Validates commit messages against Conventional Commit rules to guarantee flawless `git-cliff` changelog generation.                                                 |
| **Workspace Gate**    | `just verify`                                                                                                                                                   | `pre-push`   | Decoupled compile/lint/test suite run; executes automatically during `git push` to keep local commits instantaneous.                                                |


### 4. Execute Your First Build and Verification
Confirm that your local setup is fully functional by running the unified quality gate:
```bash
just verify
```
This will compile the workspace, run unit tests, audit licenses, check target WebAssembly compatibility, and run the linter. Once this passes cleanly, your development environment is 100% operational!

> **Note**: The official CI pipeline additionally runs `cargo llvm-cov` in place of standard `cargo test` to combine test execution with dynamic coverage instrumentation, then uploads the result to Codecov. To run the same gate locally (enforces per-crate floors, prints the coverage table), use `just test-coverage-headless`. `just coverage` opens an interactive HTML report and does not enforce floors.

### 5. Local Nix Scratch Testing (`scratch/flake.nix`)

For advanced developers testing custom Nix overlays, packages, or compiler adjustments before integrating them into the main `flake.nix`, a dedicated scratchpad flake is provided inside the `scratch/` directory:

```bash
# Evaluate the scratchpad flake in a temporary shell
nix develop path:./scratch
```

> **Note**: The `scratch/flake.nix` file is explicitly registered in the `.gitignore` allow-list to ensure it is tracked by Git, but it is purely a local developer scratch tool and is never evaluated in the official CI pipelines.

### Common Setup Issues

| Problem                               | Cause                              | Solution                                                                                                               |
|---------------------------------------|----------------------------------- |------------------------------------------------------------------------------------------------------------------------|
| `command not found: nix`              | Nix not installed                  | Install from [nixos.org](https://nixos.org/download.html) (NixOS 23.11 or later recommended)                           |
| `error: experimental Nix features...` | Flakes not enabled in Nix config   | Add `experimental-features = nix-command flakes` to `~/.config/nix/nix.conf`, then reload your shell                   |
| `flake.lock is out of date`           | Dependencies changed upstream      | Run `nix flake update && direnv reload`                                                                                |
| `direnv: command not found`           | direnv not installed               | Install direnv ([direnv.net](https://direnv.net/)) or use `nix develop` directly each time                             |
| `direnv not activating on cd`         | Shell hook not configured          | Add direnv hook to your shell config (`.bashrc`, `.zshrc`, etc.)—see [direnv setup](https://direnv.net/docs/hook.html) |
| `error: Nix sandbox: file not found`  | Untracked files in workspace       | Run `git add` to stage new files before `nix develop` or `direnv reload`                                               |
| `just hook-install failed`            | Pre-commit dependencies missing    | Ensure `nix develop` is active, then retry                                                                             |
| `cargo/rustc version mismatch`        | Nix environment not active         | Run `direnv allow` (if using direnv) or `nix develop` (direct invocation)                                              |

### Nix Requirement

Nix is a mandatory prerequisite to develop, build, or verify this repository. This guarantees that all developers and the CI runner operate on the exact same compiler version, linters, linkers, and test runtimes, eliminating any class of environment-induced compiler/tooling drift. Building or testing the codebase without Nix is unsupported, as non-deterministic environments will trigger CI verification failures. For developers on Windows, WSL2 (Windows Subsystem for Linux) provides a fully compatible environment to run Nix natively.

## Workflow

> [!TIP]
> For a complete, step-by-step example of a successful development cycle (from issue creation to commit and Pull Request), see the [Developer Workflow Exemplar](docs/dev/workflow.md).

### 1. Issue First

Every change — feature, fix, chore, or documentation update — must have a corresponding issue opened before work begins. The issue is the authoritative record of intent and must be referenced in your PR description to connect the code changes. Issues labelled `good first issue` are pre-triaged and highly suitable for first-time contributors looking to get familiar with the codebase!

Please open an issue before starting any significant work to avoid duplication and ensure alignment with the roadmap. Check [ROADMAP.md](ROADMAP.md) for planned phases and current focus areas — issues aligned with the current development phase will be prioritised for discussion.

Structured templates are provided for both **Bug Reports** and **Feature Requests** on GitHub to help compile details (including environment details and authoritative SDMX specification references). Please select the appropriate template when filing new issues.

> **Warning**: For security vulnerabilities, do not open a public GitHub issue. Instead, please follow the secure disclosure process outlined in [SECURITY.md](SECURITY.md).

### 2. Architecture Decision Records (ADRs)

For any significant structural changes, new dependencies, or pattern shifts, we use **Architecture Decision Records (ADRs)** to document the "why". ADRs must conform to the MADR (Markdown Any Decision Records) format as specified in [ADR-0001](docs/adr/0001-record-architecture-decisions.md).

To propose an architectural decision:
1. Run `just adr "Your Decision Title"` in the terminal (this uses a custom script that enforces the MADR template).
2. Fill out the generated Markdown template in `docs/adr/`.
3. Submit the ADR alongside your PR (or in its own PR for early discussion).

> **Note**: Use `just adr` rather than `adr new` directly to ensure your ADR conforms to the MADR hybrid template required by this workspace.

### 3. Branch Strategy

Always **branch from `main`** and **target `main`** when submitting a Pull Request.

Branch names must use a clean, descriptive slug to describe the change. Under our naming convention, **issue numbers are omitted from branch names** to keep slugs short and readable:

```
feat/sdmx-structure-parser
fix/xml-namespace-handling
chore/ci-cache-tuning
```

Direct commits to `main` are blocked by branch protection rules; all changes must land on `main` via an approved Pull Request passing the automated quality gates.

### 4. Commit Requirements

All commits must be **GPG-signed** (either by passing the `git commit --gpg-sign` flag or the short-form equivalent `git commit -S` or, preferably, by enabling Git's global automatic signing/autosign configuration so you do not have to pass the flag manually).

If you have not configured GPG commit signing on your local system, follow these quick steps to get set up:

1. **Generate a GPG Key**: Run `gpg --full-generate-key` and follow the prompts:
   - Select key type: `ECC and ECC` (usually option `9`).
   - Select elliptic curve: `Curve 25519` (default/option `1`, providing modern **Ed25519** for signing and **Cv25519** for encryption).
   - Ensure the email address matches your local Git configuration.
2. **Retrieve the Key ID**: Run `gpg --list-secret-keys --keyid-format=long` and identify your key ID (the 16-character string following `sec ed25519/`, e.g., `3AA5C34371567BD2`).
3. **Configure Git**: Register the key and enable automatic signing globally so that your commits are signed automatically without needing to pass the `--gpg-sign` flag:
   ```bash
   git config --global user.signingkey 3AA5C34371567BD2
   git config --global commit.gpgsign true
   ```
4. **Multiplexer / Headless Terminal Tip**: If you commit inside a terminal multiplexer (like `tmux`) or over a headless SSH connection, GPG may fail to launch the interactive passphrase prompt. Add the following line to your shell profile (`~/.zshrc`, `~/.bashrc`, or equivalent) so it is set for every new terminal session:
   ```bash
   export GPG_TTY=$(tty)
   ```

#### Branch History vs. Target History

This repository enforces a **Merge Commit** workflow (using `--no-ff` merges) for landing features on `main`.
* **Contributor Signature Role**: Your local GPG signature on your feature branch commits is strictly required to verify the provenance and authenticity of your code.
* **The Final Merge**: When your PR is approved, the maintainer will merge your branch into `main` with a standard GPG-signed merge commit. Unlike a squash-and-merge, this preserves your individual commits and GPG signatures in the permanent Git graph, establishing a complete, unbroken cryptographic chain of custody.

For comprehensive troubleshooting (including registering your public key on your GitHub account or using modern SSH-key-based signatures instead), please consult the [Official GitHub Signature Verification Guide](https://docs.github.com/en/authentication/managing-commit-signature-verification).

This repository uses [`git-cliff`](https://github.com/orhun/git-cliff) for changelog generation and [`cargo-release`](https://github.com/crate-ci/cargo-release) for version management. All commit messages must follow the [Conventional Commits specification](https://www.conventionalcommits.org/):

| Prefix                                            | Changelog section | Semantic intent       |
|---------------------------------------------------|-------------------|-----------------------|
| `feat(scope): ...`                                | Features          | New capability        |
| `fix(scope): ...`                                 | Bug Fixes         | Defect correction     |
| `feat!(scope): ...` / `fix!(scope): ...`          | Features          | Breaking API change   |
| `perf(scope): ...`                                | Performance       | Observable speed gain |
| `chore:`, `docs:`, `refactor:`, `style:`, `test:` | Maintenance       | No API change         |

> **Note**: Version bumps are driven manually via `cargo release`. The commit type provides semantic intent for the maintainer to select the appropriate bump (`patch`, `minor`, or `major`) — it does not automatically trigger a bump. Standard semver convention applies: `feat` → MINOR, `fix` → PATCH, breaking change (`!`) → MAJOR. During the pre-1.0 phase, conservative bumps are used: features and breaking changes accumulate as patch increments until Phase 5 stabilisation.

The `scope` is informational — it appears in changelog entries and `git log` to attribute changes to their area of the codebase. Use the affected crate's scope (`types`, `parsers`, `client`, `writers`) or `facade` (specifically for the facade crate at `crates/sdmx-rs`) for crate-specific work, or a descriptive term for cross-cutting changes (e.g., `ci`, `docs`, `deps`). The `facade` scope clearly distinguishes facade-specific changes from global repository-wide modifications. The authoritative list of permitted scopes is defined in [`commitlint.config.cjs`](commitlint.config.cjs); the names here are illustrative.

> [!NOTE]
> **Cross-Crate Commit Scopes**: When a commit spans multiple workspace crates (for example, introducing a core structural type in `sdmx-types` that requires a serialisation update in `sdmx-parsers`), apply the scope of the lowest-level affected crate in the dependency chain (e.g., `feat(types): ...`). If the change uniformly changes the high-level API entry point without isolated low-level additions, use the `facade` scope.

Examples of valid commit messages:
* `feat(types): add codelist representation`
* `fix(parsers): resolve xml payload clipping`
* `chore(ci): pin nix actions to SHA`

#### Closing Issue References

When your commit resolves or addresses an open issue, include the issue reference at the bottom of your commit message body. To maintain a portable, forge-agnostic Git history (e.g., when mirroring to Codeberg), observe the following rules:

1.  **Target the Issue ID only**: Always reference the underlying **Issue ID** (e.g., `#1`) rather than the platform-specific Pull Request ID.
2.  **Use Semantic Keywords**:
    *   Use **`Fixes #ISSUE_ID`** for commits correcting bugs or defects.
    *   Use **`Closes #ISSUE_ID`** for commits completing features, chores, or tasks.

Example of a complete commit message:
```text
feat(types): add codelist representation

Introduces the core structure and serialisation traits for SDMX codelists.

Closes #1
```

> [!NOTE]
> The maintainer uses a third keyword, `Resolves #ISSUE_ID`, exclusively in the final merge commit when integrating a branch onto `main`. You will see this in the Git history when your PR is merged — it is not a keyword contributors need to use.

### 5. Scaffolding Dependency Policy

During Phase 0, crates contain stub implementations. To prevent `cargo machete` from flagging legitimate-but-unused dependencies, register ignored entries in `[package.metadata.cargo-machete]` within each `Cargo.toml`.

Every ignored dependency **must** include a comment:

```toml
[package.metadata.cargo-machete]
ignored = [
  "serde",             # Phase 1: serialisation trait derives and type annotations
  "tokio",             # Phase 3: async runtime for HTTP client
  "wasm-bindgen-test", # PERMANENT: platform-specific (cfg(wasm32))
]
```

| Comment               | Meaning                                                                    |
|-----------------------|----------------------------------------------------------------------------|
| `Phase N: <reason>`   | Remove from the ignored list when you write code that uses this dependency |
| `PERMANENT: <reason>` | Platform-specific or dev-only; never removed                               |

`scripts/check-scaffolding.sh` (run as part of `just verify`) enforces that every entry has a comment and that Phase N entries are consistent with the crate's declared dependencies.

### 6. Pull Requests

**Requirements**:

- **Single Intent**: One logical change per PR. Do not bundle unrelated fixes.
- **PR Title Discipline**: **The title of your Pull Request must follow the Conventional Commits specification** (e.g., `feat(types): add codelist representation`). The PR title (and individual branch commits) will be used to drive `git-cliff` changelog generation.
- **Issue Reference**: Every PR must reference its issue (`Closes #N` or `Fixes #N` in the description).
- **Documentation**: All public items added or modified must carry `///` doc comments. The workspace enforces `missing_docs` as a warning — CI treats warnings as errors.
- **Verification Gate**: Every PR must pass the unified `just verify` gate locally before submission. *Note: In PR contexts, overdue maintenance obligations (`check-maintenance.sh`) are demoted to non-blocking warnings so that expired deadlines do not block external contributions.*
- **CI Readiness**: Your PR is **not ready for review** until all CI checks pass. GitHub branch protection requires CI to pass before merge. Monitor the CI status in your PR — all checks must show ✅ before requesting maintainer review. For detailed information about each CI check, when it runs, and how to interpret failures, see [docs/project/ci-gating.md](docs/project/ci-gating.md).
- **Coverage**: The CI pipeline enforces crate-specific coverage thresholds (e.g., 85% for `sdmx-types`, 75% for `sdmx-parsers`) via `cargo llvm-cov`. Run `just coverage` locally to verify before submitting. If the gate flags boilerplate or non-essential paths, the threshold can be adjusted. See [docs/dev/testing.md](docs/dev/testing.md) for details.

**Style Checklist**:

Before submitting, verify your code adheres to the style guide:

- [ ] Code formatted with `just fmt`
- [ ] Naming follows conventions (functions/modules: snake_case, types: CamelCase)
- [ ] All public items have rustdoc comments (`///`)
- [ ] Doc comments follow the [rustdoc conventions](docs/dev/rustdoc.md) (the public/`design_docs` split, `## Specification` citations, heading and example idioms)
- [ ] Rustdoc examples compile (`cargo test --doc`)
- [ ] Comments explain WHY, not WHAT
- [ ] No unnecessary comments or commented-out code
- [ ] Error types defined in `error.rs` using `thiserror`
- [ ] Imports organised (stdlib → external → internal) and not redundant
- [ ] Private by default; `pub` used intentionally
- [ ] Safety comments (`// SAFETY:`) explain invariants for all `unsafe` blocks

See [docs/dev/practices.md](docs/dev/practices.md) for detailed guidance.

* **Post-Merge Cleanup**: Once the maintainer approves and merges your PR, the branch on `main` will be updated. You can safely clean up your local fork repository by pulling the latest changes and deleting your feature branch:
```bash
  git checkout main
  git pull origin main
  git branch -d feat/your-feature-branch
```
(Tip: You can also safely delete the branch on your GitHub fork UI once the PR is marked as merged).

### Code Review Philosophy & Standards

For comprehensive code review standards, reviewer priorities, and self-review checklist, see [docs/project/reviews.md](docs/project/reviews.md).

## Local Quality Gates

To ensure the continuous integration pipeline passes cleanly, run the unified verification gate locally before pushing:

```bash
just verify
```

This runs the same validation suite as CI and must execute inside the active `direnv` environment. The gate coordinates five verification domains:

1. **`just verify-rust`** — Formatting, linting, tests, coverage, semver, and security audit. *(MSRV verified implicitly by clippy on default features.)*
2. **`just verify-scripts`** — Shell script linting and BATS test suite.
3. **`just verify-docs`** — Commit message format, markdown, ADRs, design docs, and links.
4. **`just verify-infra`** — Nix flake and GitHub Actions workflow validation.
5. **`just verify-maintenance`** — Maintenance obligation tracking and scaffolding validation.

**Optional**: Developers working on optional features should run `just msrv-features` to verify MSRV (1.92.0) compatibility across feature combinations. This is also checked on schedule in CI.

**Key principle**: If `just verify` passes locally, CI will pass. If any check fails locally, fix it before pushing.

For detailed information about each check and how to debug failures, see [docs/project/ci-gating.md](docs/project/ci-gating.md).

> [!TIP]
> To quickly discover commands or run specific subsets of checks, use the interactive self-discovery menus:
> * `just lint-help` 🧹 — Code quality, style, and formatting guide (run `just lint` for a fast local pre-commit gate).
> * `just audit-help` 🔒 — Security, vulnerability, and license audit guide (run `just audit-all` to run all audits).
> * `just doctor` 🏥 — System health, compiler toolchain, and env diagnostics guide.
> * `just docs-help` 📝 — Architecture decision records, design docs, and user guide manager.
> * `just maintain-help` 🔧 — Release, changelog, and repository maintenance manager.

**For a complete reference of all available `just` recipes and tooling, see [docs/dev/tooling.md](docs/dev/tooling.md).**

### Dependency Audit Checks

The supply chain audit (step 5 above) is enforced via `cargo-deny` and `cargo-machete` - any issues must be resolved before merge. For detailed policy and resolution steps, see [SECURITY.md § Supply Chain Security](SECURITY.md#supply-chain-security).

### Coverage Validation
The coverage check (step 6 above) enforces crate-specific coverage thresholds (ranging from 75% to 85% per crate) via `cargo llvm-cov`. To validate your changes locally before submission, run:
```bash
just coverage
```

#### Running Checks Locally

```bash
# Check dependency vulnerabilities, licenses, and bans
just deny

# Check for unused dependencies
just machete

# Or run both together (recommended before submitting PR)
just verify
```

#### Resolving Violations

If either check fails, follow the resolution steps below. Violations **block merge** and must be resolved before submitting a PR.

#### 1. RustSec Advisory Violation

CI output: `error: X security advisories found`

- **Preferred**: Upgrade the affected dependency to a patched version
  ```bash
  cargo update -p vulnerable_crate
  ```
- **If unavoidable**: Document the accepted risk in [deny.toml](deny.toml):
  ```toml
  [advisories]
  ignore = [
    { name = "crate-name", id = "RUSTSEC-2026-XXXX", reason = "Low severity; patched version incompatible with MSRV. Upgrade in Phase N." }
  ]
  ```

#### 2. License Violation

CI output: `error: unknown license` or `deny: disallowed license`

- **Preferred**: Remove the dependency if possible
- **If required**: Add the license to the whitelist in [deny.toml](deny.toml) only if it aligns with project policy (rarely approved):
  ```toml
  [licenses]
  allow = [
    "MIT",
    "Apache-2.0",
    # ... (add new license here with discussion in PR)
  ]
  ```

#### 3. Banned Crate Violation

CI output: `deny: explicit ban found`

- **Policy**: No exceptions. Banned crates (`native-tls`, `openssl`, `openssl-sys`) cannot be added
- **Resolution**: Use the approved alternative (e.g., `rustls` instead of `openssl`) or remove the dependency entirely
- **Rationale**: See [ADR-0013](docs/adr/0013-use-rustls-over-native-tls-for-transport-layer-security.md) for the rustls-only decision

#### 4. Unknown Registry Violation

CI output: `deny: unknown registry` or `deny: unknown git source`

- **Policy**: Dependencies must come from crates.io registry only
- **Resolution**: Either remove the dependency or find an equivalent crate published to crates.io
- **Exception process**: Rare; requires ADR discussion and deny.toml allow-list update

#### 5. Unused Dependency Warning (cargo-machete)

CI output: `warning: unused package` in `Cargo.toml`

During Phase 0, legitimate unused dependencies are expected:

- **If intentional (Phase N scaffolding)**: Document in `[package.metadata.cargo-machete]` with a phase comment:
  ```toml
  [package.metadata.cargo-machete]
  ignored = [
    "tokio",            # Phase 3: async runtime for HTTP client
    "reqwest",          # Phase 3: HTTP client library
  ]
  ```
- **If accidental**: Remove the dependency from `Cargo.toml`
- **When implementing the dependency**: Remove it from the ignored list so `cargo machete` can verify ongoing usage

For detailed scaffolding policy, see [CONTRIBUTING.md § Scaffolding Dependency Policy](#5-scaffolding-dependency-policy).

## Testing Conventions

Full testing conventions, coverage expectations, and the testing checklist are documented in [docs/dev/testing.md](docs/dev/testing.md).

**The headline**: unit tests inline in `mod tests`, integration tests in `crates/*/tests/`, HTTP mocking via `wiremock` (no real network calls). Coverage thresholds are enforced per-crate via `cargo llvm-cov` (floors: `sdmx-types` 85%, `sdmx-parsers` 75%, `sdmx-writers` 80%, `sdmx-client` 80%, `sdmx-rs` 70%; global fallback 80% — authoritative values in `codecov.yaml`). PRs that fall below a crate's floor must document the gap. Run `just test-coverage-headless` locally to see the numbers against the same floors CI enforces; `just coverage` produces a local HTML report only.

## Documentation Standards

`sdmx-rs` emphasises documentation that explains **why** decisions are made, not just **that** they were made. See [docs/dev/documentation.md](docs/dev/documentation.md) for the thinking and guidance, and [docs/dev/rustdoc.md](docs/dev/rustdoc.md) for the rustdoc authoring conventions.

## Deprecation & Breaking Changes

### Deprecation Strategy

When an API becomes superseded but must remain available for compatibility, mark it with the `#[deprecated]` attribute in code (see [docs/dev/practices.md](docs/dev/practices.md) for syntax).

**Timeline**:
- Deprecated items must remain functional and documented for **at least one minor version** before removal
- Document the deprecation in the CHANGELOG with a clear migration path
- The next major version may remove the deprecated item without further notice

**Example**:

In version 0.5.0, deprecate `parse_constraint()` in favor of `ConstraintModel::parse()`:

```rust
#[deprecated(since = "0.5.0", note = "use `ConstraintModel::parse()` instead")]
pub fn parse_constraint(input: &str) -> Result<ConstraintModel> {
    ConstraintModel::parse(input)
}
```

In version 0.6.0, the deprecated item remains. Update CHANGELOG to warn removal is coming in 1.0.0.

In version 1.0.0 (major), the deprecated item may be removed entirely.

### Breaking Changes

A breaking change is any modification that requires consumer code to update. Breaking changes require a **major version bump** (following semver).

**What constitutes a breaking change**:
- Removing or renaming public types, functions, traits, modules
- Changing function signatures (parameter types, return type)
- Removing enum variants (adding variants is not breaking; removing is)
- Changing behaviour of existing functions (even if signature stays the same)
- Upgrading MSRV (consumers may be pinned to older compiler versions)

**When breaking changes happen**:
1. Mark them explicitly in the CHANGELOG under a `### Breaking Changes` section
2. Include both the **what** (what changed) and **why** (rationale)
3. Provide a **migration path** (how consumers should update their code)

**Example CHANGELOG entry**:

```markdown
## [1.0.0] - 2026-12-15

### Breaking Changes

- **BREAKING**: Removed `parse_constraint()` in favor of `ConstraintModel::parse()`.
  Migration: Call `ConstraintModel::parse(input)` directly instead of `parse_constraint(input)`.

- **BREAKING**: `ParseError::InvalidXml` now takes `&str` instead of `String`.
  Migration: If you construct this error directly, adjust: `ParseError::InvalidXml(s.to_string())`.

- **BREAKING**: MSRV bumped to 1.92.0 (was 1.91.0).
  Migration: Upgrade your Rust toolchain to 1.92.0 or later, or pin to 0.5.x.
```

## Merge Policy

All changes land on `main` via GitHub Pull Requests. Each PR must:

- **Pass all CI checks** — Automated enforcement; CI failures block merge
- **Receive at least 1 approval** — From the maintainer (see Code Review Philosophy & Checklist above)
- **Use standard merge commits (`--no-ff`)** — Preserves all individual GPG-signed commits in the git history to maintain cryptographic provenance.

The maintainer uses a local merge workflow (`--no-ff`) to maintain the cryptographic integrity of the Git history and ensure every commit on `main` is signed and verifiable.

## Maintainer Merge Protocol

> **Maintainer-only**: The merge protocol, GPG attribution, and version support policy are documented in [docs/project/merging.md](docs/project/merging.md).

## Release Workflow

> **Maintainer-only**: The full release process — pre-release checklist, dry-run, execution, coordinated suite publishing, and failure recovery — is documented in [docs/project/releasing.md](docs/project/releasing.md).

## MSRV Policy

The workspace `rust-version` field in `Cargo.toml` defines the **Minimum Supported Rust Version (MSRV)**. The MSRV will not be raised to a version released less than **6 calendar months ago**, and bumps are **breaking changes** requiring a MAJOR version increment.

For complete policy, upgrade procedures, and troubleshooting, see [docs/project/msrv.md](docs/project/msrv.md).

## License

**Licensing model**: Contributions operate on an *inbound = outbound* basis. By submitting a contribution you confirm that you have the right to do so under the terms of your employer or any relevant agreements, and you accept that your contribution will be licensed under the same terms as this project. No CLA or DCO is required.

This project is licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

### Contribution Licensing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

### Third-Party Code & License Compatibility

When submitting a PR that includes code from other projects or references, ensure the source code's license is compatible with our dual MIT/Apache-2.0 licensing:

**Compatible Licenses**:
- MIT, Apache-2.0, Apache-1.1, ISC, BSD-2-Clause, BSD-3-Clause, 0BSD
- MPL-2.0 (for module-level code, with LICENSE notice)
- LGPL-2.1+, AGPL-3.0+ (only if clearly isolated in a separate binary/optional feature, not in core `sdmx-types`)

**Incompatible Licenses**:
- GPL-2.0, GPL-3.0 (copyleft; requires entire project relicense)
- SSPL, Commons Clause (proprietary restrictions; not open source)
- Custom proprietary licenses

**If your PR includes third-party code**:
1. Document the source in the code comment (URL + license)
2. Verify the license is in the compatible list above
3. If MPL-2.0: include a `COPYING` notice in the file
4. If LGPL: document why it's isolated from core types
5. If incompatible: either remove the code or propose an alternative approach in the PR description

For questions on license compatibility, open an issue before starting work. The maintainer can advise whether an approach is acceptable.
