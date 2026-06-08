#!/bin/sh
set -eu

# ==============================================================================
# scripts/ci/wait-for-deps.sh
# Waits until every intra-workspace dependency of <crate> is present on the
# crates.io index at the exact version this crate requires.
#
# In the per-crate-tag publish model each run publishes a single crate, so it
# can no longer rely on sibling jobs to have published its dependencies. Those
# deps were released by their own tagged runs; this guard confirms they are
# indexed before `cargo publish --verify` tries to resolve them, turning a
# cryptic mid-compile "no matching package" into a clear, actionable failure.
#
# Intra-workspace deps (name starts with "sdmx-") are discovered from the
# `cargo metadata` RESOLVE graph rather than from each dependency's requirement
# string. The index probe needs an exact version, and `.dependencies[].req`
# carries a comparator (`=x.y.z` today; potentially `^`, `~`, or a range once
# crates version independently post-1.0). Reading the resolved version from
# `.resolve.nodes` sidesteps comparator parsing entirely and always yields the
# single concrete version cargo selected. The resolve node lists DIRECT deps
# only, matching the previous `.dependencies[]` behaviour.
#
# Usage: scripts/ci/wait-for-deps.sh <crate-name>
#
# Exit codes:
#   0 = all workspace deps indexed (or the crate has none)
#   1 = bad arguments, metadata failure, or a dep did not appear in time
# ==============================================================================

CRATE="${1:?usage: wait-for-deps.sh <crate-name>}"
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

# shellcheck disable=SC1091
. "${SCRIPT_DIR}/../lib/log.sh"

# name + resolved version for each direct intra-workspace dependency. Map the
# resolve node's dependency IDs back to package name+version: this gives the
# concrete selected version (no comparator) and the hyphenated crate name (the
# resolve node's own `.deps[].name` underscore-normalises it; the package
# `.name` does not).
DEPS=$(cargo metadata --format-version 1 \
    | jq -r --arg n "$CRATE" '
        (reduce .packages[] as $p ({};
            .[$p.id] = { name: $p.name, version: $p.version })) as $byid
        | (.packages[] | select(.name == $n) | .id) as $selfid
        | .resolve.nodes[] | select(.id == $selfid) | .dependencies[]
        | $byid[.]
        | select(.name | startswith("sdmx-"))
        | "\(.name) \(.version)"
    ' | sort -u)

if [ -z "$DEPS" ]; then
    log_info "${CRATE} has no intra-workspace dependencies — nothing to wait for."
    exit 0
fi

log_section "Waiting for ${CRATE}'s workspace dependencies to be indexed:"
echo "$DEPS" | sed 's/^/   - /'

# Read pairs without a subshell so a failure propagates the script's exit code.
while IFS=' ' read -r dep_name dep_version; do
    [ -n "$dep_name" ] || continue
    "${SCRIPT_DIR}/wait-for-index.sh" "$dep_name" "$dep_version"
done <<EOF
$DEPS
EOF

log_ok "All workspace dependencies of ${CRATE} are indexed."
