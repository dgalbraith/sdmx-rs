#!/usr/bin/env bats
# ==============================================================================
# Test suite for the ruleset projection invariant.
#
# The committed forge/github/ruleset-*.json files are the PROJECTED form of a
# live GitHub ruleset — the request body with all server-owned read-only fields
# stripped (forge/README.md "Projection"). doctor-forge diffs the live ruleset,
# re-projected, against these files; that diff is only meaningful if the
# committed files are ALREADY in projected form (i.e. running the projection on
# them is a no-op). A hand edit that reintroduced a read-only field (id,
# created_at, …) would silently break the comparison's premise.
#
# This suite asserts the no-op: for every committed ruleset file, the projection
# jq from forge-spec.sh applied to the file equals the file's own canonical form.
# No forge, no `gh` — pure jq over committed artifacts.
#
# Run with: bats tests/bats/forge-projection.bats
# ==============================================================================
setup() {
    source "$BATS_TEST_DIRNAME/common.sh"

    REPO_ROOT="$BATS_TEST_DIRNAME/../.."

    # Source forge-spec.sh to reach forge_spec_ruleset_projection_jq. It is
    # SOURCED-ONLY (no top-level side effects), so this is safe.
    # shellcheck disable=SC1091
    . "$REPO_ROOT/scripts/lib/forge-spec.sh"

    command -v jq >/dev/null || skip "jq not available"
}

@test "forge projection: every committed ruleset file is already in projected form" {
    proj_jq="$(forge_spec_ruleset_projection_jq)"

    for f in "$REPO_ROOT"/forge/github/ruleset-*.json; do
        # Canonical file form (sorted keys) vs. the same file run through the
        # projection (sorted keys). Equal ⇒ the file carries no read-only fields
        # and is a projection fixed point.
        canonical="$(jq -S '.' "$f")"
        projected="$(jq -S "$proj_jq" "$f")"
        if [ "$canonical" != "$projected" ]; then
            echo "Ruleset file is NOT in projected form: $f" >&2
            echo "--- file (canonical) ---" >&2
            printf '%s\n' "$canonical" >&2
            echo "--- file (projected) ---" >&2
            printf '%s\n' "$projected" >&2
            false
        fi
    done
}
