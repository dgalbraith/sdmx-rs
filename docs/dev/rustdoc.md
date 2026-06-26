# Rustdoc Conventions

This guide defines how doc comments are authored in `sdmx-rs`. It is the *how* of
rustdoc. For *which* documentation type to write and *why*, see
[Documentation Standards](documentation.md); for API-surface and code conventions
(ergonomic traits, `#[must_use]`, naming, module organisation), see
[Practices & Code Style](practices.md).

The worked exemplars for everything below live in the `annotation` and `lexical`
modules of `crates/sdmx-types`.

---

## Two documentation layers

Every public item has two potential audiences, kept in separate layers:

- **Public (`///`)**: written for a consumer who has never read the design docs.
  What the type is, how to use it, and the SDMX spec terms it maps to. It carries
  no internal references (`D-####`, ADR numbers, `§` design-doc sections).
- **Maintainer (`design_docs`)**: design rationale, decision-register provenance,
  and why an alternative was rejected. Gated behind the `design_docs` cfg so it
  renders only in the internal build (`just docs-internal`) and never on docs.rs.

```rust
#[cfg_attr(design_docs, doc = r#"
## Design Notes

Why this shape, and the rejected alternative.

Decisions: D-0011, D-0035.
"#)]
pub struct Example;
```

At module level the same gate is an inner attribute, placed before the first item:

```rust
#![cfg_attr(design_docs, doc = r#"
## Design Notes
...
"#)]
```

Decision references use a plain `Decisions: D-####` line; the `check-decision-refs`
gate verifies every such reference resolves to `docs/decisions.md`.

The split test: a fact belongs in `///` only if a consumer needs it. Rationale,
register links, and "why not X" go to `design_docs`. Spec *terms* (`xs:decimal`,
`AnnotationURLType`) are consumer-facing and stay public; the *register* is not.

---

## The Specification block

Every type that models an SDMX construct carries a `## Specification` block citing
its place in the standard. Fields are **Type**, **Element**, **Editions**. Two house
rules:

- The colon sits outside the bold: `- **Type**:`, not `- **Type:**`.
- Use **Editions**, not "Versions", to avoid collision with the artefact-versioning
  sense of `SdmxVersion`.

There are five citation cases:

1. **Full citation**, a complexType with an element:

   ```text
   /// ## Specification
   /// - **Type**: `AnnotationURLType`
   /// - **Element**: `<AnnotationURL>`
   /// - **Editions**: SDMX 3.0 and 3.1
   ```

2. **Abstract or simple type**, which has a type but no element. State the negation
   explicitly; never omit the line. Use `N/A (Simple Type)` or `N/A (Abstract Type)`.

3. **W3C primitive**, where the type is an XML Schema built-in, not an SDMX type.
   Cite W3C: `- **Schema**: W3C XML Schema (xs)`, `- **Type**: xs:decimal`,
   `- **Element**: N/A (Primitive)`.

4. **Virtual type**, a Rust-only construct with no schema counterpart (for example a
   projection). State the virtual fact publicly (`- **Schema**: N/A (Virtual Type)`,
   `- **Type**: Rust-specific projection`); the ADR reference goes in `design_docs`.

5. **Divergent editions**, present in both editions but differently. Append
   `(Divergent)` to the Editions line; the diff mechanics and the decision reference
   go in `design_docs`.

Pure utility types (display adapters and the like) that model no SDMX construct carry
no `## Specification` block.

---

## Citing the XSD contract

For the modelled SDMX types, the authoritative grammar is fetched verbatim and embedded
beneath the `## Specification` block, foldable, so the schema sits beside the citation.
The fragment is `include_str!`'d through the `design_docs` gate, so it renders only in
dthe internal build (`just docs-internal`), never on docs.rs:

```rust
/// - **Editions**: SDMX 3.0 and 3.1
#[cfg_attr(design_docs, doc = include_str!("../docs/xsd-fragments/AnnotationURLType.md"))]
```

The fragments are generated, never authored by hand: a `[[fragment]]` entry in
[`xsd-manifest.toml`](../../crates/sdmx-types/xsd-manifest.toml) opts a schema symbol in,
`just gen-xsd-fragments` slices it verbatim, and the `check-xsd-fragments` doctor fails
CI on any drift. The directory
[README](../../crates/sdmx-types/docs/xsd-fragments/README.md) documents the full
mechanism and how to add a type.

---

## Headings

- `#` (H1) is reserved for the canonical rustdoc sections: `# Examples`, `# Errors`,
  `# Panics`, `# Safety`.
- `##` (H2) is used for custom sections: `## Specification`, `## Guarantees`,
  `## Design Notes`.
- `# Errors` lives on the fallible method, not the type, so the type keeps
  `## Specification` and `# Examples` with no clash between the two levels.

---

## Examples

- A fallible example uses `?` with a hidden `Ok` trailer rather than `.unwrap()`:

````text
/// # Examples
///
/// ```
/// use sdmx_types::SdmxDecimal;
///
/// let value: SdmxDecimal = "-3.14".parse()?;
/// assert_eq!(value.as_str(), "-3.14");
/// # Ok::<(), sdmx_types::Error>(())
/// ```
````

- Lead with `.parse()` where the type implements `FromStr`.
- Use `example.com` for illustrative URLs, never a real-looking endpoint.
- An example asserts something; it does not merely construct a value and discard it.

---

## Guarantees

State invariants a consumer can rely on under a `## Guarantees` heading, with the
property itself in a code span:

```text
/// ## Guarantees
///
/// Round-trips losslessly through its text:
/// `x.to_string().parse::<SdmxDecimal>() == Ok(x)`.
```

A guarantee is a claim; its enforcement is a property test, not the doc.

---

## Prose

No em-dashes anywhere, in doc comments, code comments, or guides. Use a comma, a
colon, or a restructured sentence instead.

---

## See also

- [Documentation Standards](documentation.md): documentation types and when to write
  each.
- [Practices & Code Style](practices.md): API-surface conventions (ergonomic traits,
  `#[must_use]`, naming, module organisation).
