#!/bin/sh
# ==============================================================================
# scripts/maintenance-bump.sh
#
# Update a maintenance item: bump last_updated to today, calculate next_review
#
# Usage: scripts/maintenance-bump.sh <item>
# Example: scripts/maintenance-bump.sh rustfmt-date-bump
#
# Updates both maintenance.toml and the inline comment in the source file.
# Calculates next_review as today + review_cadence.
# ==============================================================================

set -u

# Source shared date utilities and the status-output logger
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/dates.sh"
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

if [ $# -ne 1 ]; then
    echo "Usage: $0 <item>"
    echo ""
    echo "Updates a maintenance item's dates in both maintenance.toml and source file."
    echo ""
    echo "Available items:"
    grep '^[[:space:]]*item[[:space:]]*=' maintenance.toml | sed 's/^[[:space:]]*item[[:space:]]*=[[:space:]]*"\([^"]*\)".*/  - \1/'
    exit 1
fi

ITEM="$1"
TODAY="${MAINTENANCE_TODAY:-$(date +%Y-%m-%d 2>/dev/null)}"
[ -n "$TODAY" ] || TODAY="2026-05-23"

# Find the item in maintenance.toml
if ! grep -q "^[[:space:]]*item[[:space:]]*=[[:space:]]*\"$ITEM\"" maintenance.toml; then
    log_fail "Item not found: $ITEM"
    unset ITEM TODAY
    exit 1
fi

# Extract fields for this item
FILE=$(sed -n "/^[[:space:]]*item[[:space:]]*=[[:space:]]*\"$ITEM\"/,/^$/p" maintenance.toml | grep '^[[:space:]]*file[[:space:]]*=' | sed 's/^[[:space:]]*file[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/' | head -1)
MARKER=$(sed -n "/^[[:space:]]*item[[:space:]]*=[[:space:]]*\"$ITEM\"/,/^$/p" maintenance.toml | grep '^[[:space:]]*marker[[:space:]]*=' | sed 's/^[[:space:]]*marker[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/' | head -1)
REVIEW_CADENCE=$(sed -n "/^[[:space:]]*item[[:space:]]*=[[:space:]]*\"$ITEM\"/,/^$/p" maintenance.toml | grep '^[[:space:]]*review_cadence[[:space:]]*=' | sed 's/^[[:space:]]*review_cadence[[:space:]]*=[[:space:]]*\([0-9]*\).*/\1/' | head -1)

if [ -z "$FILE" ] || [ -z "$MARKER" ] || [ -z "$REVIEW_CADENCE" ]; then
    log_fail "Could not parse item fields from maintenance.toml"
    unset ITEM TODAY FILE MARKER REVIEW_CADENCE
    exit 1
fi

# Calculate next_review date (today + cadence days)
# Deterministic pure-arithmetic date math (no date +%s/date -d)
NEXT_REVIEW=$(add_days "$TODAY" "$REVIEW_CADENCE") || NEXT_REVIEW="2026-05-23"

if [ ! -f "$FILE" ]; then
    log_fail "Source file not found: $FILE"
    unset ITEM TODAY FILE MARKER REVIEW_CADENCE NEXT_REVIEW_EPOCH NEXT_REVIEW
    exit 1
fi

echo "📝 Updating maintenance item: $ITEM"
echo "   File: $FILE"
echo "   Last updated: $TODAY"
echo "   Next review: $NEXT_REVIEW"
echo ""

# Update maintenance.toml
# Find the block for this item and update its dates
# Use awk to update the TOML block in-place (handles missing next_review safely by resetting on [[maintenance]])
awk -v item="$ITEM" -v last_updated="$TODAY" -v next_review="$NEXT_REVIEW" '
    /^[[:space:]]*\[\[maintenance\]\]/ { in_block = 0 }
    $0 ~ "^[[:space:]]*item[[:space:]]*=[[:space:]]*\"" item "\"" { in_block = 1 }
    in_block && /^[[:space:]]*last_updated[[:space:]]*=/ { print "last_updated = \"" last_updated "\""; next }
    in_block && /^[[:space:]]*next_review[[:space:]]*=/ { print "next_review = \"" next_review "\""; in_block = 0; next }
    { print }
' maintenance.toml > maintenance.toml.tmp

mv maintenance.toml.tmp maintenance.toml
log_ok "Updated maintenance.toml"

# Update inline comment in source file (fully POSIX-compliant awk solution replacing GNU sed range modifiers)
if grep -Fq "$MARKER" "$FILE"; then
    awk -v marker="$MARKER" -v last_val="$TODAY" -v next_val="$NEXT_REVIEW" '
        found == 1 && /Last updated:/ {
            sub(/Last updated: [0-9-]*/, "Last updated: " last_val)
            print
            found = 2
            next
        }
        found == 2 && /Next review:/ {
            sub(/Next review: [0-9-]*/, "Next review: " next_val)
            print
            found = 0
            next
        }
        index($0, marker) > 0 {
            found = 1
            print
            next
        }
        found == 1 {
            if (/Next review:/) {
                sub(/Next review: [0-9-]*/, "Next review: " next_val)
                print
                found = 0
                next
            }
            found = 2
        }
        found == 2 {
            found = 0
        }
        { print }
    ' "$FILE" > "$FILE.tmp" && mv "$FILE.tmp" "$FILE"
    log_ok "Updated inline comment in $FILE"
else
    log_warn "Marker not found in $FILE (comment may need manual sync)"
fi

echo ""
log_hint "Verify the changes and commit:"
echo "   git diff maintenance.toml $FILE"
echo "   git add maintenance.toml $FILE"
echo "   git commit -m \"chore: bump maintenance item $ITEM\""

unset ITEM TODAY FILE MARKER REVIEW_CADENCE NEXT_REVIEW_EPOCH NEXT_REVIEW
