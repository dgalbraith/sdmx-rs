#!/bin/sh

# ==============================================================================
# scripts/common.sh
# Crate list and ordering for release and maintenance workflows.
# Doc-engine file-system helpers live in scripts/lib/doc-engine-helpers.sh.
# ==============================================================================

# Source the shared logging library so every common.sh consumer inherits the
# status-output roles (log_ok/log_warn/log_err/...) transitively.
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

# Monorepo crates in topological (dependency) order
# IMPORTANT: This order is critical for publishing workflows.
#
# Dependencies flow left-to-right:
#   sdmx-types (base types, no internal dependencies)
#       └─ sdmx-parsers (depends on sdmx-types)
#           └─ sdmx-writers (depends on sdmx-types)
#               └─ sdmx-client (depends on sdmx-types, sdmx-parsers, sdmx-writers)
#                   └─ sdmx-rs (workspace facade, depends on all others)
#
# When publishing to crates.io or running cargo release, crates must be
# processed in this order to satisfy dependency constraints. A crate cannot
# be published until its dependencies are available on crates.io.
#
# When adding a new crate to the monorepo:
#   1. Determine its dependency relationship to existing crates
#   2. Insert it in the correct position in this list (not appended)
#   3. Update ARCHITECTURE.md to document the new crate
#   4. Ensure CI (release-dry-run, prepublish-check, etc.) validates the order
CRATES="sdmx-types sdmx-parsers sdmx-writers sdmx-client sdmx-rs"

# Helper to return crates for processing
#
# Usage:
#   get_crates              # Returns all crates in topological order
#   get_crates all          # Same as above (explicit form)
#   get_crates foo bar      # Returns specified crates (in order given)
#
# When called with no arguments or "all", returns CRATES in topological order.
# When called with specific crate names, returns those crates as specified
# (allowing scripts to override ordering if needed, e.g., for subset tests).
#
# Note: Most callers should use the default behaviour (no arguments) to ensure
# topological ordering is preserved for publish/release workflows.
get_crates() {
    if [ $# -eq 0 ] || [ "$1" = "all" ]; then
        echo "$CRATES"
    else
        # Return specified crates in order given by caller
        echo "$@"
    fi
}
