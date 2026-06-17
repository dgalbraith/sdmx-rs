# Maintenance Policy & Obligation Tracking

The sdmx-rs project uses a maintenance ledger system to codify periodic obligations that keep the project healthy. This document explains what maintenance obligations are, why they matter, and how to manage them.

## What is Maintenance?

Maintenance work is periodic tasks that don't fit into the normal feature/bugfix cycle. Examples:
- Updating the nightly Rust toolchain version pin
- Reviewing if the MSRV (Minimum Supported Rust Version) can be bumped
- Running security audits on dependencies
- Re-baselining performance benchmarks
- Reviewing linting rules for promotion
- Evaluating test coverage thresholds

These tasks are easy to defer and hard to notice when they're overdue. The maintenance ledger makes them explicit and enforced.

## System Design

The maintenance system has three parts:

### 1. maintenance.toml — Central Ledger

A single TOML file listing all maintenance obligations:

```toml
[[maintenance]]
item = "rustfmt-date-bump"
file = "flake.nix"
marker = "# MAINTENANCE: rustfmt-date-bump"
review_cadence = 60
warn_threshold = 90
fail_threshold = 120
last_updated = "2026-05-01"
next_review = "2026-06-30"
```

**Fields**:
- `item`: identifier (slug format, unique)
- `file`: where this work happens
- `marker`: comment marker to find inline breadcrumb
- `review_cadence`: days between reviews
- `warn_threshold`: warn if not updated this many days
- `fail_threshold`: fail CI if overdue this many days
- `last_updated`: when work was last done (ISO 8601)
- `next_review`: when to review next (ISO 8601)

**Source of truth**: maintenance.toml is authoritative. Inline comments must match.

### 2. Inline Comments — Local Context

Every maintenance item has a breadcrumb comment in its source file:

```nix
# MAINTENANCE: rustfmt-date-bump
# Last updated: 2026-05-01
# Next review: 2026-06-30
# Description: Nightly rustfmt is pinned; update periodically.
nightly-fmt = inputs.rust-overlay.lib.toRustVersion {
  rustVersion = "nightly-2026-05-01";
  ...
};
```

**Format**:
```
# MAINTENANCE: <item-slug>
# Last updated: YYYY-MM-DD
# Next review: YYYY-MM-DD
# Description: [optional one-line summary]
```

**Why**: Preserves context when someone edits that file. Visible during code review.

### 3. check-maintenance.sh — Enforcement

Validates that maintenance obligations are tracked and deadlines are met:

```bash
bash scripts/check-maintenance.sh        # Check now
bash scripts/check-maintenance.sh --dry-run  # Preview without enforcing
```

**What it validates**:
1. Every item in maintenance.toml has a matching inline comment
2. Inline dates match maintenance.toml (catch drift)
3. next_review has not passed (fail if overdue)
4. Days since last_updated < warn_threshold (warn if stale)

**Exit codes**:
- `0`: All items tracked and current (or approaching deadline)
- `1`: Item(s) overdue or missing comments

## Operational Workflow

### When an item comes due

**You see**: CI fails with `❌ rustfmt-date-bump: OVERDUE (next review was 2026-06-30)`

**You do**:

1. Perform the maintenance work (e.g., update flake.nix with new nightly date)
2. Use the helper to update dates:
   ```bash
   scripts/maintenance-bump.sh rustfmt-date-bump
   ```
   This updates both `maintenance.toml` and the inline comment automatically.

3. Verify locally:
   ```bash
   bash scripts/check-maintenance.sh
   ```
   Should show: `✅ rustfmt-date-bump (updated 0 days ago, review due 2026-07-30)`

4. Commit:
   ```bash
   git add maintenance.toml flake.nix
   git commit -m "chore: bump rustfmt nightly date to 2026-06-30"
   ```

5. CI passes.

### Adding a new maintenance obligation

1. Identify what needs periodic review
2. Add entry to `maintenance.toml`:
   ```toml
   [[maintenance]]
   item = "new-obligation"
   file = "path/to/file"
   marker = "# MAINTENANCE: new-obligation"
   review_cadence = 60
   warn_threshold = 90
   fail_threshold = 120
   last_updated = "2026-05-23"
   next_review = "2026-07-23"
   ```

3. Add inline comment to the source file at the relevant location:
   ```
   # MAINTENANCE: new-obligation
   # Last updated: 2026-05-23
   # Next review: 2026-07-23
   # Description: Why this needs periodic review
   ```

4. Test:
   ```bash
   bash scripts/check-maintenance.sh
   ```

5. Commit both files.

### Checking synchronisation

Detect if maintenance.toml and inline comments are out of sync:

```bash
bash scripts/maintenance-sync.sh
```

This is useful when refactoring or moving files. Reports drift without auto-correcting.

## Phase 2+ Items

Items that apply only to future project phases are commented out in maintenance.toml:

```toml
# [[maintenance]]
# item = "benchmark-baseline-refresh"
# file = "crates/sdmx-parsers/benches/README.md"
# review_cadence = 90
# last_updated = "2026-05-23"
# next_review = "2026-08-23"
```

When Phase 2 starts:
1. Uncomment the item
2. Set `last_updated` and `next_review` to Phase 2 start date
3. Add inline comment to the source file
4. Commit

## Current Maintenance Items

| Item                           | File                         | Cadence  | Threshold                  |
|--------------------------------|------------------------------|----------|----------------------------|
| **rustfmt-date-bump**          | flake.nix                    | 60 days  | warn at 90d, fail at 120d  |
| **msrv-upgrade-window**        | Cargo.toml                   | 180 days | warn at 210d, fail at 270d |
| **dependency-audit**           | .github/workflows/ci.yml     | 90 days  | warn at 150d, fail at 210d |
| **linting-rules-review**       | Justfile                     | 90 days  | warn at 150d, fail at 180d |
| **coverage-threshold-review**  | codecov.yaml                 | 90 days  | (Phase 1+)                 |
| **signing-subkey-renewal**     | verify-signature.yml         | deadline | review by 2027-10-01       |
| **benchmark-baseline-refresh** | crates/sdmx-parsers/benches/ | 90 days  | (Phase 2+)                 |

### Item Details & Execution Routines

#### signing-subkey-renewal

* **Type**: Deadline-driven (not a rolling cadence). The maintainer's GPG signing subkey `[S]` has a fixed expiry (currently 2027-12-31). If it expires unrotated, `verify-signature.yml` stops emitting `VALIDSIG` and **every push to `main` fails** — this obligation exists to prevent that.
* **Trigger**: `next_review` is set to ~90 days before the `[S]` expiry (currently 2027-10-01). The weekly scheduled job opens a renewal tracking issue once that date passes, giving runway to rotate. The `warn`/`fail` day-count thresholds are set wide deliberately so the staleness check does not fire before the deadline-based review date does.
* **Breadcrumb location**: Beside the fingerprint allowlist in `.github/workflows/verify-signature.yml` — the trust root that fails on expiry. A date-free cross-reference also appears in [forge-setup.md](forge-setup.md) next to the `[S] Expiry` register.
* **Remediation workflow**:
  1. Rotate the signing subkey following [forge-setup.md → Signing subkey rotation](forge-setup.md#signing-subkey-rotation) (generate new `[S]`, re-export and commit the `.asc`, re-upload to GitHub). The **primary** fingerprint in the allowlist is unchanged by rotation, so `verify-signature.yml`'s allowlist needs no edit.
  2. Bump the obligation: `scripts/maintenance-bump.sh signing-subkey-renewal`.
  3. **Important**: `maintenance-bump.sh` resets `next_review` to today + `review_cadence`, which is wrong for a fixed-deadline item. Manually correct `next_review` (in both `maintenance.toml` and the `verify-signature.yml` breadcrumb) to ~90 days before the **new** subkey's expiry, and update the `[S] Expiry` column in the [forge-setup.md](forge-setup.md) key register.
  4. Run `bash scripts/check-maintenance.sh` to confirm the item is current, then GPG-sign and commit.

#### dependency-audit

* **Frequency**: 90 days (or immediately when a security advisory is received).
* **Monitoring**: GitHub Dependabot Alerts are configured in monitor-only (alerts-only) mode.
* **Remediation workflow**:
  1. Triage alerts under the GitHub **Security and quality** tab or via local `just audit-all` checks.
  2. Resolve the vulnerability using the following hierarchy:
     * **Direct Dependencies**: Run `cargo update -p <vulnerable-crate> --precise <patched-version>`.
     * **Transitive Dependencies**:
       1. Trace parent chain: `cargo tree -i <vulnerable-crate>`.
       2. Force update the transitive dependency directly: `cargo update -p <vulnerable-crate> --precise <patched-version>`.
       3. If blocked by parent constraints, update the parent dependency: `cargo update -p <parent-crate>`.
       4. Override temporarily in the root `Cargo.toml` `[patch.crates-io]` section if no upstream updates exist.
       5. If no resolution exists and the CVE is non-exploitable, document the rationale and add the advisory ID to the ignore list in `deny.toml`.
  3. Execute `just verify` locally to ensure formatting is clean, the suite compiles on MSRV, WASM checks pass, and tests succeed.
  4. GPG-sign and commit the changes, then push to `main`.
  5. Run `scripts/maintenance-bump.sh dependency-audit` to bump the maintenance ledger.

## CI Integration

`check-maintenance.sh` is run as an informational check in continuous integration:

**CI behaviour**:
- **On Pull Requests**: The check is informational (exits 0). This prevents blocking external contributors while maintaining visibility of maintenance status.
- **On default branch (`main`) pushes (post-merge)**: The check is informational (exits 0). No blocks — visibility only.
- **Scheduled Weekly Check**: A GitHub Actions cron job runs every **Saturday at 00:00 UTC**.
  - If any maintenance item is overdue, the job automatically creates a tracking GitHub Issue labelled `maintenance` specifying the exact overdue items.
  - No duplicate issues will be created if a tracking issue is already open.
  - This is the **primary enforcement mechanism** — maintainers see overdue work via GitHub Issues and prioritise accordingly.

This approach maintains discipline (obligations are visible and tracked) without false CI gates. Maintainers see weekly Issues and handle work through their normal workflow.

## Resolution Workflow & Issue Tracking

When the scheduled CI creates a tracking issue for an overdue obligation:

### Workflow

1. **CI detects overdue obligation** (Saturday 00:00 UTC)
   - Creates GitHub Issue labelled `maintenance`
   - Example issue title: `chore(maintenance): Scheduled Maintenance Review Overdue`
   - Issue body lists all overdue items

2. **Maintainer picks up the issue**
   - Reviews the overdue obligation(s)
   - Performs the maintenance work (e.g., update toolchain pin, audit dependencies)
   - Creates a feature branch: `git checkout -b chore/maintenance-<item-name>`

3. **Update maintenance dates and commit**
   - Use the helper to bump dates in both `maintenance.toml` and inline comments:
     ```bash
     scripts/maintenance-bump.sh <item>
     ```
   - Commit with conventional type `chore(maintenance):` and issue reference:
     ```bash
     git commit -m "chore(maintenance): update rustfmt pin to 2026-07-15

     Reviewed and updated nightly rustfmt version pin.

     Closes #42"
     ```
   - The `Closes #ISSUE_NUMBER` in the commit footer tells GitHub to auto-close the tracking issue when the commit is merged

4. **Merge to main**
   - Create PR, merge locally to main with GPG signature (per [MERGING.md](merging.md))
   - Push to GitHub: `git push origin main`
   - GitHub automatically closes the tracking issue when it detects the `Closes #42` metadata in the merged commit

5. **Next scheduled CI run (following Saturday)**
   - Validates the obligation is now current
   - No new issue is created if the obligation is resolved
   - The closed issue serves as an audit trail of when the work was done

### Example

```
Overdue item: rustfmt-date-bump
- GitHub Issue #42 created: "chore(maintenance): Scheduled Maintenance Review Overdue"
- Maintainer: updates flake.nix nightly pin from 2026-05-01 to 2026-07-15
- Commit: git commit -m "chore(maintenance): update rustfmt pin to 2026-07-15 — Closes #42"
- Merge: Local merge to main, push to GitHub
- Result: Issue #42 auto-closes; maintenance obligation resolved
- Audit: Git history shows when and by whom the obligation was fulfilled
```

This design uses GitHub's native issue workflow while ensuring obligations are tracked and resolved predictably.

## Troubleshooting

### "❌ marker not found"

The inline comment is missing from the source file.

**Fix**: Add the inline comment:
```
# MAINTENANCE: <item>
# Last updated: YYYY-MM-DD
# Next review: YYYY-MM-DD
```

### "❌ date mismatch"

The inline comment has different dates than maintenance.toml.

**Fix**: Run helper to re-sync:
```bash
scripts/maintenance-bump.sh <item>
```

Or manually edit both files to match.

### "❌ OVERDUE"

The next_review date has passed.

**Fix**: Do the maintenance work and update:
```bash
scripts/maintenance-bump.sh <item>
```

### "⚠️ stale"

The item hasn't been reviewed in warn_threshold days (but not yet overdue).

**Fix**: Review and update soon:
```bash
scripts/maintenance-bump.sh <item>
```

### "❌ source file not found"

The file path in maintenance.toml doesn't exist.

**Fix**: Either update maintenance.toml with the correct path, or update the file path if it moved.

## See Also

- `.just verify` recipe (includes check-maintenance)
- [maintenance.toml](../../maintenance.toml) (the ledger)
- [scripts/check-maintenance.sh](../../scripts/check-maintenance.sh) (validator)
- [scripts/maintenance-bump.sh](../../scripts/maintenance-bump.sh) (helper)
- [scripts/update-msrv.sh](../../scripts/update-msrv.sh) (automated MSRV upgrade)
