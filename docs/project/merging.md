# Merge Protocol

Maintainer-only. Documents how approved pull requests and local maintainer branches are merged onto `main` using standard merges with GPG signature preservation. Contributors do not need to read this — see [CONTRIBUTING.md](../../CONTRIBUTING.md) for the contributor workflow.

---

## CI Gates & Branch Protection

**Branch protection rulesets enforce that all commits to `main` are GPG-signed by an authorised maintainer and that its history is append-only (no force-push or deletion).** The CI-green seal on every SHA is a process guarantee upheld by the staging round-trip, not a forge-enforced barrier against a direct push (see the scope note in point 2). Together these are the cryptographic and process safeguards for the zero-trust invariant:

1. **All commits must be GPG-signed**: `required_signatures` is enforced with no bypass — every commit pushed to `main`, including merge commits made by the maintainer, must carry a verified GPG signature.
2. **The staging round-trip carries a CI-green status check to `main`**: The `Zero Trust Gate` ruleset requires the single `CI Quality Gate` aggregate check on the SHA being *merged via a pull request*. It does not gate a direct `git push` to `main`: `required_status_checks` applies to PR merges, not direct ref updates, and CI runs only after a push is accepted. The maintainer therefore upholds the seal by discipline, staging every merge for CI before fast-forwarding `main` (points 3 and 4), backed locally by the advisory `guard-main-push` pre-push hook (`scripts/guard-main-push.sh`), which refuses a push to `main` whose commit is not on a `staging-*` branch. It is an accident guard, bypassable by the maintainer (the root of trust cannot be gated against its own credentials); the guarantees the ruleset *does* enforce for every actor, at ref-update time, are signing (point 1) and append-only history.
3. **Merges happen locally via the staging branch pattern**: All merges to `main` are performed **locally with GPG signatures**, then pushed to a `staging-<slug>` branch for CI verification before being fast-forwarded to `main` (see [Scenario A](#scenario-a-merging-your-own-branch) and [Scenario B](#scenario-b-merging-a-contributors-pr)). The GitHub merge button must not be used — a GitHub-mediated merge produces a web-flow-signed commit, violating the maintainer-signed-only invariant.
4. **CI gates both PRs and the staging branch**: Workflows trigger on PR open/update and on `staging-*` pushes. Status check results are stored against the commit SHA — a SHA whose `CI Quality Gate` is green on `staging-*` carries that result when fast-forwarded to `main`, satisfying the gate without re-running CI.
5. **The single required check is an aggregator**: `CI Quality Gate` (the `ci-gate` job) passes only when every merge-gating job is `success` or legitimately `skipped`. The jobs it covers — and those deliberately excluded, such as the PR-only `semver-check` — are declared in [`forge/github/ci-gating-jobs.json`](../../forge/github/ci-gating-jobs.json) and cross-checked against the workflow by `scripts/verify-ci-gate.sh`. The full list, fail-closed rationale, and exclusions are documented in [ci-gating.md — CI Quality Gate](ci-gating.md#the-ci-quality-gate-aggregator).
6. **Weekly Validation**: The CI pipeline runs a scheduled weekly validation (Saturdays at 00:00 UTC) to ensure the release infrastructure and build system continue to work correctly, independent of new contributions.

See [forge-setup.md](forge-setup.md) for the full ruleset configuration, design rationale, and `gh` API commands.

### Check Design Philosophy

Required checks are scoped to what they validate, avoiding redundant validation. Each check targets a specific domain; checks do not duplicate each other. For example:

- **Infrastructure checks** (`nix-check`) trigger on infra file changes only, not on Rust code changes — Rust validation jobs already run inside `nix develop` and will catch sandbox issues if the Nix environment is broken.
- **Rust validation** (`test-matrix`, `clippy`, `coverage`) runs in the Nix environment and validates both code quality and environment correctness.
- **Formatting and linting** checks are domain-specific (code, docs, scripts) and trigger on relevant file changes.
- **Semantic versioning** checks run on PRs to main to ensure version bumps follow SemVer policy.

**Check Trigger Matrix — Examples of what triggers when files change**:

| Changed Files                      | nix-check  | check-workflows | test-matrix | clippy  | check-formatting | check-docs | check-scripts | msrv-verify | semver-check  |
|------------------------------------|:----------:|:---------------:|:-----------:|:-------:|:----------------:|:----------:|:-------------:|:-----------:|:-------------:|
| `flake.nix`                        |     ✅     |        —        |      —      |    —    |        ✅        |     —      |       —       |      —      |       —       |
| `.github/workflows/ci.yml`         |     ✅     |        ✅       |      —      |    —    |        —         |     —      |       —       |      —      |       —       |
| `crates/sdmx-types/src/lib.rs`     |     —      |        —        |      ✅     |    ✅   |        ✅        |     ✅     |       —       |      ✅     |      ✅*      |
| `Cargo.toml` (dependency update)   |     —      |        —        |      ✅     |    ✅   |        —         |     ✅     |       —       |      ✅     |      ✅*      |
| `rust-toolchain.toml`              |     —      |        —        |      ✅     |    ✅   |        —         |     —      |       —       |      —      |       —       |
| `docs/design/0001-*.md`            |     —      |        —        |      —      |    —    |        ✅        |     ✅     |       —       |      —      |       —       |
| `scripts/check-*.sh`               |     —      |        —        |      —      |    —    |        ✅        |     —      |       ✅      |      —      |       —       |
| `**/*.md`                          |     —      |        —        |      —      |    —    |        —         |     ✅     |       —       |      —      |       —       |
| (Weekly scheduled run)             |     ✅     |        ✅       |      ✅     |    ✅   |        ✅        |     ✅     |       ✅      |      ✅     |       ✅      |

*`semver-check` runs on PRs to main only.
"—" = does not run; "✅" = runs.

---

## Semantic Keywords

We use specific Git trailer keywords to reference and automatically close issue tracking items. These must target the underlying **Issue ID** rather than the platform-specific Pull Request ID. This keeps the Git history portable and forge-agnostic (e.g., when mirroring to Codeberg).

*   **`Fixes #ISSUE_ID`**: Used in bug fix commits to denote resolving a defect/problem.
*   **`Closes #ISSUE_ID`**: Used in chore, feature, or task commits to denote completing a requirement.
*   **`Refs #ISSUE_ID`**: Used in commits that relate to an issue without completing it.
*   **`Resolves #ISSUE_ID`**: Used in the final merge commit to denote the completion and integration of the work addressing that issue.

The four keywords are not interchangeable; they play three distinct roles. `Refs` provides bare issue *linkage* — the cross-reference that wires issue, PR, and commit together in the forge UI for navigability — and nothing more. `Fixes`/`Closes` on the **branch commit and PR** add the *completion claim* to that linkage, and their choice is type-driven (`Fixes` for bugs, `Closes` for chores/features/tasks). `Resolves` on the **merge commit** is the keyword that *indicates closure*: in this repository's local-merge model (no GitHub merge button — see [CI Gates & Branch Protection](#ci-gates--branch-protection)), an issue is only closed automatically by a closing keyword on a commit that lands on `main` — and the commit guaranteed to do that with the right keyword is the maintainer-authored merge commit. In short: `Refs` links; `Closes`/`Fixes` link and *claim* completion; `Resolves` *certifies* integration and closes the issue.

On a **multi-commit branch**, the completion claim belongs to exactly one commit per issue; every other commit links with `Refs`. The two patterns:

*   **Single issue, multiple commits**: intermediate commits carry `Refs #ISSUE_ID`; only the terminal commit that completes the requirement carries the type-driven `Fixes`/`Closes #ISSUE_ID`. The merge commit carries `Resolves #ISSUE_ID` as usual.
*   **Multiple issues under an umbrella issue**: each commit carries the type-driven `Fixes`/`Closes` for its own sub-issue and may additionally carry `Refs` for the umbrella; the merge commit enumerates one `Resolves #ISSUE_ID` per issue it integrates.

> [!IMPORTANT]
> Never include platform-specific Pull Request IDs (e.g., `#2` for PR #2) in your commit or merge messages. Always reference the underlying Issue ID (e.g., `#1`). The associated pull requests will still be automatically tracked and closed when the branch merge is detected.

---

## Remote Configuration

The examples below operate against a remote named `origin` (the conventional default). All tooling that touches the canonical `main` branch — `just check-commits`, `just release-push`, [`scripts/release-merge.sh`](../../scripts/release-merge.sh), and [`scripts/doctor-git.sh`](../../scripts/doctor-git.sh) — resolves this remote from the `SDMX_MAIN_REMOTE` environment variable, defaulting to `origin`. If your clone names the GitHub remote differently, set it once (for example in your shell profile or `.envrc`):

```sh
export SDMX_MAIN_REMOTE=github
```

Otherwise, substitute your remote name in the commands below.

**Mirroring to multiple forges**: maintainers who mirror to Codeberg as well as GitHub typically configure a fan-out push remote named `all` (a single remote with multiple push URLs) and publish with `git push all` rather than pushing to one forge. The worked examples use `git push all` wherever a mirror push is intended.

---

When a branch or PR is ready to merge:

### Scenario A: Merging Your Own Branch
If you are merging your own local branch (e.g., `chore/initialise-workspace-configuration`), your branch is already local, so no fetch is needed before merging.

> [!TIP]
> For a step-by-step example of this flow, see [Appendix A: Bootstrapping Worked Example](#appendix-a-worked-example-repository-bootstrapping--initial-workspace-merge).

1.  **Ensure main is up-to-date**:
    ```bash
    git checkout main
    git pull origin main
    ```

2.  **Merge onto main (No Fast-Forward)**:
    To preserve cryptographic signature provenance, always use `--no-ff` and GPG sign the merge commit. Specify your merge commit message using a heredoc:
    ```bash
    git merge --no-ff --gpg-sign BRANCH_NAME -F - <<'EOF'
    chore(scope): descriptive title

    Detailed description of changes.

    Resolves #ISSUE_NUMBER
    EOF
    ```

3.  **Push merge commit to a staging branch and wait for CI**:
    The Zero Trust Gate on `main` requires CI-green status checks. Push the merge commit to a `staging-<slug>` branch (derived from your branch name with slashes replaced by hyphens) so CI runs against the exact SHA:
    ```bash
    git push origin main:staging-BRANCH_SLUG
    ```
    Monitor the CI run. The staging branch has no branch-protection rules — only the resulting SHA matters. When all required checks are green, proceed.

4.  **Fast-forward `main` and push to all forges**:
    Fast-forward `main` to the staging SHA (which now carries the green seal) and push to both forges. Then clean up:
    ```bash
    git push all main
    git push origin --delete staging-BRANCH_SLUG
    git branch -d BRANCH_NAME
    ```
    GitHub's `delete_branch_on_merge` setting will auto-delete `staging-BRANCH_SLUG` when the PR closes, but deleting it explicitly here keeps the remote clean immediately.

### Scenario B: Merging a Contributor's PR
If you are merging a contributor's branch (which exists on their fork or remote and is not present on your local machine), you must fetch their PR reference first:

> [!TIP]
> For a step-by-step example of this flow, see [Appendix B: Contributor Merge Worked Example](#appendix-b-worked-example-merging-a-contributors-pull-request).

1.  **Fetch the remote PR branch to a local review branch**:
    ```bash
    git fetch origin pull/PR_NUMBER/head:review/BRANCH_NAME
    git checkout review/BRANCH_NAME
    ```

2.  **Verify cryptographic integrity**:
    Verify that the contributor's commits are cryptographically signed:
    ```bash
    git log --show-signature
    ```

3.  **Merge onto main (No Fast-Forward)**:
    ```bash
    git checkout main
    git pull origin main
    git merge --no-ff --gpg-sign review/BRANCH_NAME -F - <<'EOF'
    chore(scope): descriptive title

    Detailed description of changes.

    Resolves #ISSUE_NUMBER
    EOF
    ```

4.  **Push merge commit to a staging branch and wait for CI**:
    The Zero Trust Gate on `main` requires CI-green status checks. Push the merge commit to a `staging-<slug>` branch (derived from the PR branch name with slashes replaced by hyphens) so CI runs against the exact SHA:
    ```bash
    git push origin main:staging-BRANCH_SLUG
    ```
    Monitor the CI run. When all required checks are green, proceed.

5.  **Fast-forward `main` and push to all forges**:
    Fast-forward `main` to the staging SHA and push to both forges. Then clean up:
    ```bash
    git push all main
    git push origin --delete staging-BRANCH_SLUG
    git branch -d review/BRANCH_NAME
    ```

---

## Version Support & Compatibility

We follow semantic versioning across all crates. Current support policy:

- **Active versions**: Within the current major version, all minor releases receive security updates and critical bug fixes
- **EOL timeline**: Previous major version receives security updates for 6 months after a new major release, then EOL
- **Feature flags**: Consumer code should not rely on undocumented feature flags — they may be removed without notice
- **Breaking changes**: Documented in CHANGELOG.md with a migration path

See [ARCHITECTURE.md — Versioning Strategy](../../ARCHITECTURE.md#versioning-strategy) and [ADR-0004 — Decoupled Crate Versioning](../adr/0004-decoupled-crate-versioning-strategy.md) for the technical versioning structure.

---

## Appendix A: Worked Example (Repository Bootstrapping & Initial Workspace Merge)

[↑ Back to Scenario A](#scenario-a-merging-your-own-branch)

Below is an abbreviated walkthrough of the repository's initial bootstrap flow (`Scenario A`), focusing strictly on the Git terminal commands and GPG-signed merge syntax.

> [!NOTE]
> For the full, unabbreviated PR description checklist and commit messages used during this bootstrap, see the [Developer Workflow Exemplar](../dev/workflow.md).

### 1. Branch Creation & Initial Check-in
The feature branch is checked out, and the baseline workspace configuration is committed locally using GPG signing and a multi-line commit message referencing the underlying Issue ID:

```bash
# Checkout bootstrap feature branch
git checkout -b chore/initialise-workspace-configuration

# Commit baseline configuration with GPG signature
git commit --gpg-sign -F - <<'EOF'
chore(repo): initialise workspace configuration

Bootstraps the `sdmx-rs` repository by establishing the complete workspace
layout, Nix environment, and quality gates.

Closes #1
EOF

# Push to origin remote
git push -u origin chore/initialise-workspace-configuration
```

### 2. Pull Request Generation
The Pull Request is generated via the GitHub CLI (`gh`). As the maintainer working their own branch, `--assignee` and `--label` are set at creation time rather than as a post-triage step:

```bash
gh pr create --title "chore(repo): initialise workspace configuration" \
  --assignee "@me" \
  --label "chore" \
  --body-file - <<'EOF'
Delivers the foundational infrastructure for the `sdmx-rs` monorepo by configuring the workspace layout, a deterministic Nix-based development environment, local and CI verification quality gates, forge governance tooling, a signed-and-attested release pipeline, and architectural design documentation.

Bootstrapping the infrastructure as a single unified commit establishes the foundation and ensures that the initial environment configuration, pre-commit hooks, lint rules, and workspace crates compile and satisfy all verification constraints.

## Key Changes

- **Workspace & Cargo Monorepo**: Initialised the root `Cargo.toml` and configured the structural boundaries for the facade (`sdmx-rs`) and subcrates (`sdmx-types`, `sdmx-parsers`, `sdmx-writers`, `sdmx-client`).
- **Toolchain & Nix Environment**: Established `rust-toolchain.toml` and built the `flake.nix` devShell wrapper with cryptographically pinned system dependencies via `flake.lock`, and committed `Cargo.lock` for reproducible builds.
- **Quality Gates & Security**: Embedded compilation quality flags in `.cargo/config.toml`, constrained licences and vulnerabilities via `deny.toml`, configured `cargo-llvm-cov` / Codecov coverage, and built the GitHub Actions workflows (`ci.yml`, `publish.yml`, and `verify-signature.yml`, which uses the committed GPG public keys in `.github/maintainer-keys/` as its trust root). The `publish.yml` pipeline performs per-crate Trusted-Publishing releases with build-provenance and dual (CycloneDX and SPDX) SBOM attestation.
- **Forge Governance**: Established `scripts/forge-apply.sh` (guarded one-shot bootstrap), `scripts/doctor-forge.sh` (continuous spec verification), and `scripts/lib/forge-spec.sh` (machine-readable desired state), backed by committed ruleset and allowlist artifacts under `forge/`.
- **Pretty-Printer Configuration**: Defined `rustfmt.toml` with nightly import ordering (`imports_granularity`, `group_imports`) and doc-comment formatting, establishing the canonical code style for the workspace.
- **Diagnostic Tooling**: Bundled the `scripts/doctor-*.sh` suite, providing automated health checks for environment setup, Git hygiene, Nix configuration, hook integration, and monorepo structure, together with troubleshooting guidance.
- **Maintenance & Release Automation**: Configured `maintenance.toml` to track obligation deadlines and phases; `release.toml` orchestrates per-crate version bumps and signing; automation scripts handle MSRV upgrades, lockfile refreshes (`update-deps`, `update-flake`), and compliance synchronisation.
- **Documentation Infrastructure**: Organised documentation across architecture, design specifications, project operations (release, maintenance, and review procedures), and developer and user guides. The integrated `doc-engine.sh` script manages the ADR, design, and guide lifecycle (creation, renaming, and removal).
- **BATS Integration Tests**: Automated verification of document lifecycle (creation, removal, renaming, and validation), maintenance obligation tracking, monorepo scaffolding, and MSRV upgrade mechanics, together with the release pipeline (changelog generation, prep-release, prepublish, and semver) and the security and signature gates, summarised by a reusable `scripts/run-bats.sh` runner.
- **Hygiene & Hooks**: Established an allow-list `.gitignore`, templates for issue and PR creation, and local git hooks via `pre-commit` (integrating `gitleaks` secret scanning and nightly `rustfmt` rules) and `commitlint`. Set default `.editorconfig`, `.gitattributes`, and `CODEOWNERS` policies.

## Quality Checklist

- [x] I have run the unified quality gate (`just verify` or `nix develop --command just verify`) and all checks pass cleanly.
- [x] I have added doc-comments (`///`) to any new or modified public API items.
- [x] My commits are GPG-signed.

Closes #1
EOF
```

### 3. Non-Fast-Forward Merge, Staging, and Cleanup
The branch is merged back into `main` using standard non-fast-forward merge configurations (`--no-ff`) to preserve GPG signatures, then pushed to a staging branch for CI verification before landing on `main`:

```bash
# Sync with main and pull latest changes
git checkout main
git pull origin main

# Execute GPG-signed non-fast-forward merge.
# The merge commit body is the canonical, detailed record (see note below);
# it is abbreviated here. For the full enumerated message, see the
# Developer Workflow Exemplar — Stage 5.
git merge --no-ff --gpg-sign chore/initialise-workspace-configuration -F - <<'EOF'
chore(repo): initialise workspace configuration

Delivers the initial repository baseline for the `sdmx-rs` monorepo by
establishing the workspace layout, Nix environment, and quality gates.

Key Changes:

- [ ... full enumerated change list — see Stage 5 of workflow.md ... ]

Resolves #1
EOF

# Push to staging branch so CI runs on this exact SHA
git push origin main:staging-chore-initialise-workspace-configuration

# Once CI is green, fast-forward main and publish to every forge
git push all main

# Clean up staging branch and local feature branch
git push origin --delete staging-chore-initialise-workspace-configuration
git branch -d chore/initialise-workspace-configuration
```

> [!NOTE]
> The merge commit — not the lean branch commit — carries the **detailed, enumerated description**. It is the maintainer-authored canonical record on `main`. The branch commit stays lean; the maintainer curates the durable description at merge time. See the [Developer Workflow Exemplar — Stage 5](../dev/workflow.md#stage-5-the-merge-maintainer-integration) for the full message and the rationale.

---

## Appendix B: Worked Example (Merging a Contributor's Pull Request)

[↑ Back to Scenario B](#scenario-b-merging-a-contributors-pr)

Below is a fully worked example of merging an external contributor's Pull Request (`Scenario B`), documenting branch fetching, GPG signature verification, merge structures, and cleanup.

### 1. Fetching the Contributor's Branch
The maintainer fetches the remote PR reference directly into a dedicated local review branch and checks it out:

```bash
# Fetch Pull Request #2 Head to local review branch review/feat-sdmx-dimensions
git fetch origin pull/2/head:review/feat-sdmx-dimensions

# Switch to the review branch
git checkout review/feat-sdmx-dimensions
```

### 2. Verifying Cryptographic Integrity
Before merging, the maintainer inspects the contributor's commit logs to verify all GPG signatures are valid and trusted:

```bash
# Verify commit signatures in the branch log
git log --show-signature
```

### 3. Non-Fast-Forward Merge, Staging, and Cleanup
The maintainer switches to `main`, pulls the latest upstream changes, performs a GPG-signed non-fast-forward merge targeting the review branch, then pushes to staging for CI verification before landing on `main`:

```bash
# Update local main
git checkout main
git pull origin main

# Execute GPG-signed non-fast-forward merge.
# As in Scenario A, the merge commit body is the detailed canonical record
# the maintainer authors — the contributor's branch commit stays lean. It is
# abbreviated here; see the Developer Workflow Exemplar — Stage 5 for the
# full enumerated form and rationale.
git merge --no-ff --gpg-sign review/feat-sdmx-dimensions -F - <<'EOF'
feat(types): add core SDMX dimension definitions

Integrates core dimension type mappings and representation structures
under the SDMX-ML 3.0 schema specification.

Key Changes:

- [ ... full enumerated change list — see Stage 5 of workflow.md ... ]

Resolves #2
EOF

# Push to staging branch so CI runs on this exact SHA
git push origin main:staging-review-feat-sdmx-dimensions

# Once CI is green, fast-forward main and publish to every forge
git push all main

# Clean up staging branch and local review branch
git push origin --delete staging-review-feat-sdmx-dimensions
git branch -d review/feat-sdmx-dimensions
```

> [!NOTE]
> This is the contributor case the model is built for: the contributor's branch
> commit carried `Closes #2` (a feature — issue *linkage*, type-driven); the
> maintainer authors the **detailed merge commit** with `Resolves #2`, which is
> the canonical record on `main` and the trailer that *guarantees* closure.
> The maintainer curates quality at merge time rather than gating review on the
> contributor's commit-message wording. See [Semantic Keywords](#semantic-keywords)
> and the [Developer Workflow Exemplar — Stage 5](../dev/workflow.md#stage-5-the-merge-maintainer-integration).
