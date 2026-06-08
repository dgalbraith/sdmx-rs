<!-- TEMPLATE GUIDANCE — delete this whole comment block in the curated copy.
  Curated, user-facing release notes for the sdmx-rs FACADE. Create with
  `just new-release-notes <version>` (copies this template to
  crates/sdmx-rs/release-notes/<version>.md). This file — not the machine
  CHANGELOG.md — drives the facade's GitHub Release body. The Release title is set
  separately to a plain "sdmx-rs v<version>", so do NOT add a title heading here —
  lead straight into the summary prose below.

  A curated copy is a MANDATORY pre-tag gate: `just check-release-notes <version>`
  fails unless every section below has real content AND no template guidance
  (lines marked "GUIDANCE:" and this block) survives. Curate, do not just save.

  Audience & order: a developer deciding whether to upgrade and what it costs —
  what breaks, what's fixed, what's new, what's coming, the toolchain floor, the
  facade's feature surface, security, dependencies, provenance.

  Headings are PLAIN text (no emoji), matching the project's other documentation
  and the wider Rust-library convention. MD041 (first-line-heading) is lifted for
  this directory so the prose-first opening lints cleanly.

  Empty-state convention: keep EVERY section. If a section has nothing to report,
  state the negative explicitly (the "If none" line in each section's guidance) —
  a consumer making an upgrade decision must not have to guess whether "nothing
  changed" or "the author forgot". Never delete a section.

  The "Current MSRV" line is ALWAYS present and is a LITERAL value kept in sync by
  `just update-msrv` — do NOT tokenise or delete it.
-->

<!-- GUIDANCE: replace this line with a 1-2 sentence prose summary of the release. -->

## Breaking Changes & Migration

<!-- GUIDANCE: breaking API changes or structural migrations, each with the upgrade
     steps and a link to the driving issue where one exists. If this release bumps
     the MSRV, cross-reference it here (an MSRV increase breaks consumers on an
     older toolchain).
     If none, write: "This release contains no breaking changes." -->

## Bug Fixes

<!-- GUIDANCE: user-visible defects fixed, framed by what now works
     (* **Area**: what was broken and what now behaves correctly).
     If none, write: "No bug fixes in this release." -->

## New Features & Enhancements

<!-- GUIDANCE: notable additions, framed by consumer benefit
     (* **Feature area**: what was added and why it helps the consumer; link the
     driving issue where one exists).
     If none, write: "No new features in this release." -->

## Deprecations

<!-- GUIDANCE: APIs still working but slated for removal — the migration runway.
     State the replacement, removal timeline, and link the tracking issue where known
     (* **Item**: deprecated in favour of X; removed in a future major release).
     If none, write: "No deprecations in this release." -->

## Minimum Supported Rust Version (MSRV)

<!-- GUIDANCE: ALWAYS keep this section and the line below (kept in sync by
     `just update-msrv` — leave it). If this release BUMPED the MSRV, add bullets
     under the line explaining the trigger and the consumer impact. If unchanged,
     leave just the line. -->
* **Current MSRV**: `1.91.0`

## Feature Flags

<!-- GUIDANCE: the facade gates whole subcrates behind features (parsers, writers,
     client), so flag changes are consumer-architecture. Detail new, changed, or
     deprecated flags (e.g. added feature `csv`).
     If none, write: "No changes to Cargo feature flags." -->

## Security

<!-- GUIDANCE: security-relevant changes — fixed advisories (RUSTSEC IDs), yanked
     versions, hardening. High-signal; lead with it when present.
     If none, write: "No security advisories addressed in this release." -->

## Dependency Updates

<!-- GUIDANCE: notable dependency version changes worth a consumer's attention
     (e.g. a TLS stack or async-runtime bump). Routine lockfile churn need not be
     listed. If none, write: "No notable dependency updates in this release." -->

## Verifying Release Provenance
Every artifact is published with SLSA build provenance and dual-format (CycloneDX + SPDX) SBOMs, and every release tag and its commit are GPG-signed by a maintainer. For the exact `gh attestation verify` and `git verify-tag`/`verify-commit` commands, see [SECURITY.md — Verifying Release Provenance](../../../SECURITY.md#verifying-release-provenance).
