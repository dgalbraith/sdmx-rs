# Registry Configuration Artifacts

Machine-readable realizations of the **registry** (crates.io) configuration that
[docs/project/registry-setup.md](../docs/project/registry-setup.md) describes in
prose. This directory is the registry-plane sibling of [`forge/`](../forge/): the
docs hold the **why** (rationale, irreversible-ordering, the manual steps that stay
manual), and `scripts/doctor-registry.sh` / `scripts/registry-tp.sh` are the
**how**.

## Registry vs. forge — two planes

A **forge** (GitHub, Codeberg/Forgejo) hosts *source and collaboration*: repos,
branches, pull requests, issues, rulesets, CI. That plane is governed by
[`forge/`](../forge/) + `forge-spec.sh` / `doctor-forge.sh` / `forge-apply.sh`.

A **registry** (crates.io) distributes *built artifacts*: published crate versions,
the index, ownership, and Trusted Publishing (TP) configuration. That is this
plane.

They are deliberately kept separate — different resources, different credentials,
different tooling. crates.io is **not** a second forge, so this work uses
`registry_*` naming and its own spec library rather than extending the forge spec.

**The one sanctioned cross-plane link:** a crate's Trusted Publishing config binds
to a GitHub Actions *environment*, and that environment MUST be the forge's gating
`release` environment. So `registry-spec.sh` borrows two values from the forge
spec — the `OWNER/REPO` (`forge_spec_owner_repo`) and the environment name
(`forge_spec_release_env_name`) — binding the planes at their single legitimate
seam rather than re-declaring (and risking drift in) those values.

## Trusted Publishing

Trusted Publishing replaces a long-lived crates.io API token with an ephemeral
OIDC token issued per-job by GitHub Actions. The publish workflow
(`.github/workflows/publish.yml`) authenticates this way; the registry side is a
per-crate **TP config** on crates.io naming the trusted
`{repository_owner, repository_name, workflow_filename, environment}`.

The desired state lives in [`scripts/lib/registry-spec.sh`](../scripts/lib/registry-spec.sh):

- `registry_spec_crates` — the five publishable crates, in topological order.
- `registry_spec_tp_workflow` — `publish.yml`.
- `registry_spec_tp_repo` / `registry_spec_tp_environment` — the cross-plane
  bindings (owner/repo + `release` env).
- `registry_spec_enforcement` — the desired `trustpub_only` end state (`true`).

### Verify, don't mutate

`scripts/doctor-registry.sh` (read-only, `just doctor-registry`) asserts the live
crates.io state against the spec: that each crate has **exactly one** TP config
matching the spec binding (a stray/extra config pointing at the wrong repo or
workflow is the real supply-chain threat), and reports each crate's enforcement
state. It never mutates.

`scripts/registry-tp.sh` is **print-only**: it emits the exact `cargo publish`
reservation, TP-registration, and enforcement commands and checks their
preconditions, but it **never issues a mutating request and never holds a token**.
The maintainer runs the printed commands by hand. This is deliberate: on a public
registry a published version cannot be unpublished and enforcement removes the
token escape hatch, so the irreversible acts stay entirely in human hands. See the
runbook for the full ordered sequence.

## Why no `registry-apply.sh`

The forge has a large, drift-prone write surface, so `forge-apply.sh` earns its
keep. The registry's write surface is the opposite: a handful of irreversible,
once-ever calls (register each TP config once; enable enforcement once). Automating
those would buy little and would add a standing, token-wielding mutation tool to
the attack surface. So the registry plane keeps the *verification* half of the
pattern (`spec → doctor`) and reduces the *mutation* half to a print-only helper.
