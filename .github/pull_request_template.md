<!--
Please ensure your PR title follows Conventional Commits!
Examples:
  feat(sdmx-types): add DataConstraint model
  fix(sdmx-parsers): resolve xml namespace overlap
  chore: bump dependencies

For a complete worked exemplar of a pull request description, see:
docs/dev/workflow.md
-->

<!-- Provide a clear and concise description of the changes made in this PR. -->
<!-- Explain *why* this change is necessary if it's not immediately obvious from the issue. -->

## Key Changes

<!-- Provide a bulleted list of the main changes implemented. -->

- **[Crate/Area]**: Details...

## Related Documents

<!-- Link any ADRs, design docs, or guides that motivated or were updated by this PR. Omit if none. -->

## Quality Checklist

<!-- Please verify the following locally before submitting your PR -->
- [ ] I have run the unified quality gate (`just verify` or `nix develop --command just verify`) and all checks pass cleanly.
- [ ] I have added doc-comments (`///`) to any new or modified public API items.
- [ ] My commits are GPG-signed.

<!--
Breaking change? If this PR changes public API in a breaking way, your commit
message MUST carry `!` or a `BREAKING CHANGE:` footer — the release version bump
is derived from it, so a mislabelled `fix:` ships an undisclosed break under a
wrong version number. Nothing to tick here; just make sure the commit is right.
-->

<!-- Use 'Closes #ISSUE_ID' for features/chores, or 'Fixes #ISSUE_ID' for bug fixes as per CONTRIBUTING.md. -->
Closes #<!-- Issue ID -->
