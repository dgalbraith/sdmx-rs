#!/usr/bin/env bats
# ==============================================================================
# Test suite for scripts/doctor-forge.sh
#
# Testing approach: doctor-forge is READ-ONLY. These tests NEVER touch a live
# forge — `gh` is replaced by a PATH shim (mock_gh) that serves canned JSON from
# a per-test, mutable copy of tests/bats/fixtures/forge/. A `git` shim makes the
# offline root-commit signature check pass without real GPG (mirrors the
# verify-signature.bats pattern). Each test perturbs a single fixture (or auth
# state) to exercise one MATCH/drift path.
#
# Run with: bats tests/bats/doctor-forge.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    REPO_ROOT="$BATS_TEST_DIRNAME/../.."

    # Isolated working tree that satisfies the OFFLINE tier prerequisites.
    TMPDIR=$(mktemp -d)
    cd "$TMPDIR" || exit 1

    # Scripts + sourced libs, mirroring the real scripts/ layout the script
    # resolves relative to $0.
    mkdir -p scripts/lib lib
    cp "$REPO_ROOT/scripts/doctor-forge.sh" scripts/
    cp "$REPO_ROOT/scripts/lib/forge-spec.sh" scripts/lib/
    cp "$REPO_ROOT/scripts/lib/log.sh" scripts/lib/

    # Committed forge/github artifacts (doctor diffs live against these).
    mkdir -p forge/github
    cp "$REPO_ROOT"/forge/github/ruleset-*.json forge/github/
    cp "$REPO_ROOT/forge/github/actions-allowlist.json" forge/github/

    # OFFLINE prerequisites:
    #  - maintainer key file present
    mkdir -p .github/maintainer-keys
    echo "-----BEGIN PGP PUBLIC KEY BLOCK-----" > .github/maintainer-keys/dgalbraith.asc
    #  - primary fingerprint anchored in the CI trust root
    mkdir -p .github/workflows
    cat > .github/workflows/verify-signature.yml <<'EOF'
# trust root stub
env:
  ALLOWED_PRIMARY_FINGERPRINTS: |
    53069F0184A426465E5FF9E7FC6BB04EBF431B25
EOF

    # Git repo + signing config matching the spec, plus a root commit and the
    # origin/codeberg/all remotes the offline tier checks.
    git init --initial-branch=main -q
    git config user.name "David Galbraith"
    git config user.email "dg@lbraith.io"
    git config user.signingkey "B43D054479B0A9374BC35C167D4A0D2EE2E2ECD7!"
    git config commit.gpgsign true
    git config commit.gpgsign false   # do not actually sign in tests; the shim asserts signature
    git commit --allow-empty -m "chore(repo): establish signed repository root" -q
    git remote add origin "git@github.com:dgalbraith/sdmx-rs.git"
    git remote add codeberg "git@codeberg.org:dgalbraith/sdmx-rs.git"
    git remote add all "git@github.com:dgalbraith/sdmx-rs.git"

    # Per-test MUTABLE fixture copy so a single test can perturb one response.
    export FORGE_FIXTURES="$TMPDIR/forge-fixtures"
    cp -r "$REPO_ROOT/tests/bats/fixtures/forge" "$FORGE_FIXTURES"

    # Derive ruleset-N.json fixtures from the committed spec files so that the
    # "clean state → exit 0" tests never drift when the spec changes.  The
    # rulesets.json index drives the id→name→file mapping; each fixture is the
    # projected spec body plus a static server-owned envelope.
    while IFS= read -r entry; do
        rs_id="$(printf '%s' "$entry" | jq -r '.id')"
        rs_name="$(printf '%s' "$entry" | jq -r '.name')"
        rs_file="$(find "$REPO_ROOT/forge/github" -name 'ruleset-*.json' \
            -exec sh -c 'jq -e --arg n "$1" ".name == \$n" "$2" > /dev/null 2>&1 && printf "%s" "$2"' _ "$rs_name" {} \; | head -n1)"
        [ -z "$rs_file" ] && continue
        jq -S --argjson id "$rs_id" \
            '{id: $id,
              node_id: "RRS_mock",
              name,
              target,
              source_type: "Repository",
              source: "dgalbraith/sdmx-rs",
              enforcement,
              current_user_can_bypass: "always",
              created_at: "2026-01-01T00:00:00Z",
              updated_at: "2026-01-01T00:00:00Z",
              _links: {self: {href: ("https://api.github.com/repos/dgalbraith/sdmx-rs/rulesets/" + ($id | tostring))}},
              bypass_actors,
              conditions,
              rules}' \
            "$rs_file" > "$FORGE_FIXTURES/ruleset-${rs_id}.json"
    done < <(jq -c '.[]' "$FORGE_FIXTURES/rulesets.json")

    # Mock bin: a `git` shim that makes `git verify-commit` succeed (signed root)
    # and falls through to real git otherwise. mock_gh prepends to the same dir.
    mkdir -p "$BATS_TEST_TMPDIR/bin"
    cat > "$BATS_TEST_TMPDIR/bin/git" <<EOF
#!/bin/sh
if [ "\$1" = "verify-commit" ]; then exit 0; fi
exec $(command -v git) "\$@"
EOF
    chmod +x "$BATS_TEST_TMPDIR/bin/git"
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

teardown() {
    cd "$BATS_TEST_DIRNAME" || exit 1
    rm -rf "$TMPDIR"
}

# Run doctor-forge with the ambient CI/tooling env scrubbed (so colour/CI
# branches are deterministic) but the mock-bin PATH preserved.
run_doctor() {
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE
    run sh scripts/doctor-forge.sh
}

# ==============================================================================
# Auth gating
# ==============================================================================

@test "doctor-forge: auth absent -> online tier skipped, offline ran, exit 0" {
    mock_gh --no-auth
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    # Offline tier ran...
    [[ "$output" == *"Offline checks"* ]]
    [[ "$output" == *"Maintainer key file present"* ]]
    # ...and the online tier degraded to a warning + hint, not a hard failure.
    [[ "$output" == *"not authenticated"* ]]
    [[ "$output" == *"offline checks passed"* ]]
}

# ==============================================================================
# Full match (post-apply: allowed_actions=selected)
# ==============================================================================

@test "doctor-forge: all live state matches spec (post-apply) -> exit 0" {
    # Post-apply: actions/permissions reports allowed_actions=selected, and the
    # selected-actions body matches the committed file.
    GH_MOCK_ALLOWED_ACTIONS=selected mock_gh
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"matches spec"* ]]
}

# ==============================================================================
# Signing key: short key ID matches the full spec fingerprint (suffix match)
# ==============================================================================

@test "doctor-forge: short signing key ID matches the full spec fingerprint" {
    GH_MOCK_ALLOWED_ACTIONS=selected mock_gh
    # git accepts the 16-hex short ID (the tail of the 40-hex fingerprint), with
    # the '!' subkey pin. doctor-forge must accept it as the same key, not warn.
    git config user.signingkey "7D4A0D2EE2E2ECD7!"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"git user.signingkey pins the signing subkey"* ]]
    # Must NOT emit the mismatch warning for a valid short ID.
    [[ "$output" != *"!= spec subkey"* ]]
}

# ==============================================================================
# Security settings
# ==============================================================================

@test "doctor-forge: vulnerability alerts disabled -> exit 1" {
    GH_MOCK_VULN_ALERTS=off mock_gh
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"vulnerability-alerts = false (want true)"* ]]
}

@test "doctor-forge: Dependabot auto-fix PRs enabled -> exit 1 (signing invariant)" {
    # automated-security-fixes must stay OFF — its auto-PRs would add unsigned
    # commits. A live 'enabled' is drift.
    GH_MOCK_AUTO_FIXES=true mock_gh
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"automated-security-fixes = true (want false)"* ]]
}

@test "doctor-forge: secret scanning disabled -> warn, exit 0 (default, public-only deferral)" {
    GH_MOCK_ALLOWED_ACTIONS=selected mock_gh
    # Disable secret scanning in the live repo response.
    jq '.security_and_analysis.secret_scanning.status = "disabled"' \
        "$FORGE_FIXTURES/repo.json" > "$FORGE_FIXTURES/repo.json.tmp"
    mv "$FORGE_FIXTURES/repo.json.tmp" "$FORGE_FIXTURES/repo.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"secret_scanning = disabled"* ]]
    [[ "$output" == *"deferred until repo is public"* ]]
}

@test "doctor-forge: default workflow token permissions = write -> exit 1" {
    mock_gh
    # Loosen the default GITHUB_TOKEN to write — spec wants read (least privilege).
    jq '.default_workflow_permissions = "write"' \
        "$FORGE_FIXTURES/actions-permissions-workflow.json" > "$FORGE_FIXTURES/awf.tmp"
    mv "$FORGE_FIXTURES/awf.tmp" "$FORGE_FIXTURES/actions-permissions-workflow.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"default_workflow_permissions = write (want read)"* ]]
}

@test "doctor-forge: secret scanning disabled + FORGE_SECURITY_REQUIRED=1 -> exit 1" {
    mock_gh
    jq '.security_and_analysis.secret_scanning.status = "disabled"' \
        "$FORGE_FIXTURES/repo.json" > "$FORGE_FIXTURES/repo.json.tmp"
    mv "$FORGE_FIXTURES/repo.json.tmp" "$FORGE_FIXTURES/repo.json"
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE
    FORGE_SECURITY_REQUIRED=1 run sh scripts/doctor-forge.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"secret_scanning = disabled"* ]]
}

# ==============================================================================
# Label drift
# ==============================================================================

@test "doctor-forge: a label colour drift -> exit 1" {
    mock_gh
    # Corrupt the 'feat' label colour in the live response.
    jq '(.[] | select(.name == "feat") | .color) = "ffffff"' \
        "$FORGE_FIXTURES/labels.json" > "$FORGE_FIXTURES/labels.json.tmp"
    mv "$FORGE_FIXTURES/labels.json.tmp" "$FORGE_FIXTURES/labels.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Label drifts"* ]]
    [[ "$output" == *"feat"* ]]
}

# ==============================================================================
# Label colour case-insensitivity (F2): uppercase live colour still matches
# ==============================================================================

@test "doctor-forge: uppercase live label colour still matches (case-insensitive)" {
    GH_MOCK_ALLOWED_ACTIONS=selected mock_gh
    # GitHub returns lowercase today, but a label set elsewhere could be uppercase.
    # Upcase the 'feat' colour in the live response; spec is lowercase 0e8a16.
    jq '(.[] | select(.name == "feat") | .color) |= ascii_upcase' \
        "$FORGE_FIXTURES/labels.json" > "$FORGE_FIXTURES/labels.json.tmp"
    mv "$FORGE_FIXTURES/labels.json.tmp" "$FORGE_FIXTURES/labels.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"matches spec"* ]]
}

# ==============================================================================
# Duplicate ruleset name (F3): two live rulesets share a spec name -> drift
# ==============================================================================

@test "doctor-forge: duplicate ruleset name -> exit 1" {
    mock_gh
    # Append a second ruleset with the same name as the signing ruleset (id 1).
    jq '. + [{"id": 99, "name": "Enforce High Integrity Development", "target": "branch", "enforcement": "active"}]' \
        "$FORGE_FIXTURES/rulesets.json" > "$FORGE_FIXTURES/rulesets.json.tmp"
    mv "$FORGE_FIXTURES/rulesets.json.tmp" "$FORGE_FIXTURES/rulesets.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Duplicate ruleset name"* ]]
}

# ==============================================================================
# Repo fetch failure (B2): one error, no spurious repo-settings drift cascade
# ==============================================================================

@test "doctor-forge: repos fetch failure -> single error, no settings cascade" {
    mock_gh
    # Remove the repo.json fixture so the bare repos/{o}/{r} call 404s. The
    # rulesets/labels/etc. endpoints map to their own fixtures and still succeed.
    rm -f "$FORGE_FIXTURES/repo.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Could not query repos/"* ]]
    # The merge-flag / repo-setting keys must NOT appear as drift lines (the
    # nested-fetch fix means they are not evaluated against an empty body).
    [[ "$output" != *"allow_merge_commit ="* ]]
    [[ "$output" != *"has_projects ="* ]]
}

# ==============================================================================
# Signing-ruleset bypass regression (SECURITY invariant)
# ==============================================================================

@test "doctor-forge: signing ruleset gains a bypass actor -> exit 1" {
    mock_gh
    # Inject a bypass actor into the LIVE signing ruleset (id 1) only.
    jq '.bypass_actors = [{"actor_id": 99, "actor_type": "User", "bypass_mode": "always"}]' \
        "$FORGE_FIXTURES/ruleset-1.json" > "$FORGE_FIXTURES/ruleset-1.json.tmp"
    mv "$FORGE_FIXTURES/ruleset-1.json.tmp" "$FORGE_FIXTURES/ruleset-1.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"SECURITY"* ]]
    [[ "$output" == *"bypass"* ]]
}

# ==============================================================================
# Push-restriction (Zero Trust Gate) bypass regression (SECURITY invariant)
# ==============================================================================

@test "doctor-forge: push-restriction ruleset gains a bypass actor -> exit 1" {
    mock_gh
    # Inject a bypass actor into the LIVE push-restriction ruleset (id 2) only.
    jq '.bypass_actors = [{"actor_id": 5, "actor_type": "RepositoryRole", "bypass_mode": "always"}]' \
        "$FORGE_FIXTURES/ruleset-2.json" > "$FORGE_FIXTURES/ruleset-2.json.tmp"
    mv "$FORGE_FIXTURES/ruleset-2.json.tmp" "$FORGE_FIXTURES/ruleset-2.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"SECURITY"* ]]
    [[ "$output" == *"bypass"* ]]
}

# ==============================================================================
# Push-restriction (Zero Trust Gate) missing required_status_checks
# ==============================================================================

@test "doctor-forge: push-restriction ruleset missing required_status_checks -> exit 1" {
    mock_gh
    # Drop the required_status_checks rule from the LIVE push-restriction ruleset (id 2).
    jq '.rules = [{"type": "update"}]' \
        "$FORGE_FIXTURES/ruleset-2.json" > "$FORGE_FIXTURES/ruleset-2.json.tmp"
    mv "$FORGE_FIXTURES/ruleset-2.json.tmp" "$FORGE_FIXTURES/ruleset-2.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"drifts from forge/github/ruleset-push-restriction.json"* ]]
}

# ==============================================================================
# delete_branch_on_merge must be true
# ==============================================================================

@test "doctor-forge: delete_branch_on_merge=false -> exit 1" {
    mock_gh
    jq '.delete_branch_on_merge = false' \
        "$FORGE_FIXTURES/repo.json" > "$FORGE_FIXTURES/repo.json.tmp"
    mv "$FORGE_FIXTURES/repo.json.tmp" "$FORGE_FIXTURES/repo.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"delete_branch_on_merge = false (want true)"* ]]
}

# ==============================================================================
# Ruleset file-drift (live diverges from the committed JSON)
# ==============================================================================

@test "doctor-forge: tag ruleset rule differs from committed file -> exit 1" {
    mock_gh
    # Drop a rule from the LIVE tag ruleset (id 3) so it diverges from the file.
    jq '.rules = [{"type": "non_fast_forward"}]' \
        "$FORGE_FIXTURES/ruleset-3.json" > "$FORGE_FIXTURES/ruleset-3.json.tmp"
    mv "$FORGE_FIXTURES/ruleset-3.json.tmp" "$FORGE_FIXTURES/ruleset-3.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"drifts from forge/github/ruleset-tag-protection.json"* ]]
}

# ==============================================================================
# Release environment absent -> warn (default), not failure
# ==============================================================================

@test "doctor-forge: release environment absent -> warn, exit 0" {
    GH_MOCK_ALLOWED_ACTIONS=selected mock_gh
    # Remove the env fixture so the endpoint 404s (no environment created).
    rm -f "$FORGE_FIXTURES/environment-release.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 0 ]
    [[ "$output" == *"release environment missing"* ]]
}

@test "doctor-forge: release environment absent + FORGE_RELEASE_REQUIRED=1 -> exit 1" {
    mock_gh
    rm -f "$FORGE_FIXTURES/environment-release.json"
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE
    FORGE_RELEASE_REQUIRED=1 run sh scripts/doctor-forge.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"release environment missing"* ]]
}

# ==============================================================================
# Actions allowlist: allowed_actions=all pre-apply (expected drift, not failure)
# ==============================================================================

@test "doctor-forge: allowed_actions=all (pre-apply) -> drift line, deferred warn for selected-actions" {
    # Default fixture has allowed_actions=all (GH_MOCK_ALLOWED_ACTIONS not set).
    mock_gh
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    # allowed_actions drift must appear (it's not 'selected' yet).
    [[ "$output" == *"allowed_actions = all (want selected)"* ]]
    # selected-actions is deferred when allowed_actions != selected.
    [[ "$output" == *"Deferred: allowed_actions=all"* ]]
    [ "$status" -eq 1 ]
}

# ==============================================================================
# Actions allowlist: selected-actions body matches committed file -> ok
# ==============================================================================

@test "doctor-forge: allowed_actions=selected, selected-actions matches file -> no drift" {
    # Activate the "selected" mode fixture for actions/permissions.
    GH_MOCK_ALLOWED_ACTIONS=selected mock_gh
    # Also copy the allowlist into the test tree so the file check passes.
    cp "$REPO_ROOT/forge/github/actions-allowlist.json" forge/github/
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [[ "$output" == *"selected-actions matches committed file"* ]]
    [ "$status" -eq 0 ]
}

# ==============================================================================
# Actions allowlist: selected-actions drifts from committed file -> fail
# ==============================================================================

@test "doctor-forge: selected-actions drifts from committed file -> exit 1" {
    GH_MOCK_ALLOWED_ACTIONS=selected mock_gh
    cp "$REPO_ROOT/forge/github/actions-allowlist.json" forge/github/
    # Inject an extra pattern into the LIVE selected-actions response so it
    # diverges from the committed file.
    jq '.patterns_allowed += ["extra-org/extra-action@*"]' \
        "$FORGE_FIXTURES/actions-permissions-selected.json" \
        > "$FORGE_FIXTURES/actions-permissions-selected.json.tmp"
    mv "$FORGE_FIXTURES/actions-permissions-selected.json.tmp" \
        "$FORGE_FIXTURES/actions-permissions-selected.json"
    run_doctor
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"selected-actions drifts from"* ]]
}

# ==============================================================================
# Actions allowlist coverage: all uses: covered -> ok
# ==============================================================================

@test "doctor-forge: all workflow uses: covered by allowlist -> no coverage failure" {
    mock_gh
    # Create a workflow dir with only covered actions (github-owned + spec'd third-party).
    mkdir -p "$TMPDIR/wf-covered"
    cat > "$TMPDIR/wf-covered/covered.yml" <<'EOF'
jobs:
  build:
    steps:
      - uses: actions/checkout@abc123
      - uses: dtolnay/rust-toolchain@abc123
      - uses: DeterminateSystems/nix-installer-action@abc123
      - uses: nix-community/cache-nix-action@abc123
EOF
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE
    FORGE_WORKFLOWS_DIR="$TMPDIR/wf-covered" run sh scripts/doctor-forge.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    # Must NOT report any uncovered actions (drift/fail from the crosscheck).
    [[ "$output" != *"Uncovered action:"* ]]
}

# ==============================================================================
# Actions allowlist coverage: uncovered uses: -> fail
# ==============================================================================

@test "doctor-forge: uncovered uses: in workflow -> exit 1" {
    mock_gh
    cp "$REPO_ROOT/forge/github/actions-allowlist.json" forge/github/
    # Create a fixture workflow dir with a single uncovered action.
    mkdir -p "$TMPDIR/wf-uncovered"
    cat > "$TMPDIR/wf-uncovered/bad.yml" <<'EOF'
jobs:
  build:
    steps:
      - uses: some-org/some-unlisted-action@abc123
EOF
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE
    FORGE_WORKFLOWS_DIR="$TMPDIR/wf-uncovered" run sh scripts/doctor-forge.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    [ "$status" -eq 1 ]
    [[ "$output" == *"Uncovered action: some-org/some-unlisted-action"* ]]
    # The prescriptive message must name the allowlist file.
    [[ "$output" == *"forge/github/actions-allowlist.json"* ]]
}

# ==============================================================================
# Actions allowlist coverage: stale pattern (no matching uses:) -> warn, not fail
# ==============================================================================

@test "doctor-forge: stale allowlist pattern (no matching uses:) -> warn, exit 0" {
    mock_gh
    cp "$REPO_ROOT/forge/github/actions-allowlist.json" forge/github/
    # Use a workflow dir with only github-owned actions — all third-party
    # patterns in the allowlist become stale.
    mkdir -p "$TMPDIR/wf-github-only"
    cat > "$TMPDIR/wf-github-only/simple.yml" <<'EOF'
jobs:
  build:
    steps:
      - uses: actions/checkout@abc123
EOF
    unset GITHUB_ACTIONS CI SDMX_MAIN_REMOTE
    FORGE_WORKFLOWS_DIR="$TMPDIR/wf-github-only" run sh scripts/doctor-forge.sh
    echo "STATUS: $status" >&2
    echo "OUTPUT: $output" >&2
    # Stale pattern is a WARN, not a FAIL — must not push exit status to 1.
    [[ "$output" == *"Stale allowlist pattern"* ]]
    # Stale patterns are warnings; the only failures here are from pre-apply drift
    # (allowed_actions=all). The crosscheck itself must not add a new failure.
    [[ "$output" != *"Uncovered action:"* ]]
}
