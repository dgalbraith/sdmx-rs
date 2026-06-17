# 7. Compile-Time Query Validation via the Typestate Pattern

Date: 2026-05-20

## Status

Accepted

<!-- Valid statuses: Proposed, Accepted, Implemented, Superseded -->

---

## Summary

Enforce mandatory query parameters at compile time using the typestate pattern with phantom type parameters, ensuring only structurally valid SDMX REST queries expose the execution bridge methods and shifting interface errors from runtime responses to compile-time checks.

---

## Problem / Motivation

The SDMX REST API enforces strict structural constraints on query URLs. A structure query missing a mandatory field such as `agencyId` or `resource_type` will always produce a 400 or 404 response from the server — there is no partial success. These constraints are fully knowable at the time a query is constructed, not at the time it is executed.

The library must decide where to enforce these constraints:
- At the **call site**, preventing invalid queries from being constructed
- At **execution time**, detecting and returning errors from `.send()`
- At **compile time**, making invalid queries unrepresentable as a type

This is the foundational architectural decision for the query builder API. Every other builder design decision in this codebase follows from it.

The SDMX REST mandatory field matrix is:

| Query Type | Mandatory Fields            | Optional Fields         |
|------------|-----------------------------|-------------------------|
| Structure  | `resource_type`, `agencyId` | `resourceId`, `version` |
| Data       | `flowRef`                   | `key`, `providerRef`    |

### Decision Drivers

- **Correctness.** An invalid SDMX query has no valid outcome. The library should not allow one to be constructed.
- **Error locality.** The closer an error is caught to its source, the cheaper it is to fix. A compile error is cheaper than a runtime error; a runtime error before network I/O is cheaper than a server 400 response.
- **Exemplar value.** The library should demonstrate what Rust's type system can do in a domain where constraints are statically knowable.
- **Contributor experience.** The enforcement mechanism must be legible to contributors who did not write it.

---

## Proposed Design

**The Typestate pattern is used for all mandatory field enforcement in query builders.**

Optional fields (those that have a protocol-level default) are stored as `Option` values on the builder struct and are available at any typestate. They never gate execution and do not participate in the typestate graph.

The boundary is explicit:
- **Mandatory field** → typestate type parameter
- **Optional field** → `Option<T>` on the struct, method available on `impl<A, R>` (any state)

### Typestate Pattern Structure

Phantom type parameters encode which mandatory fields have been set. Each method that sets a mandatory field returns a new type rather than `Self`. Only the fully-specified type exposes `.send()`.

```rust
// Incomplete builders — .send() does not exist on these types
let b1 = client.structure();                  // StructureQueryBuilder<NoAgency, NoResourceType>
let b2 = client.structure().agency("BIS");    // StructureQueryBuilder<HasAgency, NoResourceType>
let b3 = client.structure().resource_type("dataflow"); // StructureQueryBuilder<NoAgency, HasResourceType>

// Complete builder — .send() exists only here
let b4 = client.structure()
    .agency("BIS")
    .resource_type("dataflow");                    // StructureQueryBuilder<HasAgency, HasResourceType>
```

Invalid queries do not compile:
```
error[E0599]: no method named `send` found for struct
`StructureQueryBuilder<HasAgency, NoResourceType>`
```

---

## Alternatives Considered

### Option A — Runtime Validation in `.send()`

```rust
pub struct StructureQueryBuilder<'a> {
    client:        &'a SdmxClient,
    agency:        Option<String>,
    resource_type: Option<String>,
    version:       Option<String>,
}

impl<'a> StructureQueryBuilder<'a> {
    pub async fn send(self) -> Result<StructureMessage, sdmx_client::Error> {
        let agency = self.agency
            .ok_or(sdmx_client::Error::MissingField("agencyId"))?;
        let resource_type = self.resource_type
            .ok_or(sdmx_client::Error::MissingField("resource_type"))?;
        // ... build URL and execute
    }
}
```

**Pros**:
- Familiar pattern. Developers coming from any language recognise `Option`-checked validation in an execution method.
- Simple struct and impl definitions; no phantom types or lifetime proliferation beyond the client reference.
- Easy to add new mandatory fields without changing the type signature.

**Cons**:
- Invalid queries compile and run. The error is only discovered when `.send()` is awaited — potentially far from where the incomplete builder was constructed.
- `Result` is returned for errors that are entirely the caller's fault and entirely avoidable. This conflates structural misuse with genuine runtime failures (network errors, server errors, parse errors).
- In async code, the await point may be separated from the builder construction by many lines or even function boundaries, making the error context poor.
- Forces callers to handle an error that should not exist.
- Does not demonstrate what Rust's type system is capable of. For an exemplar library, this is a significant omission.

**Verdict**: Correct but unsophisticated. Appropriate for a quick internal tool; wrong for an exemplar library whose purpose includes demonstrating idiomatic Rust.

### Option B — Runtime Panic in `.send()`

```rust
impl<'a> StructureQueryBuilder<'a> {
    pub async fn send(self) -> Result<StructureMessage, sdmx_client::Error> {
        let agency = self.agency
            .expect("agencyId is required; call .agency() before .send()");
        let resource_type = self.resource_type
            .expect("resource_type is required; call .resource_type() before .send()");
        // ...
    }
}
```

**Pros**:
- Simpler than Option A — no error variant for structural misuse.
- Panic message can be descriptive.

**Cons**:
- Panics in library code are poor practice. Libraries should not make decisions about whether a program should terminate.
- Still deferred to runtime; the error only appears when the code path is executed, which may not be during testing.
- Provides no compile-time signal whatsoever.
- Actively hostile to library consumers who cannot catch panics cleanly in async contexts.

**Verdict**: Worse than Option A. Rejected without further consideration.

---

## Drawbacks / Trade-offs

- All query builders in this library follow the Typestate pattern for mandatory field enforcement. This is a library-wide convention, not a per-builder choice.
- `CONTRIBUTING.md` must document the pattern and explain how to read incomplete-builder compiler errors.
- `ARCHITECTURE.md` carries the state machine diagrams and implementation blueprints as the primary reference for contributors extending the builders.
- Future query types (e.g., `MetadataQueryBuilder`, `SchemaQueryBuilder`) must identify their mandatory fields and model them as typestate parameters. See ADR-0016 for guidance on type parameter count.
- Runtime validation in `.send()` is reserved for errors that cannot be known at compile time: network failures, server errors, malformed response bodies.

---

## Questions & Resolutions

None.

---

## References

- [ARCHITECTURE.md](../../ARCHITECTURE.md) — Section 2 (Structure Query Builder)
- [ARCHITECTURE.md](../../ARCHITECTURE.md) — Section 3 (Data Query Builder)
- [ARCHITECTURE.md](../../ARCHITECTURE.md) — Section 5 (Type-Level Error Diagnostics)
- [Design Document 0006 — Builder Field Storage](0006-builder-field-storage.md)
- [ADR-0016 — Type Parameter Count Policy](../adr/0016-type-parameter-count.md)
- [CONTRIBUTING.md](../../CONTRIBUTING.md) — Reading Typestate Compiler Errors
