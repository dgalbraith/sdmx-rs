# 4. Release Publish Pipeline and Supply Chain Provenance

Date: 2026-05-31

## Status

Accepted

<!-- Valid statuses: Proposed, Accepted, Implemented, Superseded -->

---

## Summary

Design of the CI publish pipeline (`publish.yml`) that publishes the workspace crates to crates.io with cryptographically verifiable supply-chain provenance. The pipeline is triggered by signed release tags, gated on maintainer-signature verification, and publishes each crate via crates.io Trusted Publishing (OIDC) — eliminating long-lived registry credentials. For every published `.crate` artifact it generates a SLSA build-provenance attestation and two SBOM attestations (CycloneDX and SPDX), all stored in the GitHub attestation store and verifiable out-of-band via `gh attestation verify`. This design completes the final open Phase 0 item and must be in place before the `0.1.0` publication at Phase 1 completion. It aligns the release process with modern supply-chain best practice and the relevant federal standards (EO 14028, OMB M-22-18, NIST SP 800-218, SLSA, NTIA SBOM minimum elements).

---

## Problem / Motivation

The local release tooling (`cargo-release`, `git-cliff`) already produces signed version-bump commits and signed tags, establishing a strong **repository audit trail**. The `verify-signature.yml` workflow enforces that every commit on `main` and every release tag carries a valid maintainer signature anchored on the primary GPG key fingerprint. What none of this provides is **consumer-verifiable artifact provenance**: the assurance that a given `.crate` on crates.io was built from a specific, signed source commit, in an isolated CI environment, with a published record of its dependency tree.

Two distinct goals were previously conflated and must be separated:

- **Repository audit trail** — *who authorised each release.* Satisfied by local GPG signing of commits and tags. This is repository integrity.
- **Consumer-verifiable provenance** — *what was built, from what source, in what environment, with which dependencies.* Requires CI-based publishing with an OIDC build identity and build/SBOM attestations. This is artifact provenance.

GPG signatures on git commits answer the first question only. No federal framework recognises commit signatures as artifact provenance. Local (laptop) publishing is strictly worse on every supply-chain axis: it requires a long-lived API token on a developer machine, the build environment is unattestable, and the published `.crate` carries no provenance record.

**Current state**: `release.toml` declares `publish = false` with a comment deferring registry push to CI, and `releasing.md` Section 6 describes CI publishing — but no `publish.yml` exists. The only tag-triggered CI job today is `check-changelog`. A release executed as documented would therefore either stall or force a manual token-based local publish that bypasses every provenance control.

**Goals and success criteria**:

- **No long-lived publish credentials** — authentication is per-run and auto-revoked (SSDF PS.1/PS.2, OMB M-22-18).
- **Signature-gated publishing** — publishing cannot proceed unless the release tag and its history satisfy the maintainer-signed-only invariant.
- **Human approval** — a protected environment with a required reviewer decouples "a tag was pushed" from "a publish was authorised."
- **Per-crate provenance** — each published crate carries its own SLSA build-provenance attestation and its own SBOM attestations (SLSA Build L2, EO 14028, NTIA).
- **Deterministic, recoverable job structure** — topological publish order with explicit handling of crates.io indexing lag and per-crate retry granularity.

---

## Proposed Design

### Architecture / Key Decisions

#### 1. Trigger and Signature Gate

The workflow triggers on release-tag pushes only:

```yaml
on:
  push:
    tags: ['sdmx-*/v*']
```

Per-crate tags follow the existing `cargo-release` convention (e.g. `sdmx-types/v0.1.0`), not a flat `v0.1.0`. The bootstrap crate-name reservation has no tag anchor by design; its durable record is the [Bootstrap Record in releasing.md](../project/releasing.md#bootstrap-record). The glob matches this convention while excluding arbitrary tags: in GitHub tag filters `*` does not span `/`, so `sdmx-*/v*` matches `sdmx-types/v0.1.0` but never a stray `wip` or docs tag, which must not start the publish chain. (Signature *enforcement* is broader — `verify-signature.yml` runs on `tags: ['**']` so every tag object is checked regardless; only *publishing* is scoped to release tags.)

Publishing is gated on the `verify-signature` job (from `verify-signature.yml`), which verifies that the tag object and its commit history carry a valid signature whose **primary** key fingerprint is in the maintainer allowlist. It sits at the head of the `needs:` chain (`verify-signature → check-changelog → setup → publish`), so a tag with an absent, invalid, or unauthorised signature halts the run before any artifact reaches crates.io. Job ordering matters: publish must run *after* the signature gate, never in parallel with it.

#### 2. Job Topology — Per-Crate Tag Dispatch

The tag selects the crate. Each `<crate>/v<version>` tag triggers an independent run that publishes exactly that one crate:

```
verify-signature → check-changelog → setup → publish
                                        │        │
              parse tag, assert ────────┘        └──── wait-for-deps → package →
              version == Cargo.toml                    publish (verify) → index →
                                                       SBOM → attest ×3 → release
```

A batch release pushes one tag per crate (`git push --follow-tags`), producing one single-crate run per tag. Topological ordering across a batch is enforced by the **index**, not a static job chain: the `wait-for-deps` step blocks a run until the crate's own workspace dependencies are published and indexed (a no-op for `sdmx-types`). This is correct in both versioning phases — pre-1.0 lockstep (five tags → five runs) and post-1.0 per-crate (only the changed crates' tags fire).

Rationale for tag-dispatched single-crate runs rather than a static all-crate chain:

- **Tag ⇄ artifact binding**: the `setup` job asserts the tag's version equals the crate's `Cargo.toml` version before anything publishes ([`parse-release-tag.sh`](../../scripts/ci/parse-release-tag.sh)). The signed tag is the trust anchor, so the published artifact must be exactly what the tag claims — never whatever sits in the working tree. A static all-crate run cannot make this assertion because one tag does not name all five versions.
- **Correct crate selection**: only the tagged crate publishes. There is no "publish all five, rely on the idempotency guard to skip the rest" — which silently mispublishes if an unchanged crate's working-tree version diverges from what was intended.
- **Retry granularity + idempotency**: a failed run is re-run from the Actions UI; the publish guard (section 6) skips the crate if its version is already on crates.io and updates the GitHub Release in place, so re-runs never error on "version already exists".
- **Isolation**: each run performs its own ephemeral OIDC token exchange, narrowly scoped and auto-revoked.

#### 3. Authentication — Trusted Publishing (OIDC)

Each publish job authenticates to crates.io via OIDC using `rust-lang/crates-io-auth-action` (SHA-pinned per repository hygiene). No API token is stored as a CI secret or on any developer machine.

```yaml
permissions:
  id-token: write      # OIDC for TP auth and Sigstore attestations
  contents: write      # GitHub Release creation
  attestations: write  # build-provenance and SBOM attestations
```

The action exchanges the job's OIDC identity for a short-lived crates.io token (output variable `token`), auto-revoked in its post-step, passed to `cargo` via `CARGO_REGISTRY_TOKEN`. Trusted Publishers are registered per crate name in the crates.io UI (repository, workflow, environment). Once validated, **enforcement** is enabled per crate to disable API-token publishing entirely, closing the long-lived-credential vector and satisfying the ephemeral-credentials posture demanded by M-22-18 and SSDF PS.1.

#### 4. Release Environment and Human Approval

The `publish` job runs in a protected GitHub environment named `release`:

```yaml
jobs:
  publish:
    environment: release
    # ...
```

The environment is configured with a **required reviewer** (the maintainer). Because the project currently has a single maintainer, **"Allow self-reviews" must be enabled** — otherwise GitHub deadlocks the deployment, as a single reviewer cannot approve their own run by default. This gate is the human-in-the-loop control that catches accidental tag pushes and tags on non-release commits — and, since each crate is its own run, it is requested once per crate, so every crate is individually approved. Onboarding additional maintainers is purely additive (add them as reviewers); no workflow change is required.

#### 5. Packaging and the Artifact Path

Attestation requires a known, deterministic local path to the `.crate`. The artifact is produced by an explicit `cargo package` step so it exists at `target/package/<name>-<version>.crate` for the attestation and GitHub-Release steps regardless of whether the subsequent publish runs (it is skipped on an idempotent re-run — see section 6):

```bash
# Version extracted via cargo metadata (same pattern as MSRV extraction in ci.yml)
VERSION=$(cargo metadata --no-deps --format-version 1 \
  | jq -r --arg n "$CRATE_NAME" '.packages[] | select(.name == $n) | .version')

# 1. Package — writes target/package/<name>-<version>.crate (attestation subject)
cargo package --manifest-path "crates/${CRATE_NAME}/Cargo.toml"

# 2. Publish WITH verification — cargo re-packages (deterministically, same
#    artifact) and compiles the packaged crate against its registry deps.
cargo publish --manifest-path "crates/${CRATE_NAME}/Cargo.toml"
```

`cargo publish` is run **without `--no-verify`**: its build step compiles the packaged crate against the published (index) versions of its dependencies, catching the monorepo failure mode where a crate builds in-workspace via `path` deps but is broken once published (an unbumped dependency version, a file excluded by `include`/`exclude`). `cargo check --workspace` in CI cannot catch this because it resolves path deps. Since publishing is irreversible, this compile is mandatory insurance and is worth the redundant re-package the separate `cargo package` step incurs.

> [!NOTE]
> An earlier draft of this design specified `cargo publish --no-package` to reuse the packaged tarball and avoid the double-package. That flag **does not exist** in the pinned cargo (1.91); the real choice is `cargo publish` (packages + verifies) vs `--no-verify` (packages, skips the compile). We keep the verifying default. The re-package is deterministic, so the attestation subject is unaffected.

#### 6. Idempotent Publish + Indexing-Lag Poll Loop

`sdmx-parsers` depends on `sdmx-types`; `sdmx-rs` depends on all of them. crates.io does not index a new version instantly, so a dependent crate's publish can fail with "dependency not found" if it runs immediately after its dependency. Rather than rely on the operator to wait, the lag is encoded in CI: after each `cargo publish`, the job polls the live index until the version appears, with bounded backoff.

The same index check serves a second purpose — **idempotency**. Before publishing, each job asks whether its exact version is already on crates.io; if so, the publish and poll steps are skipped. This makes a re-run of a partially-completed batch safe: crates that already published are no-ops, and the run resumes at the crate that failed (see Mid-Run Failure Recovery in [releasing.md](../project/releasing.md)). Both checks share one helper, [`scripts/ci/check-published.sh`](../../scripts/ci/check-published.sh), driven from [`wait-for-index.sh`](../../scripts/ci/wait-for-index.sh) for the poll.

The probe queries the crates.io **sparse index** (`https://index.crates.io/<a2>/<b2>/<name>`), which returns newline-delimited JSON with one object per published version, and exact-matches the `vers` field:

```sh
# exists=true iff <version> is already published
jq -s -e --arg v "$VERSION" 'any(.[]; .vers == $v)' "$index_body"
```

`cargo search` is deliberately **not** used: it reports only the latest version of a crate, so it cannot confirm the presence of a specific (possibly older) version — fine for "has the just-published latest appeared?", wrong for the idempotency question. The sparse index answers both. The same index probe also backs the pre-publish `wait-for-deps` step (section 2), which blocks a run until its workspace dependencies — published by their own tagged runs — are indexed, so a dependent crate never starts its verify-compile until its dependency is resolvable. Linear backoff (10s, 20s … 150s over 16 attempts) gives a ~20-minute wait ceiling within job timeout limits — sized to cover the upper end of historical crates.io indexing slowdowns (15–20 minutes during outages); in practice indexing completes in under two minutes, so most runs exit on the first or second attempt.

#### 7. Software Bill of Materials — Dual Format, Single Source

Each crate produces **two** SBOMs from a single authoritative source:

```bash
# Primary: CycloneDX (best Rust tooling, OWASP standard)
cargo cyclonedx --manifest-path "crates/${CRATE_NAME}/Cargo.toml" --format json
# → <name>.cdx.json

# Derived: SPDX 2.3 (ISO/IEC 5962:2021, NTIA / EO 14028 named format)
cyclonedx-cli convert --input-file "<name>.cdx.json" \
  --output-format spdxjson --output-file "<name>.spdx.json"
```

CycloneDX is generated first because `cargo cyclonedx` (`cyclonedx-rust-cargo`) is the most actively maintained Rust SBOM tool and handles workspace-member scoping correctly. **SPDX is derived from the CycloneDX document via `cyclonedx-cli convert`, not generated independently** — this guarantees both SBOMs describe exactly the same dependency graph, where two independent tool runs could capture divergent snapshots. SBOM granularity is **per crate**: each published `.crate` has its own dependency surface.

#### 8. Attestations — Three Per Crate

Each crate produces three attestations, all bound to the same `.crate` artifact and stored in the GitHub attestation store:

```yaml
# SLSA build provenance — what source produced this artifact, signed by CI identity
- uses: actions/attest-build-provenance@<sha>
  with:
    subject-path: target/package/<name>-<version>.crate

# CycloneDX SBOM attestation
- uses: actions/attest-sbom@<sha>
  with:
    subject-path: target/package/<name>-<version>.crate
    sbom-path: <name>.cdx.json

# SPDX SBOM attestation
- uses: actions/attest-sbom@<sha>
  with:
    subject-path: target/package/<name>-<version>.crate
    sbom-path: <name>.spdx.json
```

`actions/attest-build-provenance` generates SLSA Build Level 2 provenance — GitHub-hosted runners supply the build isolation, and the attestation is signed via Sigstore/Fulcio under the workflow's OIDC identity. `actions/attest-sbom` binds each SBOM to the artifact under the same identity. All three are verifiable via `gh attestation verify`. Both require the `attestations: write` and `id-token: write` permissions.

#### 9. GitHub Release Creation

After a crate is published, indexed, and attested, the job creates a GitHub Release for its tag via [`scripts/ci/create-release.sh`](../../scripts/ci/create-release.sh), providing a stable home for the release notes plus the `.crate` and SBOM assets. The notes **body** is resolved by crate kind — a deliberate split that keeps the machine record and the human record separate:

- **The five `CHANGELOG.md` files are the machine record.** They are strict `git-cliff` output, gated byte-for-byte by [`check-changelog`](../../scripts/check-changelog.sh), and are **never hand-curated** — curating them would make that gate fail. `CHANGELOG.md` is not a crates.io surface (only `readme = "README.md"` is wired), so machine-purity costs nothing user-facing.
- **The facade (`sdmx-rs`) Release body is curated prose** at `crates/sdmx-rs/release-notes/<version>.md`. Because the facade `CHANGELOG.md` is machine-locked, user-facing prose lives in this separate file, which is a **mandatory pre-tag gate** ([`check-release-notes.sh`](../../scripts/check-release-notes.sh), also folded into `prepublish-check`). The gate must fire **before** `cargo release --execute`: a GitHub Release attaches to a *pushed* tag and can only be created post-tag in CI, and the tag push is the irreversible publish trigger — so "no facade release without curated prose" is enforced locally pre-tag, then `create-release.sh` deterministically materialises the Release from the file (with the machine `CHANGELOG` section as a backstop, and an empty facade body still fatal).
- **Leaf crates** (`sdmx-types`/`parsers`/`writers`/`client`) use their auto `CHANGELOG.md` section. When a pre-1.0 lockstep batch leaves a leaf with **no user-facing changes**, its section is empty — but the leaf Release must still be created, because it is the host for that crate's own SLSA build-provenance and SBOM attestations and its `.crate` asset (the settled per-crate-provenance decision, §§7–8). In that case `create-release.sh` emits a **provenance placeholder** body that names the Release as a provenance container and links the crate's `CHANGELOG` (plus the facade batch when a facade release exists at the same version). The placeholder fires **only** on a genuinely empty section, so post-1.0 leaves releasing independently with real changelogs are unaffected.

The step is **idempotent**: if the release already exists (a re-run), notes are refreshed and assets re-uploaded with `--clobber` rather than failing on "release already exists". Changelog *synchronisation* is verified separately and earlier by the `check-changelog` gate; release-note *resolution* (curated file / changelog section / placeholder) is a distinct concern and intentionally not shared with it.

#### 10. Consumer Verification

Verification instructions live in [SECURITY.md](../../SECURITY.md) — the conventional, auditor-facing home for provenance instructions post-EO-14028:

```bash
# Build provenance for the published artifact
gh attestation verify <name>-<version>.crate --repo dgalbraith/sdmx-rs
# SBOM attestations
gh attestation verify <name>.cdx.json  --repo dgalbraith/sdmx-rs
gh attestation verify <name>.spdx.json --repo dgalbraith/sdmx-rs
# Signed release tag (repository provenance)
git verify-tag sdmx-<crate>/v<version>
```

The documentation must also state the **residual gap**: `cargo` and crates.io have no native attestation check today (tracked upstream at `rust-lang/cargo#12661`); verification is out-of-band via the `gh` CLI. Stating this explicitly prevents consumers from looking for provenance in the wrong place.

---

## Alternatives Considered

### Alternative A — Local-only publishing (laptop `cargo publish`) (Rejected)

The implicit prior assumption (`release.toml` defers to CI, but no CI existed). Rejected: requires a long-lived API token on a developer machine, the build environment is unattestable, and commit signatures are repository integrity, not artifact provenance — a distinction recognised by no federal framework. Trusted Publishing obsoletes the main historical reason to publish locally (avoiding a long-lived CI token).

### Alternative B — Long-lived API token as a CI secret (Rejected)

A `CARGO_REGISTRY_TOKEN` repository secret avoids the laptop-token problem but reintroduces a long-lived credential in CI — a standing exfiltration target and a direct violation of the ephemeral-credentials posture in M-22-18 and SSDF PS.1. Trusted Publishing's per-run, auto-revoked OIDC token is the correct mechanism.

### Alternative C — Single publish job for the whole workspace (Rejected)

`cargo release --workspace` or a single job iterating all crates gives no retry granularity: a mid-sequence failure leaves partial state with no clean resume point. Per-crate-tag dispatch (section 2) isolates failures to one run per crate and allows targeted re-runs. This mirrors the existing "do not run `cargo release --workspace`" guidance in [releasing.md](../project/releasing.md).

### Alternative D — Independent SPDX generation (Rejected)

Generating SPDX with a separate tool run (rather than converting from CycloneDX) risks two divergent dependency snapshots if the environment changes between runs. Deriving SPDX from the single CycloneDX source via `cyclonedx-cli convert` guarantees an identical graph. SPDX-only generation was also rejected because current Rust SPDX tooling is less mature than the CycloneDX toolchain.

### Alternative E — No human-approval gate (fully automated on tag push) (Rejected for now)

Trusted Publishing alone would auto-publish on any tag push that passes signature verification, including accidental tags or tags on non-release commits. The `release` environment's required reviewer is a low-cost second factor. For a solo maintainer it is self-approval rather than separation of duties, but it preserves the deliberate-action property and establishes the pattern for future multi-maintainer operation.

### Alternative F — Native cargo / crates.io provenance verification (Not available)

crates.io has no native build-provenance, attestation, or SLSA support, and there is no PEP-740 equivalent for cargo (`rust-lang/cargo#12661`). The GitHub attestation store with out-of-band `gh attestation verify` is the workaround that works today. Accepted as a residual ecosystem limitation, documented for consumers.

---

## Drawbacks / Trade-offs

**Forge lock-in.** The pipeline depends on GitHub Actions OIDC for Trusted Publishing. Forgejo Actions (the Codeberg mirror) does not currently support crates.io Trusted Publishing, so a primary-forge migration would require reworking the workflow. This is an accepted dependency, already noted in [forge-setup.md](../project/forge-setup.md).

**Consumer verification friction.** Provenance is real but not verifiable through `cargo` itself; consumers must use the `gh` CLI out-of-band. This limits the practical reach of the attestations until the Rust ecosystem grows native support.

**Token-exchange multiplication.** One OIDC exchange per crate (one per tagged run, five per full release batch) is slightly more work than a single shared credential, accepted in exchange for per-run isolation and retry granularity.

**Indexing-lag latency.** The poll loop can add minutes to a release in the worst case. Mitigated by linear backoff and an early-exit on first success; typical impact is seconds.

**Bootstrap friction.** crates.io has no pending-publisher feature, so the first publish of each crate name must be a manual, token-based bootstrap before Trusted Publishing can be registered — a one-time deviation from the otherwise tokenless model.

**Maintenance surface.** Three attestations and two SBOM tools per crate add CI steps that must be kept SHA-pinned and current; SBOM tooling in the Rust ecosystem is still maturing.

---

## Questions & Resolutions

- **[Resolved]** - **Indexing poll backoff**: Exact backoff shape for the indexing poll — linear (`10s × attempt`) is the baseline; confirm against observed crates.io indexing latency during the bootstrap publish and adjust `MAX_RETRIES` / multiplier if needed.

  *Answer:* Confirmed that the linear backoff (`10s × attempt`) is robust and matches standard latency (which is typically under 2 minutes). We also harden the polling script (`wait-for-index.sh`) to detect and fail-fast on permanent registry errors (e.g. 403 Forbidden or network/configuration errors) instead of retrying to the timeout limit.

  *Revised:* The retry limit was raised from 12 (~13-minute ceiling) to 16 (~20-minute wait ceiling) so the budget covers the upper end of historical crates.io indexing slowdowns (15–20 minutes during outages) rather than only the typical case. The backoff shape, fail-fast behaviour, and idempotent re-run safety are unchanged; the only effect is a longer tail before a transient indexing lag red-lights a crate that already published successfully.

- **[Resolved]** - **SBOM toolchain provisioning**: Confirm `cargo cyclonedx` and `cyclonedx-cli` are available in the Nix devShell (for parity) or installed via SHA-pinned actions in the workflow, and pin both.

  *Answer:* Confirmed available and incorporated into the Nix devShell.

- **[Resolved]** - **GitHub Release creation mechanism**: `gh release create` vs `softprops/action-gh-release` (SHA-pinned); confirm changelog-section extraction reuses `scripts/check-changelog.sh` rather than duplicating logic.

  *Answer:* Uses `gh release create` (already a SHA-pinned-action-free CLI on the runner) wrapped in [`create-release.sh`](../../scripts/ci/create-release.sh) for idempotency. Extraction does **not** reuse `check-changelog.sh` — that script verifies changelog/history sync, a different concern from extracting one version's section; sharing would couple them. See section 9.

- **[Resolved]** - **Partial-batch failure semantics**: When a mid-chain crate fails after its predecessors published, confirm the documented recovery (re-run from the failed job; do not delete tags) matches the `needs:`-chain behaviour and the Mid-Run Failure Recovery section of [releasing.md](../project/releasing.md).

  *Answer:* The publish guard (section 6) makes both the failed-job re-run and a full-workflow re-run idempotent; already-published crates skip publishing, the GitHub Release updates in place. `releasing.md` Mid-Run Failure Recovery updated to direct re-running from the Actions UI (a `main` push does **not** re-trigger the tag-gated workflow) and to state the idempotency guarantee.

- **[Open]** - **Native cargo provenance verification**: Monitor `rust-lang/cargo#12661` for an upstream consumer-side verification path that would close the residual gap.

  *Notes/Thoughts*: Interim reliance on the GitHub offerings.

---

## References

<!-- Link to: related ADRs, design docs, downstream decisions this may spawn, spec/issue references. -->

* [ADR-0003: Workspace Crate Facade and Version Pinning Strategy](../adr/0003-workspace-crate-facade-and-version-pinning-strategy.md)
* [ADR-0004: Decoupled Crate Versioning Strategy](../adr/0004-decoupled-crate-versioning-strategy.md)
* [releasing.md — Release Workflow](../project/releasing.md)
* [forge-setup.md — Forge Setup, Rulesets, Signing Key Maintenance](../project/forge-setup.md)
* [Trusted Publishing on crates.io](https://crates.io/docs/trusted-publishing)
* [`rust-lang/crates-io-auth-action`](https://github.com/rust-lang/crates-io-auth-action)
* [`actions/attest-build-provenance`](https://github.com/actions/attest-build-provenance) · [`actions/attest-sbom`](https://github.com/actions/attest-sbom)
* [SLSA — Supply-chain Levels for Software Artifacts](https://slsa.dev/)
* [CycloneDX](https://cyclonedx.org/) · [`cyclonedx-rust-cargo`](https://github.com/CycloneDX/cyclonedx-rust-cargo)
* [SPDX](https://spdx.dev/) — ISO/IEC 5962:2021
* [NTIA — The Minimum Elements for an SBOM](https://www.ntia.gov/report/2021/minimum-elements-software-bill-materials-sbom)
* [NIST SP 800-218 — Secure Software Development Framework (SSDF)](https://csrc.nist.gov/pubs/sp/800/218/final)
* [Executive Order 14028 — Improving the Nation's Cybersecurity](https://www.federalregister.gov/documents/2021/05/17/2021-10460/improving-the-nations-cybersecurity)
* [OMB M-22-18 — Enhancing the Security of the Software Supply Chain](https://www.whitehouse.gov/wp-content/uploads/2022/09/M-22-18.pdf)
* Upstream tracking — native cargo provenance: [`rust-lang/cargo#12661`](https://github.com/rust-lang/cargo/issues/12661)

---

## Notes for Implementation

**Phase**: Phase 0 (Repository Infrastructure) — the final open Phase 0 item; required before the `0.1.0` publication at Phase 1 completion.

**Bootstrap sequence (one-time, just before the `0.1.0` publish)**:

1. Manual token-based `cargo publish` for each crate name to reserve it on crates.io (no Trusted Publishing yet).
2. Tag each crate with its baseline anchor `sdmx-<crate>/v0.0.0` via the `cargo-release` machinery (signed commit and tag) — not a flat `v0.0.0`.
3. Create the protected `release` environment (required reviewer = maintainer, self-review enabled).
4. Write and merge `publish.yml`.
5. Register Trusted Publishers per crate (repo, workflow, environment) — commands emitted by the print-only `scripts/registry-tp.sh` and verified by `just doctor-registry`; the crates.io UI is the fallback.
6. Enable enforcement per crate to disable API-token publishing (`registry-tp.sh --print-enforce`).
7. First real release (`0.1.0`) flows through the full pipeline.

The repository stays private until just before the Phase 1 publish.

**Migration & compatibility**: Additive infrastructure — no source code or public API is affected.

- `release.toml` is already correct for this model (`publish = false`, `sign-tag = true`, `push = false`): the maintainer signs and pushes the tag locally; CI publishes.
- [releasing.md](../project/releasing.md) Section 6 currently describes CI publishing as if the workflow exists; reconcile it with the actual `publish.yml` once written.
- [forge-setup.md](../project/forge-setup.md) must document the `release` environment creation and the Trusted Publisher registration steps.
- [SECURITY.md](../../SECURITY.md) must add the consumer-verification section (Key Decision 10).

**Testing strategy**:

- `actionlint` (via the existing `check-workflows` CI job) validates `publish.yml` syntax and security on infra changes.
- `just prepublish-check` (`cargo publish --dry-run` per crate, topological) validates packaging and metadata before any real publish; the `release-dry-run` CI job simulates the release sequence (plan, hooks, tag names) without packaging.
- The one-time manual bootstrap exercises packaging and naming end-to-end.
- After `0.1.0`, run the documented `gh attestation verify` and `git verify-tag` commands against the published artifacts to confirm the chain resolves end-to-end.
- With enforcement enabled, confirm an attempted API-token publish is rejected by crates.io.

**Federal-standard alignment summary**:

- SSDF PS.1/PS.2, OMB M-22-18 (ephemeral credentials): Trusted Publishing + enforcement.
- SLSA Build L2: `attest-build-provenance` on isolated GitHub-hosted runners.
- EO 14028 / NTIA SBOM minimum elements: per-crate CycloneDX + SPDX, attested.
- Residual gap: no native consumer-side provenance verification through `cargo` (ecosystem limitation; tracked).
