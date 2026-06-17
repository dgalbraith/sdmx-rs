#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/update-msrv.sh
#
# Testing approach: Integration tests for MSRV updates (raise or lower).
# Validates configuration updates, version verification, error handling,
# and different behaviour between raising (breaking) and lowering (feature).
#
# Run with: bats tests/bats/update-msrv.bats
# Or: just test-maintenance (part of full suite)
# ==============================================================================

setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"
    cd "$BATS_TEST_TMPDIR" || exit 1

    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"

    # Gitignore bin/ so mock helpers (mock_just) don't dirty the working tree
    echo "bin/" > .gitignore

    # Copy entire scripts directory
    cp -r "$REPO_ROOT/scripts" .

    # Copy files declared in manifest
    while IFS= read -r file; do
        [ -z "$file" ] && continue
        case "$file" in "#"*) continue ;; esac
        mkdir -p "$(dirname "$file")"
        cp "$REPO_ROOT/$file" "$file" \
            || { echo "ERROR: manifest file not found in repo: $file"; exit 1; }
    done < "$BATS_TEST_DIRNAME/fixtures/update-msrv-manifest.txt"

    git add .
    git commit -m "Initial test state" -q

    # Assert key files are correctly staged
    assert_file_exists_in_git "scripts/update-msrv.sh"
    assert_file_exists_in_git "Cargo.toml"
    assert_file_exists_in_git "rust-toolchain.toml"
}

teardown() {
    # BATS automatically cleans up $BATS_TEST_TMPDIR
    :
}

# ==============================================================================
# Argument Parsing Tests
# ==============================================================================

@test "update-msrv: shows usage with no args" {
    run_isolated "scripts/update-msrv.sh"
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Usage:" ]]
}

@test "update-msrv: accepts valid version format (X.Y.Z) for upgrade" {
    run_isolated "scripts/update-msrv.sh" --dry-run 1.91.0 1.92.0
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Dry-run validation complete" ]]
}

@test "update-msrv: accepts valid version format (X.Y.Z) for downgrade" {
    run_isolated "scripts/update-msrv.sh" --dry-run --downgrade 1.91.0 1.85.0
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Dry-run validation complete" ]]
}

@test "update-msrv: rejects invalid version format" {
    run_isolated "scripts/update-msrv.sh" --dry-run 1.91 1.92.0
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Invalid version format" ]]
}

@test "update-msrv: --downgrade flag is recognised" {
    run_isolated "scripts/update-msrv.sh" --help
    [ "$status" -eq 0 ]
    [[ "$output" =~ "--downgrade" ]]
}

# ==============================================================================
# Dry-Run Mode
# ==============================================================================

@test "update-msrv --dry-run: shows validation results for upgrade" {
    run_isolated "scripts/update-msrv.sh" --dry-run 1.91.0 1.92.0
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Dry-run validation complete" ]]
}

@test "update-msrv --dry-run: shows validation results for downgrade" {
    run_isolated "scripts/update-msrv.sh" --dry-run --downgrade 1.91.0 1.85.0
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Dry-run validation complete" ]]
}

@test "update-msrv --dry-run: does not modify any files" {
    ORIGINAL_CARGO=$(sha256sum Cargo.toml | awk '{print $1}')
    ORIGINAL_TOOLCHAIN=$(sha256sum rust-toolchain.toml | awk '{print $1}')

    bash scripts/update-msrv.sh --dry-run 1.91.0 1.92.0

    [ "$(sha256sum Cargo.toml | awk '{print $1}')" = "$ORIGINAL_CARGO" ]
    [ "$(sha256sum rust-toolchain.toml | awk '{print $1}')" = "$ORIGINAL_TOOLCHAIN" ]
}

@test "update-msrv --dry-run: does not stage any files" {
    bash scripts/update-msrv.sh --dry-run 1.91.0 1.92.0
    run git diff --cached --name-only
    [ -z "$output" ]
}

# ==============================================================================
# Pre-Flight Checks
# ==============================================================================

@test "update-msrv: fails with dirty git working tree" {
    echo "uncommitted change" >> Cargo.toml
    run_isolated "scripts/update-msrv.sh" 1.91.0 1.92.0
    # Dump git state if exit code is unexpected (git-state failures)
    [ "$status" -eq 1 ] || {
        echo "Expected exit 1 but got $status. Git files:"
        git ls-files
        false
    }
    [[ "$output" =~ "Git working tree is dirty" ]]
}

@test "update-msrv: fails if Cargo.toml and rust-toolchain.toml are out of sync" {
    sed -i 's/channel = "1.91.0"/channel = "1.90.0"/' rust-toolchain.toml
    git add rust-toolchain.toml
    git commit -m "Desync for testing" -q
    run_isolated "scripts/update-msrv.sh" 1.91.0 1.92.0
    # Dump git state if exit code is unexpected (git-state failures)
    [ "$status" -eq 1 ] || {
        echo "Expected exit 1 but got $status. Git files:"
        git ls-files
        false
    }
    [[ "$output" =~ "MSRV mismatch" ]]
}

@test "update-msrv: fails if current MSRV doesn't match OLD_VERSION (upgrade)" {
    run_isolated "scripts/update-msrv.sh" 1.90.0 1.92.0
    # Dump git state if exit code is unexpected (git-state failures)
    [ "$status" -eq 1 ] || {
        echo "Expected exit 1 but got $status. Git files:"
        git ls-files
        false
    }
    [[ "$output" =~ "Current MSRV" ]]
    [[ "$output" =~ "does not match OLD_VERSION" ]]
}

@test "update-msrv: fails if current MSRV doesn't match OLD_VERSION (downgrade)" {
    run_isolated "scripts/update-msrv.sh" --downgrade 1.90.0 1.85.0
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Current MSRV" ]]
}

# ==============================================================================
# Explicit Pattern Matching (Safety)
# ==============================================================================

@test "update-msrv: detects file drift in Cargo.toml (wrong version)" {
    sed -i 's/rust-version = "1.91.0"/rust-version = "1.90.0"/' Cargo.toml
    git add Cargo.toml
    git commit -m "Drift for testing" -q

    run_isolated "scripts/update-msrv.sh" --dry-run 1.91.0 1.92.0
    [ "$status" -eq 1 ]
    # Script reports MSRV mismatch when Cargo.toml and rust-toolchain.toml diverge
    [[ "$output" =~ "mismatch" ]]
}

@test "update-msrv: detects file drift in rust-toolchain.toml (wrong version)" {
    sed -i 's/channel = "1.91.0"/channel = "1.90.0"/' rust-toolchain.toml
    git add rust-toolchain.toml
    git commit -m "Drift for testing" -q

    run_isolated "scripts/update-msrv.sh" --dry-run 1.91.0 1.92.0
    [ "$status" -eq 1 ]
    # Script reports MSRV mismatch when Cargo.toml and rust-toolchain.toml diverge
    [[ "$output" =~ "mismatch" ]]
}

@test "update-msrv: provides clear error message when sed pattern doesn't match" {
    # Rename the rust-version key so grep finds no match; WORKSPACE_MSRV becomes
    # empty, triggering a sync mismatch error before the script reaches sed
    sed -i 's/rust-version = "1.91.0"/RUST_VER = "1.91.0"/' Cargo.toml
    git add Cargo.toml
    git commit -m "Format change for testing" -q

    run_isolated "scripts/update-msrv.sh" --dry-run 1.91.0 1.92.0
    [ "$status" -eq 1 ]
    [[ "$output" =~ "mismatch" ]]
}

# ==============================================================================
# Release-notes template MSRV sync
# ==============================================================================

@test "update-msrv: bumps the facade release-notes template Current MSRV line" {
    # The template carries the MSRV as a literal value so update-msrv must keep it
    # in sync. Full run required: --dry-run exits before file mutations.
    TEMPLATE="crates/sdmx-rs/release-notes/templates/template.md"
    grep -q '\*\*Current MSRV\*\*: `1.91.0`' "$TEMPLATE"   # precondition
    mock_just
    run_isolated "scripts/update-msrv.sh" 1.91.0 1.92.0
    [ "$status" -eq 0 ]
    grep -q '\*\*Current MSRV\*\*: `1.92.0`' "$TEMPLATE"
    # Only the MSRV line moves: section structure is untouched.
    grep -q '^## Breaking Changes & Migration$' "$TEMPLATE"
    grep -q '^## Minimum Supported Rust Version (MSRV)$' "$TEMPLATE"
}

# ==============================================================================
# Badge alt-text and crate README updates
# ==============================================================================

@test "update-msrv: updates README.md badge alt-text (MSRV: X.Y.Z form)" {
    # The badge alt-text uses 'MSRV: X.Y.Z' (colon-space); the badge URL uses
    # 'MSRV-X.Y.Z' (hyphen). Both forms must be updated.
    grep -q 'MSRV: 1\.91\.0' README.md   # precondition
    mock_just
    run_isolated "scripts/update-msrv.sh" 1.91.0 1.92.0
    [ "$status" -eq 0 ]
    grep -q 'MSRV: 1\.92\.0' README.md
    ! grep -q 'MSRV: 1\.91\.0' README.md
}

@test "update-msrv: updates all five crate README badges" {
    # crates/*/README.md carry the same badge form and must be kept in sync.
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        grep -q 'MSRV: 1\.91\.0' "crates/$crate/README.md"   # precondition
    done
    mock_just
    run_isolated "scripts/update-msrv.sh" 1.91.0 1.92.0
    [ "$status" -eq 0 ]
    for crate in sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs; do
        grep -q 'MSRV: 1\.92\.0' "crates/$crate/README.md"
        grep -q 'MSRV-1\.92\.0' "crates/$crate/README.md"
        ! grep -q 'MSRV: 1\.91\.0' "crates/$crate/README.md"
    done
}

@test "update-msrv: updates CONTRIBUTING.md bare-paren MSRV form" {
    # CONTRIBUTING.md line 319 uses 'MSRV (X.Y.Z)' — not caught by the bold
    # or currently-bold patterns; needs its own sed.
    grep -q 'MSRV (1\.91\.0)' CONTRIBUTING.md   # precondition
    mock_just
    run_isolated "scripts/update-msrv.sh" 1.91.0 1.92.0
    [ "$status" -eq 0 ]
    grep -q 'MSRV (1\.92\.0)' CONTRIBUTING.md
    ! grep -q 'MSRV (1\.91\.0)' CONTRIBUTING.md
}

# ==============================================================================
# Upgrade-Specific Behaviour
# ==============================================================================

@test "update-msrv (upgrade): updates maintenance.toml dates" {
    # Full run required: --dry-run exits before file mutations
    mock_just
    run_isolated "scripts/update-msrv.sh" 1.91.0 1.92.0
    [ "$status" -eq 0 ]
    grep -q 'last_updated = "' maintenance.toml
    grep -q 'next_review = "' maintenance.toml
}

@test "update-msrv (upgrade): prints BREAKING CHANGE warning" {
    # Full run required: BREAKING CHANGE is only printed in the post-update summary
    mock_just
    run_isolated "scripts/update-msrv.sh" 1.91.0 1.92.0
    [ "$status" -eq 0 ]
    [[ "$output" =~ "BREAKING" ]]
}

@test "update-msrv (upgrade): warns about 6-month policy" {
    run_isolated "scripts/update-msrv.sh" --dry-run 1.91.0 1.92.0
    [ "$status" -eq 0 ]
    [[ "$output" == *"6+ months"* ]]
}

# ==============================================================================
# Downgrade-Specific Behaviour
# ==============================================================================

@test "update-msrv --downgrade: skips maintenance.toml updates" {
    ORIGINAL_MAINTENANCE=$(sha256sum maintenance.toml | awk '{print $1}')

    # Full run required: dry-run exits before file mutations
    mock_just
    run_isolated "scripts/update-msrv.sh" --downgrade 1.91.0 1.85.0
    [ "$status" -eq 0 ]

    # maintenance.toml must be unchanged (downgrade skips review obligation updates)
    [ "$(sha256sum maintenance.toml | awk '{print $1}')" = "$ORIGINAL_MAINTENANCE" ]
}

@test "update-msrv --downgrade: is not a breaking change" {
    run_isolated "scripts/update-msrv.sh" --dry-run --downgrade 1.91.0 1.85.0
    [ "$status" -eq 0 ]
    # Should NOT mention breaking change for downgrade
    [[ ! "$output" =~ "BREAKING" ]]
}

# ==============================================================================
# Version Reference Updates
# ==============================================================================

@test "update-msrv: updates docs/project/msrv.md old MSRV example" {
    # Full run required: sed patterns for msrv.md are context-matched (most fragile)
    mock_just
    run_isolated "scripts/update-msrv.sh" 1.91.0 1.92.0
    [ "$status" -eq 0 ]
    grep -q "1\.92\.0" docs/project/msrv.md
}

@test "update-msrv: updates docs/project/msrv.md new MSRV example" {
    # Full run required: both old and new version references must be updated
    mock_just
    run_isolated "scripts/update-msrv.sh" 1.91.0 1.92.0
    [ "$status" -eq 0 ]
    # Old version should no longer appear as the new MSRV
    grep -qv "cargo +1\.91\.0" docs/project/msrv.md || true
    grep -q "1\.92\.0" docs/project/msrv.md
}

# ==============================================================================
# File Path Validation
# ==============================================================================

@test "update-msrv: handles all 6 Cargo.toml files" {
    grep -q "crates/sdmx-types/Cargo.toml" scripts/update-msrv.sh
    grep -q "crates/sdmx-parsers/Cargo.toml" scripts/update-msrv.sh
    grep -q "crates/sdmx-writers/Cargo.toml" scripts/update-msrv.sh
    grep -q "crates/sdmx-client/Cargo.toml" scripts/update-msrv.sh
    grep -q "crates/sdmx-rs/Cargo.toml" scripts/update-msrv.sh
}

@test "update-msrv: script covers crates/*/README.md in git add" {
    grep -q "crates/\*/README\.md" scripts/update-msrv.sh
}

# ==============================================================================
# Help Text and Documentation
# ==============================================================================

@test "update-msrv: script is executable" {
    [ -x "scripts/update-msrv.sh" ]
}

@test "update-msrv: script includes usage documentation" {
    grep -q "Usage:" scripts/update-msrv.sh
    grep -q "Examples:" scripts/update-msrv.sh
}

@test "update-msrv: documents --downgrade flag" {
    grep -q "\-\-downgrade" scripts/update-msrv.sh
}

@test "update-msrv: mentions raising vs lowering in docs" {
    grep -qE "raise|lower|downgrade|upgrade" scripts/update-msrv.sh
}
