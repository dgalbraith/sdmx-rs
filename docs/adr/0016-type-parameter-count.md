# 16. Type Parameter Count Reflects Domain Constraint

Date: 2026-05-20

## Status

Accepted

---

## Context

Once the Typestate pattern is adopted ([Design 0007](../design/0007-typestate-query-validation.md)), a secondary decision
arises for every new builder: how many phantom type parameters does it need?

The library currently has two builders:

- `StructureQueryBuilder<'a, A, RT>` — two type parameters (`A` for agency,
  `RT` for resource_type), because both are independently mandatory and
  independently settable in any order.
- `DataQueryBuilder<'a, A, F>` — two type parameters (`A` for agency,
  `F` for flow reference), because both `agencyID` and the flow's `resourceID`
  are independently mandatory under SDMX 3.x REST specifications.

This symmetry is intentional. The question this ADR answers is: what is
the general rule, and how should it be applied to future builders?

This decision matters beyond the current two builders. The SDMX REST API
defines additional query types — metadata queries, schema queries, and
others — each with their own mandatory field sets. As the library grows,
contributors need a clear principle for modelling new builders, not a
case-by-case judgment call.

---

## Decision Drivers

- **Correctness.** The type parameter structure must accurately model the
  domain constraints, neither over-constraining nor under-constraining
  the valid state space.
- **Complexity budget.** Each type parameter adds impl blocks, increases
  the depth of the state machine, and adds to the contributor learning
  curve. Complexity must be justified by the constraint it enforces.
- **Consistency.** Builders across the library should follow a rule a
  contributor can apply without needing to consult the original authors.
- **Exemplar value.** The library should demonstrate that Typestate
  complexity is a tool applied proportionally, not a pattern applied
  mechanically.

---

## The General Rule

> **One type parameter per independently-settable mandatory field.**

Three conditions must all be true for a field to warrant a type parameter:

1. **Mandatory.** The field has no protocol-level default. Omitting it
   produces an invalid query.
2. **Independent.** The field can be set in any order relative to other
   mandatory fields. There is no required sequencing.
3. **Caller-supplied.** The field is provided by the caller, not derived
   by the library from other fields or from context.

Optional fields — those with a protocol-level default — are stored as
`Option<T>` on the builder struct and are never type parameters.

---

## Application to Current Builders

### `StructureQueryBuilder<'a, A, RT>` — Two Parameters

SDMX REST structure query route: `/structure/{resourceType}/{agencyId}/{version}`

| Field           | Mandatory                 | Independent | Caller-supplied | Type parameter? |
|-----------------|---------------------------|:-----------:|:---------------:|-----------------|
| `resource_type` | Yes                       | Yes         | Yes             | Yes → `RT`      |
| `agencyId`      | Yes                       | Yes         | Yes             | Yes → `A`       |
| `version`       | No (defaults to `latest`) | -           | Yes             | No → `Option`   |

Both mandatory fields are independently settable. A caller may set agency
before resource_type or resource_type before agency — the protocol does not care.
This produces a two-dimensional state machine with four nodes:

```
<NoAgency, NoResourceType> → <HasAgency, NoResourceType>
<NoAgency, NoResourceType> → <NoAgency, HasResourceType>
<HasAgency, NoResourceType> → <HasAgency, HasResourceType>
<NoAgency, HasResourceType> → <HasAgency, HasResourceType>
```

Two type parameters are correct and necessary.

---

### `DataQueryBuilder<'a, A, F>` — Two Parameters

SDMX 3.x REST data query route: `/data/{context}/{agencyID}/{resourceID}/{version}`

| Field        | Mandatory                   | Independent | Caller-supplied | Type parameter?     |
|--------------|-----------------------------|:-----------:|:---------------:|---------------------|
| `agencyID`   | Yes                         | Yes         | Yes             | Yes → `A`           |
| `flow_id`    | Yes                         | Yes         | Yes             | Yes → `F` (flow ID) |
| `context`    | No (defaults to `dataflow`) | -           | Yes             | No → `Option`       |
| `version`    | No (defaults to `latest`)   | -           | Yes             | No → `Option`       |
| `dimensions` | No (defaults to `all`)      | -           | Yes             | No → `Option`       |

Both `agencyID` and `flow_id` (the unique identifier of a dataflow, not the entire flow reference)
are mandatory and independently settable. This produces a two-dimensional state machine
with four nodes:

```
<NoAgency, NoFlowId> → <HasAgency, NoFlowId>
<NoAgency, NoFlowId> → <NoAgency, HasFlowId>
<HasAgency, NoFlowId> → <HasAgency, HasFlowId>
<NoAgency, HasFlowId> → <HasAgency, HasFlowId>
```

**Naming rationale**: The type parameter `F` represents `FlowId` (marked as `NoFlowId`/`HasFlowId`
in the state machine) to make explicit that this is the unique identifier of the dataflow,
not the entire flow reference structure. The `flow_id()` method name consistently reflects
this semantic distinction.

Two type parameters are correct and necessary.

---

## Options Considered for the General Rule

### Option A — One Type Parameter per Builder (Uniform Minimum)

Every builder has exactly one type parameter representing an `IsReady` /
`NotReady` state. When all mandatory fields are set, the builder transitions
to `IsReady`.

```rust
pub struct StructureQueryBuilder<'a, State> {
    client:        &'a SdmxClient,
    agency:        Option<Cow<'a, str>>,
    resource_type: Option<Cow<'a, str>>,
    version:       Option<Cow<'a, str>>,
}

// NotReady → Ready requires both agency and resource_type to have been set
impl<'a> StructureQueryBuilder<'a, NotReady> {
    pub fn agency(mut self, ...) -> Self { ... }
    pub fn resource_type(mut self, ...) -> Self { ... }
    pub fn ready(self) -> Result<StructureQueryBuilder<'a, Ready>, Error> {
        // runtime check that both are set
    }
}
```

**Pros**:

- Uniform structure; every builder looks the same regardless of how many
  mandatory fields it has.
- Fewer impl blocks per builder.

**Cons**:

- Collapses all mandatory fields into a single state bit. The type system
  can no longer express *which* fields are missing — only that the builder
  is not ready.
- Reintroduces runtime validation (the `ready()` check) for the relationship
  between mandatory fields, which is exactly what [Design 0007](../design/0007-typestate-query-validation.md) rejected.
- Compiler errors become less informative: `not ready` rather than
  `NoResourceType`.
- Defeats the purpose of the Typestate pattern for multi-field builders.

**Verdict**: A false uniformity that sacrifices the primary benefit of
Typestate for builders with more than one mandatory field. Rejected.

---

### Option B — Maximum Type Parameters (Uniform Maximum)

Every field, mandatory or optional, gets a type parameter.

```rust
pub struct DataQueryBuilder<'a, F, K, P> {
    client:   &'a SdmxClient,
    flow:     F,     // NoFlow | HasFlow
    key:      K,     // NoKey | HasKey
    provider: P,     // NoProvider | HasProvider
}
```

**Pros**:

- Maximum type-system information at every point in the chain.
- A caller could theoretically constrain an API to require, say, a key.

**Cons**:

- Type parameters on optional fields enforce nothing. `HasKey` is not
  required to call `.send()`; neither is `NoKey`. The parameter exists
  but is inert.
- For `DataQueryBuilder` with two optional fields, this produces 2³ = 8
  possible states, of which 4 are executable (`HasFlow/*`). The four
  `NoFlow/*` states and the transitions between `HasKey`/`NoKey` and
  `HasProvider`/`NoProvider` add impl blocks that enforce no constraint.
- State machine diagrams become unreadable at scale.
- Adds significant mechanical impl block work for every optional field
  added to any builder.

**Verdict**: Type parameters without constraints are noise. Optional fields
with protocol-level defaults belong in `Option<T>`, not in the typestate
graph. Rejected.

---

### Option C — One Parameter per Independently-Settable Mandatory Field

The rule stated at the top of this document. Each type parameter corresponds
to exactly one independently mandatory, caller-supplied field. The complexity
of the state machine is proportional to the number of such fields.

**Pros**:

- Every type parameter enforces a real constraint.
- The state machine complexity matches the domain complexity — no more,
  no less.
- Straightforward to apply to new builders: identify mandatory fields,
  count them, add that many type parameters.
- Compiler errors are maximally informative: `NoResourceType` tells you
  exactly what is missing.

**Cons**:

- Different builders have different numbers of type parameters, which
  can initially look inconsistent to a contributor who has not read this
  ADR. Mitigated by documentation.
- For builders with three or more mandatory fields, the number of impl
  blocks grows as 2ⁿ transitions. This is manageable up to ~3 parameters;
  at 4+ it should prompt a review of whether the query type should be
  decomposed.

**Verdict**: The correct rule. Complexity is earned, not imposed.

---

## Decision

**Each query builder carries one phantom type parameter per
independently-settable mandatory field. Optional fields are stored as
`Option<T>` and never appear as type parameters.**

The state machine for any builder has exactly 2ⁿ nodes, where n is the
number of mandatory fields. For n = 1 this is 2 nodes; for n = 2 this is
4 nodes. If a prospective builder requires n ≥ 4 mandatory independent
fields, the design should be reviewed to determine whether the query type
should be decomposed before the Typestate graph is implemented.

---

## Consequences

- All query builders in this library follow the rule: one phantom type
  parameter per independently-settable mandatory field. This is a
  library-wide convention, not a per-builder choice.
- Contributors adding new query types must apply the three-condition test
  (mandatory, independent, caller-supplied) to determine type parameter
  count before implementing. The decision table pattern in this ADR serves
  as the reference template.
- If a prospective builder requires n ≥ 4 mandatory independent fields,
  the design must be reviewed for decomposition before the Typestate graph
  is implemented. This threshold is a signal that the query type may be
  conflating separate concerns.
- Optional fields with protocol-level defaults are always `Option<T>` on
  the struct and never appear as type parameters — regardless of how
  complex the query type is.
- `CONTRIBUTING.md` must document this rule alongside the Typestate pattern
  explanation so that contributors extending the builders apply it
  consistently.

---

## References

- [Design Document 0007 — Compile-Time Query Validation via the Typestate Pattern](../design/0007-typestate-query-validation.md)
- [Design Document 0006 — Builder Field Storage Strategy (`Cow<'static, str>`)](../design/0006-builder-field-storage.md)
- `ARCHITECTURE.md` — Section 2 (Structure Query Builder state diagram)
- `ARCHITECTURE.md` — Section 3 (Data Query Builder state diagram)
- `CONTRIBUTING.md` — Extending the query builders

---

## Applying This Rule to Future Builders

When adding a new query builder, the implementing contributor should
complete the following checklist:

1. List all fields required by the SDMX REST route for this query type.
2. For each field, determine: is it mandatory (no protocol default)?
3. For each mandatory field, determine: is it independently settable
   (no required ordering relative to other mandatory fields)?
4. The count of fields that are both mandatory and independent is the
   number of type parameters the builder needs.
5. All remaining fields are `Option<T>` on the struct.
6. Draw the state machine (2ⁿ nodes) before writing any `impl` blocks.
   If the diagram is hard to draw, the builder is too complex.

### Examples from Existing Builders

- `StructureQueryBuilder<'a, A, RT>` retains two type parameters. This is
  correct.
- `DataQueryBuilder<'a, A, F>` carries two type parameters. This is correct to model
  the mandatory context-agency-flow structure in SDMX 3.x.
- Future builders must follow the rule in this ADR. The checklist above
  is the implementation guide.
- `ARCHITECTURE.md` Section 3 documents the asymmetry between the two
  current builders and explains the principle. This ADR is the formal
  reference for that explanation.
- If a future query type has three mandatory independent fields, the
  contributor must implement 2³ = 8 nodes and document the state machine
  in `ARCHITECTURE.md` before the implementation is reviewed.
