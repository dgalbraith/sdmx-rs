# Registry Setup (crates.io)

Runbook for the **registry** plane: reserving crate names, registering Trusted
Publishers, and enabling enforcement on crates.io. This is the registry sibling of
[forge-setup.md](forge-setup.md) (the GitHub/forge plane). The plane boundary and
the artifact spec live in [`registry/README.md`](../../registry/README.md); this
document is the operator runbook.

> [!NOTE]
> crates.io is a **registry** (it distributes built artifacts), not a forge. Its
> tooling, credentials, and config are deliberately separate from the forge. The
> one cross-plane link is that a Trusted Publishing config binds to the forge's
> gating `release` environment — see [forge-setup.md — Release
> Environment](forge-setup.md#release-environment).

## Tooling

| Tool | What it does |
|------|--------------|
| `just doctor-registry` | **Read-only.** Verifies the live crates.io Trusted Publishing config + enforcement state against [`scripts/lib/registry-spec.sh`](../../scripts/lib/registry-spec.sh). Run it to confirm the live state. |
| `scripts/registry-tp.sh` | **Print-only.** Emits the exact `cargo publish` / register / enforce commands and checks their preconditions. It never mutates crates.io and never holds a token — you run the printed commands yourself. Not a `just` recipe (a guarded one-shot bootstrap tool; see the policy in [tooling.md](../dev/tooling.md)). |

Run `scripts/registry-tp.sh --print-register` to get the registration commands and
`scripts/registry-tp.sh --print-enforce` for the enforcement commands. Verifying is
`just doctor-registry`.

## Authentication

The management API has **no OAuth / headless flow** — it accepts only a personal
API token (`Authorization` header) or a web-UI session cookie. The pipeline's OIDC
token is publish-scoped and is rejected by these endpoints by design.

No API token exists (the bootstrap token is revoked). For any future management
task that needs one:

- Mint a **personal API token** at [crates.io → Account → API
  Tokens](https://crates.io/settings/tokens). Use a **minimal scope and short
  expiry**, and **revoke it as soon as the task is done**: the
  long-lived-credential window should be tiny.
- Export it as `CRATES_IO_TOKEN` for `doctor-registry` / `registry-tp` (read-only
  use). Do **not** reuse `CARGO_REGISTRY_TOKEN` (that name carries the pipeline's
  publish-only OIDC token, which the management API rejects).

## Trusted Publisher registration

Trusted Publishing replaces a long-lived crates.io token with an ephemeral OIDC
token issued per-job by GitHub Actions. Each crate name is registered separately.

A Trusted Publisher is registered for all five crates, binding repository
`dgalbraith/sdmx-rs`, workflow `publish.yml`, and environment `release`; `just
doctor-registry` verifies the live state. The procedure below is the reference for
registering a future crate.

### Prerequisites

Registration requires, per crate:

- The crate name exists on crates.io (names come into existence only by
  publishing; the existing names are recorded in the [releasing.md bootstrap
  record](releasing.md#bootstrap-record)). crates.io has **no pending-publisher
  feature**: the name must exist before a Trusted Publisher can attach to it.
- `publish.yml` is merged to the default branch; crates.io validates the workflow
  filename and environment against the default branch.
- The `release` environment exists ([forge-setup.md — Release
  Environment](forge-setup.md#release-environment)).

### Register each crate

Run `scripts/registry-tp.sh --print-register` and execute the printed command for
each unregistered crate. Each registers this binding:

| Field             | Value         |
|-------------------|---------------|
| GitHub owner      | `dgalbraith`  |
| Repository name   | `sdmx-rs`     |
| Workflow filename | `publish.yml` |
| Environment name  | `release`     |

Equivalently, in the web UI: **crates.io → Your crates → `<crate>` → Settings →
Trusted Publishing → Add a publisher**.

Verify with `just doctor-registry` — it asserts **exactly one** matching publisher
per crate (a stray or extra config pointing at the wrong repo/workflow is the real
supply-chain risk).

## Enforcement

"Require Trusted Publishing" is enabled for all five crates and no API token
exists, so token-based publishing is structurally impossible. Verify with
`REGISTRY_ENFORCEMENT_REQUIRED=1 just doctor-registry`. The emergency path (the
crate owner toggling the setting off in the web UI) is recorded in the
[releasing.md bootstrap record](releasing.md#bootstrap-record).

To enforce a future crate, run `scripts/registry-tp.sh --print-enforce` (it
refuses to print for any crate that is not yet published + registered) and execute
the printed `PATCH … {"crate":{"trustpub_only":true}}`. Web-UI equivalent:
**Settings → Trusted Publishing → Require Trusted Publishing**.

## What stays manual (and why)

- **Name reservation** — a one-time `cargo publish` with a long-lived token. It is
  not automatable (the name must exist before anything else), and `registry-tp.sh`
  only *prints* the command.
- **Running the register / enforce commands** — by deliberate design our tooling
  never holds a token or issues a mutating crates.io call. A published version
  cannot be unpublished and enforcement removes the token escape hatch, so the
  irreversible acts stay entirely in your hands at a shell prompt.
