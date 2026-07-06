# Code Style Guide

This guide documents the visual and structural conventions used across sdmx-rs. The goal is consistency, readability, and idiomatic Rust practice. Style is enforced, not debated.

We follow the [Rust API Guidelines (RFC 430)](https://rust-lang.github.io/api-guidelines/) and align with conventions in the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/). Where sdmx-rs has project-specific constraints (no_std, WASM, async), we document them explicitly.

## Code Formatting

**Rust**: Formatted with nightly rustfmt via `cargo fmt`. All code must pass `nightly rustfmt` checks.

Run locally:
```bash
just fmt
```

CI enforces formatting as part of `just verify`.

**TOML**: Formatted with taplo via `cargo fmt` (taplo is invoked as part of the unified fmt target).

**Markdown**: Validated with markdownlint-cli. Style rules are in [.markdownlint.yaml](../../.markdownlint.yaml).

**Key principle**: Don't debate formatting. If the toolchain enforces it, follow it. If the toolchain doesn't enforce it, consistency matters more than preference—pick a convention and stick to it.

## Naming Conventions

Naming conventions follow [RFC 430 — Naming Conventions](https://rust-lang.github.io/api-guidelines/naming.html).

### Functions & Methods
- **snake_case** for all function and method names ([Rust idiom](https://rust-lang.github.io/api-guidelines/naming.html#function-and-method-names-follow-snakecase-c-fn))
- Imperative verbs when appropriate: `build()`, `parse()`, `validate()`
- Descriptive names over short abbreviations: `parse_sdmx_xml()` not `parse_sx()`

```rust
// Good
pub fn parse_constraint_model(input: &str) -> Result<Model> { }

// Avoid
pub fn parse_cm(input: &str) -> Result<Model> { }
```

### Types & Traits
- **CamelCase** for structs, enums, traits ([RFC 430 — Type Naming](https://rust-lang.github.io/api-guidelines/naming.html#type-names-follow-pascalcase-c-type))
- Descriptive nouns: `ConstraintModel`, `DataflowRequest`
- Avoid abbreviations unless they are standard domain terms (e.g., `SDMX`, `DSL`, `HTTP`)

```rust
// Good
pub struct ConstraintModel { }
pub enum ContentType { }

// Avoid
pub struct CM { }
pub enum CT { }
```

### Constants
- **SCREAMING_SNAKE_CASE** (Rust idiom)
- Descriptive: `DEFAULT_TIMEOUT_SECS` not `DTS`

### Type Parameters & Lifetimes
- Single uppercase letter for generic type parameters (Rust convention): `<T>`, `<E>`
- Descriptive names only when semantically important:
  ```rust
  pub fn map<T, U>(self, f: impl Fn(T) -> U) -> Result<U>  // Generic
  pub struct Parser<Output> { }  // Semantically important
  ```

- Named lifetimes when they clarify intent:
  ```rust
  pub fn borrow<'a>(&'a self) -> &'a Data  // Explicit
  fn transform<'a>(&'a self, input: &'a str)  // Unclear—use different lifetimes or shorter scope
  ```

### Modules
- **snake_case** (Rust idiom)
- Single word when possible; compound words are acceptable if clear: `constraint_model`, `client`, `parsers`
- Organise hierarchically: `crates/sdmx-types/src/constraint/mod.rs` for the `constraint` module

## Spelling Convention

Use British by default; use American for anything owned by an external ecosystem.

- **British** — SDMX information-model terms (`Artefact`, `Organisation`, `Localised`) and all project prose (serialisation, modelling, behaviour, optimisation).
- **American** — externally-owned vocabulary: Rust/serde API identifiers (`Serialize`, the `serialize` method, `#[serde(...)]`), tool names (`rust-analyzer`), protocol tokens (the HTTP `Authorization` header), and the crates.io `serialization` keyword.

Tie-breaker: in a comment about a specific external trait's action, use the trait's spelling; for the concept broadly, British.

Two homographs turn on the referent, not the spelling:
- **`artefact`/`artifact`**: An SDMX metadata object (`IdentifiableArtefact`) is British **artefact**. A build or supply-chain output (`.crate`, GitHub Actions artifacts) is American **artifact**.
- **`licence`/`license`**: The SPDX field, `LICENSE-*` files, and proper nouns (`MIT License`) are American **license**. General project prose uses British **licence**.

Resolve both by what the word refers to, not by its surface form.

Generated and externally-sourced content is exempt and follows its source verbatim: the fetched `specs/`, the generated XSD fragments under `crates/sdmx-types/docs/xsd-fragments/` (run `just gen-xsd-fragments`), the `CODE_OF_CONDUCT.md` (the adopted Contributor Covenant), and the `LICENSE-*` files.

## Commenting Philosophy

**Comment the WHY, not the WHAT.** Code already describes what it does. Comments should explain intent, constraints, or non-obvious decisions.

### When to Comment
- **Always**: Invariants, preconditions, or subtle gotchas
  ```rust
  // Parse in reverse order because the spec defines constraints in dependency order,
  // but constraint resolution requires breadth-first traversal.
  for item in items.iter().rev() { }
  ```

- **Always**: Workarounds for specific bugs or platform quirks
  ```rust
  // FIXME: reqwest 0.11.x drops connection on idle timeout; drain response before returning
  // See https://github.com/seanmonstar/reqwest/issues/1340
  let _ = response.text().await;
  ```

- **Always**: Non-standard or surprising behaviour
  ```rust
  // Intentionally panic on invalid config; this is not a recoverable error.
  // Operator must fix the configuration file.
  panic!("Invalid configuration: {}", err);
  ```

### When NOT to Comment
- Describe self-documenting code
  ```rust
  // Bad
  // Check if the string is empty
  if name.is_empty() { }

  // Good (no comment needed)
  if name.is_empty() { }
  ```

- Restate function names or signatures
  ```rust
  // Bad
  /// Parses a constraint model
  fn parse_constraint_model(input: &str) -> Result<Model> { }

  // Good (rustdoc is enough; see Rustdoc Style section)
  ```

- Comment every line or every function
  ```rust
  // Bad
  // Increment i
  i += 1;
  // Check if i is greater than 10
  if i > 10 { }

  // Good (code is clear; no comment needed)
  i += 1;
  if i > 10 { }
  ```

**Line comments**: Use `//` for inline commentary.

**Block comments**: Use `/* */` only when necessary; prefer multiple `//` lines for readability.

### Safety Comments

Every `unsafe` block must have a `// SAFETY:` comment explaining why the unsafe operation is sound. This follows [Rustonomicon guidance](https://doc.rust-lang.org/nomicon/) and documents the invariants the compiler cannot verify.

```rust
// Good: Explains the invariant that makes this safe
unsafe {
    // SAFETY: We verified that ptr is valid and aligned before this call.
    // The data it points to is initialised and will not be mutated while this
    // reference is alive (we hold an exclusive lock).
    &*ptr
}

// Avoid: No explanation
unsafe {
    &*ptr
}
```

**Format**: `// SAFETY: [explanation of why the unsafe operation is sound]`

**Content**: Explain:
- What invariants you're relying on
- Why those invariants hold true at this point
- What could go wrong if the invariant is violated

See ADR-0002 for the policy on unsafe code (forbidden by default).

### Comment Markers
Use `# MAINTENANCE:` for periodic obligations; see [maintenance.md](../project/maintenance.md) for format and examples.

## Rustdoc Style

Rustdoc comments (`///`) are required for all public items (types, functions, methods, modules, constants, traits). Rustdoc is validated in CI: missing or incomplete docs cause compilation to fail. See [RFC 430 — Documentation](https://rust-lang.github.io/api-guidelines/documentation.html) for detailed guidelines.

The `sdmx-rs` authoring conventions (the public `///` versus `design_docs` split, the `## Specification` citation discipline, and the heading, example, and `## Guarantees` idioms) have their own guide: [Rustdoc Conventions](rustdoc.md). The baseline format below applies across the workspace.

### Format

**One-line summary (preferred)**:
```rust
/// Parses an SDMX-ML constraint model from the given string.
pub fn parse_constraint_model(input: &str) -> Result<Model> { }
```

**Multi-line with extended description**:
```rust
/// Parses an SDMX-ML constraint model from the given string.
///
/// This function validates the input against the SDMX-ML schema and
/// constructs an in-memory representation. See [`ConstraintModel`] for details.
///
/// # Errors
///
/// Returns `Err` if the input is not valid SDMX-ML or violates structural constraints.
///
/// # Examples
///
/// ```
/// let xml = r#"<ConstraintModel>...</ConstraintModel>"#;
/// let model = parse_constraint_model(xml)?;
/// ```
pub fn parse_constraint_model(input: &str) -> Result<Model> { }
```

### Sections

Use markdown headers for common sections:
- `# Errors` — when the function returns `Result`
- `# Panics` — when the function can panic (prefer avoiding panics)
- `# Safety` — when the function is unsafe
- `# Examples` — always include if the function is complex or not self-evident

**Avoid**: Restating parameter names or types; rustdoc renders the signature.

```rust
// Bad
/// Gets the value.
/// # Arguments
/// * `key` - The key to look up
pub fn get(&self, key: &str) -> Option<&Value> { }

// Good
/// Returns the value associated with `key`, or `None` if not found.
pub fn get(&self, key: &str) -> Option<&Value> { }
```

### Examples
Examples in rustdoc must compile and run. Use `#` to hide setup boilerplate:

```rust
/// # Examples
///
/// ```
/// # use sdmx_types::ConstraintModel;
/// let model = ConstraintModel::new();
/// assert!(model.is_empty());
/// ```
pub fn new() -> Self { }
```

Examples that are expected to fail should use `should_panic` or `ignore`:

```rust
/// # Examples
///
/// ```should_panic
/// let model = ConstraintModel::from_invalid_xml("<invalid");
/// ```
pub fn from_xml(input: &str) -> Self { }
```

## Module & File Organisation

### Crate Structure

Each crate has a clear purpose:
- **sdmx-types**: Domain types and data structures (no I/O, no async)
- **sdmx-parsers**: Streaming deserialisation of SDMX payloads (XML/JSON/CSV) into domain types
- **sdmx-writers**: Serialisation of domain types to SDMX formats (XML/JSON/CSV)
- **sdmx-client**: HTTP client for fetching and managing SDMX data
- **sdmx-rs**: Unified facade crate (re-exports key types and builders)

### Module Layout

A typical crate:

```
crates/sdmx-parsers/src/
├── lib.rs                    # Module declarations, re-exports, feature gating
├── error.rs                  # Custom error types
├── xml/
│   ├── mod.rs               # Submodule declaration and re-exports
│   ├── constraint.rs        # Implementation
│   └── dataflow.rs          # Implementation
└── json/
    ├── mod.rs               # Submodule declaration and re-exports
    └── constraint.rs        # Implementation
```

### Visibility Boundaries

- **Private by default.** Mark as `pub` only when necessary for the crate's public API.
- **Module contents**: Use `pub use` in `mod.rs` to expose public types and functions.
  ```rust
  // crates/sdmx-parsers/src/xml/mod.rs
  pub use constraint::parse_constraint_model;
  pub use dataflow::parse_dataflow;
  ```

- **Crate re-exports**: The facade crate (sdmx-rs) re-exports key types from implementation crates.
  ```rust
  // crates/sdmx-rs/src/lib.rs
  pub use sdmx_types::{ConstraintModel, Dataflow};
  ```

### File Naming

- Use `mod.rs` for module roots; do not use module name as file (Rust convention)
  ```
  // Good
  src/constraint/mod.rs

  // Avoid
  src/constraint.rs (used only for single-file modules with no submodules)
  ```

- Split large modules into submodules and organise logically:
  ```
  src/constraint/
  ├── mod.rs           # Declares submodules, re-exports public types
  ├── model.rs         # ConstraintModel definition
  ├── rule.rs          # Rule types
  └── validation.rs    # Validation logic
  ```

## Code Organisation Patterns

### Imports

- **Organise imports**: stdlib → external crates → internal crates
  ```rust
  use std::collections::HashMap;

  use serde::{Deserialize, Serialize};

  use crate::constraint::ConstraintModel;
  use crate::error::ParseError;
  ```

- **Use `use` statements, not `mod::`** (idiomatic Rust)
  ```rust
  // Good
  use crate::constraint::ConstraintModel;
  let model = ConstraintModel::new();

  // Avoid
  let model = crate::constraint::ConstraintModel::new();
  ```

### Type Definitions

- Group related types in the same module
  ```rust
  // src/constraint/model.rs
  pub struct ConstraintModel { }
  pub enum ConstraintType { }
  pub struct Rule { }
  ```

- Implement traits near the type or in a logical grouping module
  ```rust
  // src/constraint/model.rs
  impl ConstraintModel { }
  impl From<XmlElement> for ConstraintModel { }
  ```

### Collection Construction

- **Empty vectors use `Vec::new()`; reserve `vec![...]` for non-empty literals.** Both forms compile identically for the empty case, so keeping the `vec!` macro for elements-bearing literals keeps it meaningful: its presence always signals content.
  ```rust
  // Good
  let components = Vec::new();
  let codes = vec![code_a, code_b];

  // Avoid
  let components = vec![];
  ```

### String Construction

- **String literals become `String` through `String::from("x")`.** The literal-conversion forms compile identically; `String::from` names the constructed type as the leading token, matching `None::<T>` and `Vec::new()`, where `"x".to_string()` reaches the value through the `Display` machinery and `"x".into()` names no target at all. This applies in doc examples as in tests and implementation code. `.to_string()` remains correct on non-literal receivers, where it is Display rendering rather than literal conversion.
  ```rust
  // Good
  let id = String::from("CL_FREQ");
  let rendered = version.to_string(); // Display on a value, not a literal conversion

  // Avoid
  let id = "CL_FREQ".to_string();
  let id: String = "CL_FREQ".into();
  ```

## Fixture Construction

- **Test fixtures build validated types through their constructors (`new()` / `parse()`), never by struct literal.** A constructor-routed fixture is a value the type actually admits and stays coupled to the invariants as they change: tighten a rule and every affected fixture fails loudly, where a literal would keep compiling with a now-illegal value. The invariant-free pub-field carriers (`LocalisedText`, `DimensionRef`, `TextFormat`, ...) have no constructor and are legitimately literal-built. A fixture that deliberately needs a state the constructor forbids (exercising a defensive backstop, for example) stays literal with a one-line comment naming the intent, so the bypass reads as deliberate. This split is a semantic judgement no check script can enforce; this documented convention is the only defense against regression.
  ```rust
  // Good: routed through the validating constructor
  let agency = Agency::new(metadata("SDMX"), Vec::new()).unwrap();
  // Good: an invariant-free carrier has no constructor to route through
  let text = LocalisedText { language: Some("en".to_string()), text: "Frequency".to_string() };

  // Avoid: a literal of a validated type bypasses its invariants
  let agency = Agency { metadata: metadata("SDMX"), contacts: Vec::new() };

### Error Handling

All crates use `thiserror` for error types. See ADR-0006 for detailed conventions and [RFC 430 — Error Handling](https://rust-lang.github.io/api-guidelines/errors.html).

**Philosophy**: Return `Result<T>` for recoverable errors; panic only for truly unrecoverable conditions (invariant violations, configuration errors that should never happen in production).

**Error definition** goes in `error.rs`:
```rust
// crates/sdmx-parsers/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid XML: {0}")]
    InvalidXml(String),

    #[error("Missing required element: {0}")]
    MissingElement(String),
}
```

**Using errors**:
- **`?` operator (preferred)**: Propagate errors up the call stack when the caller should decide how to handle it
  ```rust
  pub fn parse(input: &str) -> Result<Model> {
      let xml = validate_xml(input)?;  // Error propagates
      Ok(Model::from(xml))
  }
  ```

- **`unwrap()` (avoid in libraries)**: Only in tests, examples, or binary code where panic is acceptable
  ```rust
  // OK in tests or bin
  let model = parse(input).unwrap();

  // Avoid in library code
  let model = parse(input).unwrap();  // Panics on error—caller has no choice
  ```

- **`expect(msg)` (use sparingly)**: When an error would indicate a serious invariant violation, use `expect()` with a clear message explaining why this should never happen
  ```rust
  // Good: message explains the invariant
  let model = self.get_cached_model()
      .expect("Model should exist after validation passed");

  // Avoid: message repeats the error
  let model = parse(input).expect("failed to parse");
  ```

**Error context**: Add context as errors propagate when it clarifies the operation
```rust
pub fn load_from_file(path: &Path) -> Result<Model> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ParseError::FileError(format!("{}: {}", path.display(), e)))?;
    parse(&content)
}
```

### Trait Implementations

**Location**: Implement traits on a type in the same module where the type is defined. See [RFC 430 — Trait implementations](https://rust-lang.github.io/api-guidelines/type-safety.html#types-eagerly-implement-common-traits-c-common-traits).

```rust
// src/constraint/model.rs
pub struct ConstraintModel { }

impl ConstraintModel {
    pub fn new() -> Self { }
}

impl From<XmlElement> for ConstraintModel {
    fn from(element: XmlElement) -> Self { }
}

impl std::fmt::Display for ConstraintModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { }
}
```

**Idiomatic patterns**:

- **Prefer `From` over custom constructors** when the conversion is infallible and semantically clear
  ```rust
  // Good: From makes intent clear
  impl From<XmlElement> for ConstraintModel { }
  let model: ConstraintModel = xml_element.into();

  // Avoid: Custom constructor for simple conversions
  impl ConstraintModel {
      pub fn from_xml(element: XmlElement) -> Self { }
  }
  ```

- **Use `TryFrom` for fallible conversions**
  ```rust
  impl TryFrom<&str> for ConstraintModel {
      type Error = ParseError;
      fn try_from(input: &str) -> Result<Self, ParseError> { }
  }
  ```

- **Blanket implementations** (e.g., implementing a trait for all `T` that satisfy bounds) go with the trait definition, not the types
  ```rust
  // In the trait module, not in the type's module
  impl<T: Display> MyTrait for T { }
  ```

- **Marker traits** (zero-method traits like `Send`, `Sync`) use `impl Trait for Type {}` with no body
  ```rust
  // Signal that ConstraintModel is safe to send across threads
  unsafe impl Send for ConstraintModel { }
  ```

#### Trade-off: Trait bounds in function signature vs impl block

Choose based on clarity and reusability:

```rust
// Good: Simple, bounded trait; clear from signature
fn process<T: Clone>(item: T) { }

// Better: Complex bounds; use where clause for readability
fn process<T>(item: T)
where
    T: Clone + Display + Send,
    T::Error: std::error::Error,
{ }

// Good: Many type parameters; impl block is clearer
impl<T, U> From<T> for U
where
    T: IntoIterator<Item = U>,
{ }
```

### Builder Pattern

The facade crate uses builders for complex construction (e.g., `ClientBuilder`). Builders follow the pattern:

```rust
pub struct ClientBuilder {
    timeout: Option<Duration>,
    base_url: String,
}

impl ClientBuilder {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            timeout: None,
            base_url: base_url.into(),
        }
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    pub fn build(self) -> Result<Client> {
        Client::new(self.base_url, self.timeout)
    }
}
```

**Guidelines**:
- Do not implement `Default` on builders; use `::new()` for clarity
- Make field visibility private; expose only through builder methods
- Return `self` from configuration methods to enable chaining
- The `build()` method is the only one that consumes (no `&mut self` after calling `build()`)

**When NOT to use**:
- Simple types with 1-2 optional fields → just use function parameters
- Immutable-only APIs → consider just returning a pre-configured value
- When type parameters would be complex → may be simpler to use function overloads

### Derive Traits

Prefer deriving standard traits when they make semantic sense. Manual implementations should be reserved for types with non-obvious behaviour.

**Standard derives (prefer when applicable)**:

```rust
#[derive(Clone, Copy, Debug)]
pub struct ModelId(u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintModel {
    rules: Vec<Rule>,
}
```

**When to derive vs implement manually**:

| Trait             | Derive | Manual                               | When                                                                        |
| ------------------|:------:|--------------------------------------|-----------------------------------------------------------------------------|
| `Clone`           | ✅     | Needed for non-trivial cloning logic | Most types; use manual for expensive clones that need custom logic          |
| `Debug`           | ✅     | Rarely                               | Most types; manual only if you want custom formatting                       |
| `Default`         | ✅     | For complex initialisation           | Simple types; use manual when `Default` has non-obvious behaviour            |
| `PartialEq`, `Eq` | ✅     | Often needed                         | If all fields are `PartialEq`, derive; manual for custom equality semantics |
| `Display`         | ❌     | ✅ Always                            | No derive; requires explicit `fmt` implementation                           |
| `From`, `Into`    | ❌     | ✅ Always                            | Implement `From`; `Into` is automatic                                       |

#### Example: Manual implementation when semantics matter

```rust
// Derive works fine
#[derive(Clone, Debug)]
pub struct Config { timeout: Duration }

// Manual: PartialEq ignores internal counters (intentional)
impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.base_url == other.base_url
        // Don't compare internal state (request counts, etc.)
    }
}
```

**Guidelines**:
- Derive when the behaviour is obvious and correct
- Document why you manually implement a trait (unusual semantics)
- `#[derive(Debug)]` is almost always desirable for debugging

### Lexical newtype trait surface

The lexical newtypes (D-0027, extended by the D-0070..D-0072 grammar series) validate their grammar at construction and fork on storage: `SdmxDecimal`, `SdmxInteger`, `SdmxTimePeriod`, and `SdmxTimeRange` store the raw lexeme losslessly; `SdmxVersion` and `VersionRef` are raw-free (their canonical grammars admit one lexeme per value, so the parsed decomposition is stored and the text is reconstructed); and the `ObservationalTimePeriod` union preserves its members' lexemes. The standard-trait surface is deliberate: it exposes the lexical form for reading and parsing without dissolving the validated boundary, with each row's scope following the storage class.

| Trait                            | Provided | Rationale                                                                                                                                                                                                                                                                                                  |
| -------------------------------- |:--------:|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `Display`                        | ✅       | Renders the lexical form: the stored lexeme verbatim on the lexeme-storing types, the reconstructed canonical lexeme on the raw-free grammars (one spelling per value, D-0070); never a normalisation.                                                                                                     |
| `FromStr`                        | ✅       | Delegates to `new()`, so `"…".parse()` is the idiomatic, validating constructor.                                                                                                                                                                                                                           |
| `AsRef<str>`                     | ✅       | Explicit borrow of the stored lexeme, on the lexeme-storing types only; deliberately absent on `SdmxVersion`/`VersionRef`, whose lexeme is reconstructed, so there is nothing to borrow.                                                                                                                   |
| `PartialEq` / `Eq`               | ✅       | String identity over the stored raw; structural over the raw-free decomposition, the same partition by canonicity (D-0070). Lossless-distinct: `SdmxVersion` treats `1.0.0-rc` and `1.0.0`, and `3.1` and `3.1.0`, as unequal.                                                                             |
| `PartialEq<str>` / `<&str>`      | ✅       | String identity with the stored lexeme, on the lexeme-storing types only (the raw-backed newtypes and the `ObservationalTimePeriod` union); the raw-free `SdmxVersion`/`VersionRef` take none (D-0074).                                                                                                    |
| `Ord` / `PartialOrd`             | ❌       | Deferred. No member admits a total order consistent with its lossless-distinct `Eq` (numeric on the decimals, chronological on the time types, legacy-vs-semantic on the versions), so a type-level `Ord` would violate the `Ord`/`Eq` contract (D-0060); precedence is an explicit method or wrapper.     |
| `Borrow<str>`                    | ❌       | Conditional only, and expressible only on the lexeme-storing types. It would require `Eq`/`Hash`/`Ord` consistent with `str`, locking the equality/ordering semantics and foreclosing the `SdmxVersion` precedence question. Adopt only if a `str`-keyed lookup need arises and the semantics are settled. |
| `Deref<str>`                     | ❌       | Rejected. Auto-deref would surface every `str` method as the newtype's own API, dissolving the validated boundary; `AsRef<str>` gives explicit raw access instead.                                                                                                                                         |

The single-`String` newtypes (`SdmxDecimal`, `SdmxInteger`, `SdmxTimeRange`) additionally provide `From<Self> for String` and an inherent `into_inner()`. A *consuming* unwrap moves the value out rather than coercing it while the newtype is live, so it does not dissolve the validated boundary the way `Deref` would. `SdmxVersion` holds only its parsed decomposition (there is no stored lexeme to unwrap), and the remaining lexeme-storing types (`SdmxTimePeriod` with its retained kind, the `ObservationalTimePeriod` union) expose no such unwrap.

Decisions: D-0027, D-0060, D-0070, D-0071, D-0072, D-0074.

### Const and Const Functions

Use `const` for values known at compile time; use `const fn` for functions that can be evaluated at compile time.

```rust
// Good: Compile-time constant
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

// Good: Const function (no allocations, no I/O)
pub const fn model_id(id: u64) -> u64 {
    id
}

// Avoid: Runtime initialisation
pub const PATHS: Vec<&'static str> = vec!["a", "b"];  // Error: vec! not const
```

**When to use `const fn`**:
- Simple computations (arithmetic, bitwise, comparisons)
- Type-level programming (const generics, type constants)
- Initialisation of `const` values

**When NOT to use `const fn`**:
- Any heap allocation (Vec, String, Box)
- I/O operations
- Function pointers or closures that aren't `const`
- Complex control flow that isn't evaluated at compile time

**Guidelines**:
- Only mark a function `const fn` if its body can actually be evaluated at compile time
- Const functions must not panic; errors are compile-time errors, not runtime panics
- Use `const fn` sparingly; prioritise clarity over compile-time evaluation unless there's a clear benefit

## Core Trade-offs

This guide documents conventions, but they serve readability and maintainability—not dogma. When conventions don't fit, understand the trade-off and make an intentional choice.

### Module Granularity

**Convention**: One responsibility per module; split large modules into submodules.

**Trade-off**: Many small modules (fine granularity) vs fewer large modules (coarse granularity)

**Decision**: Start coarse; split when a module exceeds ~500 lines or has clearly separable concerns. Prefer keeping related logic together over perfect organisation.

```rust
// Coarse: Related parsing logic stays together
pub mod xml {
    pub fn parse_constraint(input: &str) -> Result<ConstraintModel> { }
    pub fn parse_dataflow(input: &str) -> Result<Dataflow> { }
}

// Fine: Separated by domain type (when xml/ grows large)
pub mod xml {
    pub use constraint::parse_constraint;
    pub use dataflow::parse_dataflow;
    mod constraint;
    mod dataflow;
}
```

### Generic Complexity

**Convention**: Use generics for flexibility; keep type parameters reasonable.

**Trade-off**: Flexibility (generic code works with many types) vs clarity (too many type parameters hurt readability and compilation time)

**Decision**: Limit to ≤ 3 type parameters in a function signature; move complex bounds to `where` clauses. If a generic is complex enough to confuse users, document it with examples.

```rust
// Good: Simple, clear
pub fn map<T, U>(item: T, f: impl Fn(T) -> U) -> U { }

// Avoid: Too many bounds in signature
pub fn process<T, U, V, E>(a: T, b: U, c: V) -> Result<(T, U), E>
where
    T: Clone + Debug + Send,
    U: Into<String>,
    V: Iterator,
    E: std::error::Error,
{ }

// Better: Document or simplify
pub type ComplexResult<T> = Result<T, ParseError>;
pub fn process<T, U>(a: T, b: U) -> ComplexResult<(T, U)>
where
    T: Clone + Debug + Send,
    U: Into<String>,
{ }
```

### Type Safety vs Convenience

**Convention**: Prefer explicit types (newtypes) over type aliases when semantically distinct.

**Trade-off**: Type safety (can't mix types) vs convenience (shorter, less wrapping)

**Decision**: Use newtypes for domain concepts; use aliases for clarity on complex signatures.

```rust
// Good: Semantic distinction; can't accidentally swap
pub struct ModelId(u64);
pub struct RuleId(u64);

// Good: Clarity alias for complex type
pub type ParseResult<T> = Result<T, ParseError>;

// Avoid: Semantic type as alias
pub type EntityId = u64;  // Too easy to mix with other u64
```

## Type-Level Programming

Leverage Rust's type system to encode invariants and prevent invalid states at compile time.

**Newtype for semantic distinction**:

```rust
// Good: Type-safe; can't accidentally mix types
pub struct ModelId(u64);
pub struct RuleId(u64);

let model: ModelId = ModelId(42);
let rule: RuleId = RuleId(42);
// let wrong: ModelId = rule;  // ❌ Compile error—types don't match
```

**Phantom types for compile-time tracking (advanced)**:

```rust
use std::marker::PhantomData;

// Encode builder state in the type system
pub struct QueryBuilder<T> {
    query: String,
    _state: PhantomData<T>,
}

pub struct AgencySet;
pub struct ResourceTypeSet;

impl QueryBuilder<()> {
    pub fn agency(self, id: &str) -> QueryBuilder<AgencySet> { }
}

impl QueryBuilder<AgencySet> {
    pub fn resource_type(self, ty: &str) -> QueryBuilder<(AgencySet, ResourceTypeSet)> { }
}

// Only fully-specified queries can be executed
impl QueryBuilder<(AgencySet, ResourceTypeSet)> {
    pub fn build(self) -> Result<Query> { }
}
```

**Guidelines**:
- Use newtypes when types should not be interchangeable
- Use phantom types when you need to track state or prevent certain combinations at compile time (advanced pattern; see [Design 0007](../design/0007-typestate-query-validation.md) for sdmx-rs's typestate query builder)
- Avoid phantom types for simple cases; they add cognitive overhead
- Document why a phantom type is necessary; it's not obvious to readers

## Async/Await Patterns

The project uses **Tokio** as its async runtime (ADR-0011). Follow these patterns for async code.

**When to write async functions**:
- Functions that perform I/O (network, disk)
- Functions that need to be cancellable or time-bounded
- Library functions that might be called from async contexts

**Async function signatures**:

```rust
// Good: Async function with clear types
pub async fn fetch_data(url: &str) -> Result<Data> {
    // Network I/O here
    Ok(Data { })
}

// Good: Returning a Future (for complex scenarios)
pub fn fetch_data_lazy(url: &str) -> impl std::future::Future<Output = Result<Data>> {
    async move {
        // Network I/O here
        Ok(Data { })
    }
}

// Avoid: Blocking I/O in async code
pub async fn load_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)?  // ❌ Blocks the executor!
}

// Better: Use tokio::fs for async I/O
pub async fn load_file(path: &Path) -> Result<String> {
    tokio::fs::read_to_string(path).await
}
```

**Guidelines**:
- Use `.await` at the appropriate boundary; don't hold locks across `.await` points
  ```rust
  // Bad: Lock held across await
  let guard = data.lock();
  let result = async_operation().await;
  drop(guard);

  // Good: Lock released before await
  let result = {
      let guard = data.lock();
      guard.do_something()
  };
  async_operation().await;
  ```

- Avoid blocking operations in async code; use tokio equivalents
- Spawned tasks should be awaited or explicitly detached
  ```rust
  // Good: Task is awaited
  let handle = tokio::spawn(async { /* work */ });
  handle.await?;

  // Good: Task is detached (background work is OK to ignore)
  tokio::spawn(async { /* fire-and-forget logging */ });
  ```

**Blocking API Bridge**:

For synchronous APIs that need async implementations, see [Design 0005](../design/0005-synchronous-and-blocking-api-execution-bridge.md). The project provides blocking wrappers that bridge sync and async contexts without exposing complexity to users.

## Feature-Gated Code

Organise feature-gated code to minimise complexity:

```rust
// Good: Feature guard at module level
#[cfg(feature = "async")]
pub mod async_client;

#[cfg(feature = "streaming")]
pub use streaming::parse_stream;

// Avoid: Scattered cfg guards in function bodies
pub fn process(data: &[u8]) -> Result<Model> {
    #[cfg(feature = "streaming")]
    return parse_stream(data);
    #[cfg(not(feature = "streaming"))]
    return parse_all(data);
}
```

**Document features** in the crate's README and feature descriptions in `Cargo.toml`:

```toml
[features]
default = ["std"]
std = []
async = ["tokio"]
streaming = []
```

## Deprecation

Mark deprecated items with the `#[deprecated]` attribute and provide a migration path:

```rust
#[deprecated(
    since = "0.2.0",
    note = "use `ConstraintModel::parse()` instead"
)]
pub fn parse_constraint(input: &str) -> Result<ConstraintModel> {
    ConstraintModel::parse(input)
}
```

For deprecation timeline and breaking change policy, see [CONTRIBUTING.md](../../CONTRIBUTING.md).

## Copy vs Clone

**Copy** and **Clone** are distinct concepts with implications for API design and performance.

**Copy**: Implicit duplication via bitwise copy (fast, zero overhead). Only small types should implement Copy.

```rust
// Good: Small, primitive-like types
#[derive(Copy, Clone)]
pub struct ModelId(u64);

#[derive(Copy, Clone)]
pub enum Constraint { Include, Exclude }

// Avoid: Large or heap-allocated types
#[derive(Copy, Clone)]  // Bad! Clones entire Vec
pub struct Rules(Vec<Rule>);
```

**Clone**: Explicit duplication (can be expensive). Implement when copying is necessary but not implicit.

```rust
// Good: Expensive operation is explicit
#[derive(Clone)]
pub struct ConstraintModel {
    rules: Vec<Rule>,
    metadata: HashMap<String, String>,
}

let model2 = model1.clone();  // Obvious that copying happens
```

**Trade-off**: Copy is convenient (no `.clone()` calls) but signals to users "this is cheap to copy." Don't implement Copy on types where copying is expensive (Vec, HashMap, String, I/O handles).

**Guidelines**:
- Implement Copy only if the type is small (typically ≤ 2 pointer widths) and copying is semantically correct
- Types containing heap data (Vec, String, HashMap) should not be Copy
- Newtype wrappers around Copy types are usually Copy (`struct Id(u64)`)
- If you implement Copy, always implement Clone (required by the trait)

## Visibility Patterns

**Default to private.** Mark as `pub` only when the type/function is part of the public API contract. See [RFC 430 — Sealed traits](https://rust-lang.github.io/api-guidelines/type-safety.html#sealed-traits-protect-against-downstream-implementations-c-sealed) and [visibility design](https://rust-lang.github.io/api-guidelines/flexibility.html).

**`pub` vs `pub(crate)` strategy**:

```rust
// Public API: Other crates depend on this
pub struct ConstraintModel { }
pub fn parse(input: &str) -> Result<ConstraintModel> { }

// Crate-internal: Other modules need this, but it's not part of the public API
pub(crate) fn validate_syntax(input: &str) -> Result<()> { }

// Module-internal: Only visible to this module
fn helper() { }
```

**Re-export pattern** for facade crates and logical grouping:

```rust
// src/lib.rs in sdmx-rs (facade crate)
pub use sdmx_types::{ConstraintModel, Dataflow};
pub use sdmx_parsers::parse;
pub use sdmx_client::Client;

// Hides implementation crate names; users do:
// use sdmx_rs::ConstraintModel;  ✅
// NOT: use sdmx_types::ConstraintModel;
```

**Public module structure**:

```rust
// src/lib.rs
pub mod constraint;      // Public module; users can see it
pub(crate) mod internal; // Crate-internal; not documented

// src/constraint/mod.rs
pub use model::ConstraintModel;
pub use validation::validate;
// Other exports: internal impl details remain private
```

**Decision framework**:
- Expose only types and functions users will directly call
- Hide implementation modules (like `internal`, `utils`)
- Use `pub use` in `mod.rs` to shape the public API
- Re-export from the facade crate to create a unified API surface

## no_std Compatibility

The project targets `no_std` with `alloc` (ADR-0005). Code must compile in both `std` and `no_std` contexts.

**Key constraints**:
- No `std` imports; use `core` and `alloc` instead
  ```rust
  // Good
  use core::fmt;
  use alloc::string::String;
  use alloc::vec::Vec;

  // Avoid
  use std::fmt;  // Fails in no_std
  ```

- Prefer `alloc` types when heap allocation is needed
  ```rust
  pub fn parse(input: &[u8]) -> Result<alloc::vec::Vec<Item>> { }
  ```

- Avoid `panic!` and `unwrap()` in library code; they require `std` panic handler in no_std
  ```rust
  // Better: return Result
  pub fn new(value: u32) -> Result<NonZeroU32> { }
  ```

- String handling: use `&str` where possible; `String` requires allocation
  ```rust
  pub fn parse_name(input: &str) -> Result<()> { }  // ✅ No allocation

  pub fn to_name_string(&self) -> String { }  // ⚠️ Requires alloc
  ```

**Testing**: Code must compile with and without `std`. CI validates `no_std` targets. See crate's `lib.rs` for feature-gating `std` dependencies.

## WASM Considerations

The project targets WebAssembly (WASM) compilation (ADR-0007). Code targeting WASM should follow these guidelines:

**Compilation targets**:
- `wasm32-unknown-unknown` — Headless WASM (no JS imports)
- No async at present; blocking I/O only

**Constraints**:
- No threads; single-threaded execution
- Minimal memory overhead; allocations are visible to JS
- No `std::env`, `std::fs`, or other I/O; these don't exist in WASM

**Patterns**:
```rust
// Good: Pure computation; works in WASM
pub fn validate(model: &ConstraintModel) -> Result<()> { }

// Avoid: System I/O; doesn't work in WASM
pub fn load_from_file(path: &Path) -> Result<ConstraintModel> { }
```

**Testing**: CI validates WASM compilation for `sdmx-types`, `sdmx-parsers`, `sdmx-writers`, and `sdmx-rs` with `--target wasm32-unknown-unknown`.
