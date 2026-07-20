# sdmx-types Examples

Examples demonstrating `sdmx-types` usage are planned; none exist yet.

## Running Examples

Examples will live in this crate's `examples/` directory and will run with:

```bash
cargo run --example <name>
```

---

## Planned Examples

| Example                 | Purpose                                                                                                      |
|-------------------------|--------------------------------------------------------------------------------------------------------------|
| construct_codelist      | Create a `Codelist` with items and validation; basic type creation and validation                            |
| construct_datastructure | Create a `DataStructureDefinition` with dimensions/attributes/measures; nested structures                    |
| construct_constraint    | Create a `ConstraintModel`: handle SDMX 3.0 vs 3.1 divergence                                                |
| trait_hierarchy         | Demonstrate the `IdentifiableArtefact` → `MaintainableArtefact` trait pattern; shows the abstraction pattern |
