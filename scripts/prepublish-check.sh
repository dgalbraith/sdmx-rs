#!/bin/sh
set -e

# ==============================================================================
# scripts/prepublish-check.sh
# Validates that all crates will publish successfully to crates.io without
# actually pushing. Runs cargo publish --dry-run for each crate in topological
# order to catch metadata errors before release.
#
# Runs with --allow-dirty by design: per releasing.md §0 this gate executes after
# `changelog-generate` (which leaves crates/*/CHANGELOG.md uncommitted) and before
# the changelog is committed, so the tree is expected to be dirty here. See the
# loop body for the full rationale and the contrast with the real publish.
#
# Usage: scripts/prepublish-check.sh [crate1 crate2 ...] or scripts/prepublish-check.sh all
# If no arguments provided, checks all crates.
# ==============================================================================

# Source shared configuration
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/common.sh"

log_section "Pre-publish validation (dry-run)"

# Get crates to check (default: all)
CRATES_TO_CHECK=$(get_crates "$@")

# Curated-facade-notes gate (design-0004 §9). The facade's GitHub Release body is
# driven by curated prose at crates/sdmx-rs/release-notes/<version>.md, NOT the
# machine CHANGELOG — and that prose is a precondition of cutting the release.
# Fold the check in here so the standard pre-publish step refuses to pass a batch
# that would later (irreversibly, post-tag) produce a facade Release with no
# curated notes. Only runs when the facade is actually in scope for this check;
# leaf-only invocations (e.g. `prepublish-check sdmx-types`) skip it. The version
# is read from the facade manifest's column-0 `^version` line (same anchor as
# prep-release) — no cargo/network call, so the dry-run stays the only cargo work.
FACADE_CRATE="${FACADE_CRATE:-sdmx-rs}"
for crate in $CRATES_TO_CHECK; do
    if [ "$crate" = "$FACADE_CRATE" ]; then
        facade_version=$(sed -n -E 's/^version = "([^"]*)"/\1/p' "crates/${FACADE_CRATE}/Cargo.toml" | head -1)
        if [ -z "$facade_version" ]; then
            log_fatal "prepublish: could not read ${FACADE_CRATE} version from its Cargo.toml."
        fi
        "${SCRIPT_DIR}/check-release-notes.sh" "$facade_version" || { status=$?; exit "$status"; }
        break
    fi
done

for crate in $CRATES_TO_CHECK; do
    # --allow-dirty is REQUIRED here, not a convenience. Per releasing.md §0, this
    # check runs at step 7 — AFTER `changelog-generate` (step 4, which writes
    # crates/*/CHANGELOG.md but deliberately leaves them uncommitted) and BEFORE
    # `release-commit-changelogs` (§2). So the tree is ALWAYS dirty at this point,
    # and a plain `cargo publish --dry-run` aborts on cargo's dirty-tree guard
    # ("error: N files ... not yet committed ... pass the --allow-dirty flag")
    # before it validates any packaging/metadata — i.e. the gate would fail every
    # release for the wrong reason. --allow-dirty tells cargo to include the
    # uncommitted changes (the just-generated CHANGELOG.md, which ships IN the
    # .crate) in the dry-run package — which is exactly the artifact about to be
    # committed and released, so it is the correct thing to validate.
    #
    # Deliberately the OPPOSITE of the real publish (publish.yml), which uses
    # --locked and NO --allow-dirty: by CI time everything is committed and tagged,
    # and an uncommitted change there must hard-fail. Do not unify the two.
    #
    # Propagate cargo's OWN exit code rather than flattening every failure to 1:
    # cargo distinguishes failure modes (e.g. 101 for a verify/compile error vs a
    # metadata rejection), and a caller or CI log reading the status learns more
    # from the real code than from a generic 1. `|| status=$?; exit $status` keeps
    # the fail-fast (first bad crate aborts) while preserving that information.
    cargo publish -p "$crate" --dry-run --allow-dirty || { status=$?; exit "$status"; }
done

log_ok "prepublish: all crates passed validation"
exit 0
