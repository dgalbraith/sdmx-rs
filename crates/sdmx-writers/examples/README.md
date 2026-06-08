# sdmx-writers Examples

Examples demonstrating `sdmx-writers` usage.

## Running Examples

```bash
cargo run --example <name>
```

---

## Planned Examples

| Example        | Purpose                                                          |
|----------------|------------------------------------------------------------------|
| serialize_xml  | Write SDMX structures to SDMX-ML (XML) format                    |
| serialize_json | Write SDMX structures to SDMX-JSON format                        |
| roundtrip      | Parse → serialize → parse; verify serialization is lossless      |
| target_version | Demonstrate version-aware serialization (SDMX 3.0 vs 3.1 output) |
