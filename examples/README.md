# Runnable Examples

Complete, runnable examples demonstrating library usage.  See individual crates for examples:

- [sdmx-types examples](../crates/sdmx-types/examples/README.md): Type construction and validation
- [sdmx-parsers examples](../crates/sdmx-parsers/examples/README.md): Parsing CSV/JSON/XML
- [sdmx-writers examples](../crates/sdmx-writers/examples/README.md): Serialisation
- [sdmx-client examples](../crates/sdmx-client/examples/README.md): HTTP queries and streaming
- [sdmx-rs examples](../crates/sdmx-rs/examples/README.md): End-to-end usage

---

## Running Examples

All examples are in the `examples/` directory of the respective crates and can be executed with:

```bash
cargo run --example <name> -p <crate-name>
```

---

## Using Examples in Your Own Code

Each example is designed to be:
- **Self-contained** — runnable immediately without additional setup
- **Educational** — comments explain each step
- **Copy-paste friendly** — easy to adapt to your use case

---

## Contributing Examples

To add a new example:
1. Create `crates/<crate-name>/examples/my-example.rs`
2. Ensure it compiles and runs with `cargo run --example my-example`
3. Document the purpose in the crate README and update the `crates/<crate-name>/examples/README.md`
4. Update [docs/guides/](../docs/guides/README.md) with a tutorial if it demonstrates a complex pattern

See [CONTRIBUTING.md](../CONTRIBUTING.md) for development workflow.
