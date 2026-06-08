# sdmx-rs Examples (facade)

Complete, runnable examples demonstrating library usage.  See individual crates for examples:

- [sdmx-types examples](../../sdmx-types/examples/README.md): Type construction and validation
- [sdmx-parsers examples](../../sdmx-parsers/examples/README.md): Parsing CSV/JSON/XML
- [sdmx-writers examples](../../sdmx-writers/examples/README.md): Serialization
- [sdmx-client examples](../../sdmx-client/examples/README.md): HTTP queries and streaming


## Running Examples

All examples are in the `examples/` directory of the respective crates and can be executed with:

```bash
cargo run --example <name> -p <crate-name>
```

---

## Planned Examples

| Example             | Purpose                                     |
|---------------------|---------------------------------------------|
| end_to_end_workflow | Demonstrate the end-to-end usage of the API |

---

## Contributing Examples

To add a new example:
1. Create `crates/<crate-name>/examples/my-example.rs`
2. Ensure it compiles and runs with `cargo run --example my-example -p <crate-name>`
3. Document the purpose in the crate README and update the `crates/<crate-name>/examples/README.md`
4. Update [docs/guides/](../../../docs/guides/README.md) with a tutorial if it demonstrates a complex pattern

See [CONTRIBUTING.md](../../../CONTRIBUTING.md) for development workflow.
