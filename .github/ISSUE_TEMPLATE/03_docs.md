---
name: Documentation Task
about: Propose a documentation update, new guide, or API documentation improvements.
labels: docs
---
<!--
  SUMMARY
  Provide a crisp, two-to-three sentence summary of the documentation task.
  State the immediate technical goal and which guides or public APIs are affected.
-->

## Key Deliverables

<!--
  Break the documentation task into concrete, reviewable deliverables.
  Each item should map roughly to a pull request or logical unit of work.
  Add or remove lines as needed — keep the `- [ ]` prefix so items render as checkboxes.
-->

- [ ] <!-- e.g., Draft new section/guide inside docs/ -->
- [ ] <!-- e.g., Update existing README or contributing instructions -->
- [ ] <!-- e.g., Document public API items with rustdoc comments -->

## Verification

<!--
  Standard quality checks — edit only if this task warrants additions or exceptions.
-->

- [ ] Markdown files format and lint cleanly with zero warnings (`just verify`).
- [ ] Ensure all local internal markdown links are valid and resolve.
- [ ] Public API documentation generates without compiler warnings (`just docs`).

## Dependencies & References

<!--
  - List any issues that must be resolved before this one can begin.
  - Link relevant architectural decisions (ADRs) or design documents below.
-->

- **Pre-requisites**: None <!-- List blocking issue numbers, e.g., #123 -->
- **Resources**: None <!-- Cite relevant ADRs, design docs, or external URIs -->
