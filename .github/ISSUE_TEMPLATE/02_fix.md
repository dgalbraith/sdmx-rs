---
name: Bug Report
about: Report a bug or defect in the sdmx-rs framework.
labels: fix
---
<!--
  SUMMARY
  Provide a crisp, two-to-three sentence summary of the bug.
  State what is broken, when it happens, and the architectural impact if left unresolved.
-->

## Steps to Reproduce

<!--
  Describe exactly how to reproduce the unexpected behaviour.
-->

1. Go to '...'
2. Execute command '...'
3. See error '...'

**Minimal Reproducible Example:**

```rust
// Paste your standalone MRE code here
```

## Expected Behaviour

<!-- State what you expected to happen instead. -->

## Observed Behaviour

<!-- Attach error messages, compiler warnings, or stack traces here. -->

```text
[Paste logs, error messages, panic outputs compiler errors, or stack traces here]
```

## Key Deliverables

<!--
  Break the fix into concrete, reviewable deliverables.
  Each item should map roughly to a pull request or logical unit of work.
  Add or remove lines as needed — keep the `- [ ]` prefix so items render as checkboxes.
-->

- [ ] <!-- e.g., Add failing regression test demonstrating the bug -->
- [ ] <!-- e.g., Implement fix in the target component -->
- [ ] <!-- e.g., Verify all integration gates pass cleanly -->

## Verification

<!--
  Standard quality checks — edit only if this fix warrants additions or exceptions.
-->

- [ ] Code builds cleanly with zero warnings under `clippy::pedantic`.
- [ ] Add explicit unit/integration regression tests confirming the fix.
- [ ] Automated CI pipelines (`verify`) pass 100% on the fix branch.

## Dependencies & References

<!--
  - List any related issues or dependencies.
  - Detail environment specifics if they are relevant to reproducing the bug.
-->

- **Environment**:
  - **OS**: None <!-- e.g., Ubuntu 24.04, macOS Sequoia -->
  - **Rust Version**: None <!-- e.g., rustc 1.91.0 -->
  - **Crate & Version**: None <!-- e.g., sdmx-parsers v0.2.0, or main at commit abc1234 -->
- **Pre-requisites**: None <!-- List blocking issue numbers, e.g., #123 -->
- **Resources**: None <!-- Cite relevant spec sections or external URIs -->
