# Forge Setup

Maintainer-only. Documents how to configure forge hosting for this repository from scratch. Contributors do not need to read this — see [CONTRIBUTING.md](../../CONTRIBUTING.md) for the contributor workflow.

This repository is hosted on GitHub as its primary forge, with a private Codeberg mirror maintained via dual-push. Each forge section below is self-contained.

Set these shell variables before running any commands in this document. The defaults reflect the live repository; adjust for a different target:

```bash
OWNER="dgalbraith"
REPO="sdmx-rs"
SIGNING_EMAIL="dg@lbraith.io"  # maintainer's GPG signing identity
```

---

## GitHub

### Prerequisites

- The `gh` CLI installed and authenticated (`gh auth login`).
- The maintainer's GPG public key uploaded to their GitHub account under **Settings → SSH and GPG keys**.
- The maintainer's signing email added as a verified address on their GitHub account under **Settings → Emails**. This is required for GitHub to show the "Verified" badge on signed commits and for `required_signatures` enforcement to accept pushes.

Verify `gh` authentication before applying any configuration:

```bash
gh auth status
```

---

### Local Maintainer Git Configuration

Configure the local clone to sign with the maintainer key **before** creating the signed root commit (step 3 below) — without this, `git commit --gpg-sign` cannot select the right key and the `required_signatures` ruleset rejects the push. This is local client configuration, distinct from the GitHub-side key *registration* in [step 2](#2-register-the-signing-key).

```bash
git config --global user.name      "David Galbraith"
git config --global user.email     "dg@lbraith.io"                              # MUST match the GitHub-verified signing email
git config --global user.signingkey B43D054479B0A9374BC35C167D4A0D2EE2E2ECD7!   # signing [S] subkey; trailing ! pins this subkey specifically
git config --global commit.gpgsign true
git config --global tag.gpgsign    true                                         # releases sign tags, not only commits
```

> [!IMPORTANT]
> `user.email` must exactly match a **verified** address on the GitHub account (see Prerequisites). A mismatch makes commits show as "Unverified" despite a valid signature, and `required_signatures` rejects the push. The signing key is the `[S]` subkey from the [maintainer key register](#current-maintainer-key-register); the trailing `!` pins it so GPG does not fall back to the primary or another subkey.
>
> The signing-subkey fingerprint changes on [subkey rotation](#signing-subkey-rotation) — update `user.signingkey` locally as part of that procedure. Interactive signing also needs `GPG_TTY`, exported by the repository's `.envrc`.

---

### Base Repository Setup

Complete these steps once when creating the repository. The sequence matters: the signing key must be registered before the root commit so that GitHub can verify it, the root commit establishes a signed history anchor before any branch protection is active, and labels are reset before issue workflows depend on them.

#### 1. Create the repository

```bash
gh repo create "${OWNER}/${REPO}" \
  --private \
  --description "A Rust implementation of the SDMX (Statistical Data and Metadata Exchange) standard." \
  --clone
cd "${REPO}"
```

> [!NOTE]
> Create as private initially. Visibility can be changed to public once the repository is ready (`gh repo edit "${OWNER}/${REPO}" --visibility public`). Changing visibility later has no effect on git history or rulesets.

#### 2. Register the signing key

The maintainer's GPG public key must be registered with GitHub before the root commit, so that GitHub can verify the signature on push.

**Upload the public key**:

```bash
gh api --method POST user/gpg_keys \
  -F armored_public_key="$(gpg --armor --export "${SIGNING_EMAIL}")"
```

Or upload manually: **GitHub → Settings → SSH and GPG keys → New GPG key**.

**Verify key registration and email status**:

Verifying GPG key registration and email state requires the `admin:gpg_key` and `user` scopes, which are not needed for BAU operations. Acquire them, run the checks, then relinquish them by restoring the BAU token scope:

```bash
# Acquire verification scopes
gh auth refresh -h github.com -s admin:gpg_key -s user

# Verify the key is registered and the signing email is verified
gh api user/gpg_keys | jq -r '.[] | "\(.key_id)\t\(.emails[].email)"'
gh api user/emails | jq -r '.[] | "\(.email)\tverified=\(.verified)"'

# Relinquish — restore BAU token scope
gh auth refresh -h github.com -s admin:public_key -s gist -s read:org -s repo
```

The signing email must appear as `verified=true`. Unverified addresses cause commits to show as "Unverified" even with a valid signature, and `required_signatures` will reject pushes.

#### 3. Establish the signed root commit

Create an empty root commit to anchor the signed history before any code lands. This is commit 0 — the immutable foundation of the audit trail.

```bash
git commit --allow-empty --gpg-sign -m "chore(repo): establish signed repository root"
git push -u origin main
```

Verify the root commit is signed:

```bash
git log -1 --format="%H %G? %GK %GS"
# G? = G (good signature), GK = signing key fingerprint, GS = signer name
```

#### 4. Reset labels

GitHub creates nine default labels on every new repository. The setup below deletes all of them and recreates exactly the labels this project needs. This ensures the label set is fully owned, drift-proof, and reproducible — no inherited state from GitHub defaults.

The label set has two orthogonal axes that can be combined on a single issue:

- **Commit-type labels** — the nature of the change that will close the issue, aligned with Conventional Commits
- **Triage labels** — the state or audience of the issue, independent of change type

> [!NOTE]
> GitHub's native "not planned" close reason covers the `wontfix` use case without needing a label. Use it when closing issues that will not be addressed.

**Delete all default labels**:

```bash
for label in \
  "bug" "documentation" "duplicate" "enhancement" \
  "good first issue" "help wanted" "invalid" "question" "wontfix"; do
  gh label delete "${label}" --yes 2>/dev/null && \
    echo "Deleted: ${label}" || echo "Not found (already deleted?): ${label}"
done
```

**Create the full label set**:

```bash
# Commit-type labels — align with Conventional Commit types
gh label create "feat"          --color "0e8a16" --description "New feature implementations and structural enhancements"
gh label create "fix"           --color "d93f0b" --description "Targeted bug fixes and code corrections"
gh label create "docs"          --color "0075ca" --description "Documentation-only modifications and additions"
gh label create "refactor"      --color "0052cc" --description "Code changes that neither fix a bug nor add a feature"
gh label create "perf"          --color "006b75" --description "Code changes that explicitly improve execution performance"
gh label create "test"          --color "fef2c0" --description "Adding missing tests or correcting existing test suites"
gh label create "ci"            --color "0369a1" --description "Changes to continuous integration configurations, scripts, and automation workflows"
gh label create "build"         --color "93c5fd" --description "Changes affecting the build system, workspace layouts, or external dependencies"
gh label create "chore"         --color "5319e7" --description "Repository maintenance, toolchain shifts, and meta-configuration updates"

# Triage labels — cross-cutting, stack with commit-type labels
gh label create "maintenance"      --color "ffa500" --description "Scheduled maintenance obligations, dependency reviews, and toolchain upkeep"
gh label create "breaking"         --color "9b1c1c" --description "Introduces a breaking change; requires migration notes and semver major bump"
gh label create "good first issue" --color "7057ff" --description "Well-scoped issue suitable for first-time contributors"
gh label create "help wanted"      --color "ec4899" --description "Maintainer is requesting outside input or contributions"
gh label create "duplicate"        --color "cfd3d7" --description "Tracks a problem already reported in another issue"
```

The complete palette and its semantic logic:

| Label              | Colour    | Family           | Semantic alignment                       |
|--------------------|-----------|------------------|------------------------------------------|
| `feat`             | `#0e8a16` | Green            | Additive / growth                        |
| `fix`              | `#d93f0b` | Red-orange       | Corrective                               |
| `breaking`         | `#9b1c1c` | Dark red         | Highest severity — distinct from `fix`   |
| `docs`             | `#0075ca` | Blue             | Informational                            |
| `refactor`         | `#0052cc` | Deep blue        | Structural                               |
| `ci`               | `#0369a1` | Steel blue       | Infrastructure (not corrective)          |
| `build`            | `#93c5fd` | Medium-pale blue | Tooling / supporting                     |
| `perf`             | `#006b75` | Teal             | Technical improvement                    |
| `chore`            | `#5319e7` | Purple           | Meta / admin                             |
| `test`             | `#fef2c0` | Pale yellow      | Supporting work                          |
| `maintenance`      | `#ffa500` | Amber            | Scheduled obligations / toolchain upkeep |
| `good first issue` | `#7057ff` | Violet           | Well-scoped for first-time contributors  |
| `help wanted`      | `#ec4899` | Pink             | Community / participation                |
| `duplicate`        | `#cfd3d7` | Grey             | Already reported in another issue        |

Verify the full label set:

```bash
gh api "repos/${OWNER}/${REPO}/labels" | jq -r '.[] | "\(.name)\t#\(.color)\t\(.description)"' | sort
```

After the initial bootstrap, use `just update-labels` to apply colour or
description changes — it PATCHes existing labels and POSTs new ones. It does
not delete labels (the delete pass is bootstrap-only) and does not perform
renames.

**Renaming a label** is non-destructive at the API level (the label ID is
preserved and all tagged issues/PRs follow the rename automatically), but
`just update-labels` cannot perform renames because the spec only carries the
desired name, not the old name. Perform renames manually:

```bash
gh api --method PATCH "repos/${OWNER}/${REPO}/labels/{old-name}" \
    -f new_name="{new-name}" -f color="{color}" -f description="{description}"
```

After the manual rename, `just doctor-forge` will confirm the live label matches
the spec. No spec change is needed — `scripts/lib/forge-spec.sh` already
carries the new name.

---

### Repository Settings

Disable merge methods that would allow GitHub-mediated (web-flow-signed) merges. Only standard merge commits are permitted — squash and rebase both create new commits not signed by the maintainer.

```bash
gh api --method PATCH "repos/${OWNER}/${REPO}" \
  -F allow_squash_merge=false \
  -F allow_rebase_merge=false \
  -F allow_merge_commit=true \
  -F allow_auto_merge=false \
  -F delete_branch_on_merge=true
```

> [!NOTE]
> GitHub requires at least one merge method to be enabled. `allow_merge_commit=true` satisfies this requirement. The merge button remains visible in the GitHub UI but must not be used — doing so produces a web-flow-signed commit that violates the maintainer-signed-only invariant and will be detected by the `verify-signature` CI job.
>
> `delete_branch_on_merge=true` automatically removes `feature/*` and `staging-*` branches when their PR is merged or closed. `main` is protected against deletion by Ruleset 1 regardless of this setting; `release/*` branches are local-only and are unaffected.

---

### Branch Rulesets

The repository uses three rulesets on `main` and release tags. All bypass actors have been removed — no role may push to `main` without a CI-verified SHA. The staging branch pattern (see [workflow.md — Normal Development Flow](../dev/workflow.md#stage-5-the-merge-maintainer-integration)) is the mechanism that satisfies this without requiring re-running CI on every push.

> [!NOTE]
> The desired forge state below (rulesets, labels, merge flags, the `release` environment) is captured as machine-readable artifacts under [`forge/`](../../forge/README.md) and tooling: `just doctor-forge` verifies live config against that spec (read-only); `scripts/forge-apply.sh` applies it as a guarded one-shot bootstrap; and `just update-rulesets`, `just update-labels`, and `just update-actions-allowlist` handle idempotent post-bootstrap updates to those three surfaces. The registry plane is the sibling [registry-setup.md](registry-setup.md).

#### Design rationale

The zero-trust invariant requires:

1. Every commit on `main` is GPG-signed by a maintainer — not by GitHub's web-flow key.
2. Every commit on `main` carries a green CI seal before the forge accepts the push.
3. Force-pushes and deletions are blocked.

These three requirements cannot be satisfied by a single ruleset because GitHub's bypass is whole-ruleset — a bypass actor added to satisfy requirement 2 would also exempt them from requirement 1. The solution is two `main` rulesets with different rules and no bypass actors on either:

- **High Integrity ruleset** — enforces signing, deletion protection, and force-push protection with **no bypass**. Applies to everyone, always.
- **Zero Trust Gate ruleset** — enforces the `update` rule (push restriction) and `required_status_checks` with **no bypass**. A commit SHA that earns a green CI seal on a `staging-*` branch carries that result when fast-forwarded to `main`; the gate sees green and accepts the push without re-running CI.

The staging branch pattern (push merge commit to `staging-<slug>`, CI runs there, fast-forward to `main`) is the mechanism that satisfies all three requirements without exempting anyone. See [merging.md](merging.md) for the merge workflow.

> [!IMPORTANT]
> Do not add bypass actors to either `main` ruleset. Doing so would either exempt them from `required_signatures` (High Integrity ruleset) or allow pushing an unverified SHA to `main` (Zero Trust Gate ruleset). There is no legitimate use case for bypass — the staging pattern covers everything that direct-push bypass was previously used for.

#### Ruleset 1 — Enforce High Integrity Development

Enforces signing, deletion protection, and force-push protection. No bypass actors — applies to all actors unconditionally.

```bash
gh api --method POST "repos/${OWNER}/${REPO}/rulesets" --input - <<'EOF'
{
  "name": "Enforce High Integrity Development",
  "target": "branch",
  "enforcement": "active",
  "bypass_actors": [],
  "conditions": {
    "ref_name": {
      "exclude": [],
      "include": ["~DEFAULT_BRANCH"]
    }
  },
  "rules": [
    {"type": "deletion"},
    {"type": "non_fast_forward"},
    {"type": "required_signatures"}
  ]
}
EOF
```

#### Ruleset 2 — Zero Trust Gate

Restricts pushes to `main` and requires the single aggregate status check to pass. No bypass actors — every push to `main`, including maintainer pushes, must carry a CI-green SHA. The staging branch pattern satisfies this: a SHA that earns a green check on `staging-*` carries that result when fast-forwarded to `main`.

The gate requires exactly **one** context — `CI Quality Gate` — rather than enumerating every individual job. That context is the `ci-gate` aggregator job in `.github/workflows/ci.yml`, which passes only when every merge-gating job is `success` or legitimately `skipped` (see [ci-gating.md — CI Quality Gate](ci-gating.md#the-ci-quality-gate-aggregator)). A single context avoids the brittle requirement that this list stay byte-identical to each job's `name:` field, and it sidesteps the path-filter deadlock: jobs that legitimately skip on a given SHA would otherwise register as missing-and-therefore-unsatisfied required checks.

`strict_required_status_checks_policy` is `false` — the branch does not need to be up-to-date with `main` before the push, only the SHA's own check needs to be green.

The gating set the aggregator covers (and the jobs deliberately excluded, such as the PR-only `semver-check`) is declared in [`forge/github/ci-gating-jobs.json`](../../forge/github/ci-gating-jobs.json) and cross-checked against the workflow by `scripts/verify-ci-gate.sh`.

```bash
gh api --method POST "repos/${OWNER}/${REPO}/rulesets" --input - <<'EOF'
{
  "name": "Zero Trust Gate",
  "target": "branch",
  "enforcement": "active",
  "bypass_actors": [],
  "conditions": {
    "ref_name": {
      "exclude": [],
      "include": ["~DEFAULT_BRANCH"]
    }
  },
  "rules": [
    {"type": "update"},
    {
      "type": "required_status_checks",
      "parameters": {
        "strict_required_status_checks_policy": false,
        "required_status_checks": [
          {"context": "CI Quality Gate"}
        ]
      }
    }
  ]
}
EOF
```

> [!IMPORTANT]
> The context string above is the `name:` field of the `ci-gate` job in `.github/workflows/ci.yml` (`CI Quality Gate`). Verify it against a live CI run on a `staging-*` branch before pushing this ruleset to the forge — a mismatch between the string here and the string GitHub records for the check will cause the gate to block all pushes to `main`. Run `gh api "repos/${OWNER}/${REPO}/commits/<SHA>/check-runs" | jq -r '.check_runs[].name'` against a green staging SHA to confirm the exact string.

#### Ruleset 3 — Protect Release Tags

Prevents deletion, force-updates, and unsigned pushes of release tags. Targets the per-crate tag pattern used by `cargo-release` (`sdmx-{crate}/v{version}`). No bypass actors — release tags must be signed by everyone, including the maintainer; there is no legitimate reason to delete or rewrite a pushed release tag.

```bash
gh api --method POST "repos/${OWNER}/${REPO}/rulesets" --input - <<'EOF'
{
  "name": "Protect Release Tags",
  "target": "tag",
  "enforcement": "active",
  "bypass_actors": [],
  "conditions": {
    "ref_name": {
      "exclude": [],
      "include": [
        "refs/tags/sdmx-*/v*"
      ]
    }
  },
  "rules": [
    {"type": "deletion"},
    {"type": "non_fast_forward"},
    {"type": "required_signatures"}
  ]
}
EOF
```

This pattern matches `publish.yml`'s trigger (`tags: [ 'sdmx-*/v*' ]`) and `release.toml`'s tag format (`tag-name = "{{crate_name}}/v{{version}}"`). A `v*` pattern would silently miss every real release tag.

#### Verifying the rulesets

```bash
gh api "repos/${OWNER}/${REPO}/rulesets" | jq -r '.[] | "\(.id)\t\(.name)\t\(.target)\t\(.enforcement)"'
```

After the initial bootstrap, use `just update-rulesets` to apply changes to
ruleset JSON files — it queries by name and POSTs (create) or PUTs (update) as
appropriate, and is safe to re-run. Do not re-run `scripts/forge-apply.sh` for
this purpose.

#### Onboarding a future maintainer

Grant the collaborator the **Maintain** or **Admin** role on the repository. No ruleset edits are needed — the staging pattern applies equally to all maintainers.

```bash
gh api --method PUT "repos/${OWNER}/${REPO}/collaborators/USERNAME" \
  -F permission=maintain
```

The new maintainer must follow the same staging-branch flow as all other maintainers: merge locally, push to `staging-<slug>`, wait for CI, fast-forward to `main`. There is no bypass path for any role.

---

### Exporting and Replicating Rulesets

To export a ruleset (e.g. for backup, replication to another repo, or version control):

```bash
# List all rulesets and their IDs
gh api "repos/${OWNER}/${REPO}/rulesets" | jq -r '.[] | "\(.id)\t\(.name)"'

# Export a specific ruleset by ID
gh api "repos/${OWNER}/${REPO}/rulesets/<ID>" > ruleset-export.json

# Strip read-only fields and POST to a target repo
TARGET="other-repo"
gh api "repos/${OWNER}/${REPO}/rulesets/<ID>" \
  | jq '{name, target, enforcement, bypass_actors, conditions, rules}' \
  | gh api --method POST "repos/${OWNER}/${TARGET}/rulesets" --input -
```

> [!NOTE]
> `bypass_actors` entries use role-based `actor_id` constants (`2` = Maintain, `5` = Admin) that are GitHub-wide and portable across repos and organisations. User-specific or team-specific bypass actors carry IDs that may not resolve correctly across targets and should be reviewed before replication.

---

### Security Settings

The repository's GitHub security toggles are part of the forge spec
([`scripts/lib/forge-spec.sh`](../../scripts/lib/forge-spec.sh)): `just
doctor-forge` verifies them against the live repo, and `scripts/forge-apply.sh`
sets them during bootstrap. Two groups, by availability and endpoint:

**Always-on (available on private repos, applied at bootstrap):**

| Setting | Desired | Why |
|---------|:-------:|-----|
| Dependabot **vulnerability alerts** | **enabled** | Monitor-mode advisories (`SECURITY.md`). Endpoint: `PUT/GET …/vulnerability-alerts`. |
| Dependabot **automated security fixes** (auto-PRs) | **disabled** | Auto-PRs would introduce unsigned/bot-signed commits, violating the signed-history invariant. Endpoint: `…/automated-security-fixes` (DELETE to disable). |
| **Private vulnerability reporting** | **enabled** | Lets researchers report privately, aligning with the disclosure process in `SECURITY.md`. Endpoint: `PUT …/private-vulnerability-reporting`. |
| **Default workflow token permissions** | **read** | Least-privilege for `GITHUB_TOKEN`; a job that needs write requests it explicitly (`publish.yml` does). Endpoint: `PUT …/actions/permissions/workflow`. |
| **Actions can approve PRs** | **disabled** | The Actions bot must not approve pull requests. Same endpoint. |
| **`allowed_actions`** | **`selected`** | Restricts *which* actions may run at all, to a committed allowlist (`forge/github/actions-allowlist.json`). Orthogonal to `sha_pinning_required`: SHA-pinning stops tag-mutation of an action you already use; `selected` stops an unreviewed or typo-squatted action from entering a workflow at all. `forge-apply` PUTs the allowlist **then** PATCHes this setting (order matters — the allowlist must exist before the mode flips). Endpoint: `PATCH …/actions/permissions`. |
| **`has_wiki` / `has_pages`** | **false** | Reduce unmanaged-content attack surface. Repo PATCH. |
| **`allow_update_branch`** | **false** | No GitHub-side branch updates — they would create commits outside the signed local-merge flow. Repo PATCH. |

> [!WARNING]
> Flipping `allowed_actions=selected` makes CI **refuse to run** any `uses:`
> reference not covered by `forge/github/actions-allowlist.json`. Before running
> `forge-apply`, confirm `just doctor-forge` reports **zero** "Uncovered action"
> failures — that is the proof the allowlist is complete for the current workflows.
>
> **Maintenance obligation:** when adding a new third-party action to a workflow,
> add its `org/name@*` entry to `forge/github/actions-allowlist.json` in the
> **same commit**. `doctor-forge` will FAIL if a workflow references an unlisted
> action. See `docs/dev/tooling.md` → *Adding a GitHub Action* for the two-step
> checklist.

Apply / verify:

```bash
scripts/forge-apply.sh          # sets all of the above (guarded one-shot bootstrap)
just update-rulesets            # update rulesets post-bootstrap (POST-or-PUT by name)
just update-labels              # update labels post-bootstrap (PATCH-or-POST by name)
just update-actions-allowlist   # push committed allowlist live (PUT)
just doctor-forge               # verifies live state matches spec (read-only)
```

> [!NOTE]
> `doctor-forge` asserts `automated-security-fixes` is **off** — re-enabling
> Dependabot's auto-PRs is drift, not an improvement, because of the signing
> invariant. Dependency bumps are made by the maintainer via `just update-deps`
> (signed), not by bot PRs.

**Secret scanning and Push Protection:** see below. `doctor-forge` reports a
disabled reading for these as a failure by default; `FORGE_SECURITY_REQUIRED=0`
downgrades it to a warning, for clones pushed to private repositories that
cannot enable these settings without GitHub Advanced Security.

#### Secret Scanning & Push Protection

This is the **server-side** secret control and the only layer that *prevents* a leak rather than detecting one after the fact. The local pre-commit/pre-push hooks and the CI `check-secrets` job are all bypassable (`git push --no-verify`, uninstalled hooks) and, for CI, only flag a secret *after* it has reached the remote. **Push Protection rejects the push at the forge before the offending history is accepted** — closing the force-push leak vector that the bypassable layers cannot.

The three layers are defence-in-depth, not redundancy:

| Layer | Stage | Prevents leak reaching remote? | Bypassable? |
|-------|-------|:------------------------------:|:-----------:|
| pre-commit / pre-push hooks (`gitleaks`) | local | Yes (if installed) | Yes (`--no-verify`) |
| CI `check-secrets` (`just secrets-scan`) | runner | No — detects after push lands | No, but too late |
| **Push Protection** | **forge** | **Yes — rejects the push** | Only via logged, explicit bypass |

Secret scanning and Push Protection are free only on public repositories; on a private repository they require GitHub Advanced Security (a paid add-on). Both settings are applied with:

```bash
gh api --method PATCH "repos/${OWNER}/${REPO}" --input - <<'EOF'
{
  "security_and_analysis": {
    "secret_scanning": { "status": "enabled" },
    "secret_scanning_push_protection": { "status": "enabled" }
  }
}
EOF
```

Verify both report `enabled`:

```bash
gh api "repos/${OWNER}/${REPO}" --jq '.security_and_analysis | {
  secret_scanning: .secret_scanning.status,
  push_protection: .secret_scanning_push_protection.status
}'
```

> [!NOTE]
> Push Protection is GitHub-native and does not mirror to Codeberg — the same forge-portability caveat as the [branch rulesets](#branch-rulesets). On a forge migration it would be rebuilt against the target's equivalent feature (Forgejo secret scanning), not transferred. The local `gitleaks` hooks and CI job, being git/Nix-based, travel with the repo and remain the cross-forge baseline.

---

### Release Environment

The `release` GitHub environment gates every publish job in `publish.yml`. Each job requires manual maintainer approval before it runs. Create it once; no further changes are needed unless reviewers are added.

#### Create the environment

Navigate to **GitHub → Settings → Environments → New environment**, name it `release`, and save. Then configure it via the API:

```bash
# Create the environment
gh api --method PUT "repos/${OWNER}/${REPO}/environments/release"

# Add the maintainer as a required reviewer
MAINTAINER_ID=$(gh api user --jq '.id')
gh api --method PUT "repos/${OWNER}/${REPO}/environments/release" \
  --input - <<EOF
{
  "reviewers": [
    {"type": "User", "id": ${MAINTAINER_ID}}
  ],
  "prevent_self_review": false
}
EOF
```

> [!IMPORTANT]
> `prevent_self_review` must be `false` for a solo maintainer. With self-review disabled, the maintainer cannot approve their own deployment and publish will deadlock indefinitely. Enable it only when a second maintainer is available to approve.

#### Verify

```bash
gh api "repos/${OWNER}/${REPO}/environments/release" \
  | jq '{name: .name, reviewers: .reviewers, prevent_self_review}'
```

#### Onboarding additional maintainers

Add them as reviewers in the environment settings. No structural change to `publish.yml` is needed.

---

### Trusted Publisher Registration

Trusted Publishing (registering each crate's publisher on crates.io and, later,
enabling enforcement) is a **registry-plane** concern, not a forge one. It is
documented in its own runbook: [registry-setup.md](registry-setup.md).

The forge provides the binding it depends on — the protected `release` environment
([above](#release-environment)) — which the crates.io Trusted Publisher config
references by name. Once that environment exists and `publish.yml` is merged to the
default branch, follow [registry-setup.md — Trusted Publisher
registration](registry-setup.md#trusted-publisher-registration). Verify the live
binding with `just doctor-registry`.

---

### Signing Key Maintenance

Each maintainer operates a primary-with-subkeys GPG key structure. Initial registration is covered in [Base Repository Setup — step 2](#2-register-the-signing-key). This section covers the ongoing maintenance: the key register, adding new maintainers, and subkey rotation.

The recommended key architecture is:

| Role                                             | Description                                      | Expiry             |
|--------------------------------------------------|--------------------------------------------------|--------------------|
| Primary [C] — certify only, identity anchor      | Air-gapped; never used for day-to-day operations | None (recommended) |
| Signing [S] — used for all commit/tag signatures | Exported to the daily working environment        | Set (e.g. 2 years) |
| Encryption [E]                                   | Optional                                         | Set                |
| Authentication [A]                               | Optional                                         | Set                |

**CI signature verification anchors on the primary key fingerprint, not the signing subkey.** This ensures that rotating the signing subkey (at expiry or proactively) requires no CI or allowlist changes — only re-committing the updated public key file.

#### Current maintainer key register

Update this table whenever a maintainer is added, removed, or rotates a subkey.

| Maintainer | Signing Email   | Primary [C] Fingerprint                    | Signing [S] Fingerprint                    | [S] Expiry |
|------------|-----------------|--------------------------------------------|--------------------------------------------|------------|
| dgalbraith | `dg@lbraith.io` | `53069F0184A426465E5FF9E7FC6BB04EBF431B25` | `B43D054479B0A9374BC35C167D4A0D2EE2E2ECD7` | 2027-12-31 |

The public key for each maintainer is committed at `.github/maintainer-keys/<username>.asc` and is the trust source for CI signature verification.

> [!NOTE]
> **Subkey expiry is a tracked maintenance obligation.** The `[S]` expiry above is monitored by the `signing-subkey-renewal` item in [maintenance.toml](../../maintenance.toml). Its scanner-tracked breadcrumb lives beside the fingerprint allowlist in [`.github/workflows/verify-signature.yml`](../../.github/workflows/verify-signature.yml) (the trust root that fails when a subkey expires). The weekly scheduled maintenance job opens a renewal tracking issue ahead of the deadline — see [maintenance.md](maintenance.md). Follow [Signing subkey rotation](#signing-subkey-rotation) below when it fires.

#### Adding a maintainer

1. Obtain the new maintainer's GPG public key (armoured `.asc` export of their full public key).
2. Verify the key fingerprint out-of-band before trusting it.
3. Allowlist the key in [`.gitignore`](../../.gitignore). The `maintainer-keys/` directory is deny-by-default, so add an explicit entry — this makes adding a key to the CI trust root a deliberate, reviewed act:
   ```gitignore
   !/.github/maintainer-keys/<username>.asc
   ```
4. Commit the key file: `git commit -S .github/maintainer-keys/<username>.asc .gitignore -m "chore(keys): add <username> maintainer key"`.
5. Add the new maintainer's primary key fingerprint to the CI allowlist in `.github/workflows/verify-signature.yml`.
6. Add a row to the maintainer key register above.
7. Grant the collaborator the Maintain or Admin role (see [Onboarding a future maintainer](#onboarding-a-future-maintainer)).

#### Signing subkey rotation

Run this procedure before a signing subkey's expiry date.

1. Bring the air-gapped primary key into a secure environment.
2. Generate a new signing subkey: `gpg --edit-key <PRIMARY_FINGERPRINT>` → `addkey` → `save`.
3. Export the updated public key: `gpg --armor --export <SIGNING_EMAIL> > .github/maintainer-keys/<username>.asc`.
4. Point the local clone at the new subkey *before* the next signed commit, or step 5 signs with the old (expiring) key: `git config --global user.signingkey <NEW_SUBKEY_FPR>!` (see [Local Maintainer Git Configuration](#local-maintainer-git-configuration)).
5. Commit the updated key file (signed commit): `git commit -S .github/maintainer-keys/<username>.asc -m "chore(keys): rotate signing subkey"`.
6. Re-upload to the GitHub account: `gh api --method POST user/gpg_keys -F armored_public_key="$(gpg --armor --export <SIGNING_EMAIL>)"`.
7. The CI allowlist and `verify-signature.yml` require no changes — the primary key fingerprint is unchanged.
8. Update the `[S] Expiry` column in the maintainer key register above.

> [!NOTE]
> A scheduled maintenance check monitors signing subkey expiry and opens a tracking issue ~90 days before the expiry date. This is the `signing-subkey-renewal` obligation in [maintenance.toml](../../maintenance.toml) (tracked at the allowlist in `verify-signature.yml`). The primary key fingerprint in the CI allowlist is the durable trust anchor; subkey rotation is transparent to all downstream verification.
>
> After rotating, bump the obligation, then correct `next_review` to ~90 days before the **new** subkey's expiry (`maintenance-bump.sh` resets it to today + cadence, which is not what a fixed-deadline item wants):
> ```bash
> scripts/maintenance-bump.sh signing-subkey-renewal
> # then edit maintenance.toml + the verify-signature.yml breadcrumb so
> # next_review = (new [S] expiry − 90 days); update the [S] Expiry table above
> ```

---

## Codecov

Codecov provides PR coverage annotations and a dashboard for tracking per-crate coverage trends over time. It is **non-blocking** — a Codecov outage or token failure never fails CI; the coverage gate itself is enforced locally by `cargo llvm-cov` in the `coverage` job. Codecov is dashboard and PR annotation only.

### Integration setup

1. **Install the Codecov GitHub App**: visit [codecov.io](https://codecov.io), sign in with GitHub, and grant access to the `sdmx-rs` repository.

2. **Retrieve the repository upload token**: in the Codecov dashboard, navigate to **sdmx-rs → Settings → General** and copy the upload token.

3. **Add the token as a GitHub Actions secret**:
   ```bash
   gh secret set CODECOV_TOKEN --body "<token>"
   ```
   The secret name must be exactly `CODECOV_TOKEN` — this is what `ci.yml` references in the `codecov/codecov-action` upload step.

4. **Verify**: trigger a CI run (push or PR) and confirm the "Upload Coverage to Codecov" step succeeds and the dashboard shows coverage data.

### Token expiry

`CODECOV_TOKEN` expiry is the primary in-our-control failure vector for the upload step. When it expires, CI emits a warning annotation ("Codecov upload failed (non-blocking)") but does not fail. Refresh the token via the Codecov dashboard and re-run `gh secret set CODECOV_TOKEN` as above. Token expiry is tracked as a maintenance obligation in [maintenance.toml](../../maintenance.toml) under `codecov-token-renewal`.

### Configuration

Coverage thresholds and PR patch targets are declared in [`codecov.yaml`](../../codecov.yaml), which is the authoritative source. See [testing.md](../dev/testing.md) for the per-crate floor rationale.

---

## Codeberg

Codeberg serves as a private read-only mirror maintained via dual-push from the maintainer's local clone. Its purpose is continuity — all code, commits, tags, and signatures are present on Codeberg at all times, so a forge migration is a routing change rather than a data migration.

**Current status**: mirror only. Issues, CI, and rulesets remain on GitHub. Migration tooling (`tea`, Forgejo import API) would be used to bring those across at migration time.

**Dual-push setup** (fan-out push remote):

The canonical GitHub remote is `origin` (created by `gh repo create --clone` in [Base Repository Setup](#1-create-the-repository)); this is the name the `main`-branch tooling defaults to via `SDMX_MAIN_REMOTE`. The commands below add `codeberg` and a fan-out `all` remote *alongside* it — they do not replace `origin`. If your GitHub remote is named something else, either rename it (`git remote rename <name> origin`) or set `SDMX_MAIN_REMOTE` (see [merging.md](merging.md#remote-configuration)).

```bash
git remote add codeberg "git@codeberg.org:${OWNER}/${REPO}.git"

# Configure a fan-out push remote named 'all'
git remote add all "git@github.com:${OWNER}/${REPO}.git"
git remote set-url --add --push all "git@github.com:${OWNER}/${REPO}.git"
git remote set-url --add --push all "git@codeberg.org:${OWNER}/${REPO}.git"

# Verify
git remote -v
```

Subsequent pushes use `git push all` (or `git push all --follow-tags` for releases) to publish to both forges simultaneously. See [merging.md](merging.md) for the merge workflow and `SDMX_MAIN_REMOTE` configuration.

**Migration considerations**:

- Git history, commits, tags, and GPG signatures travel with the push automatically — no action needed at migration time.
- Issues, PRs, and CI configuration do not live in git and require tooling-assisted migration.
- The `publish.yml` CI workflow uses GitHub Actions OIDC (Trusted Publishing). Forgejo Actions does not currently have crates.io Trusted Publishing support. The publish workflow would require rework if the primary forge changes.
- GitHub-specific contexts (`github.*`) and GitHub Actions action SHAs may not resolve on Forgejo and would need adjustment.
