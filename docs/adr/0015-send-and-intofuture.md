# 15. `.send()` as Primary Execution Method with `IntoFuture` as Secondary

Date: 2026-05-20

## Status

Accepted

---

## Context

Once a Typestate query builder reaches its fully-specified state, the caller
must be able to execute the query. There are two idiomatic mechanisms in the
Rust async ecosystem for expressing this:

1. An explicit `.send()` method that returns a `Future`
2. Implementing `std::future::IntoFuture`, which allows the builder itself
   to be `.await`ed directly

These are not mutually exclusive, but one must be designated the primary
documented path. This choice affects call-site ergonomics, compiler error
legibility, ecosystem familiarity, and the extensibility of the API.

`IntoFuture` was stabilised in Rust 1.64. It is a legitimate, stable feature
and its use in builder APIs is a recognized pattern. The decision here is
therefore not whether to use it, but what role it plays relative to `.send()`.

**Note on `Send` Guarantee**: Builders store `Cow<'static, str>` for string
fields (see [Design 0006](../design/0006-builder-field-storage.md)), ensuring all data is
`'static` by construction. This guarantees that builders and their returned
futures are unconditionally `Send + 'static`, allowing them to be freely spawned
onto background task pools via `tokio::spawn()` or similar mechanisms without
any lifetime constraints or conversion steps at the execution boundary.

---

## Decision Drivers

- **Ecosystem familiarity.** Rust async HTTP libraries (`reqwest`, `surf`,
  `hyper`) universally use `.send()` as their execution boundary. Contributors
  arriving from these libraries should find the convention familiar.
- **Execution boundary legibility.** In an exemplar codebase, the point where
  query construction ends and I/O begins should be unambiguous.
- **Compiler error quality.** Errors involving a missing `.send()` call are
  more readable than errors involving `IntoFuture` trait bounds.
- **API extensibility.** The execution path must not foreclose future options
  such as blocking execution or dry-run modes.
- **Exemplar value.** `IntoFuture` is a real and useful Rust feature that
  the library should demonstrate, even if it is not the primary path.

---

## Options Considered

### Option A — `.send()` Only

```rust
impl StructureQueryBuilder<HasAgency, HasResourceType> {
    pub async fn send(self) -> Result<StructureMessage, sdmx_client::Error> {
        // ...
    }
}
```

Call site:

```rust
let result = client.structure()
    .agency("BIS")
    .resource_type("dataflow")
    .send()
    .await?;
```

**Pros**:

- Single, unambiguous execution path.
- Matches `reqwest`, `surf`, and `hyper` conventions exactly.
- Compiler errors are straightforward: `no method named send found` is
  self-explanatory to any Rust developer.
- Simplest to document and test.

**Cons**:

- Does not demonstrate `IntoFuture`, a stable and genuinely useful Rust
  feature that is directly applicable to builder APIs.
- One additional method call at every call site compared to direct `.await`.

**Verdict**: Correct and safe, but leaves a demonstrable language feature
on the table. For an internal tool this is fine; for an exemplar library
it is a missed opportunity.

---

### Option B — `IntoFuture` Only

```rust
impl IntoFuture for StructureQueryBuilder<HasAgency, HasResourceType> {
    type Output     = Result<StructureMessage, sdmx_client::Error>;
    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // execution logic inline
        })
    }
}
```

Call site:

```rust
let result = client.structure()
    .agency("BIS")
    .resource_type("dataflow")
    .await?;
```

**Pros**:

- Cleanest possible call site. The query reads as a pure expression.
- Fully demonstrates `IntoFuture` as the idiomatic mechanism.

**Cons**:

- The builder *becomes* a future on `.await`. Storing the builder in a
  variable and later awaiting it is valid but reads oddly:

  ```rust
  let query = client.structure().agency("BIS").resource_type("dataflow");
  // query is a StructureQueryBuilder, not a Future
  // IntoFuture::into_future() is called implicitly by .await
  let result = query.await?; // correct but surprising
  ```

- Compiler errors involving `IntoFuture` bounds are currently less readable
  than those involving a plain method:

  ```
  // With .send() only:
  error[E0599]: no method named `send` found for struct `StructureQueryBuilder<...>`

  // With IntoFuture only:
  error[E0277]: `StructureQueryBuilder<HasAgency, NoResourceType>` cannot be converted
  to a future
  ```

  The second form is accurate but requires the reader to understand the
  `IntoFuture` mechanism to act on it.

- Forecloses a future `.send_blocking()` path without API redesign, since
  the "execute this builder" concept is now exclusively tied to `IntoFuture`.
- Documentation examples using `.await` directly on the builder will appear
  to execute without an explicit call — the implicit `into_future()` is
  invisible to readers unfamiliar with the trait.

**Verdict**: Ergonomically appealing but makes the execution boundary
implicit, reduces compiler error legibility, and limits future extensibility.
Wrong as the sole path.

---

### Option C — `.send()` Primary, `IntoFuture` Delegates to It

```rust
impl StructureQueryBuilder<HasAgency, HasResourceType> {
    /// Execute the query. This is the primary execution method.
    pub async fn send(self) -> Result<StructureMessage, sdmx_client::Error> {
        let url = format!(/* ... */);
        self.client.execute_get(url).await
    }
}

impl IntoFuture for StructureQueryBuilder<HasAgency, HasResourceType> {
    type Output     = Result<StructureMessage, sdmx_client::Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'static>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}
```

Both call sites work and produce identical results:

```rust
// Primary: explicit, used in all documentation examples
let result = client.structure()
    .agency("BIS")
    .resource_type("dataflow")
    .send()
    .await?;

// Secondary: valid, IntoFuture delegates to .send()
let result = client.structure()
    .agency("BIS")
    .resource_type("dataflow")
    .await?;
```

**Pros**:

- `.send()` provides the familiar, legible, ecosystem-consistent execution
  boundary.
- `IntoFuture` is demonstrated as a real, working feature — not suppressed.
- All execution logic lives in `.send()`; `IntoFuture` is a one-line
  delegation with no duplicated implementation.
- Future addition of `.send_blocking()` or `.send_with_options()` is
  straightforward — `.send()` remains the canonical execution path.
- Compiler errors on incomplete builders reference `.send()`, which is
  more legible than `IntoFuture` bound errors.
- Documentation examples use `.send().await` consistently; the `IntoFuture`
  path is documented in `ARCHITECTURE.md` as an available but non-primary
  alternative.

**Cons**:

- Slightly more implementation surface than either option alone. Mitigated
  by the one-line delegation — there is no meaningful maintenance cost.
- A contributor encountering `IntoFuture` in the source without reading
  `ARCHITECTURE.md` may not understand why it exists. Mitigated by the
  doc comment on the `impl` block.

**Verdict**: The correct choice. Delivers ecosystem familiarity and compiler
error legibility via `.send()`, while demonstrating `IntoFuture` as a
first-class feature. No logic is duplicated.

---

## Decision

**`.send()` is the primary execution method on all fully-specified query
builders. `IntoFuture` is implemented on all fully-specified builders,
delegating to `.send()` with no duplicated logic.**

All documentation examples in this codebase use `.send().await`. The
`IntoFuture` path is documented in `ARCHITECTURE.md` Section 4 and is
available to callers who prefer it.

---

## Implementation

### Direct Use of `Cow<'static, str>` Fields at Execution

When `.send()` is invoked, all `Cow<'static, str>` fields can be used directly
in string formatting and network calls without conversion. The `Cow` type
automatically deref-coerces to `&str`, and the lifetime is guaranteed to be
`'static` by construction:

```rust
impl StructureQueryBuilder<HasAgency, HasResourceType> {
    pub async fn send(self) -> Result<StructureMessage, sdmx_client::Error> {
        // Cow<'static, str> fields can be used directly in format strings.
        // No conversion needed; all data is already owned or 'static-borrowed.
        let url = format!(
            "{}/structure/{}/{}/{}/{}",
            self.client.base_url,
            &self.resource_type.0,
            &self.agency.0,
            self.resource_id.as_deref().unwrap_or("all"),
            self.version.as_deref().unwrap_or("latest"),
        );
        self.client.execute_get(url).await
    }
}
```

**Rationale**: All string fields are `Cow<'static, str>`, guaranteeing that
every value is either a `&'static str` (zero-cost borrowing) or an owned
`String` (allocation happened at the call site). There is no borrowed-and-local
data to defer or convert at execution time. This simplifies both the execution
path and the reasoning about when allocations occur.

### Unconditional `'static` Futures

The futures returned by both `.send()` and `IntoFuture` are unconditionally
`'static`:

```rust
impl IntoFuture for StructureQueryBuilder<HasAgency, HasResourceType> {
    type Output     = Result<StructureMessage, sdmx_client::Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'static>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}
```

These futures can be freely spawned onto background task pools:

```rust
tokio::spawn(client.structure().agency("BIS").resource_type("dataflow").send())
```

There are no lifetime constraints, and all data the future needs is owned or
`'static`-borrowed.

---

## Consequences

- Every fully-specified builder in this library implements both `.send()`
  and `IntoFuture`. This is a library-wide convention.
- All execution logic resides in `.send()`. `IntoFuture::into_future()`
  always delegates to `self.send()` and contains no other logic.
- **Unconditional `'static` Futures**: Both `.send()` and `IntoFuture` return
  `'static` futures without exception. This is guaranteed by the use of
  `Cow<'static, str>` in all string fields (see [Design 0006](../design/0006-builder-field-storage.md)).
  Builders can be freely spawned onto background task pools via `tokio::spawn()`.
- **Simplified Execution**: The `.send()` method does not need to call
  `.into_owned()` or perform any field conversions. All data is already
  appropriately owned or `'static`-borrowed by construction.
- `ARCHITECTURE.md` Section 4 documents both paths, explains the rationale
  for `.send()` as primary, and notes that both resolve identically.
- If a blocking execution path is added in future (`send_blocking()`), it
  is added as a method on the fully-specified builder, consistent with
  this pattern. It does not affect `IntoFuture`.
- A boxed future type (`Pin<Box<dyn Future<Output = ...> + Send + 'static>>`)
  is used as the associated `IntoFuture` type. This accommodates the stable
  Rust constraint where anonymous, internal futures returned by HTTP client
  libraries like `reqwest` cannot be named inside structs without unstable
  compiler features. The overhead of a single heap allocation (`Box::pin`)
  is negligible relative to the overall network I/O, DNS resolution, TLS
  handshake, and stream parsing execution costs.

---

## References

- `ARCHITECTURE.md` — Section 4 (Execution Model)
- [Design Document 0007 — Compile-Time Query Validation via the Typestate Pattern](../design/0007-typestate-query-validation.md) (establishes which builders have a fully-specified state that exposes execution)
- [`std::future::IntoFuture`](https://doc.rust-lang.org/std/future/trait.IntoFuture.html): Stabilised in Rust 1.64
- `reqwest` — `.send()` convention reference
