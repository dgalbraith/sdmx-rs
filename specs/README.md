# Pinned SDMX Schemas

The official **SDMX XML schemas** (.xsd) that the sdmx-rs workspace is modelled against, pinned to upstream releases and fetched on demand: the repository tracks the pins, not the schema files. The decision register ([docs/decisions.md](../docs/decisions.md)) cites them as the source of truth for the domain model in `sdmx-types`, and the parsers in `sdmx-parsers` validate against them.

## Provenance

For each SDMX edition the table below records its upstream source, release tag, and commit SHA. The pin covers the whole release at that commit; from it the build extracts the `schemas/` subtree (the full `.xsd` set), which is what the workspace models against.

| Version | Source                                                                                                                              | Tag      | Commit                                     | Contents                                            |
|---------|-------------------------------------------------------------------------------------------------------------------------------------|----------|--------------------------------------------|-----------------------------------------------------|
| 3.0     | [sdmx-twg/sdmx-ml](https://github.com/sdmx-twg/sdmx-ml) ([release v3.0.0](https://github.com/sdmx-twg/sdmx-ml/releases/tag/v3.0.0)) | `v3.0.0` | `29f1a3d856c4259429f5ec0eae811653adc5cdb5` | `schemas/` subtree (30 `.xsd`, incl. W3C `xml.xsd`) |
| 3.1     | [sdmx-twg/sdmx-ml](https://github.com/sdmx-twg/sdmx-ml) ([release v3.1.0](https://github.com/sdmx-twg/sdmx-ml/releases/tag/v3.1.0)) | `v3.1.0` | `182248b3c8030b595187dca51ca341d5ff839c24` | `schemas/` subtree (30 `.xsd`, incl. W3C `xml.xsd`) |

The exact pins (per-edition commit + per-file sha256 + blob URLs) live in [`sources.toml`](sources.toml), the single source of truth for both the Nix fetch in `flake.nix` and the `fetch-specs` verify gate. Each fetch is tied to the immutable commit and re-checks every file against its recorded sha256, so a materialised `specs/` is reproducible and integrity-checked regardless of how GitHub packages the download.

In each case the fetched files are the contents of the release's `schemas/` directory, verbatim, with the internal `xs:include` / `xs:import` relative paths intact.

When re-pinning a set to a new release, record the change here (old → new tag/SHA, date, reason) rather than overwriting silently.

## Rights and licensing

The repository tracks pins and tooling only, never the schema files. The `.xsd` files are fetched from upstream at the pinned commit, at build time, and are not redistributed here. The SDMX XML schemas are copyright the SDMX initiative ([sdmx.org](https://sdmx.org)); the bundled `xml.xsd` is copyright W3C, fetched within the SDMX release. The project's MIT/Apache-2.0 licence covers sdmx-rs code only and does not extend to these schemas. See [`NOTICE`](NOTICE) for the full statement.

## Fetching

`just fetch-specs` materialises the pinned `schemas/` trees into `specs/` via the Nix fixed-output derivation and re-verifies each file's sha256 against `sources.toml`. It is idempotent: a `.sha256.stamp` records the verified pin, so a re-run on an up-to-date tree is a no-op. The FOD output is content-addressed and cached (the Nix store locally, a repo-scoped Actions cache in CI), so warm runs never refetch.

## Layout

```
specs/
├── README.md         (this file)
├── 3.0/
│   └── schemas/      SDMX 3.0 schemas (*.xsd) from v3.0.0
└── 3.1/
    └── schemas/      SDMX 3.1 schemas (*.xsd) from v3.1.0
```

The upstream `schemas/` subdirectory is preserved (not flattened) so the schemas' internal `xs:include` / `xs:import` relative references resolve unchanged. The `3.0/` and `3.1/` trees are fetched on demand and gitignored (not committed); only this `README.md`, `sources.toml`, and `NOTICE` are tracked.

Both versions are carried because the workspace targets **both** SDMX 3.0 and 3.1, and their structural divergence (notably data constraints — see [ADR-0008](../docs/adr/0008-model-sdmx-3-0-and-3-1-divergence-with-a-unified-constraintmodel.md)) is normalised into a single canonical domain model.

## Citing a schema from the decision register

The decision register cites schemas by **pinned upstream blob URL**, not by local path: the `.xsd` files are fetched on demand and are absent from a fresh checkout (see [Layout](#layout)). A `Spec ref` row links to the cited type at the pinned commit with a `#L<start>-L<end>` anchor onto its definition, for example `https://github.com/sdmx-twg/sdmx-ml/blob/<commit>/schemas/SDMXCommon.xsd#L219-L255`. The link text names the file and edition (e.g. `SDMXCommon.xsd 3.1`) and the anchor lands on the cited `complexType`/`simpleType`. The `<commit>` is the per-edition `rev` recorded in [`sources.toml`](sources.toml), so a re-pin moves every link by editing one field; the `#L` anchors are recomputed at re-pin, since they shift if upstream reflows a file.
