# Testing Guide

This guide documents the testing strategy, patterns, and infrastructure for sdmx-rs. Testing is not optional—the project enforces per-crate coverage floors calibrated to the complexity and criticality of each crate, with a separate patch coverage target applied to new code in PRs. [`codecov.yaml`](../../codecov.yaml) is the authoritative source for current thresholds.

## Testing Philosophy

**Why we test**:
- Verify correctness: SDMX parsing must be bulletproof; invalid data should never silently corrupt state
- Catch regressions: refactoring should be safe; tests are a safety net
- Document behaviour: tests are executable specifications; they clarify intent
- Enable confidence: deployments should be boring; comprehensive tests make that possible

**What we optimise for**:
1. **Clarity over coverage chasing** — 80% coverage of important paths beats 100% coverage of trivial code
2. **Meaningful assertions** — tests that assert useful properties, not just "it runs without crashing"
3. **Test maintainability** — tests are code; they need the same care as production code
4. **Realistic scenarios** — use actual SDMX spec examples; avoid synthetic happy-path-only tests

**What we don't do**:
- Over-mock: real behaviour is better than mock behaviour; mocks hide integration failures
- Test internals: test public APIs and behaviour, not implementation details
- Ignore flaky tests: flaky tests are worse than no tests; they destroy confidence
- Treat coverage as a goal: 80% is a floor, not a ceiling

---

## Testing Pyramid & Structure

```
         /\
        /  \  Integration Tests (Crate-level: API contracts)
       /----\
      /      \
     /        \  Unit Tests (Function-level: behaviour)
    /----------\
   /            \
  /              \ Doc Tests (Examples: API usage)
 /----------------\
```

**Distribution**:
- **Unit tests**: 60–70% of test count (fast, focused, numerous)
- **Integration tests**: 20–30% of test count (verify crate contracts)
- **Doc tests**: 5–10% of test count (API examples that compile and run)
- **Property-based tests**: 5–10% (fuzzing, roundtrip validation)

**Coverage by crate**:
- **sdmx-types**: 85%+ (domain types are critical; comprehensive validation)
- **sdmx-parsers**: 75–80% (streaming logic inherently complex; rare branches hard to hit)
- **sdmx-writers**: 80%+ (serialisation is straightforward)
- **sdmx-client**: 80%+ (HTTP logic, error handling well-covered by mocks)
- **sdmx-rs (facade)**: 70%+ (thin wrapper; coverage is secondary to integration tests)

Coverage is measured via `cargo llvm-cov` and enforced in CI. Thresholds vary by crate criticality (see [codecov.yaml](../../codecov.yaml)):
- **sdmx-types**: 85% target, 0% threshold (no regressions for domain types)
- **sdmx-parsers**: 75% target, 1% threshold (streaming complexity tolerated)
- **sdmx-writers, sdmx-client**: 80% target, 1% threshold
- **sdmx-rs**: 70% target, 1% threshold (facade pattern, integration tests primary)

---

## Test Organisation

### Unit Tests: Collocated in Modules

Place unit tests in the same file as the code they test, at the bottom of the module:

```rust
// src/constraint/model.rs

pub struct ConstraintModel {
    rules: Vec<Rule>,
}

impl ConstraintModel {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_model_is_empty() {
        let model = ConstraintModel::new();
        assert!(model.rules.is_empty());
    }

    #[test]
    fn test_add_rule_increases_count() {
        let mut model = ConstraintModel::new();
        let rule = Rule { /* ... */ };
        model.add_rule(rule);
        assert_eq!(model.rules.len(), 1);
    }
}
```

**Rationale**:
- Tests stay close to the code they test (easier to maintain)
- `#[cfg(test)]` modules are compiled out in release builds (zero overhead)
- Module-level access to private items is natural and necessary
- Reviewers see code and tests in the same PR diff

### Integration Tests: In `tests/` Directory

Integration tests verify crate-level contracts and interact with the public API:

```
crates/sdmx-parsers/
├── src/
│   ├── lib.rs
│   └── xml/
│       └── constraint.rs
└── tests/
    ├── common/
    │   └── fixtures.rs          # Shared test utilities
    ├── integration_parse_constraint.rs
    └── integration_parse_dataflow.rs
```

Each test file in `tests/` is compiled as a separate binary:

```rust
// crates/sdmx-parsers/tests/integration_parse_constraint.rs
use sdmx_parsers::parse_constraint_model;
use std::fs;

#[test]
fn test_parse_valid_constraint_model() {
    let xml = fs::read_to_string("tests/fixtures/constraint_valid.xml")
        .expect("fixture not found");
    let model = parse_constraint_model(&xml)
        .expect("parsing failed");
    assert_eq!(model.rules().len(), 3);
}

#[test]
fn test_parse_invalid_xml_returns_error() {
    let xml = "<invalid></xml>";
    let result = parse_constraint_model(xml);
    assert!(result.is_err());
}
```

**Rationale**:
- Separate binaries enforce use of public API only (no `pub(crate)` shortcuts)
- Integration tests are slow; keep unit tests fast and numerous
- Crate-level tests catch API design issues early

### Doc Tests: Embedded Examples

Include doc tests for complex types and functions; they serve as executable documentation:

```rust
/// Parses an SDMX constraint model from an XML string.
///
/// # Examples
///
/// ```
/// use sdmx_parsers::parse_constraint_model;
///
/// let xml = r#"
///   <ConstraintModel id="CM1">
///     <Rules>...</Rules>
///   </ConstraintModel>
/// "#;
/// let model = parse_constraint_model(xml)?;
/// assert_eq!(model.id(), "CM1");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_constraint_model(input: &str) -> Result<ConstraintModel> {
    // ...
}
```

**Rationale**:
- Examples stay in sync with API (compiler enforces this)
- Users see real usage patterns
- Doc tests are slower; use sparingly for non-trivial APIs

**When NOT to include**:
- Simple functions with obvious usage
- Functions that require heavy setup (network mocks)
- Examples better shown in README or EXAMPLES.md

---

## Fixtures & Test Data

### Fixture Organisation

Store test data by crate and domain type:

```
crates/sdmx-parsers/tests/fixtures/
├── constraint/
│   ├── valid/
│   │   ├── simple.xml
│   │   ├── complex_rules.xml
│   │   └── nested_dimensions.xml
│   ├── invalid/
│   │   ├── missing_element.xml
│   │   ├── malformed_syntax.xml
│   │   └── schema_violation.xml
│   └── README.md                 # Documents fixture purpose
├── dataflow/
│   ├── valid/
│   │   └── simple.json
│   └── invalid/
│       └── missing_dimensions.json
└── README.md                     # Overall fixture guide
```

### Fixture Sources

**Real SDMX Examples**:
- Use official spec examples (SDMX 3.0 and 3.1 reference documents)
- Document source in fixture README:
  ```markdown
  # Test Fixtures

  ## constraint/valid/complex_rules.xml
  - Source: SDMX-ML Technical Specification, Section 4.2.1
  - Purpose: Validates multi-level constraint evaluation with dimension relationships
  - Generated: 2026-05-23 from spec version 3.1.0
  ```

**Synthetic Examples**:
- Keep synthetic examples minimal; label them clearly
- Use for edge cases not in the spec (empty collections, boundary values)
- Tag with `# SYNTHETIC` in a comment

### Fixture Loading Utilities

Create a `tests/common/fixtures.rs` module for reusable fixture loading:

```rust
// crates/sdmx-parsers/tests/common/fixtures.rs
use std::fs;
use std::path::{Path, PathBuf};

pub struct FixtureLoader;

impl FixtureLoader {
    fn base_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
    }

    pub fn load_constraint(name: &str) -> String {
        let path = Self::base_path()
            .join("constraint")
            .join("valid")
            .join(format!("{}.xml", name));
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("fixture not found: {:?}: {}", path, e))
    }

    pub fn load_constraint_invalid(name: &str) -> String {
        let path = Self::base_path()
            .join("constraint")
            .join("invalid")
            .join(format!("{}.xml", name));
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("fixture not found: {:?}: {}", path, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_loader_loads_valid() {
        let xml = FixtureLoader::load_constraint("simple");
        assert!(!xml.is_empty());
    }
}
```

Declare as a module in test files:

```rust
// tests/integration_parse_constraint.rs
mod common;
use common::fixtures::FixtureLoader;

#[test]
fn test_parse_simple_constraint() {
    let xml = FixtureLoader::load_constraint("simple");
    let model = parse_constraint_model(&xml).expect("parsing failed");
    assert!(!model.rules().is_empty());
}
```

---

## Mocking & Test Doubles

### HTTP Mocking for sdmx-client

Use `wiremock` for HTTP mocking; it provides predictable stub responses without needing a real HTTP server:

```rust
// crates/sdmx-client/tests/integration_fetch.rs
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::method;
use sdmx_client::Client;

#[tokio::test]
async fn test_fetch_constraint_model_success() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Register mock response
    Mock::given(method("GET"))
        .and(path("/constraint/CM1"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(r#"<ConstraintModel id="CM1">...</ConstraintModel>"#))
        .mount(&mock_server)
        .await;

    // Test client behaviour
    let client = Client::new(mock_server.uri())
        .expect("client creation failed");
    let model = client.fetch_constraint("CM1")
        .await
        .expect("fetch failed");
    assert_eq!(model.id(), "CM1");
}

#[tokio::test]
async fn test_fetch_constraint_model_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/constraint/MISSING"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let client = Client::new(mock_server.uri())
        .expect("client creation failed");
    let result = client.fetch_constraint("MISSING").await;
    assert!(result.is_err());
}
```

**Rationale**:
- `wiremock` is deterministic (no real network calls)
- Responses are predictable (no flaky tests)
- Mock setup is explicit (easy to understand test intent)

**When NOT to mock**:
- Parser tests: test with real XML/JSON (fixtures, not mocks)
- Type construction: test with real instances, not mocks
- Serialisation: use actual output, not mocked output

### Avoiding Mock Drift

Keep mocks in sync with reality:
1. Derive mocks from real API responses (capture and store in fixtures)
2. Document mock assumptions in code:
   ```rust
   // MOCK: This response assumes the API returns 200 OK for valid requests.
   // If the API changes response format, update the mock response body.
   ```
3. Add integration tests that hit the real API (optional, for pre-release verification)

---

## Property-Based Testing & Fuzzing

### Domain-Type Property Tests (Phase 1)

`sdmx-types` carries an in-crate property suite: in-module `proptest!` blocks beside the example tests, with the shared lexeme generators in the crate-private `test_strategy` module. Three conventions govern it:

- **Generation routes through the validated constructors.** Strategies emit grammar-valid lexemes (or the components one is formatted from) and the properties construct values through `new()`/`from_str()`, the same single write path production code uses. A generator that bypassed the constructor would have the same defect as a `Deserialize` that did (design 0010 §7), and would spend its case budget on bizarre invariants that can never arrive in reality.
- **Property tests complement, never replace, the example tests.** The deterministic example tests enumerate every boundary and hit every branch identically on every run, so they remain the coverage backbone under the crate's 0%-tolerance coverage gate; a branch reached only probabilistically by a generator would make that gate flaky. Property tests add fuzzed breadth on top.
- **The property suite is wasm-excluded by design.** The properties verify platform-independent protocol invariants of pure `no_std + alloc` logic, so the same generated cases yield identical verdicts on every target and wasm re-execution verifies nothing new: wasm capability belongs to the `cargo check` gate, and wasm runtime behaviour to the example tests running there. `proptest` is therefore a `cfg(not(target_arch = "wasm32"))` dev-dependency, and the property modules carry the same target gate. Revisit trigger: any `cfg(target_arch)`-conditional code entering the library.

Discovered failing seeds are written to `src/proptest-regressions/` automatically and are committed, so a found failure replays on every subsequent run (the `.gitignore` allowlists the `.txt` seeds).

### Fuzzing Strategy (Phase 5)

Fuzz targets exercise parsers with random/malformed input to find panics and undefined behaviour:

```
crates/sdmx-parsers/fuzz/
├── Cargo.toml
├── fuzz_targets/
│   ├── parse_constraint_xml.rs
│   ├── parse_dataflow_json.rs
│   └── parse_csv_stream.rs
└── corpus/
    ├── parse_constraint_xml/
    │   ├── simple.xml
    │   └── complex.xml
    └── parse_dataflow_json/
        └── simple.json
```

A fuzz target:

```rust
// fuzz/fuzz_targets/parse_constraint_xml.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use sdmx_parsers::parse_constraint_model;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Parser must never panic; it should always return Result
        let _ = parse_constraint_model(s);
    }
});
```

Run fuzzing locally:

```bash
cargo fuzz run parse_constraint_xml -- -max_len=1000 -timeout=10
```

CI runs fuzzing as part of pre-release checks (ADR-0010).

### Roundtrip Property Tests (Phase 2+)

Use `proptest` to verify parse-serialize-parse equivalence:

```rust
#[cfg(test)]
mod prop_tests {
    use proptest::prelude::*;
    use sdmx_types::ConstraintModel;
    use sdmx_parsers::parse_constraint_model;
    use sdmx_writers::write_constraint_model;

    proptest! {
        #[test]
        fn prop_parse_serialize_roundtrip(model in arb_constraint_model()) {
            let xml = write_constraint_model(&model).expect("serialize failed");
            let parsed = parse_constraint_model(&xml).expect("parse failed");
            prop_assert_eq!(model, parsed);
        }
    }

    fn arb_constraint_model() -> impl Strategy<Value = ConstraintModel> {
        // Generate random but valid constraint models
        "[a-z]{1,10}".prop_map(|id| ConstraintModel::new(id))
    }
}
```

Rationale: Roundtrip tests catch serialisation bugs that unit tests miss.

---

## Coverage Expectations & Exceptions

### Coverage Floor: 80%

The project enforces 80% coverage across all crates (measured via `cargo-llvm-cov`). This is **not a goal**—it's a floor. Gaps below 80% are unacceptable unless explicitly documented.

### Per-Crate Exceptions

Some code is inherently hard to cover:

**sdmx-parsers (75% target with 1% tolerance)**:
- Streaming decoders have many rare branches (error conditions, malformed input)
- Some paths only trigger on specific byte sequences (hard to hit with fixtures)
- Example: `handle_rare_encoding_edge_case()` may have <50% coverage
- These exceptions are acceptable; see [codecov.yaml](../../codecov.yaml) for crate-specific thresholds

**sdmx-types (85% target with 0% tolerance)**:
- Domain types are the foundation; every branch must be tested
- Validation logic is critical; all error paths must be covered
- No threshold tolerance: any regression blocks merge (prevents coverage drift)

### Documenting Coverage Gaps

When intentionally leaving code uncovered, document it:

```rust
#[cfg_attr(coverage, no_coverage)]  // Rarely-hit error condition; covered by fuzzing
pub fn handle_corrupted_state() -> Result<()> {
    // ...
}
```

Or explain in a code comment:

```rust
// COVERAGE: This branch handles corrupted XML that violates the schema.
// Real-world violations are rare; fuzzing covers this indirectly.
// See fuzz/corpus/parse_constraint_xml/malformed.xml
if invalid_state {
    return Err(ParseError::CorruptedState);
}
```

Run coverage locally:

```bash
just coverage  # Generates HTML report
```

---

## Common Testing Patterns & Recipes

### Pattern 1: Testing Error Cases

Always test the error path:

```rust
#[test]
fn test_parse_missing_required_element() {
    let xml = r#"<ConstraintModel></ConstraintModel>"#;  // Missing Rules
    let result = parse_constraint_model(xml);
    assert!(result.is_err());

    // Verify the error type is specific
    match result {
        Err(ParseError::MissingElement(elem)) => {
            assert_eq!(elem, "Rules");
        }
        _ => panic!("unexpected error type"),
    }
}
```

### Pattern 2: Testing Invariants

Verify that invalid states are impossible:

```rust
#[test]
fn test_constraint_model_always_normalised() {
    let model = ConstraintModel::new("CM1");

    // The model should be in a normalised state immediately after construction
    // (all rules validated, no duplicates, etc.)
    assert!(model.is_normalised());
}
```

### Pattern 3: Testing Builder Pattern

Verify that builders enforce required fields:

```rust
#[test]
fn test_client_builder_requires_base_url() {
    // This should not compile (if the API is well-designed):
    // let client = ClientBuilder::new().build();  // Error: missing base_url

    // This should compile:
    let client = ClientBuilder::new("https://api.example.com")
        .timeout(Duration::from_secs(30))
        .build()
        .expect("client creation failed");

    assert_eq!(client.base_url(), "https://api.example.com");
}
```

### Pattern 4: Testing Async Code

Use `#[tokio::test]` for async tests:

```rust
#[tokio::test]
async fn test_fetch_with_timeout() {
    let client = create_test_client();

    let result = tokio::time::timeout(
        Duration::from_millis(100),
        client.fetch_constraint("CM1"),
    )
    .await;

    // Verify timeout behaviour
    assert!(result.is_err());
}
```

### Pattern 5: Testing no_std Compatibility

Verify code compiles in `no_std` environments:

```bash
cargo test --no-default-features --lib
```

This runs unit tests without `std`. Integration tests require `std` (they use the test harness).

---

## Running Tests

### Local Development

```bash
# Run all tests
just test

# Run tests for a specific crate
cargo test -p sdmx-parsers

# Run tests matching a pattern
cargo test parse_constraint

# Run with output (useful for debugging)
cargo test -- --nocapture

# Run a single test
cargo test test_parse_simple_constraint -- --exact
```

### Coverage Report (Local)

```bash
# Generate HTML coverage report
just coverage

# Opens in browser; navigate to `target/llvm-cov/html/index.html`
```

### CI Testing Matrix

CI runs tests on:
- **Stable Rust** (latest stable)
- **MSRV Rust** (current specified version)
- **WASM target** (wasm32-unknown-unknown, no_std)
- **Multiple OS** (Ubuntu, macOS, Windows)

All must pass before merge.

---

## Test Naming Conventions

Use descriptive names that explain what is being tested:

**Good**:
```rust
#[test]
fn test_parse_constraint_with_multiple_dimensions() { }

#[test]
fn test_constraint_parsing_fails_on_missing_rules() { }

#[test]
fn test_dataflow_builder_requires_structure_type() { }
```

**Avoid**:
```rust
#[test]
fn test_constraint() { }  // Too vague

#[test]
fn test_1() { }  // Meaningless

#[test]
fn it_works() { }  // Not descriptive
```

**Pattern**: `test_[unit]_[condition]_[expected_outcome]`
- `parse_constraint_with_multiple_dimensions` ✅
- `constraint_parsing_fails_on_missing_rules` ✅
- `dataflow_builder_requires_structure_type` ✅

---

## Test Maintenance

### Updating Tests When APIs Change

When you change a public function signature:
1. The compiler will show you all tests that need updating
2. Update the test code and assertions
3. If the test is no longer relevant, delete it (don't leave it skipped)

### Handling Flaky Tests

If a test fails intermittently:
1. **Identify the root cause** — usually related to timing, randomness, or test pollution
2. **Fix the root cause** — don't add sleeps or retries as a bandaid
3. **Add a comment** explaining why the test previously flaked and how it was fixed

Example:

```rust
#[tokio::test]
async fn test_concurrent_requests_are_isolated() {
    // FLAKINESS FIX: Previously failed due to test isolation issue.
    // Each test now creates its own mock server instance to avoid port conflicts.
    // See: https://github.com/dgalbraith/sdmx-rs/issues/42

    let mock_server = MockServer::start().await;  // Unique port per test
    // ...
}
```

### Skipping Tests Temporarily

Use `#[ignore]` for temporarily skipped tests (e.g., pending implementation):

```rust
#[test]
#[ignore = "Pending implementation of feature X; see issue #42"]
fn test_parse_csv_stream() {
    // ...
}
```

Run ignored tests explicitly:

```bash
cargo test -- --ignored
```

**Never use `#[ignore]` without documenting why or when to re-enable.**

---

## Shell Script Testing (BATS)

All BATS tests in `tests/bats/` must use the `run_isolated()` helper to execute commands with a clean, isolated environment. This prevents CI runner variables (e.g., `GITHUB_EVENT_NAME`) from leaking into tests and causing silent failures that only appear in CI.

```bash
# All test invocations use run_isolated()
run_isolated "./doc-engine.sh" add adr "My ADR"
run_isolated "scripts/check-maintenance.sh" --force
run_isolated "scripts/update-msrv.sh" --dry-run 1.91.0 1.92.0
```

**Environment Setup**:

Each test file's `setup()` function declares what environment variables it needs:

```bash
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"
    cd "$BATS_TEST_TMPDIR"

    # Declare test environment (exported once per test file)
    export MAINTENANCE_TODAY="2026-08-15"
}
```

The `run_isolated()` helper automatically passes through declared environment variables while clearing all others, ensuring test isolation.

**Rationale**: See [ADR-0020: Shell Script Test Environment Isolation](../adr/0020-shell-script-test-environment-isolation.md) for the decision drivers and risk analysis that led to this policy. In brief: CI-only test failures from environment variable pollution are expensive to debug and can be completely prevented with isolated environments.

---

## Testing no_std & WASM Targets

### no_std Unit Tests

Unit tests compile in `no_std` context (with `alloc`):

```bash
cargo test --no-default-features --lib
```

This verifies:
- No `std` imports leak into library code
- Allocations are explicit (via `alloc::vec::Vec`, `alloc::string::String`)
- Error handling doesn't rely on `std::panic`

### WASM Compilation Check

Integration tests require `std`, but crates must compile for WASM:

```bash
just check-wasm
```

This verifies:
- `sdmx-types` compiles for `wasm32-unknown-unknown`
- `sdmx-parsers` compiles for WASM
- `sdmx-writers` compiles for WASM
- Feature combinations work correctly

---

## Phase 1 Testing Checklist

Before marking a crate as "complete" in Phase 1:

- [ ] 80%+ coverage (or documented exceptions)
- [ ] Unit tests for all public functions
- [ ] Integration tests for major workflows
- [ ] Error cases tested (not just happy path)
- [ ] Doc tests included for complex APIs
- [ ] Fixtures organised and documented
- [ ] `just test` passes locally
- [ ] `just coverage` shows no coverage regressions
- [ ] `cargo test --no-default-features --lib` passes (no_std check)
- [ ] `just check-wasm` passes
- [ ] Tests run in CI without flakiness

---

## Known Limitations & Future Plans

Testing capabilities expand as the project matures — property-based roundtrip tests, `criterion` benchmarks, coverage-guided fuzzing, and end-to-end tests against real SDMX registries each land in the phase that introduces the code they exercise. [ROADMAP.md](../../ROADMAP.md) is the authoritative per-phase schedule; this guide documents the patterns and conventions for each once they are active.

---

## References

- [RFC 430 — Testing](https://rust-lang.github.io/api-guidelines/documentation.html#tests-are-informative)
- [ADR-0010: Parser Fuzzing Suite](../adr/0010-parser-fuzzing-suite-and-panic-profile-configuration.md)
- [ADR-0005: no_std + alloc Strategy](../adr/0005-adopt-no-std-with-alloc-for-sdmx-types-and-sdmx-parsers.md)
- [codecov.yaml Configuration](../../codecov.yaml)
- [Justfile Testing Recipes](../../Justfile)
