# Release Workflow

Maintainer-only. Documents how to cut releases using `cargo-release` and `git-cliff`. Contributors do not need to read this — see [CONTRIBUTING.md](../../CONTRIBUTING.md) for the contributor workflow.

This repository uses **`cargo-release`** and **`git-cliff`** to manage version bumps, changelog generation, and Git releases natively from your terminal. Every release follows a **version-named release branch** (`release/sdmx-rs/<version>`) with cryptographic GPG signatures on all commits and tags, providing a strong audit trail. The branch name carries the version so it lines up with the CI staging branch (`staging-release-sdmx-rs-<version>`) used in [§5](#5-push-to-github).

**Release model**: Two phases, governed by [ADR-0004](../adr/0004-decoupled-crate-versioning-strategy.md) and [ROADMAP.md — Versioning Strategy](../../ROADMAP.md):

- **Pre-1.0 (current)**: All crates release together in lockstep at the same version. Every release batch bumps every crate, even those without changes. This ensures a consistent, predictable `0.x.y` series while the API is still evolving.
- **Post-1.0**: Per-crate independent versioning. Only crates that actually changed are released in a given batch. The facade `sdmx-rs` is updated to pin to the new compatible versions.

The instructions below cover both phases. Phase-specific guidance is called out where the two workflows differ.

---

## 0. Pre-Release Checklist

**Complete these steps in order before executing the release**:

- [ ] **[Section 0](#0-pre-release-setup)**: Pre-release setup (sync, scaffolding, create branch, dry-run, pre-publish)
- [ ] **[Section 1](#1-review-changelogs--curate-facade-release-notes)**: Review changelogs & curate facade release notes
- [ ] **[Section 2](#2-commit-changelogs)**: Commit changelogs as a signed checkpoint
- [ ] **[Section 3](#3-execute-the-release)**: Execute cargo release for each crate
- [ ] **[Section 4](#4-merge-release-branch-back-to-main)**: Merge release branch back to main
- [ ] **[Section 5](#5-push-to-github)**: Stage on CI, await green gate, land on main and push tags
- [ ] **[Section 6](#6-ci-publishes-to-github--cratesio)**: Monitor CI publishing to GitHub & crates.io

---

## 0. Pre-Release Setup

[↑](#0-pre-release-checklist)

**Sync with origin, verify scaffolding, create release branch, run dry-run, and pre-publish validation**:

1. **All commits are GPG-signed and pushed to `main`**:
   The release workflow starts from `main`; verify your development work is committed, tested, and in sync with `origin`.

2. **Verify scaffolding dependencies are clean**:
   ```bash
   scripts/check-scaffolding.sh
   ```
   At phase boundaries (e.g., releasing 0.2.0 to mark Phase 1 complete), all ignored entries should either be actively used in the implementation (and removed from the ignored list), or `PERMANENT` with a clear reason. If this check warns about unused Phase N entries, either implement them now or move their phase annotation to Phase N+1.

3. **Decide which crates to release**:

   > **Pre-1.0**: Release **all crates together** at the same version, regardless of which ones have changes. Skip this review step — the crate list is always the full set.
   >
   > **Post-1.0**: Review which crates have changes since their last tag. Only run `cargo release` for crates actually releasing this batch:
   > ```bash
   > git tag -l "sdmx-*/v*" --sort=-version:refname | head -5
   > ```

4. **Create release branch and prepare the lockstep batch**:
   ```bash
   git checkout -b release/sdmx-rs/<version>   # e.g. release/sdmx-rs/0.2.0
   just prep-release <version>                  # e.g. just prep-release 0.2.0
   ```

   > **Pre-1.0 ONLY.** `prep-release` bumps **every** crate's `Cargo.toml` to `<version>` and creates one signed `chore(release): prepare release batch <version>` commit. This single commit touches every crate's manifest, which is what gives a **no-op** crate (one with no real changes this batch) a path-touching commit in its `--include-path` scope. Without it, `git-cliff` would find zero in-range commits for that crate and omit its release section entirely, and `cargo-release` would later skip the crate (no version bump). Running it first means `cargo release` (section 3) executes on the pre-bumped tree and does **not** re-bump.
   >
   > **Post-1.0**: skip this step — decoupled versioning means only crates with real changes release, and no-op release sections don't occur.

5. **Generate changelogs**:
   ```bash
   just changelog-generate
   ```
   This generates all `CHANGELOG.md` files in `crates/*/CHANGELOG.md` and `crates/sdmx-rs/CHANGELOG.md` (do not commit yet). A no-op crate's section renders as "No user-facing changes in this release." — this is correct (see [cliff.toml](../../cliff.toml)).

6. **Dry-run the release sequence**:

   > [!NOTE]
   > **Version increment rules**: Version bumps are derived automatically from conventional commit types since the last tag:
   > - `fix:` or `perf:` → patch version
   > - `feat:` → minor version
   > - `feat!` or `BREAKING CHANGE:` footer → major version
   > - `refactor:`, `chore:`, `docs:`, `style:`, `test:`, `ci:`, `build:`, and other types → **no version bump** (crate does not need releasing)
   >
   > If a crate has only non-version-bumping commits since its last tag, `cargo-release` will skip it. See [cliff.toml](../../cliff.toml) and [release.toml](../../release.toml) for exact configuration. Review the dry-run output to confirm which crates are actually releasing.

   ```bash
   just release-dry-run sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs
   ```
   Review the output for version increments, the derived publish order, and any release-config or hook errors. The dry-run simulates the release sequence only (plan, hooks, tag names); it does not compile or package anything, and packaging is validated separately at step 8. If issues appear, fix them on a feature branch, merge to main, create a fresh release branch, and restart from section 0.

7. **Curate the facade release notes** (see [§1](#1-review-changelogs--curate-facade-release-notes) for the full guidance):
   The facade's user-facing prose lives in a curated file that drives its GitHub Release body. Scaffold it from the template, curate every section, then verify — before pre-publish validation, because the gate enforces it:
   ```bash
   just new-release-notes <version>
   $EDITOR crates/sdmx-rs/release-notes/<version>.md  # fill every section; delete the template guidance
   just check-release-notes <version>
   ```

8. **Pre-publish validation**:
   Validate that all crates will publish successfully to crates.io without actually publishing:
   ```bash
   just prepublish-check
   ```
   This runs `cargo publish --dry-run` for each crate in topological order (catching missing license, oversized package, invalid metadata) **and** re-runs `check-release-notes` for the facade, so the curated notes from step 7 must already exist.

   > [!NOTE]
   > This check proves packaging, metadata, licences, and compilation, not registry lookup. To package a crate for the dry-run, cargo strips the workspace `path` from each intra-workspace dependency and would resolve the bare `version = "=X"` pin against crates.io. That pinned version is never on the index at check time: the in-tree pins are `=0.0.0`, and at release time `prep-release` rewrites them to the not-yet-published batch version. So `prepublish-check` injects a `[patch.crates-io]` overlay (via cargo `--config`, mutating no file) that points each `sdmx-*` dependency at its workspace path, and the verify step compiles and validates against the local sources. Resolving the real pins against the registry is left to publish time: `publish.yml` blocks each crate on `wait-for-deps` until its dependencies are indexed, then the real publish resolves them per crate. Because the overlay drops the "dependency must already be published" precondition, this check runs from section 0 as written on every release, including the first.

---

## 1. Review Changelogs & Curate Facade Release Notes

[↑](#0-pre-release-checklist)

There are **two distinct artefacts**, and conflating them is a trap the gates will catch:

1. **`CHANGELOG.md` (all five crates) — the machine record.** These are strict `git-cliff` output and **must not be hand-edited**: the `check-changelog` gate diffs each one byte-for-byte against what `git-cliff` regenerates, with no facade exemption, so any manual edit makes that gate fail. Review them only to confirm the auto-generated content is accurate; if something reads wrong, fix the underlying **commit messages** (on a feature branch, merged to main) and regenerate — never edit the `CHANGELOG.md` directly.

2. **`crates/sdmx-rs/release-notes/<version>.md` (facade only) — the curated, user-facing record.** This is where human curation lives. Because `CHANGELOG.md` is machine-locked, the facade's user-facing prose is authored in a **separate, curated file** that drives the facade's GitHub Release body. Write it for a *consumer of the facade*:
   - Explain breaking changes clearly, with migration guidance.
   - Highlight new capabilities and notable behaviour changes.
   - Note significant dependency or MSRV updates.
   - Use language facade users (not implementers) care about — e.g. instead of "refactor(parsers): restructure constraint validation", write "Constraint validation now reports clearer errors."

   ```bash
   # Scaffold from the template; curate every section; then verify the gate:
   just new-release-notes <version>                  # e.g. just new-release-notes 0.2.0
   $EDITOR crates/sdmx-rs/release-notes/<version>.md
   just check-release-notes <version>
   ```

   This file is a **mandatory precondition** of cutting a facade release. `just check-release-notes` (also folded into `prepublish-check`, §0 step 8) fails unless it exists, carries **every required section** (state the negative — e.g. "No bug fixes in this release." — never delete a section), and retains **no unedited template guidance**. It **must** pass before `cargo release --execute` (§3), because the release tag push is irreversible and the GitHub Release that consumes this file is only created afterward in CI.

> [!NOTE]
> **Why two files.** `CHANGELOG.md` is the machine record (gated, never curated); `release-notes/<version>.md` is the curated facade record (mandatory, drives the Release body). Leaf crates (`sdmx-types`, `sdmx-parsers`, `sdmx-writers`, `sdmx-client`) are **not** curated — their GitHub Release body is the auto changelog section, or a provenance placeholder when a lockstep batch leaves a crate with no user-facing changes. See [docs/design/0004 §9](../design/0004-release-publish-pipeline-and-supply-chain-provenance.md) and [`crates/sdmx-rs/release-notes/README.md`](../../crates/sdmx-rs/release-notes/README.md).

---

## 2. Commit Changelogs

[↑](#0-pre-release-checklist)

Once you've reviewed and edited the changelogs:

```bash
just release-commit-changelogs
```

This creates a signed checkpoint of the changelog prep work before executing `cargo release`.

> [!NOTE]
> If you want to verify changelog sync manually (`just check-changelog`), run it **after** this commit, not before. The check requires a clean working tree and will abort with `Git working tree is dirty` while the generated changelogs from section 1 are still uncommitted — this is a safe, no-op failure, but a confusing one if run too early.

---

## 3. Execute the Release

[↑](#0-pre-release-checklist)

> [!NOTE]
> **Optional re-dry-run**: If you edited Cargo.toml files or reconsidered crate selection during changelog curation (section 1–2), re-run `just release-dry-run` before executing below. If issues appear, fix on a feature branch, merge to main, create a fresh release branch, and restart from section 0.

Run `cargo release --execute` for each crate you're releasing, in topological order.

> **Pre-1.0 — release all crates together at the same version**:
> Pass the **same `<version>`** you gave `just prep-release` in section 0 (e.g. `0.2.0`). The tree is already bumped to it, so cargo-release confirms that version rather than re-bumping (you'll see "Publishing", not "Upgrading"). The positional is the target VERSION, and `-p` selects the package — `cargo release` takes a *version*, never a bare crate name, as its positional argument.
> ```bash
> cargo release -p sdmx-types   <version> --execute
> cargo release -p sdmx-parsers <version> --execute
> cargo release -p sdmx-writers <version> --execute
> cargo release -p sdmx-client  <version> --execute
> cargo release -p sdmx-rs      <version> --execute
> ```

<!-- -->

> **Post-1.0 — only release crates that actually changed;** unchanged crates keep their current version:
> Each crate may take a different version. Pass the target VERSION as the positional (or a bump LEVEL such as `minor`/`patch` to derive it); `-p` selects the package.
> ```bash
> # Example: only types and client are releasing this batch
> cargo release -p sdmx-types  <version> --execute
> cargo release -p sdmx-client <version> --execute
> cargo release -p sdmx-rs     <version> --execute   # facade always updates to pin new versions
> ```

Each `--execute` invocation:
- Bumps the version in that crate's `Cargo.toml`
- **Rewrites the facade's exact pin to this crate** — `cargo-release` updates the matching `sdmx-<crate> = { version = "=X.Y.Z", … }` requirement in `crates/sdmx-rs/Cargo.toml` to the new version (default `dependent-version = "upgrade"`), preserving the `=` operator mandated by [ADR-0003](../adr/0003-workspace-crate-facade-and-version-pinning-strategy.md). Because the facade is released **last** in the topological order, by the time `cargo release -p sdmx-rs` runs its pins already point at the new versions — no manual pin editing is ever required, in either phase.
- Runs the pre-release hook (`git-cliff` generates CHANGELOG.md)
- Creates a signed release commit
- Creates a signed annotated tag (e.g., `sdmx-types/v2.1.0`)
- Leaves everything local (`push = false` in `release.toml`)

> [!NOTE]
> **Intentional asymmetry**: The `--execute` step omits `--no-confirm` deliberately — you must manually approve version increments, commits, and tags before they are stamped. **This friction is deliberate.** It forces you to review what you're about to sign.
>
> **Why not a just recipe for this step?** Deliberately manual to preserve the approval prompt. If a crate's `cargo release` fails mid-execution, you'll see it immediately, debug it, fix it on a new branch, merge, and start a fresh release batch.
>
> **Tmux / headless terminals**: Ensure your GPG agent (`gpg-agent` with `pinentry-curses` or `pinentry-tty`) is configured to capture passphrase entry, or pass `--no-confirm` if you're confident.

**If one crate's release fails**:

1. Abort early (Ctrl-C) — nothing has been pushed
2. Fix the underlying issue on a feature branch (merge to main)
3. Create a fresh release branch and restart from section 0
4. Do not try to "resume" the partial release — it's cleaner to restart

---

## 4. Merge Release Branch Back to Main

[↑](#0-pre-release-checklist)

Now that all version bumps and tags are on the release branch, merge back to `main` with a signed merge commit. This creates an audit milestone:

```bash
just release-merge
```

This script automatically:
- Checks out `main` and pulls from origin
- Extracts version numbers from each crate's Cargo.toml
- Creates a signed merge commit with a structured message listing Released and Unchanged crates
- Groups all releases from this batch in one merge

The script **stops at a local merge and does not push** — this gap is a deliberate review gate. Before pushing (section 5), inspect the proposed merge commit while it is still local and discardable:

```bash
git show HEAD   # review the message, diff, and signature
```

If anything is wrong, abort cheaply — the local merge has touched nothing on the remote, so `git reset --hard @{1}` (or a fresh release branch) discards it with no consequence. Pushing is the first step toward the irreversible, tag-triggered publish chain, so the human checkpoint precedes it by design. The CI `verify-tag-on-main` gate (section 6) is a backstop for this invariant, not a replacement for the review.

---

## 5. Push to GitHub

[↑](#0-pre-release-checklist)

Two commands handle the staging-then-land sequence:

```bash
# Push merge commit to a staging branch and block until the CI Quality Gate is green.
# Exits 0 on success; exits 1 immediately on any check failure or after a 12-minute
# timeout — investigate before proceeding.
SDMX_MAIN_REMOTE=all just stage-merge <version>

# Once stage-merge exits 0: fast-forward main, push all per-crate tags, clean up staging.
SDMX_MAIN_REMOTE=all just release-push <version>
```

`stage-merge` pushes `HEAD` to `staging-release-sdmx-rs-<version>` (matched by the CI `staging-*` trigger), then polls the GitHub check-run API until the Quality Gate is terminal. The poll requires `gh auth login`; if `gh` is not authenticated it degrades gracefully — warns and exits 0 with the manual hint to run `release-push` once you have confirmed CI is green yourself.

`release-push` first **re-validates that local `HEAD` still matches the CI-verified staging commit**: it fetches `origin/staging-release-sdmx-rs-<version>` (the exact SHA `stage-merge` earned a green Quality Gate on) and refuses to proceed unless `HEAD` equals it. This closes the window between `stage-merge` and `release-push` where local `HEAD` could drift — a new commit, an amend, a different branch checked out — which would otherwise land source CI never saw on `main` and fire the irreversible publish on it. It is **fail-closed**: if the staging branch cannot be fetched (you never ran `stage-merge`, or the remote is unreachable), it aborts and tells you to stage first. The fetch always targets `origin` (where CI runs) even when `SDMX_MAIN_REMOTE` fans pushes out to a mirror.

Once `HEAD` is confirmed, `release-push` fast-forwards `main` via `HEAD:refs/heads/main` (no `--follow-tags`), then pushes tags in a separate step. The split avoids the race between the code push and `publish.yml`'s `verify-tag-on-main` gate, which fires on tag push and asserts the tag commit is already reachable from `main`.

`SDMX_MAIN_REMOTE=all` fans out to every forge mirror in one operation — the `all` remote pushes to both GitHub and Codeberg simultaneously, keeping them in full lockstep including tags. Both commands default to `origin` (GitHub only) without the override, so a contributor's clean clone works without any remote configuration.

This sends to **all forge mirrors**:
- The merge commit to `main`
- All per-crate tags (e.g., `sdmx-types/v2.1.0`, `sdmx-client/v1.0.1`)

> [!NOTE]
> Tags are pushed **after** the merge commit lands on `main`. `verify-tag-on-main.sh` (gate 4 in `publish.yml`) asserts the tag commit is reachable from `origin/main` — pushing tags before the merge would cause that gate to fail.

---

## 6. CI Publishes to GitHub & crates.io

[↑](#0-pre-release-checklist)

After pushing (section 5), **each `<crate>/v<version>` tag triggers its own independent `publish.yml` run** that publishes exactly that one crate. Pushing all tags together (`just release-push` runs `git push --tags`) therefore starts one run per released crate; cross-crate ordering is handled by the index (a run waits for its own dependencies to be published), not by a single chained pipeline.

Within a run, the four pre-publish checks (steps 1–4) are a **strict chain of separate jobs** (`needs:` edges in `publish.yml`), not parallel gates: each blocks until the previous passes, so **the first red job in the Actions UI is the cause — every job after it shows "didn't run," not a second failure.** Step 5 is a manual approval stop; steps 6–9 are the ordered steps of the `publish` job itself. Reading the run top-down tells you exactly where it stopped.

1. **Gate — verify maintainer signature** — `verify-signature.yml` (reusable workflow). Nothing downstream runs unless **both the tag object and the commit it points at** are maintainer-signed. Verifying the tag commit too (not just the tag wrapper) is load-bearing: publishing keys off the *tag commit*, so a signed tag around an unsigned commit must not be enough to publish.
2. **Gate — validate changelogs** — `check-changelog` job (`scripts/check-changelog.sh`). `needs: [verify-signature]`.
3. **Gate — resolve tag** — `setup` job parses the tag into crate + version and **asserts the tag version matches the crate's `Cargo.toml`** (the signed tag must agree with the artifact); aborts on mismatch. `needs: [check-changelog]`.
4. **Gate — verify source is in main** — `verify-tag-on-main` job asserts the tag's commit is reachable from `origin/main` (`scripts/ci/verify-tag-on-main.sh`). `needs: [setup]`. This enforces that nothing publishes whose source is not on the canonical branch — the backstop for the section 4 review gate. Because publishing keys off the *tag commit* (not the merge commit), a tag pushed before its merge lands on `main` would otherwise publish orphaned source; this gate refuses it.
5. **Manual approval (human stop)** — the `publish` job declares `environment: release` and `needs: [setup, verify-tag-on-main]`. Once gate 4 passes, GitHub **pauses and prompts the maintainer for approval** before the job starts. A run that looks "stuck" here is waiting on *you*, not on CI — approve it in the Actions UI to proceed. Steps 6–9 below are the ordered steps of this job, running once approved.
6. **Wait for workspace dependencies** — blocks until the crate's own `sdmx-*` dependencies are indexed (no-op for `sdmx-types`).
7. **Publish to crates.io** — authenticate via Trusted Publishing (ephemeral OIDC token), package, publish (with verification), then poll the index. Publish + index-poll are **skipped if the crate's exact version is already on crates.io** (sparse-index check), so re-running is safe — see **Mid-Run Failure Recovery**.
8. **Generate and attest SBOMs** — CycloneDX and SPDX SBOMs; three attestations written to the GitHub attestation store: build provenance (SLSA L2), CycloneDX SBOM, SPDX SBOM.
9. **Create GitHub Release** — created (or, on a re-run, updated in place) for the crate tag, with SBOM files attached as assets. The notes body depends on the crate: the **facade** uses its curated `crates/sdmx-rs/release-notes/<version>.md` (CHANGELOG section as a backstop); a **leaf** uses its auto `CHANGELOG.md` section, or a provenance placeholder when a lockstep batch left it with no user-facing changes (the Release still exists to host that crate's attestations and `.crate`).

Monitor the CI runs. If publishing fails, see **Mid-Run Failure Recovery** below.

> [!WARNING]
> **Handling missed breaking changes**: If a breaking change was merged under an incorrect commit type (e.g., `fix` instead of `feat!`), do not force it out as a patch release. Reword the commit on `main` via the normal contributor flow (a fresh PR), then start over on a new release branch. Do not rewrite history on the release branch to fix it in place: the release commits and tags `cargo release` stamps point at specific hashes, and rewording rewrites those commits and orphans the tags — the same hazard described in [What Not to Do](#what-not-to-do) ("Don't amend the release commit").

---

## What Not to Do

These mistakes are easy to make but have costly consequences. Avoid them:

- **❌ Don't run `cargo release --workspace --execute`**
  - If one crate fails mid-run, partial state is hard to recover
  - Post-1.0: also releases unchanged crates, inflating version numbers unnecessarily
  - Instead: release each crate individually in topological order (see section 3)

- **❌ Don't skip `just prepublish-check` before executing**
  - Saves a few seconds locally but costs CI time later
  - Catches metadata errors that would fail the publish job
  - Instead: always run prepublish-check as part of section 0, step 8

- **❌ Don't edit crate versions in Cargo.toml manually**
  - cargo-release has logic for managing versions and dependencies
  - Manual edits bypass that logic and can break the release
  - Instead: let cargo-release handle all version bumps via --execute

- **❌ Don't amend the release commit after it's been created**
  - Cargo-release creates signed commits and tags pointing to a specific hash
  - Amending the commit changes the hash, breaks the tag pointer
  - All subsequent work is now out of sync
  - Instead: if you need changes, abort and start a fresh release batch

- **❌ Don't run `git merge --no-ff -S ...` manually**
  - That's what `just release-merge` does (and does it correctly)
  - The script handles version extraction, change detection, and message formatting
  - Instead: always use `just release-merge`

- **❌ Don't force-push after the merge commit is on main**
  - Force-pushing rewrites history and breaks tags
  - The merge commit is now immutable
  - Instead: if you must undo, use `git revert` (creates a new commit)

- **❌ Don't delete tags if publish fails**
  - Tags are permanent after being pushed to GitHub
  - Deleting them locally doesn't affect what's on crates.io
  - Deleting them is confusing and hides what happened
  - Instead: see **Mid-Run Failure Recovery** for proper handling

---

## Bootstrap Record

The five crate names were reserved on crates.io on 2026-07-09 by publishing synthetic placeholders at `0.1.0-alpha.1`; the API token used was revoked afterwards. A pre-release version never matches an ordinary version requirement, so the placeholders are invisible to dependency resolution and were not yanked.

| Crate          | Registry index checksum at publish                                 |
|----------------|--------------------------------------------------------------------|
| `sdmx-types`   | `0d1bac4e9b71274162a7c4394f01246d4879a049f884c560ef197f044f43c554` |
| `sdmx-parsers` | `e2154cfdab22066f2ecbea37f7a3124cf959367d527401576b39a7d5ceb04a2a` |
| `sdmx-writers` | `d5263f51138a8fa40bf83b2890a89a87d11ff705efdc6dc53eba2c6a019b31a8` |
| `sdmx-client`  | `b27c6168ed5b4131f4de1dbdd4e232652dba88b5a013573998e7efbaf5a9588b` |
| `sdmx-rs`      | `8d1d9978e4d005206bf7e937663f6f52853b0e72c93cd414890c3c6fa486ef4a` |

The in-tree `=0.0.0` pins reference a version that was never published and never will be; this is harmless because `prep-release` rewrites every pin to the batch version at release time.

> [!IMPORTANT]
> **Never create or push a `sdmx-*/v0.0.0` git tag.** `0.0.0` is never published, and a `v0.0.0` tag would match `publish.yml`'s `tags: ['sdmx-*/v*']` trigger and fire the publish workflow for a version that must not exist on the registry. The first tags the pipeline ever sees are the `sdmx-<crate>/v0.1.0-alpha.2` rehearsal tags. As a backstop, `publish.yml`'s tag validation rejects any tag naming the `0.0.0` core before the publish path can act on it.

"Require Trusted Publishing for all new versions" is enabled for all five crates and no API token exists, so token-based publishing is structurally impossible. If a Trusted Publisher binding ever proves misconfigured, the emergency path is the crate owner toggling that setting off first in the crates.io web UI (**crates.io → Your crates → `<crate-name>` → Settings → Trusted Publishing**). Verify the standing state at any time with `REGISTRY_ENFORCEMENT_REQUIRED=1 just doctor-registry`; registry procedures live in [registry-setup.md](registry-setup.md).

---

## Mid-Run Failure Recovery

If a crate's publish run fails (e.g., crates.io indexing lag causes the `sdmx-writers` run to fail while the `sdmx-types` run already succeeded):

1. **Do not delete tags locally** — crates already published to crates.io are permanent.
2. **Do not force-push** — the merge commit is already on `main`.
3. **Wait 2–5 minutes** for crates.io indexing to catch up.
4. **Re-run the failed run from the GitHub Actions UI** (its **Re-run jobs** button). Each run is idempotent: the publish step checks the crates.io sparse index first and skips its version if already published, and the GitHub Release step updates an existing release in place rather than failing. So a re-run is a no-op for a crate that actually succeeded and a clean retry for one that did not.
   > [!NOTE]
   > `publish.yml` triggers on **tag push only** (`on: push: tags`). Pushing an empty commit to `main` will **not** re-trigger it — re-run from the Actions UI, which replays the workflow against the original tag ref. (Deleting and re-pushing the tag would also re-trigger, but tag deletion is discouraged — see [What Not to Do](#what-not-to-do).)
5. **If a specific crate's publish failed permanently**:
   - Investigate the crate's code (compilation error, dependency issue)
   - Fix it on a new branch
   - Merge to `main`
   - Run a new release batch with just that crate

---

## Rollback

If you need to rollback a release that's already merged to `main`:

1. **Revert the merge commit**:
   ```bash
   git revert -m 1 <merge-commit-hash> --gpg-sign
   git push origin main
   ```
   This creates a new signed commit that undoes the release.

2. **Do not delete the tags** — they're permanent on crates.io and in the repository history.

3. **If a crate was already published, yank it on crates.io.** Use the web UI — **crates.io → Your crates → `<crate>` → version → Yank** — which authenticates via your crates.io web session:
   > [!IMPORTANT]
   > `cargo yank` is **not** available here. Trusted Publishing issues only an ephemeral, publish-scoped token inside the CI job; it leaves no API token on the maintainer's machine, and yank requires one. Rather than mint a temporary token during an incident — reintroducing exactly the long-lived credential the pipeline was built to eliminate — yank from the web UI.
   >
   > Yanking prevents new dependency resolutions against that version but does not affect existing lockfiles — users with `Cargo.lock` checked in will continue using the yanked version unless they explicitly resolve or update.

4. **Fix the issues** on a new branch and prepare a new release batch.
