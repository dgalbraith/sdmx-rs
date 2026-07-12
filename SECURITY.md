# Security Policy

This document outlines the security policies and vulnerability reporting procedures for the `sdmx-rs` project.

## Supported Versions

Currently, `sdmx-rs` is in active, pre-1.0 early development. As such, there are no long-term support (LTS) versions. Until a stable `1.0.0` release is reached, security fixes are applied exclusively to the main development branch (`main`) and published as part of the latest pre-1.0.0 crate releases. Downstream consumers are encouraged to always upgrade to the latest published package versions to receive security updates.

| Version   | Supported | Description                                                         |
|:---------:|:---------:|---------------------------------------------------------------------|
| `main`    | ✅        | Active development branch; all security patches are applied here.   |
| `< 1.0.0` | ❌        | Legacy pre-1.0 releases are not backported; consumers must upgrade. |

## Rust Compiler & Toolchain Vulnerabilities

`sdmx-rs` is built with a specific Minimum Supported Rust Version (MSRV) documented in [docs/project/msrv.md](docs/project/msrv.md). If a **critical security vulnerability** is discovered in a Rust compiler version that `sdmx-rs` supports:

### Detection & Response

- **Monitoring**: Check [Rust Security Advisories](https://rustsec.org/) and [Rust GitHub Releases](https://github.com/rust-lang/rust/releases) for compiler CVEs affecting the `sdmx-rs` MSRV version
- **Impact Assessment**: A compiler vulnerability affects all binaries compiled with that Rust version and may be unfixable from userland (e.g., incorrect optimisations, code-generation bugs)
- **Emergency Response**: An **emergency MSRV bump** may be issued immediately, outside the normal 6-month policy window, with a corresponding MAJOR version increment to `sdmx-rs`

### For Users

**Action Items**:
1. Keep your Rust toolchain up-to-date: Run `rustup update` regularly
2. If your Rust compiler version has an active CVE (visible on [RustSec](https://rustsec.org/)), upgrade immediately—do not wait for `sdmx-rs` to bump its MSRV
3. Monitor `sdmx-rs` releases: an emergency MSRV bump will be accompanied by a detailed security advisory explaining the compiler CVE and required action

**Why the asymmetry?** Compiler vulnerabilities are often unfixable from the library side. We can only bump our MSRV to exclude vulnerable versions. Your Rust upgrade is the actual fix.

### For Contributors & Maintainers

When investigating or fixing issues, always check whether the Rust compiler version in use has any known CVEs:

```bash
# Check current Rust version
rustc --version

# Check RustSec database for compiler vulnerabilities
cargo audit

# Visit GitHub Rust releases for detailed CVE information
# https://github.com/rust-lang/rust/releases/tag/<your-msrv>
```

**Documentation Requirement**: When an emergency MSRV bump occurs due to a compiler CVE, the changelog entry must clearly explain:
1. What the Rust compiler CVE is and its impact on `sdmx-rs`
2. Why the emergency bump was necessary (deviating from 6-month policy)
3. What version(s) of Rust are affected and required
4. Timeline and severity (e.g., "All users should upgrade immediately")

## Reporting a Vulnerability

We take the security of this project seriously. If you discover a security vulnerability, please do **not** open a public issue.

### Response Expectations

`sdmx-rs` is a community-driven project maintained by a single developer. I am committed to the following timelines on a best-effort basis. If personal commitments or the complexity of the report necessitate an extension, I will communicate this to you promptly. Your patience is appreciated as we work together to keep the project secure.

### Reporting Channels

1. **Via GitHub Private Vulnerability Reporting (Preferred)**: Navigate to the **Security** tab of the repository on GitHub, click **Advisories**, and then click **Report a vulnerability**. This provides a secure, structured channel to discuss the issue and coordinate a fix.
2. **Via Email (Fallback)**: You can also report vulnerabilities directly to: **dg@lbraith.io**

### Report Contents

Please include the following information in your report:
- A description of the vulnerability and its potential impact.
- Steps to reproduce the issue.
- The affected crate(s) (`sdmx-types`, `sdmx-parsers`, `sdmx-writers`, `sdmx-client`, or `sdmx-rs`).
- Any proposed mitigation or fix (if available).

You should receive a response acknowledging receipt of the vulnerability within **5 business days**, and a preliminary assessment within **15 business days**. I am committed to verifying the issue and coordinating a patch as quickly as the project's capacity allows.

## Supply Chain Security

`sdmx-rs` employs strict supply chain security measures to ensure dependency integrity and licensing compliance.

### Security Policy

- **Vulnerability Management**: Only dependencies with no active RustSec advisories are permitted
- **License Control**: Only approved open-source licenses (MIT, Apache-2.0, BSD-3-Clause, ISC, CDLA-Permissive-2.0, CC0-1.0, Unicode-3.0)
- **TLS Backend**: `rustls` (pure Rust) enforced; `native-tls`, `openssl`, and `openssl-sys` explicitly banned (see [ADR-0013](docs/adr/0013-use-rustls-over-native-tls-for-transport-layer-security.md))
- **Registry Control**: Dependencies sourced only from crates.io; unknown registries and git sources denied
- **Code Safety**: `unsafe` code globally forbidden (see [ADR-0002](docs/adr/0002-workspace-wide-safety-policy-banning-unsafe-code.md))
- **Commit Integrity**: All commits to `main` and all release tags must be GPG-signed by an allowlisted maintainer — enforced at the forge by a no-bypass `required_signatures` ruleset and verified for maintainer authorship by CI (see [Vulnerability Monitoring & Remediation](#vulnerability-monitoring--remediation-dependabot-alerts))
- **CI-Verified Integrity (Zero Trust Gate)**: Beyond authorship, the *intent* is that every SHA on `main` has passed the `CI Quality Gate`, a **fail-closed** aggregator that passes only when every merge-gating job is `success` or legitimately `skipped` (an unknown CI result or a path-filter failure blocks rather than passes). This is an intent, not a guaranteed invariant, because enforcement differs by layer:
  - **Forge scope:** `required_status_checks` gates *pull-request merges*, not direct ref updates. A `git push` straight to `main` is not blocked by it, as CI runs only after the push is accepted.
  - **Forge-enforced for every actor:** the guarantees the forge enforces at ref-update time, for every actor including the repository owner, are `required_signatures` and the append-only rules (`non_fast_forward` and `deletion`).
  - **Maintainer discipline:** the staging round-trip therefore upholds the green-seal intent, backed locally by an advisory pre-push guard (`scripts/guard-main-push.sh`) that refuses a push to `main` whose commit is not on a `staging-*` branch. It is bypassable (`git push --no-verify`, or `SDMX_ALLOW_DIRECT_MAIN=1`) and active only if hooks are installed (checked by `just doctor-hooks`), so it is an accident guard, not a barrier: a sole maintainer, as the root of trust, cannot be cryptographically gated against their own valid credentials.
  - **Hard guarantees:** signed commits and append-only history; CI additionally re-verifies the *content* of every staged SHA (see [ci-gating.md](docs/project/ci-gating.md#the-ci-quality-gate-aggregator) and [merging.md](docs/project/merging.md)).
- **Actions SHA-pinning**: Every `uses:` reference in CI workflows is pinned to a full 40-hex commit SHA, not a mutable tag or branch — enforced by `actionlint` in CI and by `sha_pinning_required=true` at the forge. Prevents a trusted action's tag from being silently redirected to different code after the workflow is reviewed.
- **Actions allowlist**: `allowed_actions=selected` at the forge restricts which third-party actions may run at all, via a committed allowlist (`forge/github/actions-allowlist.json`). Orthogonal to SHA-pinning: pinning stops version drift on an action you already use; the allowlist stops an unreviewed or typo-squatted action entering the pipeline in the first place. Every new third-party action must be explicitly added to the allowlist in the same commit as the workflow change — `doctor-forge` fails if a workflow references an unlisted action.

### Secret Leak Prevention

Distinct from dependency integrity above, committed secrets (API keys, tokens, private keys) are guarded by three defence-in-depth layers:

- **Local hooks** (`gitleaks`, pre-commit and pre-push) — catch secrets before they leave the developer's machine. Advisory: bypassable with `git push --no-verify` or if hooks are not installed.
- **CI `check-secrets`** — `just secrets-scan` runs on every push and PR (unconditionally, not path-filtered, over full git history). Blocks merge, but detects only *after* the push reaches the remote.
- **Push Protection** (GitHub server-side) — rejects a secret-bearing push at the forge before history is accepted. This is the only layer that *prevents* exposure rather than detecting it, and the one that closes the force-push vector.

The local and CI layers are git/Nix-based and portable across forges; Push Protection is GitHub-native. The settings and their apply/verify procedure are documented in [forge-setup.md — Secret Scanning & Push Protection](docs/project/forge-setup.md#secret-scanning--push-protection).

> [!NOTE]
> Secret scanning and Push Protection are enabled on this repository.

### Vulnerability Monitoring & Remediation (Dependabot Alerts)

`sdmx-rs` uses three complementary provenance mechanisms that cover different surfaces and bind them together:

- **Repo provenance (commits and tags)**: All commits to `main` and all release tags must be GPG-signed by a listed maintainer, and every SHA reaching `main` must carry the `CI Quality Gate` seal. These guarantees hold at different strengths:
  - **Prevention (Forge-Enforced):** A no-bypass ruleset makes the forge reject any unsigned commit or tag (`required_signatures`), and the append-only rules block history rewrites. These bind direct pushes as well as PR merges.
  - **Prevention (Maintainer Discipline):** Because `required_status_checks` does not block direct pushes, the green-seal intent is upheld via the maintainer's staging round-trip and the advisory `guard-main-push` local hook.
  - **Detection and Authorship:** The `verify-signature.yml` workflow independently verifies that the signing key belongs to an **allowlisted maintainer** (the ruleset alone guarantees only that an object is signed by *some* valid key). Automated PR generation (Dependabot updates) is disabled to prevent unsigned or bot-signed commits from entering the history.
  - **Verification (on demand):** `just doctor-provenance` audits `main`'s actual history against the claims above: every commit and annotated tag signed by a maintainer key allowlisted *as of that object*, and which merges reached `main` through the staging CI round-trip versus a direct push. With `--root-fpr` (a founding fingerprint obtained out-of-band) it is an independent audit; without it, a self-consistency check against the repo's self-declared root, not an independent verdict.
- **Artifact provenance** (published crates): `publish.yml` uses Trusted Publishing (ephemeral OIDC tokens) to authenticate to crates.io, and attaches SLSA L2 build provenance and SBOM attestations to every published `.crate` file via the GitHub attestation store. The Trusted Publishing *configuration itself* (which repository, workflow, and environment crates.io trusts, plus enforcement state) is verifiable with `just doctor-registry`, which asserts exactly one publisher per crate matching the spec — a stray publisher binding is detected, not assumed away. The crates.io API token used for that setup is a short-lived, minimal-scope personal token, revoked after use; no long-lived registry credential is held by the pipeline or the tooling (see [registry-setup.md](docs/project/registry-setup.md)).
- **Binding repo to artifact provenance**: publishing is triggered by a pushed release tag, so the published artifact is bound to a *tag commit* rather than to `main` directly. The `verify-tag-on-main` gate in `publish.yml` closes that gap — it refuses to publish unless the tag's commit is reachable from `origin/main`. A crate therefore cannot go live unless its exact source is both maintainer-signed (repo provenance) **and** present on the canonical branch.

1. **GitHub scanning only**: We use GitHub Dependabot alerts in monitor-only mode. Automated pull request generation (Dependabot updates) is disabled to preserve the GPG-signed commit invariant.
2. **Local Triage & Transitive Resolution**: When a vulnerability is flagged, the maintainer resolves it using the following hierarchy:
   * **Direct Dependencies**: Run `cargo update -p <vulnerable-package> --precise <patched-version>`.
   * **Transitive (Indirect) Dependencies**:
     1. Trace the parent chain: `cargo tree -i <vulnerable-package>` to identify which direct dependency imports it.
     2. Force-update the transitive package: `cargo update -p <vulnerable-package> --precise <patched-version>`. This succeeds if the patch is semver-compatible with the parent's constraints.
     3. If blocked by constraints, check for and apply updates to the parent dependency: `cargo update -p <parent-package>`.
     4. If no upstream updates exist, temporarily override the dependency in the root `Cargo.toml` using `[patch.crates-io]`.
     5. If no resolution exists and the vulnerability is non-exploitable under our configuration, document the rationale and add the advisory ID to the ignore list in `deny.toml`.
3. **Verification**: The maintainer runs `just verify` locally to ensure the update compiles on MSRV, complies with license/banned rules, and passes all tests.
4. **GPG-signed push**: Once verified, the commit is GPG-signed and pushed to `main`.

### Dependency Audit Automation

These policies are **automatically enforced on every pull request** via two complementary mechanisms:

#### 1. Cargo-Deny ([deny.toml](deny.toml))

**What it checks**:
- **Vulnerabilities**: RustSec advisory database; CI fails if any active advisories are present
- **Licenses**: Enforces whitelist from [deny.toml](deny.toml); CI fails on unknown or disallowed licenses
- **Banned Crates**: Explicitly denies `native-tls`, `openssl`, `openssl-sys` per the rustls-only policy
- **Registry Sources**: Denies unknown registries and non-crates.io git sources; CI fails on compliance violations

**How violations are handled**:
- Active RustSec advisories → **CI fails** (blocks merge)
- Unknown or disallowed license → **CI fails** (blocks merge)
- Banned crate dependency → **CI fails** (blocks merge)
- Accepted/low-risk advisories → can be documented in [deny.toml](deny.toml) `ignore` section with rationale

#### 2. Cargo-Machete

Detects unused dependencies in workspace manifests during Phase 0 scaffolding:
- Warns if dependencies are declared in `Cargo.toml` but unused in code
- During early phases, legitimate unused dependencies are documented in `[package.metadata.cargo-machete]` per [CONTRIBUTING.md § Scaffolding Dependency Policy](CONTRIBUTING.md#5-scaffolding-dependency-policy)

### Violations & CI Enforcement

CI will **block any PR** where cargo-deny or cargo-machete checks fail:
- Active RustSec advisories → CI fails (blocks merge)
- Unknown or disallowed license → CI fails (blocks merge)
- Banned crate dependency → CI fails (blocks merge)
- Unknown registry source → CI fails (blocks merge)

For resolution steps and local workflow, see [CONTRIBUTING.md § Dependency Audit Checks](CONTRIBUTING.md) (how to run locally, how to resolve violations).

### Maintenance Review

Dependency audit policies are reviewed on a 30-day cadence (tracked in [maintenance.toml](maintenance.toml) under `dependency-audit`). During reviews, new RustSec advisories, license compatibility, and crate-banning decisions are evaluated.

## Verifying Release Provenance

Every published `.crate` file has three attestations written to the GitHub attestation store by `publish.yml`: build provenance (SLSA L2), a CycloneDX SBOM, and an SPDX SBOM. Release tags are GPG-signed by the maintainer.

### Verifying attestations

```bash
# Build provenance (SLSA L2) — confirms the .crate was built by publish.yml on GitHub-hosted runners
gh attestation verify <name>-X.Y.Z.crate --repo dgalbraith/sdmx-rs

# CycloneDX SBOM attestation
gh attestation verify <name>.cdx.json --repo dgalbraith/sdmx-rs

# SPDX SBOM attestation
gh attestation verify <name>.spdx.json --repo dgalbraith/sdmx-rs
```

### Verifying the release tag and its commit

Verify **both** the tag object and the commit it points to. The commit is the
actual source being published (publishing keys off the *tag commit*, not the
merge commit), so a maintainer signature on the tag wrapper alone is not enough.
This reproduces locally what the `verify-signature` gate enforces on every
release — it checks both the tag and its commit.

```bash
# Confirm the release tag carries a valid maintainer GPG signature
git verify-tag sdmx-types/vX.Y.Z

# Confirm the commit the tag points to — the published source — is also
# maintainer-signed
git verify-commit "sdmx-types/vX.Y.Z^{commit}"
```

### Verifying the source is on the canonical branch

`publish.yml` enforces (via the `verify-tag-on-main` gate) that a crate is only published when its tag commit is reachable from `main`. You can independently confirm this for a published version:

```bash
# Resolve the tag to its commit and confirm it is an ancestor of origin/main.
# Exit status 0 means the published source is on the canonical branch.
git fetch origin main --tags
git merge-base --is-ancestor "sdmx-types/vX.Y.Z^{commit}" origin/main \
  && echo "✅ on main" || echo "❌ NOT on main"
```

### Residual gap

`cargo` has no native attestation verification command. Verification of build provenance and SBOMs is currently out-of-band via the `gh` CLI as shown above. This is a known gap in the Rust/crates.io ecosystem (tracked upstream at [rust-lang/cargo#12661](https://github.com/rust-lang/cargo/issues/12661)).
