# MSRV Policy and Upgrade Procedures

This document outlines the Minimum Supported Rust Version (MSRV) policy for the `sdmx-rs` workspace and provides comprehensive upgrade procedures.

## MSRV Policy

The workspace `rust-version` field in `Cargo.toml` defines the **Minimum Supported Rust Version (MSRV)**. That field is the canonical source of truth; `rust-toolchain.toml` pins the exact compiler version used for local development and CI.

**Policy for bumping the MSRV**:

- The MSRV will not be raised to a version released less than **6 calendar months ago**. This means any Rust version that has been stable for at least 6 months is a valid MSRV floor. SDMX tooling is frequently integrated into enterprise data infrastructure where toolchain upgrades lag behind upstream; a 6-month window gives consumers a predictable and reasonable migration horizon.
- An MSRV bump is a **breaking change** for consumers and must be accompanied by a `MAJOR` (or during the pre-1.0 phase, a deliberate version note) version increment following standard semver convention.
- A dependency update that silently raises the effective MSRV is treated as a breaking change under the same policy. Contributors must verify that a new dependency compiles cleanly under the declared `rust-version` before merging.
- The CI pipeline enforces the declared MSRV by running `cargo check` and `cargo test` under the version pinned by `rust-toolchain.toml`. No exceptions are made for "compile-only" changes.

### MSRV Declaration Layout (Why Per-Crate, Not Inherited)

The `[workspace.package].rust-version` field is the **canonical** MSRV. Each member crate, however, **redeclares the same value as a literal** (`rust-version = "1.92.0"`) rather than inheriting it via `rust-version.workspace = true`. This duplication is deliberate, for two independent reasons:

- **Tooling compatibility.** `cargo-msrv` (used by the scheduled `check-msrv` CI job) reads crate manifests directly and cannot resolve a workspace-inherited `rust-version` — an inherited value would leave it unable to determine the MSRV. Manifest-parsing tools generally need the literal present. (Note: `cargo metadata` *does* flatten inheritance, so the `msrv-verify` job's extraction would work either way; the literal is required for the weakest tool in the chain.)
- **Automated updates.** [`scripts/update-msrv.sh`](../../scripts/update-msrv.sh) reads `[workspace.package].rust-version` as the source of truth (cross-checking it against `rust-toolchain.toml`), then rewrites the literal in every manifest with a `sed` substitution keyed on the exact `rust-version = "<old>"` string. A crate using `rust-version.workspace = true` would silently be skipped by that substitution.

**Do not** replace the per-crate literals with `rust-version.workspace = true`: it would break both `cargo-msrv` resolution and the update script. Keep `[workspace.package].rust-version` as the canonical anchor and let `just update-msrv` keep the literals in sync.

### Exception: Critical Compiler Security Vulnerabilities

If a **critical security vulnerability** is discovered in a Rust compiler version that `sdmx-rs` supports:

- An **emergency MSRV bump may be issued immediately**, outside the normal 6-month policy window
- This is treated as a breaking change requiring a MAJOR version increment
- A detailed security advisory will accompany the release explaining:
  - The Rust compiler CVE and its impact
  - Why the emergency bump was necessary
  - The new minimum Rust version required
  - Upgrade path and timeline for consumers

**Rationale**: Compiler security vulnerabilities affect all code compiled with that Rust version. Holding MSRV stable in the face of a compiler CVE would expose downstream consumers to unmitigable risk. User safety takes priority over stability guarantees.

**Example**: If Rust 1.91.0 is discovered to have a critical code-generation bug (e.g., incorrect optimisation affecting cryptographic operations), `sdmx-rs` would immediately bump MSRV to the next stable release with the fix, even if the current MSRV is only 2 months old.

## MSRV Update Procedures

### Raising MSRV (Breaking Change)

Use the automated update script to raise MSRV with policy enforcement:

```bash
just update-msrv 1.91.0 1.92.0
```

**What the script does (raise)**:
- ✅ Validates that new MSRV is 6+ months old (policy enforcement)
- ✅ Checks git state and file consistency (pre-flight validation)
- ✅ Compares clippy output between old and new MSRV (warns on divergence)
- ✅ Updates all Cargo.toml files (workspace + 5 crates)
- ✅ Updates rust-toolchain.toml, maintenance.toml, README, and docs/project/msrv.md
- ✅ Sets 6-month review obligation in maintenance.toml
- ✅ Runs full verification suite (`just verify`)
- ✅ Stages files for review (developer commits manually)
- ✅ Prints breaking-change reminder and suggested commit message

**Dry-run preview (no modifications)**:
```bash
just update-msrv --dry-run 1.91.0 1.92.0
```

### Lowering MSRV (Feature, Non-Breaking)

Lower MSRV opportunistically when code compatibility is discovered:

```bash
just update-msrv --downgrade 1.91.0 1.85.0
```

**What the script does (downgrade)**:
- ✅ Checks git state and file consistency (pre-flight validation)
- ✅ Updates all Cargo.toml files (workspace + 5 crates)
- ✅ Updates rust-toolchain.toml, README, and docs/project/msrv.md
- ✅ Skips maintenance.toml (non-breaking feature, no review obligation)
- ✅ Runs full verification suite (`just verify`)
- ✅ Stages files for review (developer commits manually)
- ✅ Notes non-breaking status in commit message

**Dry-run preview (no modifications)**:
```bash
just update-msrv --dry-run --downgrade 1.91.0 1.85.0
```

### Manual Path (For Understanding or Troubleshooting)

If you need to verify the steps manually, or the script fails, follow these. The Nix
flake provisions the toolchain from `rust-toolchain.toml` at shell entry, so edit the
pins first, then run every check inside a fresh `nix develop` that re-reads them. That
is why no step uses a `cargo +<version>` selector: that is a rustup feature, and this
workspace has no rustup, so the toolchain comes from the pinned file, not the command.

1. **Capture the lint baseline on the current toolchain**, before changing any pins, so
   a lint added by the new toolchain can be told apart from a pre-existing one:
   ```bash
   cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee /tmp/old-msrv.txt
   ```

2. **Update the version pins** to the new MSRV:
   ```bash
   # rust-toolchain.toml: channel = "1.92.0"
   # Cargo.toml [workspace.package] rust-version = "1.92.0", and the same literal in
   #   each crate manifest: crates/{sdmx-types,sdmx-parsers,sdmx-writers,sdmx-client,sdmx-rs}/Cargo.toml
   ```

3. **Verify under the new toolchain in a fresh dev shell**. Re-entering `nix develop`
   re-reads the rewritten `rust-toolchain.toml` and provisions the new compiler, so a
   bare `cargo`/`just` resolves to it:
   ```bash
   nix develop --command bash -c '
     cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee /tmp/new-msrv.txt
     cargo check --workspace --all-targets
     cargo nextest run --workspace
     cargo check --target wasm32-unknown-unknown -p sdmx-types -p sdmx-parsers
   '
   # A lint introduced by the new toolchain appears in new-msrv.txt alone:
   diff /tmp/old-msrv.txt /tmp/new-msrv.txt
   ```

   If new lints fire, either suppress them with `#[allow(...)]` if they are false
   positives, or fix the code to satisfy the new lint.

4. **Update the remaining files**:
   ```bash
   # maintenance.toml: last_updated and next_review dates (6 months from today)
   # README.md and crates/*/README.md: badge and MSRV section version numbers
   # CONTRIBUTING.md: MSRV references
   ```

5. **Run the full gate under the new toolchain**:
   ```bash
   nix develop --command just verify
   ```

### Commit and Version Bump

Once all checks pass, commit with:
```bash
git add Cargo.toml crates/*/Cargo.toml rust-toolchain.toml maintenance.toml README.md docs/project/msrv.md
git commit --gpg-sign -m "chore(msrv): raise minimum supported Rust version to 1.92.0"
```

> [!IMPORTANT]
> **BREAKING CHANGE**
>
> Per policy: MSRV raises require a **MAJOR version increment**. Update your crate versions before release. `cargo-release` will prompt you for version confirmation — approve MAJOR only.
