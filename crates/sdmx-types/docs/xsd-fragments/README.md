# XSD Contract Fragments

Each `*.md` file here (other than this README) holds one SDMX type's verbatim
`xs:complexType` / `xs:simpleType` definition, sliced from the vendored schemas under
`specs/` and wrapped in a collapsible `<details>` block. The matching Rust type
`include_str!`s its fragment into the `design_docs` rationale, so the authoritative
grammar renders directly under that type's `## Specification`, but only in the internal
docs (`just docs-internal`), never on docs.rs or a normal build.

## Generated: do not edit the fragments by hand

The fragments are produced from [`../../xsd-manifest.toml`](../../xsd-manifest.toml) by
`scripts/gen-xsd-fragments.sh`:

```sh
just gen-xsd-fragments     # regenerate (after a manifest edit or a schema re-vendor)
just check-xsd-fragments   # verify they are fresh and correctly wired (runs in verify-docs)
```

The `check-xsd-fragments` doctor fails CI on any drift, so a hand-edit to a fragment will
not survive. This README is the one authored file in the directory.

## Adding a type

Add a `[[fragment]]` entry to [`../../xsd-manifest.toml`](../../xsd-manifest.toml) (its
header documents the fields: `symbol`, `file`, `editions`, `rust`), run
`just gen-xsd-fragments`, then add the emitted `include_str!` line under the Rust item's
`## Specification`. `just check-xsd-fragments` confirms the wiring.

## Filenames

- `<Symbol>.md`: the type is structurally identical across SDMX 3.0 and 3.1.
- `<Symbol>.3.0.md` + `<Symbol>.3.1.md`: the type diverges structurally between editions
  (e.g. `MaintainableType` gains `isPartialLanguage` in 3.1).
