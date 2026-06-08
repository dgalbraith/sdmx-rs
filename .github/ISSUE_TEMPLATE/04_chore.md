---
name: Maintenance Chore
about: Perform a repository maintenance task, dependency bump, or CI update.
labels: chore
---
<!--
  SUMMARY
  Provide a crisp, two-to-three sentence summary of the chore or maintenance task.
  State the immediate technical goal and why this maintenance is necessary.
-->

## Key Deliverables

<!--
  Break the chore into concrete, reviewable deliverables.
  Each item should map roughly to a pull request or logical unit of work.
  Add or remove lines as needed — keep the `- [ ]` prefix so items render as checkboxes.
-->

- [ ] <!-- e.g., Bump target dependency versions in Cargo.toml -->
- [ ] <!-- e.g., Update CI action scripts or lint tool configurations -->
- [ ] <!-- e.g., Synchronize repository allow-lists or configs -->

## Verification

<!--
  Standard quality checks — edit only if this chore warrants additions or exceptions.
-->

- [ ] Code builds cleanly with zero warnings under `clippy::pedantic` (if code is modified).
- [ ] Verify dependency audit and license compliance gates pass cleanly (`just verify`).
- [ ] Automated CI pipelines (`verify`) pass 100% on the chore branch.

## Dependencies & References

<!--
  - List any issues that must be resolved before this one can begin.
  - Link relevant upgrade instructions or upstream dependency advisories below.
-->

- **Pre-requisites**: None <!-- List blocking issue numbers, e.g., #123 -->
- **Resources**: None <!-- Cite relevant upstream issues, security advisories, or specs -->
