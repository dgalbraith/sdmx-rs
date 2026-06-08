# 3. Typestate Query Builders Design

Date: 2026-05-24

## Status

Proposed

<!-- Valid statuses: Proposed, Accepted, Implemented, Superseded -->

---

## Summary

Design query builder APIs for SDMX REST endpoints (structure and data queries) using the **typestate pattern** to enforce compile-time validation of mandatory fields. This design ensures invalid queries cannot be constructed, shifting validation from runtime (server errors) to compile-time (type errors), eliminating entire classes of bugs and improving ergonomics for library users.

---

## Problem / Motivation

SDMX REST endpoints require specific mandatory fields (e.g., agency ID for structure queries, flow ID for data queries). If a developer forgets to set these fields, they'll only discover the error at runtime when the server rejects the request.

**Pain points**:
- Runtime errors are discovered late (after code is written, tested, deployed)
- Error messages are opaque (server error, not developer-friendly)
- Testing coverage must include error paths (increases test burden)

**Goals**:
- Make it **impossible to construct invalid queries** at compile-time
- Shift validation from runtime to type-checking
- Provide clear, self-describing compiler errors that guide developers toward valid states
- Maintain ergonomic API (builder pattern, flexible field order)

**Constraints**:
- Must support flexible field-setting order (both `.agency().resource_type()` and `.resource_type().agency()` are valid)
- Must work with `#[derive]` where possible (minimize boilerplate)
- Error messages should be understandable to Rust developers (not cryptic type parameter noise)

---

## Proposed Design

Use **typestate pattern**: Represent builder state as generic type parameters. Each parameter tracks whether a mandatory field has been set (`HasAgency` vs `NoAgency`). The `.send()` method only exists when all parameters are in `Has*` state, preventing invalid queries at compile time.

### Architecture / Key Decisions

#### 1. Marker Types for State

Each mandatory field has marker types:
- `NoAgency` → agency not yet set
- `HasAgency` → agency is set

Builder is generic: `StructureQueryBuilder<AgencyState, ResourceTypeState, ...>`

#### 2. State Transitions via Builder Methods

Each builder method (`.agency()`, `.resource_type()`) transforms the type:
```rust
impl<R, F> StructureQueryBuilder<NoAgency, R, F> {
    fn agency(self, id: impl Into<String>) -> StructureQueryBuilder<HasAgency, R, F> {
        // consume self, set agency, return new builder state
    }
}
```

#### 3. Send Only When Complete

The `.send()` method only exists when all mandatory fields are `Has*`:
```rust
impl StructureQueryBuilder<HasAgency, HasResourceType, HasVersion> {
    async fn send(self) -> Result<...> { /* send to server */ }
}
```

Attempting to call `.send()` on incomplete state produces a compile error:
```
error[E0599]: no method named `send` found for struct
    `StructureQueryBuilder<HasAgency, NoResourceType, NoVersion>`
```

#### 4. Type Parameter Ordering

Each type parameter corresponds to one mandatory field, in a consistent order. See [ADR-0016](../adr/0016-type-parameter-count.md) for the naming convention.

### Examples / Pseudo-code

**Incomplete builder (compile error)**:
```rust
let client = SdmxClient::new("https://registry.sdmx.org/rest/v1")?;
let result = client
    .structure()
    .agency("ECB")
    .send()  // ❌ ERROR: Missing resource_type!
    .await?;
```

Compiler output:
```
error[E0599]: no method named `send` found for struct
    `StructureQueryBuilder<HasAgency, NoResourceType>` in the current scope
```

**Complete builder (compiles)**:
```rust
let result = client
    .structure()
    .agency("ECB")
    .resource_type("dataflow")  // ✅ Now set!
    .send()
    .await?;
```

**Field order is flexible**:
```rust
let result = client
    .structure()
    .resource_type("dataflow")   // Can be first
    .agency("ECB")               // Or second
    .send()
    .await?;
```

---

## Alternatives Considered

### Alternative 1: Runtime Validation

Call `.send()` anytime; validate mandatory fields at runtime, return `Err(...)` if incomplete.

**Pros**:
- Simpler to implement (no type parameters)
- Familiar to users from other libraries
- Smaller type signatures

**Cons**:
- Errors only caught at runtime (late feedback)
- Must test error paths extensively
- Entire class of bugs possible: users forget to set fields, tests pass locally, production fails
- Server error messages are opaque

**Verdict**: Rejected. Fails to meet core goal of compile-time guarantee.

### Alternative 2: Builder Validation with `.build()`

Builder has all methods available; `.build()` validates, returns `Result<Query, ValidationError>`.

**Pros**:
- Single builder struct (simple to understand)
- Validation is explicit

**Cons**:
- Errors still at runtime (late feedback)
- No compile-time guarantee of validity
- `.build()` can fail, requiring error handling everywhere

**Verdict**: Rejected. Same problem as Alternative 1.

### Alternative 3: Typestate Pattern (Chosen)

Use generic type parameters to encode state; `.send()` only exists when valid.

**Pros**:
- Compile-time guarantee of validity
- Impossible to construct invalid queries
- Clear, self-describing error messages (type parameters tell you what's missing)
- Flexible field-setting order
- Zero runtime overhead (types erased)

**Cons**:
- Type parameter noise (can seem cryptic to newcomers)
- Requires understanding of marker types and generics
- Larger compiled types (but negligible at runtime)

**Verdict**: Accepted. Meets all goals and constraints.

---

## Drawbacks / Trade-offs

### 1. Learning Curve & Type-Level Error Diagnostics

Type parameters are less familiar to Rust developers. Seeing `StructureQueryBuilder<HasAgency, NoResourceType>` in an error can be confusing at first.

**Mitigations**:
- Type parameter names are self-describing (`HasAgency`, `NoResourceType`, `HasFlowId`) to make compiler output readable without prior knowledge of the pattern.
- Comprehensive error-resolution guide (CONTRIBUTING.md) with step-by-step troubleshooting for common mistakes.
- Mermaid state machine diagrams above document valid transitions clearly.

**Known Limitation**: Custom diagnostic messages via `#[rustc_on_unimplemented]` would improve errors (e.g., *"call `.resource_type(\"...\")` before `.send()`"*), but this attribute is nightly-only and unsuitable for a stability-targeting library. It is tracked as a future improvement if/when it stabilises. Stable workarounds (`#[deprecated]` on phantom `send()` impls) produce warnings rather than errors and are not used here.

### 2. Type Parameter Verbosity

Complex queries might have many type parameters, leading to verbose type signatures in code. Mitigation: [ADR-0016](../adr/0016-type-parameter-count.md) limits this to 3-4 parameters per builder.

### 3. Compilation Time

Monomorphization of generic builders for each combination of states increases compile times slightly. Mitigation: builder methods are small and inline-friendly; real impact is negligible for most projects.

### 4. API Stability

Changing mandatory fields is a breaking change (adds new type parameter). Mitigation: Document field stability in rustdoc; breaking changes require major version bump (standard semver).

### 5. Ergonomic Trade-off

Marker types can feel heavyweight for simple builders. For small builders, runtime validation might be simpler. Mitigation: Typestate pattern only used for mandatory fields; optional fields use `.option()` methods that don't affect state.

---

## Implementation Blueprints

### Structure Query Builder

#### State Machine

```mermaid
graph TD
    Start[SdmxClient::structure] :::start
    Start -->|returns| Empty["StructureQueryBuilder<br/>(NoAgency, NoResourceType)"]

    Empty -->|.agency| AgencyBound["StructureQueryBuilder<br/>(HasAgency, NoResourceType)"]
    Empty -->|.resource_type| ResourceBound["StructureQueryBuilder<br/>(NoAgency, HasResourceType)"]

    AgencyBound -->|.resource_type| Ready["StructureQueryBuilder<br/>(HasAgency, HasResourceType)"]
    ResourceBound -->|.agency| Ready

    Ready -->|optional: .resource_id| Ready
    Ready -->|optional: .version| Ready

    Ready -->|.send() / .await| Executed[Query Sent / Result Resolved]:::core

    classDef start fill:#1e293b,stroke:#3b82f6,stroke-width:2px,color:#f8fafc;
    classDef core  fill:#1e293b,stroke:#f59e0b,stroke-width:2px,color:#f8fafc;
    classDef default fill:#1e293b,stroke:#475569,stroke-width:1px,color:#f8fafc;
```

#### Code Blueprint

```rust
use std::borrow::Cow;

// 1. Phantom typestate markers.
//    HasAgency and HasResourceType carry their values directly so the type system
//    guarantees field presence without a separate runtime Option check.
//    All fields use Cow<'static, str> to ensure the builder is 'static.
pub struct NoAgency;
pub struct HasAgency(Cow<'static, str>);
pub struct NoResourceType;
pub struct HasResourceType(Cow<'static, str>);

// 2. The unified builder.
//    No lifetime parameters. The builder is unconditionally 'static because
//    all Cow fields are Cow<'static, str>. This allows the builder and all its
//    returned futures to be freely spawned on background tasks (e.g., tokio::spawn).
pub struct StructureQueryBuilder<A, RT> {
    client:        SdmxClient,
    agency:        A,
    resource_type: RT,
    resource_id:   Option<Cow<'static, str>>,   // specific artefact ID; defaults to "all"
    version:       Option<Cow<'static, str>>,   // defaults to "latest"
}

// 3. Optional fields available on any state.
//    Callers are not forced into a specific call order for optional parameters.
impl<A, RT> StructureQueryBuilder<A, RT> {
    /// Specific resource identifier within the artefact type and agency.
    /// Defaults to `all` if not set, returning all matching artefacts.
    pub fn resource_id(mut self, id: impl Into<Cow<'static, str>>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    /// Version of the artefact. Defaults to `latest` if not set.
    pub fn version(mut self, version: impl Into<Cow<'static, str>>) -> Self {
        self.version = Some(version.into());
        self
    }
}

// -------------------------------------------------------------------------
// State transitions
// Transitions 1 & 2 both depart from the same initial state and are combined
// into one impl block. Transitions 3 & 4 each occupy their own block.
// -------------------------------------------------------------------------

// Transitions 1 & 2: NoAgency/NoResourceType → HasAgency/NoResourceType
//                    NoAgency/NoResourceType → NoAgency/HasResourceType
impl StructureQueryBuilder<NoAgency, NoResourceType> {
    pub fn agency(
        self,
        id: impl Into<Cow<'static, str>>,
    ) -> StructureQueryBuilder<HasAgency, NoResourceType> {
        StructureQueryBuilder {
            client:        self.client,
            agency:        HasAgency(id.into()),
            resource_type: self.resource_type,
            resource_id:   self.resource_id,
            version:       self.version,
        }
    }

    pub fn resource_type(
        self,
        name: impl Into<Cow<'static, str>>,
    ) -> StructureQueryBuilder<NoAgency, HasResourceType> {
        StructureQueryBuilder {
            client:        self.client,
            agency:        self.agency,
            resource_type: HasResourceType(name.into()),
            resource_id:   self.resource_id,
            version:       self.version,
        }
    }
}

// Transition 3: HasAgency/NoResourceType → HasAgency/HasResourceType
// Caller set agency first, now completes with resource_type.
impl StructureQueryBuilder<HasAgency, NoResourceType> {
    pub fn resource_type(
        self,
        name: impl Into<Cow<'static, str>>,
    ) -> StructureQueryBuilder<HasAgency, HasResourceType> {
        StructureQueryBuilder {
            client:        self.client,
            agency:        self.agency,
            resource_type: HasResourceType(name.into()),
            resource_id:   self.resource_id,
            version:       self.version,
        }
    }
}

// Transition 4: NoAgency/HasResourceType → HasAgency/HasResourceType
// Caller set resource_type first, now completes with agency.
impl StructureQueryBuilder<NoAgency, HasResourceType> {
    pub fn agency(
        self,
        id: impl Into<Cow<'static, str>>,
    ) -> StructureQueryBuilder<HasAgency, HasResourceType> {
        StructureQueryBuilder {
            client:        self.client,
            agency:        HasAgency(id.into()),
            resource_type: self.resource_type,
            resource_id:   self.resource_id,
            version:       self.version,
        }
    }
}

// -------------------------------------------------------------------------
// Execution state
// Only StructureQueryBuilder<HasAgency, HasResourceType> exposes .send().
// All other states are inert: they can accumulate optional fields and
// transition, but they cannot be executed.
// -------------------------------------------------------------------------

impl StructureQueryBuilder<HasAgency, HasResourceType> {
    /// Execute the query. This is the primary execution method.
    /// The builder is consumed; mandatory fields are guaranteed by the type system.
    /// The returned future is unconditionally 'static and can be spawned on background tasks.
    pub async fn send(self) -> Result<StructureMessage, sdmx_client::Error> {
        let mut url = self.client.base_url.clone();
        {
            let mut segments = url.path_segments_mut()
                .map_err(|_| sdmx_client::Error::InvalidBaseUrl)?;

            segments.push("structure");
            segments.push(&self.resource_type.0);                             // guaranteed: HasResourceType
            segments.push(&self.agency.0);                                    // guaranteed: HasAgency
            segments.push(self.resource_id.as_deref().unwrap_or("all"));
            segments.push(self.version.as_deref().unwrap_or("latest"));
        }
        self.client.execute_get(url).await
    }
}

/// `IntoFuture` delegates to `.send()`, allowing callers to `.await` the
/// builder directly. `.send()` is preferred in all documentation examples.
/// See 'Execution Model' in ARCHITECTURE.md for the full rationale (ADR-0015).
impl IntoFuture for StructureQueryBuilder<HasAgency, HasResourceType> {
    type Output     = Result<StructureMessage, sdmx_client::Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'static>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}
```

---

### Data Query Builder

#### State Machine

```mermaid
graph TD
    Start[SdmxClient::data] :::start
    Start -->|returns| Empty["DataQueryBuilder<br/>(NoAgency, NoFlowId)"]

    Empty -->|.agency| AgencyBound["DataQueryBuilder<br/>(HasAgency, NoFlowId)"]
    Empty -->|.flow_id| FlowBound["DataQueryBuilder<br/>(NoAgency, HasFlowId)"]

    AgencyBound -->|.flow_id| Ready["DataQueryBuilder<br/>(HasAgency, HasFlowId)"]
    FlowBound -->|.agency| Ready

    Ready -->|optional: .context| Ready
    Ready -->|optional: .version| Ready
    Ready -->|optional: .raw_key| Ready
    Ready -->|optional: .dimensions| Ready

    Ready -->|.send() / .await| Executed[Query Sent / Result Resolved]:::core

    classDef start fill:#1e293b,stroke:#3b82f6,stroke-width:2px,color:#f8fafc;
    classDef core  fill:#1e293b,stroke:#f59e0b,stroke-width:2px,color:#f8fafc;
    classDef default fill:#1e293b,stroke:#475569,stroke-width:1px,color:#f8fafc;
```

#### Code Blueprint

```rust
use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;

// 1. Phantom typestate markers.
//    These live in sdmx_client::query::data and do not conflict with the
//    identically-named markers in sdmx_client::query::structure.
//    All fields use Cow<'static, str> to ensure the builder is 'static.
pub struct NoAgency;
pub struct HasAgency(Cow<'static, str>);
pub struct NoFlowId;
pub struct HasFlowId(Cow<'static, str>);

// 2. The unified builder.
//    No lifetime parameters. The builder is unconditionally 'static because
//    all Cow fields are Cow<'static, str>.
//    context and version are optional with SDMX 3.x protocol defaults.
//    dimensions are expressed as query parameters, not path segments.
pub struct DataQueryBuilder<A, F> {
    client:     SdmxClient,
    agency:     A,
    flow:       F,
    context:    Option<Cow<'static, str>>,   // defaults to "dataflow"
    version:    Option<Cow<'static, str>>,   // defaults to "latest"
    raw_key:    Option<Cow<'static, str>>,   // optional positional key fallback: "A.USD.EUR..A"
    dimensions: Option<Cow<'static, str>>,   // SDMX 3.x query params: "c[FREQ]=A+M&c[REF_AREA]=US"
}

// 3. Optional fields available on any state.
impl<A, F> DataQueryBuilder<A, F> {
    /// Artefact context type. Defaults to `dataflow` if not set.
    pub fn context(mut self, ctx: impl Into<Cow<'static, str>>) -> Self {
        self.context = Some(ctx.into());
        self
    }

    /// Version of the dataflow. Defaults to `latest` if not set.
    pub fn version(mut self, version: impl Into<Cow<'static, str>>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Positional path key fallback for legacy migration.
    /// Example: `"A.USD.EUR..A"`
    pub fn raw_key(mut self, key: impl Into<Cow<'static, str>>) -> Self {
        self.raw_key = Some(key.into());
        self
    }

    /// Dimension filter expressed as SDMX 3.x query parameters.
    /// Example: `"c[FREQ]=A+M&c[REF_AREA]=US+GB"`
    pub fn dimensions(mut self, dims: impl Into<Cow<'static, str>>) -> Self {
        self.dimensions = Some(dims.into());
        self
    }
}

// -------------------------------------------------------------------------
// State transitions — same four-transition pattern as StructureQueryBuilder.
// -------------------------------------------------------------------------

// Transitions 1 & 2: NoAgency/NoFlowId → HasAgency/NoFlowId
//                    NoAgency/NoFlowId → NoAgency/HasFlowId
impl DataQueryBuilder<NoAgency, NoFlowId> {
    pub fn agency(
        self,
        id: impl Into<Cow<'static, str>>,
    ) -> DataQueryBuilder<HasAgency, NoFlowId> {
        DataQueryBuilder {
            client:     self.client,
            agency:     HasAgency(id.into()),
            flow:       self.flow,
            context:    self.context,
            version:    self.version,
            raw_key:    self.raw_key,
            dimensions: self.dimensions,
        }
    }

    pub fn flow_id(
        self,
        id: impl Into<Cow<'static, str>>,
    ) -> DataQueryBuilder<NoAgency, HasFlowId> {
        DataQueryBuilder {
            client:     self.client,
            agency:     self.agency,
            flow:       HasFlowId(id.into()),
            context:    self.context,
            version:    self.version,
            raw_key:    self.raw_key,
            dimensions: self.dimensions,
        }
    }
}

// Transition 3: HasAgency/NoFlowId → HasAgency/HasFlowId
impl DataQueryBuilder<HasAgency, NoFlowId> {
    pub fn flow_id(
        self,
        id: impl Into<Cow<'static, str>>,
    ) -> DataQueryBuilder<HasAgency, HasFlowId> {
        DataQueryBuilder {
            client:     self.client,
            agency:     self.agency,
            flow:       HasFlowId(id.into()),
            context:    self.context,
            version:    self.version,
            raw_key:    self.raw_key,
            dimensions: self.dimensions,
        }
    }
}

// Transition 4: NoAgency/HasFlowId → HasAgency/HasFlowId
impl DataQueryBuilder<NoAgency, HasFlowId> {
    pub fn agency(
        self,
        id: impl Into<Cow<'static, str>>,
    ) -> DataQueryBuilder<HasAgency, HasFlowId> {
        DataQueryBuilder {
            client:     self.client,
            agency:     HasAgency(id.into()),
            flow:       self.flow,
            context:    self.context,
            version:    self.version,
            raw_key:    self.raw_key,
            dimensions: self.dimensions,
        }
    }
}

// -------------------------------------------------------------------------
// Execution state: only DataQueryBuilder<HasAgency, HasFlowId> can send.
// -------------------------------------------------------------------------

impl DataQueryBuilder<HasAgency, HasFlowId> {
    /// Execute the query. This is the primary execution method.
    /// The returned future is unconditionally 'static and can be spawned on background tasks.
    pub async fn send(self) -> Result<DataMessage, sdmx_client::Error> {
        let context = self.context.as_deref().unwrap_or("dataflow");
        let version = self.version.as_deref().unwrap_or("latest");
        let key = self.raw_key.as_deref().unwrap_or("all");

        // Flow reference is a single comma-separated path segment per SDMX 3.x spec
        let flow_ref = format!(
            "{},{},{},{}",
            context,
            &self.agency.0,   // guaranteed: HasAgency
            &self.flow.0,     // guaranteed: HasFlowId
            version,
        );

        let mut url = format!(
            "{}/data/{}/{}",
            self.client.base_url,
            flow_ref,
            key,
        );
        if let Some(dims) = &self.dimensions {
            url.push('?');
            url.push_str(dims);
        }
        self.client.execute_get(url).await
    }
}

/// `IntoFuture` delegates to `.send()`. See 'Execution Model' in ARCHITECTURE.md.
impl IntoFuture for DataQueryBuilder<HasAgency, HasFlowId> {
    type Output     = Result<DataMessage, sdmx_client::Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'static>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}
```

---

## Questions & Resolutions

- **[Open]** - **Nested Builders**: If a query contains sub-builders (e.g., constraint expressions), should sub-builders also use typestate? Or should they be pre-validated before passing to parent?

- **[Open]** - **Optional Fields Becoming Mandatory**: If a Phase N adds a new mandatory field to a query (e.g., "version" becomes required), how do we migrate users with existing code? The type change is breaking.

- **[Open]** - **Error Message UX**: Can we improve compiler error messages with custom error traits to make `NoResourceType` feel less cryptic? Should we provide `#[derive(BuilderState)]` macros to reduce boilerplate?

- **[Open]** - **Builder Cloning**: Should builders be cloneable (to re-use partially-set builders)? This adds complexity (must capture state in Clone impl). For now, assume builders are consumed.

- **[Open]** - **Blocking API Bridge**: How do typestate builders interact with the blocking wrapper (Design 0005)? Must async `.send()` be duplicated in blocking variant?

---

## References

* [Design 0007: Compile-Time Query Validation via the Typestate Pattern](0007-typestate-query-validation.md) — Design decision to use typestate pattern
* [ADR-0016: Type Parameter Count](../adr/0016-type-parameter-count.md) — Naming convention and limits for type parameters
* [Design 0005: Synchronous and Blocking API Execution Bridge](0005-synchronous-and-blocking-api-execution-bridge.md) — Integration with blocking wrapper
* [ARCHITECTURE.md § API Design & Ergonomics](../../ARCHITECTURE.md#api-design--ergonomics) — High-level API strategy
* [CONTRIBUTING.md § Typestate Compiler Errors](../../CONTRIBUTING.md#typestate-compiler-errors-understanding--resolving) — User-facing error resolution guide

---

## Notes for Implementation

**Phase**: Phase 3 (HTTP Client Implementation)

**Design Status**: This document is marked "Proposed" because the true test of this design is in implementation. Until builders are actually built and tested against real use cases, details may need adjustment based on what we learn about:
- Whether `Cow<'static, str>` feels ergonomic for callers
- Whether state machine complexity is worth the compile-time guarantee
- Whether error messages are clear enough without custom derive macros
- Whether the blocking API bridge (Design 0005) integrates cleanly with typestate builders

**Implementation Steps**:
1. Define marker types (`HasAgency`, `NoAgency`, etc.) in `sdmx-client/src/query/state.rs`
2. Implement `StructureQueryBuilder<A, RT>` generic struct with marker type parameters
3. Implement builder methods (`.agency()`, `.resource_type()`, etc.) with state transitions
4. Implement `.send()` and `IntoFuture` only for fully-specified state (`HasAgency`, `HasResourceType`)
5. Implement identical pattern for `DataQueryBuilder<A, F>` with two state parameters
6. Validate `'static` bounds on builders (ensure `.spawn()` safety)
7. Add comprehensive rustdoc examples for each builder state
8. Add compile-fail tests (via `trybuild` or similar) to verify incomplete builders don't compile
9. Integrate with blocking API (Design 0005): ensure `build_sync()` works with builders

**Testing Strategy**:
- Unit tests for state transitions (verify `.agency()` correctly transforms type)
- Integration tests for complete queries (verify `.send()` executes end-to-end)
- Compile-fail tests (verify incomplete builders produce expected `E0599` errors)
- Async spawning tests (verify builders are `Send + 'static`)

**Documentation to Add**:
- Step-by-step error resolution guide in CONTRIBUTING.md (referenced in Drawbacks section above)
- Rustdoc examples for common builder patterns (string literals, owned strings, optional fields)
- Tutorial: *"Implementing Typestate Builders in Rust"* (exemplar material for library patterns)

**Breaking Changes**: None in Phase 3. If future phases add mandatory fields to an existing builder, that's a breaking change (major version bump).

**Dependencies**: None new (generics and `Cow<'static, str>` are stable Rust).
