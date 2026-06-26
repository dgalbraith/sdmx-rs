# XSD Contract Fragments

Each `*.md` file here (other than this README) holds one SDMX type's verbatim
`xs:complexType` / `xs:simpleType` definition, fetched and sliced at build time
from the pinned schemas under `specs/` and wrapped in a collapsible `<details>`
block. The matching Rust type `include_str!`s its fragment into the `design_docs`
rationale, so the authoritative grammar renders directly under that type's
`## Specification`, but only in the internal docs (`just docs-internal`), never on
docs.rs or a normal build.

## Generated: do not edit the fragments by hand

The fragments are produced from [`../../xsd-manifest.toml`](../../xsd-manifest.toml) by
`scripts/gen-xsd-fragments.sh`:

```sh
just gen-xsd-fragments     # regenerate (after a manifest edit or a schema re-pin)
just check-xsd-fragments   # verify the symbol citation + include_str! wiring (runs in verify-docs)
```

The `check-xsd-fragments` doctor verifies every fragment is wired to its Rust item and
that its schema is pinned in `specs/sources.toml`; the fragments themselves are
regenerated from the pinned schemas on each build, so a hand-edit will not survive. This
README is the one authored file in the directory.

## Adding a type

Add a `[[fragment]]` entry to [`../../xsd-manifest.toml`](../../xsd-manifest.toml) (its
header documents the fields: `symbol`, `file`, `editions`, `rust`), run
`just gen-xsd-fragments`, then add the emitted `include_str!` line under the Rust item's
`## Specification`. `just check-xsd-fragments` confirms the wiring.

## Filenames

- `<Symbol>.md`: the type is structurally identical across SDMX 3.0 and 3.1.
- `<Symbol>.3.0.md` + `<Symbol>.3.1.md`: the type diverges structurally between editions
  (e.g. `MaintainableType` gains `isPartialLanguage` in 3.1).

## Rights

The fragments are verbatim slices of the SDMX XML schemas, copyright the SDMX initiative,
fetched on demand from upstream and not redistributed by this repository (they render only
in the internal `design_docs` build, never in the published crate or on docs.rs). They
carry the same rights as the schemas they are sliced from. See
[`../../../../specs/NOTICE`](../../../../specs/NOTICE) for the full statement.
