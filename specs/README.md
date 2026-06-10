# Vendored SDMX Schemas

The official **SDMX XML schemas** (.xsd) that the sdmx-rs workspace is modelled against, copied from upstream releases and version-pinned in this repository. The decision register ([docs/decisions.md](../docs/decisions.md)) cites them as the source of truth for the domain model in `sdmx-types`, and the parsers in `sdmx-parsers` validate against them.

## Provenance

Each vendored set records its upstream source, pinned release tag, commit SHA, and retrieval date. Only the `schemas/` subtree is vendored — the full `.xsd` set. Release `docs/`, `documentation/`, and `samples/` are not vendored; official sample messages belong with parser conformance fixtures under `crates/*/tests/`.

| Version | Source                                                                                                                              | Tag      | Commit                                     | Retrieved  | Vendored                                            |
|---------|-------------------------------------------------------------------------------------------------------------------------------------|----------|--------------------------------------------|------------|-----------------------------------------------------|
| 3.0     | [sdmx-twg/sdmx-ml](https://github.com/sdmx-twg/sdmx-ml) ([release v3.0.0](https://github.com/sdmx-twg/sdmx-ml/releases/tag/v3.0.0)) | `v3.0.0` | `29f1a3d856c4259429f5ec0eae811653adc5cdb5` | 2026-06-09 | `schemas/` subtree (30 `.xsd`, incl. W3C `xml.xsd`) |
| 3.1     | [sdmx-twg/sdmx-ml](https://github.com/sdmx-twg/sdmx-ml) ([release v3.1.0](https://github.com/sdmx-twg/sdmx-ml/releases/tag/v3.1.0)) | `v3.1.0` | `182248b3c8030b595187dca51ca341d5ff839c24` | 2026-06-09 | `schemas/` subtree (30 `.xsd`, incl. W3C `xml.xsd`) |

Source tarballs:

- 3.0 — `https://github.com/sdmx-twg/sdmx-ml/archive/refs/tags/v3.0.0.tar.gz`
- 3.1 — `https://github.com/sdmx-twg/sdmx-ml/archive/refs/tags/v3.1.0.tar.gz`

In each case the vendored files are the contents of the release's `schemas/` directory, copied verbatim with the internal `xs:include` / `xs:import` relative paths intact.

When updating a vendored set, record the change here (old → new tag/SHA, date, reason) rather than overwriting silently.

## Layout

```
specs/
├── README.md         (this file)
├── 3.0/
│   └── schemas/      SDMX 3.0 schemas (*.xsd) from v3.0.0
└── 3.1/
    └── schemas/      SDMX 3.1 schemas (*.xsd) from v3.1.0
```

The upstream `schemas/` subdirectory is preserved (not flattened) so the schemas' internal `xs:include` / `xs:import` relative references resolve unchanged.

Both versions are carried because the workspace targets **both** SDMX 3.0 and 3.1, and their structural divergence (notably data constraints — see [ADR-0008](../docs/adr/0008-model-sdmx-3-0-and-3-1-divergence-with-a-unified-constraintmodel.md)) is normalised into a single canonical domain model.

## Citing a schema from the decision register

Reference the pinned path, e.g. `specs/3.1/schemas/SDMXCommon.xsd`, so "Source: SDMXCommon.xsd 3.1" in a `D-00xx` entry resolves to a concrete, version-tracked file.
