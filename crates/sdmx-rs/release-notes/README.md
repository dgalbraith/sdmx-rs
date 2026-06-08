# Facade Release Notes (curated)

Human-written, user-facing release notes for the `sdmx-rs` **facade** crate, one
file per released version: `release-notes/<version>.md` (e.g. `0.2.0.md`,
`1.0.0-rc.1.md`).

## Why this is separate from `CHANGELOG.md`

The five per-crate `CHANGELOG.md` files are **strict `git-cliff` output** — the
machine record, generated from Conventional Commits and verified byte-for-byte by
the `check-changelog` CI gate. That gate has no facade exemption, so the facade
changelog **cannot** also be hand-curated without the two fighting.

User-facing prose therefore lives here instead. For the facade, this curated file
**drives the GitHub Release body** ([`create-release.sh`](../../../scripts/ci/create-release.sh)
prefers it over the machine `CHANGELOG.md` section). Leaf crates
(`sdmx-types`, `sdmx-parsers`, `sdmx-writers`, `sdmx-client`) are **not** curated
here — their Release body is the auto changelog section (or a provenance
placeholder when empty).

## The gate

A non-empty `release-notes/<version>.md` is a **mandatory precondition** of
cutting a facade release — enforced locally **before** the (irreversible) release
tag is pushed:

```bash
just check-release-notes <version>
```

This also runs as part of `just prepublish-check` and is re-checked as a backstop
in CI. See [docs/project/releasing.md](../../../docs/project/releasing.md) §1 and
[docs/design/0004](../../../docs/design/0004-release-publish-pipeline-and-supply-chain-provenance.md) §9.

## Writing a version's notes

Start from the template — copy it to `<version>.md` and fill each section:

```bash
cp templates/template.md <version>.md   # e.g. 0.2.0.md, run from this directory
```

The template's **Current MSRV** line carries the live MSRV as a literal value
(kept in sync automatically by `just update-msrv`), so a fresh copy always starts
from the correct floor. Cover what a *consumer of the facade* cares about, not
implementation detail:

- Breaking changes, stated plainly, with migration guidance.
- New capabilities and notable behaviour changes.
- Significant dependency or MSRV updates.

Prefer user-facing language: instead of *"refactor(parsers): restructure
constraint validation"*, write *"Constraint validation now reports clearer
errors."*
