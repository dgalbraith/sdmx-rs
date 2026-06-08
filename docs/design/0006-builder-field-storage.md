# 6. Builder Field Storage Strategy for Typestate Query Builders

Date: 2026-05-20

## Status

Accepted

<!-- Valid statuses: Proposed, Accepted, Implemented, Superseded -->

---

## Summary

Optimize string field storage in typestate query builders by using `Cow<'static, str>`, eliminating lifetime parameters from the builders and returned futures to enable background task spawning via `tokio::spawn` while maintaining zero-cost allocation for string literals.

---

## Problem / Motivation

The Typestate query builders (`StructureQueryBuilder`, `DataQueryBuilder`) carry string-valued fields that are set by the caller and consumed when the query is executed. Specifically:

- `StructureQueryBuilder` carries `agency_id`, `resource_type`, and optionally `version`
- `DataQueryBuilder` carries `flow_ref`, and optionally `key` and `provider`

These fields are set once during builder construction, never mutated, and consumed in a single `format!()` call at execution time. The builder itself carries a lifetime `'a` purely for the string values stored inside the `Cow<'a, str>` fields, while owning the `SdmxClient` instance cheaply via a clone to remain `'static` when only static string inputs are used.

The question is: what type should be used to store these string fields?

This decision is architecturally visible. The storage type affects:
- The lifetimes on the builder struct and its impl blocks
- The ergonomics of the call site for callers holding owned strings
- The honesty of the documented allocation claims
- What the library teaches as idiomatic Rust

All three options were evaluated against the exemplar objective: the library should demonstrate genuinely idiomatic Rust, not simplified Rust.

In practice, SDMX agency identifiers and resource names are overwhelmingly static string literals in application code (`"BIS"`, `"ECB"`, `"dataflow"`, `"codelist"`). Runtime-constructed strings (from config files, CLI arguments, or user input) represent a secondary but legitimate use case.

### Decision Drivers

- **Correctness of documented claims.** The architecture documentation must not assert zero-cost allocation if the implementation allocates unconditionally.
- **Call-site ergonomics.** The dominant caller pattern (string literals) must be friction-free. Owned string callers must not be forced to restructure their code around borrow scopes.
- **Exemplar value.** The implementation should demonstrate idiomatic library API design, including appropriate handling of the owned-vs-borrowed string tension that appears in most real Rust libraries.
- **Lifetime legibility.** The builder already carries `'a` for the client reference. Additional lifetimes must not obscure the Typestate pattern, which is the primary architectural teaching point.
- **Stability.** All chosen types must be stable Rust. No nightly features.

---

## Proposed Design

**`Cow<'static, str>` for all mandatory and optional string fields in Typestate query builders.**

Builders no longer carry a lifetime parameter `'a`. All builder types are unconditionally `'static`:

```rust
use std::borrow::Cow;

pub struct NoAgency;
pub struct HasAgency(Cow<'static, str>);
pub struct NoResourceType;
pub struct HasResourceType(Cow<'static, str>);

// No lifetime parameters on the builder:
pub struct StructureQueryBuilder<A, RT> {
    client:        SdmxClient,
    agency:        A,
    resource_type: RT,
    resource_id:   Option<Cow<'static, str>>,
    version:       Option<Cow<'static, str>>,
}

impl<A, RT> StructureQueryBuilder<A, RT> {
    pub fn resource_id(mut self, id: impl Into<Cow<'static, str>>) -> Self {
        self.resource_id = Some(id.into());
        self
    }
}

impl StructureQueryBuilder<NoAgency, NoResourceType> {
    pub fn agency(
        self,
        id: impl Into<Cow<'static, str>>,
    ) -> StructureQueryBuilder<HasAgency, NoResourceType> {
        StructureQueryBuilder {
            client: self.client,
            agency: HasAgency(id.into()),
            resource_type: self.resource_type,
            resource_id: self.resource_id,
            version: self.version,
        }
    }
}
```

The `impl` blocks accept `impl Into<Cow<'static, str>>` to allow callers to pass `&'static str`, `String`, or `Cow<'static, str>` directly. Callers with non-`'static` borrowed `&str` must explicitly convert them to `String` (or use string methods like `.to_string()` or `.to_owned()`) at the call site.

This ensures the builder, and all futures it produces, are unconditionally `'static` and can be freely spawned onto background task pools.

### Documentation Layering Strategy

To prevent `Cow` from obscuring the Typestate pattern in `ARCHITECTURE.md`, documentation is structured in two layers:

1. **Primary section (Typestate)**: State machine diagrams and transition logic use simplified type names. The focus is on valid state transitions, not field storage.
2. **Implementation note (Storage)**: A dedicated subsection and this Design Document explain the `Cow` choice, its rationale, and its behavior at the call site.

This ensures a reader learning the Typestate pattern is not simultaneously required to understand `Cow`, while a reader implementing or extending the builders has the full picture.

---

## Alternatives Considered

### Option A — `String` (Owned)

```rust
pub struct HasAgency(String);
pub struct HasResourceType(String);
```

The builder owns its string fields unconditionally.

**Pros**:
- Simplest implementation; the Typestate pattern is the sole thing demanding attention in the code and documentation.
- No lifetime proliferation; the existing `'a` on the client reference is unambiguous.
- Call sites are identical in appearance to the other options for the common literal case (`agency("BIS")` — `&str` coerces and `.into()` is called).

**Cons**:
- Allocates unconditionally on every field set, regardless of whether the input is a literal or an owned string.
- The documented claim of "zero-cost / stack-allocated" field storage is false and must be either dropped or significantly qualified.
- Slightly below idiomatic for a public-facing library API: a Rust library that accepts only `String` (or implicitly converts everything to `String`) is accepting unnecessary allocations from callers who already have `&str`.
- For an exemplar, this teaches a shortcut rather than the right pattern.

**Verdict**: Acceptable for an internal tool. Wrong for a library with unknown usage.

### Option B — `&'a str` (Borrowed)

```rust
pub struct HasAgency<'s>(&'s str);
pub struct HasResourceType<'s>(&'s str);

pub struct StructureQueryBuilder<'a, 's, A, RT> {
    client:        &'a SdmxClient,
    agency:        A,    // A = HasAgency<'s> when bound
    resource_type: RT,
}
```

Or collapsing to a single lifetime where string inputs can be assumed to outlive the client borrow:

```rust
pub struct StructureQueryBuilder<'a, A, RT> {
    client:        &'a SdmxClient,
    agency:        A,    // A = HasAgency<'a> when bound
    resource_type: RT,
}
```

**Pros**:
- Genuinely zero-cost for all inputs: no allocation occurs at any point during builder construction.
- Validates the documented claim without qualification.
- Demonstrates lifetime threading through phantom types — a legitimate and advanced Rust teaching moment.

**Cons**:
- Callers with owned strings must borrow explicitly and manage the borrow scope:
  ```rust
  let agency_id: String = config.agency_id();
  // agency_id must outlive the builder
  let result = client.structure()
      .agency(&agency_id)   // explicit borrow required
      .resource_type("dataflow")
      .send()
      .await?;
  ```
  This is correct Rust but is ergonomically surprising in a fluent builder API and will cause friction for the common case of runtime-sourced identifiers.
- Two distinct lifetime concerns (`'a` for the client, `'s` for string inputs) compete for mental bandwidth in the same struct definition and impl blocks. This risks making the Typestate pattern — the primary architectural teaching point — harder to follow for readers who are not already fluent in lifetime parameterisation.
- If the string lifetime is collapsed into `'a` (simpler), callers must ensure their strings live as long as the client reference, which is a non-obvious constraint to document and enforce.

**Verdict**: Technically correct but imposes lifetime complexity that obscures the primary lesson and creates real call-site friction. Not the right choice for a library targeting broad real-world use.

### Option C — `Cow<'a, str>`

```rust
use std::borrow::Cow;

pub struct HasAgency<'a>(Cow<'a, str>);
pub struct HasResourceType<'a>(Cow<'a, str>);
```

`Cow<'a, str>` is an enum over `&'a str` (borrowed) and `String` (owned). It borrows when it can and owns when it must, with the distinction resolved at the call site rather than imposed on the caller.

**Pros**:
- **Zero-cost for the dominant case.** String literals — the overwhelming majority of SDMX agency and resource identifiers in practice — are `&'static str` and are stored as the `Borrowed` variant with no allocation.
- **Ergonomic for owned strings.** Callers with `String` values pass them directly; ownership is moved into the `Owned` variant without requiring the caller to hold a borrow scope:
  ```rust
  // Literals: zero allocation
  client.structure().agency("BIS").resource_type("dataflow").send().await?;

  // Owned strings: moved in, no copy, no borrow scope management
  let agency_id: String = config.agency_id();
  client.structure().agency(agency_id).resource_type("dataflow").send().await?;
  ```
- **Documented claim is honest.** Zero-cost for literals; a single allocation for owned strings. Both are accurately documentable without qualification.
- **Demonstrates an important library pattern.** `Cow` for flexible ownership is idiomatic in Rust library APIs that accept string-like inputs. Using it here teaches the pattern in a realistic context.
- **Single lifetime on the builder.** The `'a` on `Cow<'a, str>` is the only lifetime parameter on the builder, as the builder owns the `SdmxClient` instance cheaply (which internally relies on reqwest's Arc-based connection pool) instead of borrowing it.
- **Stable.** `std::borrow::Cow` is stable Rust and has been since 1.0.

**Cons**:
- `Cow` is unfamiliar to intermediate Rust developers. Readers encountering it for the first time may fixate on it rather than the Typestate pattern. This is mitigated by the documentation layering strategy described below.
- Three patterns are now present simultaneously in the builder code: Typestate, lifetimes, and `Cow`. This is manageable with clear documentation but requires deliberate layering.
- A small runtime branch exists on `Cow` deref (checking the variant), though this is entirely negligible and occurs only at the `format!()` call site, not during builder construction.

**Verdict**: Introduces lifetime contamination in async contexts.

---

## Drawbacks / Trade-offs

- Builder type signatures contain NO lifetime parameters. Types like `StructureQueryBuilder<A, RT>` are used instead of `StructureQueryBuilder<'a, A, RT>`.
- Zero-cost for `&'static str` literals (no allocation or conversion)
- Single allocation for `String` and non-`'static` `&str` (conversion to owned at call site or via `.into()`)
- All input data is guaranteed to be `'static` by construction

---

## Questions & Resolutions

None.

---

## References

- [std::borrow::Cow documentation](https://doc.rust-lang.org/std/borrow/enum.Cow.html)
- [ARCHITECTURE.md](../../ARCHITECTURE.md) — API Design & Ergonomics, Section 1 (Allocation Policy)
- [ARCHITECTURE.md](../../ARCHITECTURE.md) — API Design & Ergonomics, Section 2 (Structure Builder Blueprint)
- [ARCHITECTURE.md](../../ARCHITECTURE.md) — API Design & Ergonomics, Section 3 (Data Builder Blueprint)
- Related: `crates/sdmx-client/src/query/structure.rs` (Phase 3), `crates/sdmx-client/src/query/data.rs` (Phase 3)
