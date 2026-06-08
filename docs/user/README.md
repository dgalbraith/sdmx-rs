# Using sdmx-rs

Quick-start documentation for library users.

## Quick Start

1. **Add to your Cargo.toml**:
   ```toml
   [dependencies]
   sdmx-rs = "0.1"  # Once published to crates.io
   ```

2. **Check the [Getting Started](getting-started.md)** guide.

3. **Check the [Examples](../../examples/README.md)** directory.

4. **Explore the [API Documentation](https://docs.rs/sdmx-rs)** (once published) or locally:
   ```bash
   cargo doc --open
   ```

---

## Documentation Structure

- **[Getting Started](getting-started.md)** — First query, basic usage
- **[User Guides](../guides/README.md)** — Detailed tutorials by use case
- **[Examples](../../examples/README.md)** — Runnable code samples
- **[API Reference](https://docs.rs/sdmx-rs)** — Type and function documentation (rustdoc) via the facade crate re-exports

---

## Common Questions

**Q: Where do I find API documentation?**
A: See [docs.rs/sdmx-rs](https://docs.rs/sdmx-rs) or run `cargo doc --open` locally.

**Q: How do I report bugs or request features?**
A: Open an issue on [GitHub](https://github.com/dgalbraith/sdmx-rs/issues).

**Q: Is there an MSRV? What's the support policy?**
A: See [docs/project/msrv.md](../../docs/project/msrv.md) for version information and [SECURITY.md](../../SECURITY.md) for support versions.

---

## Current Phase

Phase 0 (infrastructure) is complete; the library is in **Phase 1** (core domain types), under active development. User-facing client APIs will be available in Phase 3+.

See [ROADMAP.md](../../ROADMAP.md) for detailed timeline.
