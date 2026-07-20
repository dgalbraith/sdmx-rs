# sdmx-client Examples

Examples demonstrating `sdmx-client` usage are planned; none exist yet.

## Running Examples

Examples will live in this crate's `examples/` directory and will run with:

```bash
cargo run --example <name>
```

---

## Planned Examples

| Example             | Purpose                                                                                     |
|---------------------|---------------------------------------------------------------------------------------------|
| fetch_observations  | Initialise a client; construct a type-safe query; execute a request and handle the response |
| stream_observations | Stream large observation datasets with backpressure handling                                |
| error_recovery      | Handle network errors and retry strategies                                                  |
