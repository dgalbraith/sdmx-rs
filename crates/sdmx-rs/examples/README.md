# sdmx-rs Examples (facade)

Runnable examples demonstrating library usage are planned; none exist yet. See the individual crate example indexes:

- [sdmx-types examples](https://github.com/dgalbraith/sdmx-rs/blob/main/crates/sdmx-types/examples/README.md): Type construction and validation
- [sdmx-parsers examples](https://github.com/dgalbraith/sdmx-rs/blob/main/crates/sdmx-parsers/examples/README.md): Parsing CSV/JSON/XML
- [sdmx-writers examples](https://github.com/dgalbraith/sdmx-rs/blob/main/crates/sdmx-writers/examples/README.md): Serialisation
- [sdmx-client examples](https://github.com/dgalbraith/sdmx-rs/blob/main/crates/sdmx-client/examples/README.md): HTTP queries and streaming


## Running Examples

Examples will live in the `examples/` directory of the respective crates and will run with:

```bash
cargo run --example <name> -p <crate-name>
```

---

## Planned Examples

| Example             | Purpose                                     |
|---------------------|---------------------------------------------|
| end_to_end_workflow | Demonstrate the end-to-end usage of the API |

---

See [CONTRIBUTING.md](https://github.com/dgalbraith/sdmx-rs/blob/main/CONTRIBUTING.md) for the development workflow, including how to add examples.
