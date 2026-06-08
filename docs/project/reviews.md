# Code Review Philosophy & Standards

Code review is a collaborative teaching opportunity—reviewers help maintainers enforce consistency, catch subtle bugs, and ensure the codebase remains maintainable and performant. This document outlines review standards and what to expect from the process.

## For Reviewers

Reviewers use this guide to evaluate code consistently. These standards are not punitive but serve to maintain the project's high architectural and engineering standards.

### What Reviewers Prioritize

1. **Type Safety & Correctness**
   - Does the type system enforce the invariant being addressed?
   - Are phantom types / typestate patterns used where validation is domain-critical?
   - Could this state ever be invalid at runtime?

2. **Error Handling & Propagation**
   - Are all error paths tested (not just happy path)?
   - Is context added as errors propagate (using `map_err()` or custom error constructors)?
   - Are `panic!()` / `unwrap()` used only where truly unrecoverable (invariant violation, not logic error)?

3. **Memory & Performance Discipline**
   - Are allocations explicit and intentional (not sprinkled throughout)?
   - Are expensive types cloned unnecessarily across crate boundaries?
   - For streaming parsers: does the code avoid building full DOM trees?
   - For async code: are locks held across `.await` points?

4. **Async Safety (for sdmx-client work)**
   - Are all spawned tasks `.await`-ed or explicitly detached?
   - Does the code avoid blocking operations inside async contexts?
   - Are lifetimes clean (no lifetime contamination across task boundaries)?

5. **Concurrency & Thread Safety**
   - If `Send`/`Sync` are claimed, are they justified?
   - Are shared types behind thread-safe abstractions (Arc, Mutex)?
   - Is `Arc<SdmxClient>` actually needed, or can clients be cloned directly?

6. **Documentation Completeness**
   - Do public APIs have rustdoc comments with examples?
   - Do complex behaviors document the WHY (not the WHAT)?
   - Are invariants and safety concerns documented?

7. **Testing Coverage**
   - Does the PR maintain or improve the crate-specific coverage floors (ranging from 70% to 85%; see CONTRIBUTING.md)?
   - Are error cases tested, not just happy paths?
   - Do tests verify behavior, not just "it compiles"?

8. **Architecture & Dependency Boundaries**
   - Does the change respect crate boundaries (types → parsers → client)?
   - Is new complexity isolated, or does it leak across module boundaries?
   - Would a standalone feature benefit from its own module/crate?

### Reviewer Red Flags

Conditions that typically warrant rejection or significant revision:

- ❌ Public API with no doc comments
- ❌ Error handling via `unwrap()` in library code
- ❌ Large clones of complex types across crate boundaries
- ❌ `async` code holding locks across `.await`
- ❌ Untested error paths
- ❌ New dependencies added without ADR discussion
- ❌ Coverage regression without documented exception

### When to Approve

A PR is ready to merge when:

- ✅ Code is clear and maintainable
- ✅ Changes are scoped to a single intent
- ✅ All public APIs are documented
- ✅ Tests verify correctness (not just coverage)
- ✅ Type system prevents invalid states where applicable
- ✅ Performance implications are considered

---

## For Contributors

Contributors should familiarize themselves with these review standards **before** submission. Use them for self-review:

### Pre-Submission Self-Review Checklist

Before opening a PR, verify your code against the review priorities above:

- [ ] **Type Safety**: Does the type system enforce invariants? No unexpected runtime states possible?
- [ ] **Error Handling**: All error paths tested? Context added as errors propagate? No unwrap/panic in library code?
- [ ] **Memory**: Allocations explicit and intentional? No unnecessary clones across boundaries? No DOM accumulation in parsers?
- [ ] **Async Safety (if applicable)**: All tasks awaited? No locks across .await? Clean lifetimes?
- [ ] **Concurrency**: Send/Sync claims justified? Shared types behind abstractions?
- [ ] **Documentation**: Public APIs have rustdoc? Complex behaviors document WHY?
- [ ] **Tests**: Maintain crate-specific coverage floors? Error cases tested? Tests verify behavior?
- [ ] **Architecture**: Respects crate boundaries? Complexity isolated?
- [ ] **No red flags**: No unwrap, no undocumented dependencies, no coverage regressions

See [CONTRIBUTING.md § Style Checklist](../../CONTRIBUTING.md#style-checklist) for code formatting and style requirements.

---

## Review Process

1. **Contributor opens PR** with issue reference and passes local `just verify`
2. **Maintainer reviews** against the standards above
3. **Feedback loop**: Contributor addresses comments; maintainer re-reviews
4. **Approval**: Maintainer approves when all standards are met
5. **Merge**: Merge to `main` via standard merge commit (`--no-ff`) to preserve cryptographic provenance

Reviewers prioritize thoroughness over speed—expect substantive feedback on non-trivial changes.
