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
| `just doctor-registry` | **Read-only.** Verifies the live crates.io Trusted Publishing config + enforcement state against [`scripts/lib/registry-spec.sh`](../../scripts/lib/registry-spec.sh). Run it after each step below to confirm. |
| `scripts/registry-tp.sh` | **Print-only.** Emits the exact `cargo publish` / register / enforce commands and checks their preconditions. It never mutates crates.io and never holds a token — you run the printed commands yourself. Not a `just` recipe (a guarded one-shot bootstrap tool; see the policy in [tooling.md](../dev/tooling.md)). |

Run `scripts/registry-tp.sh --print-register` to get the registration commands and
`scripts/registry-tp.sh --print-enforce` for the enforcement commands. Verifying is
`just doctor-registry`.

## Authentication

The management API has **no OAuth / headless flow** — it accepts only a personal
API token (`Authorization` header) or a web-UI session cookie. The pipeline's OIDC
token is publish-scoped and is rejected by these endpoints by design.

- Mint a **personal API token** at [crates.io → Account → API
  Tokens](https://crates.io/settings/tokens). Use a **minimal scope and short
  expiry**, and **revoke it after setup** — the long-lived-credential window should
  be tiny.
- Export it as `CRATES_IO_TOKEN` for `doctor-registry` / `registry-tp` (read-only
  use). Do **not** reuse `CARGO_REGISTRY_TOKEN` (that name carries the pipeline's
  publish-only OIDC token, which the management API rejects).

## Trusted Publisher registration

Trusted Publishing replaces a long-lived crates.io token with an ephemeral OIDC
token issued per-job by GitHub Actions. Each crate name must be registered
separately.

### Prerequisites

- All five crate names must already exist on crates.io (reserved via the bootstrap
  publish — see [releasing.md](releasing.md#first-time-bootstrap-sequence)).
  crates.io has **no pending-publisher feature**: the name must exist before a
  Trusted Publisher can attach to it.
- `publish.yml` must be merged to the default branch — crates.io validates the
  workflow filename and environment against the default branch.
- The `release` environment must exist ([forge-setup.md — Release
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
Trusted Publishing → Add a publisher**. Register all five: `sdmx-types`,
`sdmx-parsers`, `sdmx-writers`, `sdmx-client`, `sdmx-rs`.

Verify with `just doctor-registry` — it asserts **exactly one** matching publisher
per crate (a stray or extra config pointing at the wrong repo/workflow is the real
supply-chain risk).

## Enforcement (after the first successful TP publish)

Once Trusted Publishing is confirmed working end-to-end, disable API-token
publishing per crate. Run `scripts/registry-tp.sh --print-enforce` (it refuses to
print for any crate that is not yet published + registered) and execute the printed
`PATCH … {"crate":{"trustpub_only":true}}`. Web-UI equivalent: **Settings →
Trusted Publishing → Require Trusted Publishing**.

> [!WARNING]
> Do not enable enforcement until the first real release has completed
> successfully through the Trusted Publishing path. Enabling it while the OIDC
> binding is unproven removes the token fallback needed to diagnose and recover
> from a misconfiguration.

Verify with `REGISTRY_ENFORCEMENT_REQUIRED=1 just doctor-registry`.

## What stays manual (and why)

- **Name reservation** — a one-time `cargo publish` with a long-lived token. It is
  not automatable (the name must exist before anything else), and `registry-tp.sh`
  only *prints* the command.
- **Running the register / enforce commands** — by deliberate design our tooling
  never holds a token or issues a mutating crates.io call. A published version
  cannot be unpublished and enforcement removes the token escape hatch, so the
  irreversible acts stay entirely in your hands at a shell prompt.
