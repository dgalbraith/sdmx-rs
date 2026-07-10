# sdmx-rs developer tasks
set shell := ["bash", "-euo", "pipefail", "-c"]

# Parameter-naming convention:
#   UPPER  — a variadic list of named DOMAIN entities the recipe understands
#            (e.g. `*CRATES`). Empty means "all": pass `{{CRATES}}` straight
#            through and let the script default via `get_crates "$@"` (single
#            source of truth in common.sh), NOT an in-recipe `{{if}}` branch.
#   lower  — a SCALAR (`version`, `target`) or an OPAQUE passthrough forwarded
#            verbatim to a script that does its own parsing (`*args` →
#            update-msrv.sh handles --downgrade/--dry-run). Lower-case signals
#            "the Justfile does not interpret this", upper signals "this is a
#            domain list the Justfile/script reasons about".

# Remote hosting the canonical `main` branch — used as the baseline for commit
# linting and as the target for release push/pull operations. Defaults to the
# conventional `origin`; override for mirrored-forge setups that name it
# differently (e.g. `SDMX_MAIN_REMOTE=github just verify`). The scripts read the
# same variable directly, so a single override is honoured everywhere.
# Do not hardcode `origin`: this is what makes a forge migration (e.g. to
# Codeberg) a one-variable routing change rather than a multi-file edit.
main_remote := env_var_or_default("SDMX_MAIN_REMOTE", "origin")

default:
    @just --list

# ============================================================================
# 1. Environment Setup
# ============================================================================

# One-shot local onboarding: register git hooks, then verify the environment. Idempotent.
setup: hook-install
    @echo ""
    @just doctor-env

# Install and configure local git hooks across all pipelines (commit, message, push)
hook-install:
    pre-commit install --hook-type pre-commit --hook-type commit-msg --hook-type pre-push

# ============================================================================
# 2. Unified Verification & Quality Gates
# ============================================================================

# Fast verification gate for active development (subset of full verify)
verify-minimal:
    cargo check --workspace --locked
    cargo clippy --workspace --locked -- -D warnings
    cargo nextest run --workspace --locked

# Run the complete verification quality gate in parallel (CI equivalent)
[parallel]
verify: verify-rust verify-wasm verify-scripts verify-docs verify-security verify-infra verify-maintenance
    @./scripts/lib/log.sh log_ok "verify: all verification phases passed"

# Run all verification gates sequentially (useful for detailed tracing/debugging)
verify-linear: verify-rust verify-wasm verify-scripts verify-docs verify-security verify-infra verify-maintenance
    @./scripts/lib/log.sh log_ok "verify: all verification phases passed"

# Verify Rust codebase formatting, clippy, public + internal docs build + doctests, semver, and test coverage
verify-rust: check-format clippy check-conventions docs docs-internal test-doc semver-check coverage-gate release-dry-run
    @./scripts/lib/log.sh log_ok "verify-rust: all gates passed"

# Verify repository shell script linting and testing
verify-scripts: shellcheck verify-test-manifests test-scripts check-shebangs
    @./scripts/lib/log.sh log_ok "verify-scripts: all gates passed"

# Verify documentation integrity: code comments, commit messages, markdown, ADRs, design docs
verify-docs: check-commits _verify-adr-quiet _verify-design-quiet _verify-guide-quiet md-check link-check check-decision-refs check-xsd-fragments verify-ci-gate
    @./scripts/lib/log.sh log_ok "verify-docs: all gates passed"

# Verify security & supply chain: secret leak scan, dependency advisories/licenses, unused dependencies
verify-security: secrets-scan deny machete
    @./scripts/lib/log.sh log_ok "verify-security: all gates passed"

# Verify infrastructure (Nix flake and GitHub Actions workflows) integrity
verify-infra: nix-check test-workflows
    @./scripts/lib/log.sh log_ok "verify-infra: all gates passed"

# Verify maintenance obligations are tracked and up-to-date
verify-maintenance: check-scaffolding check-maintenance
    @./scripts/lib/log.sh log_ok "verify-maintenance: all gates passed"

# --- verify-docs sub-gate helpers ---

# Validate commit messages follow Conventional Commits specification
check-commits:
    @commitlint --from {{ main_remote }}/main --to HEAD
    @./scripts/lib/log.sh log_ok "check-commits: commit messages follow Conventional Commits"

# Validate documentation links: reachability plus absolute-file:// ban
link-check: md-link-check local-link-check

# Validate that all Markdown documentation contains active, working links.
# The generated xsd-fragments are excluded: gen-xsd-fragments rewrites them in
# place (racing this scan under `[parallel] verify`) and check-xsd-fragments
# already verifies them byte-exact against the pinned schemas.
md-link-check:
    @find . -name '*.md' -not -path '*/templates/*' -not -path '*/target/*' -not -path '*/.direnv/*' -not -path '*/docs/xsd-fragments/*' -print0 | xargs -0 lychee --offline --no-progress

# Ban machine-specific absolute file:// links across Markdown, TOML and Rust
local-link-check:
    @./scripts/check-local-links.sh

# Validate every decision-register reference (D-NNNN) in crate source resolves to docs/decisions.md
check-decision-refs:
    @./scripts/check-decision-refs.sh

# --- verify-scripts sub-gate helpers ---

# Validate that all scripts in scripts/ use the POSIX-portable #!/bin/sh shebang
check-shebangs:
    @./scripts/check-shebangs.sh

# Verify update-msrv test manifest lists all workspace crate Cargo.toml files
verify-test-manifests:
    @./scripts/check-test-manifests.sh

# --- verify-infra sub-gate helpers ---

# Validate GitHub Action workflows for syntax errors and SHA-pinning compliance
test-workflows: verify-ci-gate
    @actionlint

# Cross-check the CI Quality Gate: the ci-gate needs: list and ci-gating.md against the gating manifest
verify-ci-gate:
    @./scripts/verify-ci-gate.sh

# --- verify-maintenance sub-gate helpers ---

# Validate maintenance obligations are tracked and deadlines met
check-maintenance:
    @./scripts/check-maintenance.sh --force

# Validate that ignored dependencies are properly scaffolded and documented
check-scaffolding:
    @./scripts/check-scaffolding.sh

# ============================================================================
# 3. Code Quality, Formatting & Linting
# ============================================================================

# Formatting, style, and static analysis diagnostics guide
lint-help:
    @echo "🧹 sdmx-rs Linting & Style"
    @echo ""
    @echo "Run all format and static analysis checks:"
    @echo "  just lint                   # Run all linting and formatting checks"
    @echo ""
    @echo "Rust:"
    @echo "  just fmt                    # Format Rust and TOML files under nightly rules"
    @echo "  just check-format           # Check formatting of Rust code and TOML manifests"
    @echo "  just check                  # Type-check all workspace packages without producing binaries"
    @echo "  just clippy                 # Run strict clippy static analysis (pedantic, nursery)"
    @echo "  just docs                   # Build documentation and check for comment warnings"
    @echo "  just docs-internal [pkgs]   # Build internal docs (design_docs notes + private items)"
    @echo ""
    @echo "Formats:"
    @echo "  just toml-fmt               # Format workspace TOML files"
    @echo "  just toml-check             # Validate formatting of workspace TOML manifests"
    @echo "  just md-fmt                 # Format workspace Markdown files"
    @echo "  just md-check               # Lint Markdown files for structure and style"
    @echo "  just shellcheck             # Lint project shell scripts (scripts/*.sh)"

# Run all local style, formatting, and static analysis checks sequentially
lint: check-format clippy md-check shellcheck

# === Rust ===

# Format all Rust files and TOML manifests using project-pinned nightly rules
fmt: toml-fmt
    @./scripts/run-fmt.sh

# Check formatting standards for all Rust code and TOML manifests
check-format: toml-check
    cargo fmt --check

# Run cargo check on all workspace packages and targets
check:
    cargo check --workspace --all-targets --locked

# MAINTENANCE: linting-rules-review
# Last updated: 2026-05-30
# Next review: 2026-08-28
# Evaluate clippy pedantic/nursery lints for promotion to default-warn
# Run strict clippy static analysis under warnings-as-errors and pedantic lints
clippy:
    cargo clippy --workspace --all-targets --locked -- -D warnings

# Enforce greppable source conventions (typed None, empty vec) that no clippy lint covers, across crate source
check-conventions:
    @./scripts/check-conventions.sh

# Generate workspace documentation and verify all public items carry doc comments without warnings
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --locked --quiet

# Generate internal docs: the design_docs rationale layer plus private items (default whole workspace; pass `-p <crate>` to scope). Fetches the pinned schemas + regenerates the fragments first (design_docs include_str!s them).
docs-internal pkgs="--workspace": fetch-specs gen-xsd-fragments
    RUSTDOCFLAGS="--cfg design_docs -D warnings" cargo doc {{ pkgs }} --no-deps --document-private-items --all-features --locked --quiet

# === Formats ===

# Format all TOML files in the workspace
toml-fmt:
    RUST_LOG=warn taplo fmt

# Check formatting of all TOML files in the workspace
toml-check:
    RUST_LOG=warn taplo fmt --check

# Fix Markdown formatting issues automatically
md-fmt:
    markdownlint-cli2 --fix "**/*.md" "#target" "#.direnv" "#.git" "#node_modules"

# Lint all Markdown documentation files for structure and syntax style.
md-check:
    markdownlint-cli2 "**/*.md" "#target" "#.direnv" "#.git" "#node_modules"

# Lint all repository shell scripts for errors and bad practices (two-tier severity: .sh full, .bats warning+)
shellcheck:
    @find scripts tests/bats -type f \( -name '*.sh' -o -name '*.bash' \) -print0 | xargs -0 shellcheck
    @find tests/bats -type f -name '*.bats' -print0 | xargs -0 shellcheck -x --severity=warning

# ============================================================================
# 4. Testing & Coverage
# ============================================================================

# Testing and coverage diagnostics guide
test-help:
    @echo "🧪 sdmx-rs Testing & Coverage"
    @echo ""
    @echo "Unit and integration tests:"
    @echo "  just test                   # Run all workspace unit and doc tests"
    @echo "  just test-scripts           # Run BATS tests for shell scripts"
    @echo "  just test-wasm              # Execute the no_std crates' WASM subset under Node/V8"
    @echo ""
    @echo "Code coverage:"
    @echo "  just coverage               # Generate interactive HTML coverage report (local)"
    @echo "  just test-coverage-headless # Standard CI gate: workspace-run, Codecov-matching per-crate floors"
    @echo "  just test-coverage-strict   # Strict audit: isolated per-crate suites, no cross-crate spillover"
    @echo ""

# Run all unit and doc tests in the workspace
test: && test-doc
    cargo nextest run --workspace --locked

# Run the workspace doctests (nextest does not execute them)
test-doc:
    cargo test --doc --workspace --locked

# Run BATS tests for shell scripts and maintenance system (summarised)
test-scripts:
    @./scripts/run-bats.sh tests/bats/

# Run source-based test coverage and generate an interactive local HTML report (local developers only; omit --open in CI/headless)
coverage:
    cargo llvm-cov --workspace --locked --html --open

# Standard coverage gate: workspace-run, per-crate floors, emits lcov.info
test-coverage-headless:
    @COVERAGE_REPORT=1 ./scripts/ci/run-coverage.sh

# verify gate variant of test-coverage-headless: identical floors, table-suppressed for clean verify output
coverage-gate:
    @./scripts/ci/run-coverage.sh

# Strict isolated coverage: per-crate suites only, no cross-crate spillover. Not wired into verify.
test-coverage-strict:
    cargo llvm-cov --package sdmx-types   --locked --fail-under-lines 85
    cargo llvm-cov --package sdmx-writers --locked --fail-under-lines 80
    cargo llvm-cov --package sdmx-client  --locked --fail-under-lines 80
    cargo llvm-cov --package sdmx-parsers --locked --fail-under-lines 75
    cargo llvm-cov --package sdmx-rs      --locked --fail-under-lines 70

# ============================================================================
# 5. Compliance, Audit & Portability
# ============================================================================

# Compliance, auditing, and dependency diagnostics guide
audit-help:
    @echo "🔒 sdmx-rs Audit, Compliance & Portability"
    @echo ""
    @echo "Run all dependency audits:"
    @echo "  just audit-all              # Run deny, machete, and outdated audits"
    @echo ""
    @echo "Dependency audits:"
    @echo "  just deny                   # Check licenses, sources, and advisories"
    @echo "  just machete                # Scan for unused workspace dependencies"
    @echo "  just outdated               # List outdated workspace crate versions"
    @echo "  just audit-safety           # Audit dependency tree for unsafe Rust usage"
    @echo ""
    @echo "  (to remediate stale deps: just update-deps / update-flake — see maintain-help)"
    @echo ""
    @echo "Portability & Compliance checks:"
    @echo "  just semver-check           # Verify semantic versioning API compliance"
    @echo "  just check-wasm             # Verify the no_std crates compile for the WASM target"
    @echo "  just msrv-features          # Verify MSRV (declared in rust-toolchain.toml) across feature combinations"
    @echo "  just nix-check              # Validate Nix Flake schema and evaluation"
    @echo "  just bloat [target]         # Profile compile size for a build target"

# Run all local compliance, dependency, and security audits
audit-all: deny machete outdated

# Scan git history for committed secrets (keys, tokens, private keys)
secrets-scan:
    @./scripts/ci/secrets-scan.sh

# Check dependencies against denylist security advisories and version rules
deny:
    cargo deny check

# Detect unused/dead dependencies in the workspace Cargo manifests
machete:
    cargo machete

# Check for outdated dependencies in the workspace
outdated:
    cargo outdated --workspace --depth 1

# Audit dependency tree for unsafe code (advisory only — non-failing; unsafe in deps is reviewed separately)
audit-safety:
    @cargo geiger --manifest-path "$PWD/crates/sdmx-rs/Cargo.toml" --all-features \
        || ./scripts/lib/log.sh log_info "audit-safety: cargo geiger exited non-zero (advisory only — unsafe in deps is reviewed separately, not a gate failure)"

# Verify semantic versioning compliance across workspace
semver-check:
    @./scripts/check-semver.sh

# Verify no_std crates compile for wasm32; also checks parsers-only native target and test harness (--no-run)
check-wasm:
    @./scripts/lib/log.sh log_section "Checking the workspace compiles for the WASM target..."
    @cargo check -p sdmx-types -p sdmx-parsers -p sdmx-writers --target wasm32-unknown-unknown --locked --quiet
    @cargo check -p sdmx-rs --target wasm32-unknown-unknown --no-default-features --locked --quiet
    @cargo check -p sdmx-rs --target wasm32-unknown-unknown --no-default-features --features parsers --locked --quiet
    @cargo check -p sdmx-rs --no-default-features --features parsers --locked --quiet
    @cargo test -p sdmx-rs --no-default-features --features parsers --no-run --locked --quiet
    @./scripts/lib/log.sh log_ok "check-wasm: workspace compiles for the WASM target"

# Verify the wasm target end-to-end: compile check plus headless Node/V8 test execution
verify-wasm: check-wasm test-wasm
    @./scripts/lib/log.sh log_ok "verify-wasm: all gates passed"

# Execute the no_std crates' wasm test subset under Node/V8 (wasm-pack + nodejs)
test-wasm:
    @./scripts/run-wasm-tests.sh

# Verify MSRV compatibility across feature combinations (no-default + all-features); manual/scheduled check only
msrv-features:
    cargo check --all --no-default-features --locked
    cargo check --all --all-features --locked

# Validate Nix Flake schema and outputs (--quiet + grep suppress progress trace and benign cross-platform warnings)
nix-check:
    @nix flake check --quiet 2> >(grep -v -E "warning: The check omitted these incompatible systems|Use '--all-systems' to check all" >&2)

# Analyse compiled binary size and functions taking up the most space
bloat target="wasm32-unknown-unknown":
    cargo bloat --workspace --target {{target}} --release

# ============================================================================
# 6. Fuzzing & Performance Benchmarking
# ============================================================================

# Run a specific cargo-fuzz target (e.g. just fuzz parse_xml)
fuzz target:
    cargo fuzz run {{target}}

# Run a fuzz target for a short smoke test (10 seconds) to verify compilation
fuzz-check target:
    @./scripts/run-fuzz-check.sh {{target}}

# Run all performance benchmarks in the workspace
bench:
    cargo bench --workspace

# ============================================================================
# 7. Document Management
# ============================================================================

# Document management diagnostic guide (lists checks to run)
docs-help:
    @echo "📝 sdmx-rs Document Management"
    @echo ""
    @echo "Architecture Decision Records (ADRs):"
    @echo "  just adr <title>                  # Create a new ADR"
    @echo "  just adr-rename <old> <new_title> # Rename an ADR and update ledger"
    @echo "  just adr-remove <target>          # Remove an ADR"
    @echo "  just verify-adr                   # Verify ADR ledger formatting"
    @echo ""
    @echo "Design Documents:"
    @echo "  just design <title>               # Create a new Design Doc"
    @echo "  just design-rename <old> <new>    # Rename a Design Doc and update ledger"
    @echo "  just design-remove <target>       # Remove a Design Doc"
    @echo "  just verify-design                # Verify Design Doc ledger formatting"
    @echo ""
    @echo "User Guides:"
    @echo "  just guide <title>                # Create a new User Guide"
    @echo "  just guide-rename <old> <new>     # Rename a User Guide and update ledger"
    @echo "  just guide-remove <target>        # Remove a User Guide"
    @echo "  just verify-guide                 # Verify User Guide ledger formatting"
    @echo ""
    @echo "XSD Contract Fragments (design_docs layer):"
    @echo "  just fetch-specs                  # Materialise the pinned SDMX schemas (fetch + sha-verify)"
    @echo "  just gen-xsd-fragments            # Regenerate fragments from xsd-manifest.toml (apply)"
    @echo "  just check-xsd-fragments          # Verify fragments are wired to their types (doctor)"
    @echo ""

# Create a new Architecture Decision Record using the custom MADR template (non-interactive)
adr title:
    @./scripts/doc-engine.sh add adr "{{title}}"

# Safely remove an Architecture Decision Record (ADR) interactively
adr-remove target:
    @./scripts/doc-engine.sh remove adr "{{target}}"

# Safely rename an Architecture Decision Record (ADR) interactively
adr-rename old_target new_title:
    @./scripts/doc-engine.sh rename adr "{{old_target}}" "{{new_title}}"

# Verify the completeness and formatting of the Architecture Decision Record (ADR) ledger
verify-adr:
    @./scripts/doc-engine.sh verify adr

[private]
_verify-adr-quiet:
    @./scripts/doc-engine.sh verify adr --quiet

# Create a new Design Document
design title:
    @./scripts/doc-engine.sh add design "{{title}}"

# Safely remove a Design Document interactively
design-remove target:
    @./scripts/doc-engine.sh remove design "{{target}}"

# Safely rename a Design Document interactively
design-rename old_target new_title:
    @./scripts/doc-engine.sh rename design "{{old_target}}" "{{new_title}}"

# Verify the completeness and formatting of the Design Document ledger
verify-design:
    @./scripts/doc-engine.sh verify design

[private]
_verify-design-quiet:
    @./scripts/doc-engine.sh verify design --quiet

# Create a new User Guide
guide title:
    @./scripts/doc-engine.sh add guide "{{title}}"

# Safely remove a User Guide interactively
guide-remove target:
    @./scripts/doc-engine.sh remove guide "{{target}}"

# Safely rename a User Guide interactively
guide-rename old_target new_title:
    @./scripts/doc-engine.sh rename guide "{{old_target}}" "{{new_title}}"

# Verify the completeness and formatting of the User Guides ledger
verify-guide:
    @./scripts/doc-engine.sh verify guide

[private]
_verify-guide-quiet:
    @./scripts/doc-engine.sh verify guide --quiet

# --- XSD contract fragments (design_docs) ---
# Pipeline: fetch-specs (materialise the pinned schemas) -> gen-xsd-fragments
# (generate the fragments into the real $OUT) -> check-xsd-fragments (verify
# wiring; verify-docs gate). The schemas + fragments are fetched/generated on
# demand (never committed), so gen + the gates declare the chain as dependencies.

# Materialise the pinned SDMX schemas on demand (Nix FOD fetch + per-file sha-verify; idempotent)
fetch-specs:
    @./scripts/fetch-specs.sh

# Generate sdmx-types XSD contract fragments (apply; run when adding a manifest entry or re-pinning)
gen-xsd-fragments: fetch-specs
    @./scripts/gen-xsd-fragments.sh

# Verify the XSD contract fragments are correctly wired to their types (doctor).
check-xsd-fragments: fetch-specs gen-xsd-fragments
    @./scripts/check-xsd-fragments.sh

# ============================================================================
# 8. Diagnostics (`just doctor` System)
# ============================================================================

# Project health diagnostic guide (lists checks to run)
doctor:
    @echo "🏥 sdmx-rs Project Diagnostics"
    @echo ""
    @echo "Nix essentials:"
    @echo "  just doctor-devshell         # Verify all declared Nix packages available"
    @echo "  just doctor-nix              # Flake validation, experimental features, lock age"
    @echo "  just doctor-direnv           # .envrc trust, shell load, env vars"
    @echo "  just doctor-git              # GPG config, commit signing, hook status"
    @echo "  just doctor-workspace        # Cargo structure, dependency graph"
    @echo ""
    @echo "Development workflow:"
    @echo "  just doctor-hooks            # Pre-commit hook installation + test"
    @echo "  just doctor-toolchain        # MSRV, rustfmt, cargo tools availability"
    @echo "  just doctor-quick            # Fast triage (check + clippy + test)"
    @echo ""
    @echo "CI/Documentation:"
    @echo "  just doctor-ci               # Compare local vs. CI pipeline checks"
    @echo "  just doctor-docs             # Broken links, ADR references, structure"
    @echo "  just doctor-monorepo         # Workspace member health, versions"
    @echo ""
    @echo "Governance (forge & registry config vs. spec, read-only):"
    @echo "  just doctor-forge            # Live forge (GitHub) config vs. spec"
    @echo "  just doctor-registry         # Live registry (crates.io) TP config vs. spec"
    @echo "  just doctor-provenance       # Main's signed history as-of the allowlist + CI round-trip"
    @echo ""
    @echo "Environment diagnostic (quick sanity check):"
    @echo "  just doctor-env              # Nix/direnv/toolchain/hooks overview"
    @echo ""

# Environment diagnostics (Nix, direnv, toolchain)
doctor-env:
    @./scripts/doctor-env.sh

# --- Nix Essentials ---

# Nix devshell package validation
doctor-devshell:
    @./scripts/doctor-devshell.sh

# Nix flake diagnostics
doctor-nix:
    @./scripts/doctor-nix.sh

# direnv integration diagnostics
doctor-direnv:
    @./scripts/doctor-direnv.sh

# Git configuration diagnostics (GPG signing, branch status)
doctor-git:
    @./scripts/doctor-git.sh

# Cargo workspace structure diagnostics
doctor-workspace:
    @./scripts/doctor-workspace.sh

# --- Development Workflow ---

# Pre-commit hook installation + functionality test
doctor-hooks:
    @./scripts/doctor-hooks.sh

# Rust toolchain + required tools validation
doctor-toolchain:
    @./scripts/doctor-toolchain.sh

# Fast triage (check + clippy + test)
doctor-quick:
    @./scripts/doctor-quick.sh

# --- CI/Documentation ---

# CI/local verification alignment check
doctor-ci:
    @./scripts/doctor-ci.sh

# Documentation structure and link validation
doctor-docs:
    @./scripts/doctor-docs.sh

# Workspace member health and consistency
doctor-monorepo:
    @./scripts/doctor-monorepo.sh

# --- Governance ---

# Forge configuration diagnostics (read-only: live forge state vs. spec)
doctor-forge:
    @./scripts/doctor-forge.sh

# Registry configuration diagnostics (read-only: live crates.io TP state vs. spec)
doctor-registry:
    @./scripts/doctor-registry.sh

# Repo-provenance audit (read-only: main's signed history as-of the allowlist + CI round-trip)
doctor-provenance *ARGS:
    @./scripts/doctor-provenance.sh {{ARGS}}

# ============================================================================
# 9. Maintenance
# ============================================================================

# Project maintenance diagnostics guide
maintain-help:
    @echo "🔧 sdmx-rs Maintenance"
    @echo ""
    @echo "Dependency & toolchain refresh (mutates lockfiles; review + sign manually):"
    @echo "  just update-deps [crates]    # Refresh Cargo.lock (all or named crates), then validate"
    @echo "  just update-flake            # Refresh flake.lock (Nix inputs), then validate"
    @echo "  just update-msrv <o> <n>     # Raise/lower MSRV (add --downgrade to lower)"
    @echo "  just update-specs <ed> <ref> # Re-pin an SDMX schema edition (commit + NAR hash + shas)"
    @echo ""
    @echo "Forge governance (maintainer-only; idempotent):"
    @echo "  just update-rulesets           # Apply spec rulesets to live forge"
    @echo "  just update-labels             # Apply spec labels to live forge"
    @echo "  just update-actions-allowlist  # Push committed actions allowlist to live forge"

# Refresh Cargo.lock (semver-compatible); validates via verify-rust. No commit — review diff and sign manually.
update-deps *CRATES:
    @./scripts/update-deps.sh {{CRATES}}

# Refresh flake.lock (Nix inputs); validates via verify-infra. No commit — review diff and sign manually.
update-flake:
    @./scripts/update-flake.sh

# Re-pin an SDMX schema edition into specs/sources.toml (resolve tag -> commit + NAR hash + per-file shas, TOFU). No commit — review diff, then recompute decisions.md #L anchors.
update-specs edition ref:
    @./scripts/update-specs.sh {{ edition }} {{ ref }}

# Raise or lower MSRV across all manifests and files; pass --downgrade to lower, --dry-run to preview
update-msrv *args:
    @./scripts/update-msrv.sh {{args}}

# Apply spec rulesets to live forge (POST to create, PUT to update by name). Aborts on duplicate names.
update-rulesets *args:
    @./scripts/update-rulesets.sh {{args}}

# Apply spec labels to live forge (PATCH to update, POST to create); does not delete or rename.
update-labels *args:
    @./scripts/update-labels.sh {{args}}

# Push committed actions-allowlist.json to live forge (single PUT); edit + commit the file first.
update-actions-allowlist *args:
    @./scripts/update-actions-allowlist.sh {{args}}

# ============================================================================
# 10. Release Pipeline
# ============================================================================

# Commit and release pipeline guide
release-help:
    @echo "🚀 sdmx-rs Commit & Release Pipeline"
    @echo ""
    @echo "Pre-release checks:"
    @echo "  just check-changelog        # Verify CHANGELOG.md sync status with git history"
    @echo ""
    @echo "Release & Lifecycle management:"
    @echo "  just prep-release <version> # Pre-1.0: bump ALL crates + signed batch commit (run before cargo release)"
    @echo "  just changelog-generate     # Generate changelogs for all crates (review before committing)"
    @echo "  just release-dry-run        # Run dry-run release simulation (specify crates or all)"
    @echo "  just release-commit-changelogs  # Commit generated changelogs as signed checkpoint"
    @echo "  just new-release-notes <v>  # Scaffold curated facade release notes from template"
    @echo "  just check-release-notes <v>   # Gate: curated release notes exist and are complete"
    @echo "  just prepublish-check       # Validate all crates will publish (dry-run, topological order)"
    @echo "  just release-merge          # Merge release branch to main with auto-generated commit message"
    @echo "  just stage-merge <version>  # Push merge commit to staging branch; wait for CI, then release-push"
    @echo "  just release-push <version> # Fast-forward main + push tags after CI is green on staging branch"

# Verify CHANGELOG.md files are in sync with git history (used in pre-release checklist)
check-changelog *CRATES:
    @./scripts/check-changelog.sh {{CRATES}}

# Pre-1.0: bump every crate to <version> in one signed batch commit; run BEFORE cargo release --execute
prep-release version:
    @./scripts/prep-release.sh {{version}}

# Generate CHANGELOG.md for all crates without committing (review before committing)
changelog-generate *CRATES:
    @./scripts/changelog-generate.sh {{CRATES}}

# Non-destructive dry-run release simulation; skipped locally if git tree is dirty
release-dry-run *CRATES:
    @./scripts/release-dry-run.sh {{CRATES}}

# Commit generated changelogs as a signed checkpoint before cargo release
release-commit-changelogs:
    @./scripts/release-commit-changelogs.sh

# Scaffold curated facade release-notes/<version>.md from template; refuses to overwrite an existing file
new-release-notes version:
    @./scripts/new-release-notes.sh {{version}}

# Gate: curated release-notes/<version>.md exists and is complete; must pass before cargo release --execute
check-release-notes version:
    @./scripts/check-release-notes.sh {{version}}

# Dry-run publish all crates in topological order; validates publishability without touching crates.io
prepublish-check *CRATES:
    @./scripts/prepublish-check.sh {{CRATES}}

# Merge release branch to main with auto-generated commit message listing released crates
release-merge:
    @./scripts/release-merge.sh

# Push the local merge commit to a CI staging branch; wait for the Quality Gate, then run release-push
stage-merge version:
    @./scripts/release-stage.sh {{version}}

# Fast-forward main, push all tags, clean up staging branch (run after CI is green on stage-merge)
release-push version:
    @./scripts/release-push.sh {{version}}
