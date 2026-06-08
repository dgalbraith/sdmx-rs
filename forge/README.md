# Forge Configuration Artifacts

Machine-readable realizations of the forge configuration described in prose at
[docs/project/forge-setup.md](../docs/project/forge-setup.md). This directory holds
the **what** (request bodies the tooling consumes); the doc holds the **why**
(rationale, one-shot/manual steps, invariants explained); the scripts are the **how**:

| Script                                | Role                                            |
|---------------------------------------|-------------------------------------------------|
| `scripts/forge-apply.sh`              | Guarded one-shot bootstrap — initial setup only |
| `scripts/doctor-forge.sh`             | Read-only drift checker — live vs. spec         |
| `scripts/update-rulesets.sh`          | Idempotent ruleset update — POST-or-PUT by name |
| `scripts/update-labels.sh`            | Idempotent label update — PATCH-or-POST by name |
| `scripts/update-actions-allowlist.sh` | Idempotent allowlist push — single PUT          |

The split mirrors the repo's existing doc/artifact separation (ADR prose in
`docs/adr/` + mechanism in `scripts/doc-engine.sh`): `docs/` stays 100% prose for
humans, and API-request-body JSON lives here instead.

## Layout

Top-level `forge/` is forge-neutral; each forge gets a subdirectory:

- `forge/github/` — GitHub rulesets as projected JSON.

This mirrors the repo's by-kind top-level pattern (`tests/`, `scripts/`,
`crates/`). Migrating the primary forge to another host (e.g. Codeberg/Forgejo) is
additive — drop a `forge/codeberg/` alongside, with zero relocation of the GitHub
artifacts. `.github/` is deliberately **not** used: it is GitHub's own namespace,
self-contradicting for forge-*portability* artifacts.

## Rulesets (`forge/github/ruleset-*.json`)

Each file is the **projected** form of a GitHub repository ruleset — the request
body with all server-owned read-only fields stripped, so the committed file
round-trips losslessly through both apply and verify:

- `ruleset-signing.json` — *Enforce High Integrity Development*. Signing,
  deletion, and force-push protection on the default branch.
- `ruleset-push-restriction.json` — *Main Pushes Restricted to Maintainers*.
  The `update` rule with Maintain (`actor_id` 2) and Admin (`actor_id` 5)
  role-based bypass.
- `ruleset-tag-protection.json` — *Protect Release Tags*. Deletion, force-push,
  and signature protection on `refs/tags/sdmx-*/v*`.

### Projection

The committed files are produced by stripping read-only fields from the live API
response. To re-baseline a file from live (a deliberate capture of live → file):

```bash
gh api "repos/${OWNER}/${REPO}/rulesets/<ID>" \
  | jq '{name, target, enforcement,
         bypass_actors: [.bypass_actors[]? | {actor_id, actor_type, bypass_mode}],
         conditions, rules}' \
  > forge/github/ruleset-<name>.json
```

The read-only fields removed are: `id`, `created_at`, `updated_at`, `node_id`,
`_links`, `source`, `source_type`, `current_user_can_bypass`.

`scripts/doctor-forge.sh` applies the *same* projection to the live ruleset and
diffs it against the committed file; `scripts/forge-apply.sh` feeds the committed
file directly as the request body (`gh api --input <file>`) — no shell
reconstruction of the JSON.

### INVARIANT — empty bypass on the signing ruleset

`ruleset-signing.json` **must** keep `"bypass_actors": []`. Adding any actor here
would exempt them from `required_signatures`, letting unsigned commits onto the
default branch. All bypass for *push access* belongs in
`ruleset-push-restriction.json` only — that is the whole point of splitting the two
rulesets (GitHub's bypass is per-ruleset, so the maintainer can bypass the `update`
rule to push directly while still being bound by signing enforcement).
`doctor-forge` asserts this invariant on the live signing ruleset and fails if the
bypass list is non-empty.

## Actions allowlist (`forge/github/actions-allowlist.json`)

This file is the literal body of the GitHub
`PUT /repos/{OWNER}/{REPO}/actions/permissions/selected-actions` request — fed
with `gh api --input` verbatim by `forge-apply`. It is also the compare target
for `doctor-forge`'s `selected-actions` diff.

When `allowed_actions=selected` is active (part of the forge baseline), GitHub
will refuse to run any `uses:` reference not covered by this file. That is the
operational gate that makes the allowlist a supply-chain control.

### Body shape

```json
{
  "github_owned_allowed": true,
  "verified_allowed": false,
  "patterns_allowed": [
    "<org>/<name>@*",
    ...
  ]
}
```

- **`github_owned_allowed: true`** — all `actions/*` actions (GitHub-owned) are
  permitted without explicit listing.
- **`verified_allowed: false`** — do NOT blanket-trust GitHub's "verified
  creator" set. Every non-GitHub-owned action must be named explicitly. This is
  deliberate; the verified list is a third-party allowlist we don't review.
- **`patterns_allowed`** — one `org/name@*` entry per non-GitHub-owned action
  used in the workflows. The `@*` suffix is the *name axis* only; the *SHA axis*
  is separately enforced by `sha_pinning_required=true` (already in the spec)
  and actionlint. The two controls are orthogonal:
  - `sha_pinning_required` — prevents tag-mutation of an action you already use.
  - `allowed_actions=selected` — restricts *which* actions may enter at all.

### Projection

`doctor-forge` sorts `patterns_allowed` before comparing so list order is not
spurious drift:

```bash
jq '{github_owned_allowed, verified_allowed, patterns_allowed: (.patterns_allowed | sort)}' \
    forge/github/actions-allowlist.json
```

### Maintenance obligation

When adding a **new** third-party `uses:` to any workflow, add its
`org/name@*` entry to this file in the **same commit**. `doctor-forge` will
`FAIL` if a workflow references an unlisted action — that is the early-warning
that the allowlist is incomplete *before* the maintainer flips `allowed_actions`
to `selected`. See `docs/dev/tooling.md` for the two-step checklist a
contributor follows when adding an action.

## `forge-apply` pre-run guard

`forge-apply` is **initial setup, not an idempotent reconcile**. Before applying
it refuses to run if the repo already looks configured, on two independent
signals:

- **Rulesets present** — re-`POST`ing would create duplicate rulesets.
- **Issues *or PRs* present** — the label DELETE+CREATE pass would strip labels
  from live items, losing triage state. Note GitHub's `/issues` endpoint returns
  PRs too, so a lone bot PR (e.g. Dependabot) also trips the guard.

The guard **fails closed**: if it cannot *determine* the live state (a `gh`
error, an unparseable response), it BLOCKS rather than assuming an empty repo —
the whole point is to avoid a destructive apply against a populated repo.

The only override is `--force` (or `FORGE_APPLY_FORCE=1`), which bypasses the
guard **wholesale** — including re-enabling the label DELETE pass. There is no
partial escape hatch for labels alone; if you only meant to re-apply rulesets on
a repo that has issues, apply that step manually instead of forcing.

**Post-bootstrap changes** (updating existing rulesets, label attributes, or the
actions allowlist) belong to the dedicated update scripts, not to `forge-apply`:

```bash
just update-rulesets             # POST-or-PUT rulesets by name (safe to re-run)
just update-labels               # PATCH-or-POST labels by name (no delete pass)
just update-actions-allowlist    # PUT the committed allowlist file live
```

These scripts are idempotent and safe to run on a live repo with data.

## Portable intent

The JSON here is GitHub's ruleset schema, but the *intent* it encodes — signed
history, maintainer-only pushes, protected signed tags — is forge-neutral. On a
forge migration the rulesets are rebuilt against the target's equivalent feature
(Forgejo rulesets), not transferred verbatim. A future `forge/spec.toml`
portable-intent layer could express that intent once and realize it per forge; it
is deferred until a second forge actually exists to realize against.
