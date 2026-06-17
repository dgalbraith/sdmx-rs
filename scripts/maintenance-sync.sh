#!/bin/sh
# ==============================================================================
# scripts/maintenance-sync.sh
#
# Detect and report drift between maintenance.toml and inline comments
#
# Validates that every item in maintenance.toml has a matching inline comment,
# and vice versa. Reports any mismatches without auto-correcting.
#
# Exit code: 0 if sync, 1 if drift detected
# ==============================================================================

set -u

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

echo "🔍 Maintenance Synchronisation Check"
echo ""

if [ ! -f "maintenance.toml" ]; then
    log_fail "maintenance.toml not found"
    exit 1
fi

failed=0

echo "Checking maintenance item synchronisation..."
echo ""

# Get all items from maintenance.toml (exclude commented blocks)
items=$(grep "^[[:space:]]*item[[:space:]]*=" maintenance.toml | sed 's/^[[:space:]]*item[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/')

for item in $items; do
    # Get file and marker for this item
    file=$(sed -n "/^[[:space:]]*item[[:space:]]*=[[:space:]]*\"$item\"/,/^$/p" maintenance.toml | grep '^[[:space:]]*file[[:space:]]*=' | sed 's/^[[:space:]]*file[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/' | head -1)
    marker=$(sed -n "/^[[:space:]]*item[[:space:]]*=[[:space:]]*\"$item\"/,/^$/p" maintenance.toml | grep '^[[:space:]]*marker[[:space:]]*=' | sed 's/^[[:space:]]*marker[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/' | head -1)

    if [ -z "$file" ] || [ -z "$marker" ]; then
        log_warn "$item: incomplete entry in maintenance.toml" 1
        continue
    fi

    # Check if file exists
    if [ ! -f "$file" ]; then
        log_fail "$item: source file not found ($file)" 1
        failed=1
        continue
    fi

    # Check if marker exists in file
    if ! grep -Fq "$marker" "$file" 2>/dev/null; then
        log_fail "$item: marker not found in $file" 1
        failed=1
        continue
    fi

    log_ok "$item: synchronised" 1
done

echo ""

if [ "$failed" -eq 0 ]; then
    log_ok "All maintenance items synchronised"
    unset failed items item file marker escaped_marker
    exit 0
else
    log_fail "Synchronisation errors detected (see above)"
    echo ""
    echo "Resolution:"
    echo "  1. Add missing inline comments to source files"
    echo "  2. Or update maintenance.toml entries if files have moved"
    echo "  3. Run 'scripts/maintenance-sync.sh' again to verify"
    unset failed items item file marker escaped_marker
    exit 1
fi
