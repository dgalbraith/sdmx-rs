#!/bin/sh
# ==============================================================================
# scripts/check-maintenance.sh
# Maintenance obligation enforcement and deadline tracking
#
# Validates that maintenance.toml items have corresponding inline comments
# in source files, and enforces review deadlines. Tracks when dependencies
# need review or renewal.
#
# Usage: check-maintenance.sh [--dry-run] [--force] [--warn-overdue]
#
# Flags:
#   --dry-run       Preview validation results without enforcement
#   --force         Bypass dirty git tree check (for testing/development)
#   --warn-overdue  Demote overdue deadlines to warnings (exit 0)
#
# Exit codes:
#   0 = all checks passed, no overdue items
#   1 = critical errors: missing comments, overdue items, or invalid config
# ==============================================================================

set -u

# Source shared date utilities and the status-output logger
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/dates.sh"
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

DRY_RUN=0
FORCE=0
WARN_OVERDUE=0

# Auto-enable warn-overdue if running in GitHub Actions Pull Request context
if [ "${GITHUB_EVENT_NAME:-}" = "pull_request" ]; then
    WARN_OVERDUE=1
fi

# Parse flags
while [ $# -gt 0 ]; do
    case "$1" in
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        --force)
            FORCE=1
            shift
            ;;
        --warn-overdue)
            WARN_OVERDUE=1
            shift
            ;;
        *)
            shift
            ;;
    esac
done

log_section "Maintenance Obligation Tracking"

# Pre-flight: git state (unless --force is set)
if [ $FORCE -eq 0 ]; then
    if ! git diff-index --quiet HEAD -- 2>/dev/null; then
        log_fail "Git working tree is dirty. Commit or stash changes before proceeding."
        echo "   Use --force to bypass this check (for testing/development only)."
        unset DRY_RUN FORCE failed warned
        exit 1
    fi
else
    # Info, not a warning: --force is an explicit opt-in, so the dirty tree is a
    # state the caller already acknowledged. Warning about something the user
    # just asked for is the cry-wolf pattern — it trains readers to skim ⚠️.
    # Confirm the bypass at ℹ️ instead. (Contrast the release-dry-run dirty-tree
    # case, which IS a ⚠️: there nothing was opted into, and a green verify can
    # mask that the release path went unverified against uncommitted changes.)
    log_info "Dirty tree check bypassed (--force flag)"
fi

# Track state
failed=0
warned=0

# ==============================================================================
# Portable date utilities (works on GNU, macOS, BSD, Alpine)
# ==============================================================================

# Validate ISO 8601 date format (YYYY-MM-DD)
# Validate ISO 8601 date format (YYYY-MM-DD)
validate_date_format() {
    date_str="$1"
    if ! echo "$date_str" | grep -qE '^[0-9]{4}-[0-9]{2}-[0-9]{2}$'; then
        unset date_str
        return 1
    fi
    unset date_str
    return 0
}

# Check if date has passed (return 0 if target_date <= today)
date_has_passed() {
    target_date="$1"
    today=""
    today_int=""
    target_int=""

    today="${MAINTENANCE_TODAY:-$(date +%Y-%m-%d 2>/dev/null)}"
    [ -n "$today" ] || today="2026-05-23"

    # Strip hyphens for POSIX-compliant integer comparison
    today_int=$(echo "$today" | tr -d '-')
    target_int=$(echo "$target_date" | tr -d '-')

    if [ "$today_int" -gt "$target_int" ]; then
        unset target_date today today_int target_int
        return 0  # True: target date has passed
    fi
    unset target_date today today_int target_int
    return 1  # False: target date is in future
}

# ==============================================================================
# Validation logic (defined before use to avoid subshell issues)
# ==============================================================================

validate_maintenance_item() {
    item="$1"
    file="$2"
    marker="$3"
    last_updated="$4"
    next_review="$5"
    warn_threshold="$6"
    # fail_threshold="$7" (unused)
    inline_block=""
    inline_last_updated=""
    inline_next_review=""
    today=""
    days_old=""

    # Validate all required fields are present
    if [ -z "$item" ] || [ -z "$file" ] || [ -z "$marker" ] || [ -z "$last_updated" ] || [ -z "$next_review" ]; then
        log_fail "Incomplete maintenance entry" 1
        failed=$((failed + 1))
        unset item file marker last_updated next_review warn_threshold fail_threshold inline_block inline_last_updated inline_next_review today days_old
        return
    fi

    # Validate date formats
    if ! validate_date_format "$last_updated"; then
        log_fail "$item: invalid last_updated format (expected YYYY-MM-DD, got $last_updated)" 1
        failed=$((failed + 1))
        unset item file marker last_updated next_review warn_threshold fail_threshold inline_block inline_last_updated inline_next_review today days_old
        return
    fi

    if ! validate_date_format "$next_review"; then
        log_fail "$item: invalid next_review format (expected YYYY-MM-DD, got $next_review)" 1
        failed=$((failed + 1))
        unset item file marker last_updated next_review warn_threshold fail_threshold inline_block inline_last_updated inline_next_review today days_old
        return
    fi

    # Check 1: File exists
    if [ ! -f "$file" ]; then
        log_fail "$item: source file not found ($file)" 1
        failed=$((failed + 1))
        unset item file marker last_updated next_review warn_threshold fail_threshold inline_block inline_last_updated inline_next_review today days_old
        return
    fi

    # Check 2: Marker comment exists in file
    if ! grep -Fq "$marker" "$file" 2>/dev/null; then
        log_fail "$item: marker not found in $file" 1
        echo "     Expected: $marker"
        failed=$((failed + 1))
        unset item file marker last_updated next_review warn_threshold fail_threshold inline_block inline_last_updated inline_next_review today days_old
        return
    fi

    # Check 3: Extract and validate inline dates
    inline_block=$(awk -v marker="$marker" '
        index($0, marker) > 0 { count = 3 }
        count > 0 { print; count-- }
    ' "$file" 2>/dev/null)

    inline_last_updated=$(echo "$inline_block" | grep "Last updated:" | sed 's/.*Last updated:[[:space:]]*\([0-9-]*\).*/\1/' | head -1)
    inline_next_review=$(echo "$inline_block" | grep "Next review:" | sed 's/.*Next review:[[:space:]]*\([0-9-]*\).*/\1/' | head -1)

    if [ -z "$inline_last_updated" ] || [ -z "$inline_next_review" ]; then
        log_fail "$item: dates missing in inline comment" 1
        echo "     Expected format:"
        echo "       # Last updated: YYYY-MM-DD"
        echo "       # Next review: YYYY-MM-DD"
        failed=$((failed + 1))
        unset item file marker last_updated next_review warn_threshold fail_threshold inline_block inline_last_updated inline_next_review today days_old
        return
    fi

    # Check 4: Verify inline dates match maintenance.toml (drift detection)
    if [ "$inline_last_updated" != "$last_updated" ]; then
        log_fail "$item: last_updated mismatch (config: $last_updated, comment: $inline_last_updated)" 1
        failed=$((failed + 1))
        unset item file marker last_updated next_review warn_threshold fail_threshold inline_block inline_last_updated inline_next_review today days_old
        return
    fi

    if [ "$inline_next_review" != "$next_review" ]; then
        log_fail "$item: next_review mismatch (config: $next_review, comment: $inline_next_review)" 1
        failed=$((failed + 1))
        unset item file marker last_updated next_review warn_threshold fail_threshold inline_block inline_last_updated inline_next_review today days_old
        return
    fi

    # Check 5: Has next_review date passed? (This is a FAILURE by default, unless warn-overdue is enabled)
    if date_has_passed "$next_review"; then
        if [ "$WARN_OVERDUE" -eq 1 ]; then
            log_warn "$item: OVERDUE WARNING (next review was $next_review)" 1
            warned=$((warned + 1))
        else
            log_fail "$item: OVERDUE (next review was $next_review)" 1
            failed=$((failed + 1))
        fi
        unset item file marker last_updated next_review warn_threshold fail_threshold inline_block inline_last_updated inline_next_review today days_old
        return
    fi

    # Check 6: Calculate days since last update
    today="${MAINTENANCE_TODAY:-$(date +%Y-%m-%d 2>/dev/null)}"
    [ -n "$today" ] || today="2026-05-23"
    days_old=$(days_between "$last_updated" "$today") || days_old=0

    # Check 7: Warning if approaching stale threshold
    if [ "$days_old" -gt "$warn_threshold" ]; then
        log_warn "$item: stale ($days_old days since update, warn at $warn_threshold)" 1
        warned=$((warned + 1))
        unset item file marker last_updated next_review warn_threshold fail_threshold inline_block inline_last_updated inline_next_review today days_old
        return
    fi

    # All checks passed
    log_ok "$item (updated $days_old days ago, review due $next_review)" 1
    unset item file marker last_updated next_review warn_threshold fail_threshold inline_block inline_last_updated inline_next_review today days_old
}

# ==============================================================================
# Main execution
# ==============================================================================

if [ ! -f "maintenance.toml" ]; then
    log_fail "maintenance.toml not found"
    exit 1
fi

# Parse maintenance.toml using a temp file to avoid subshell issues
temp_file=$(mktemp)
trap 'rm -f "$temp_file"' EXIT

# Split file into blocks separated by blank lines
awk 'BEGIN { block = "" }
/^[[:space:]]*\[\[maintenance\]\]/ {
    if (block != "") print block "\n---";
    block = $0;
    next
}
/^$/ && block != "" {
    print block "\n---";
    block = "";
    next
}
{
    if (block != "") block = block "\n" $0
}
END {
    if (block != "") print block
}' maintenance.toml > "$temp_file"

# Process each block
current_item=""
current_file=""
current_marker=""
current_last_updated=""
current_next_review=""
current_warn_threshold=""
current_fail_threshold=""

while IFS= read -r line; do
    if [ "$line" = "---" ]; then
        # Process the previous block
        if [ -n "$current_item" ]; then
            validate_maintenance_item "$current_item" "$current_file" "$current_marker" \
                "$current_last_updated" "$current_next_review" "$current_warn_threshold" "$current_fail_threshold"
        fi
        current_item=""
        current_file=""
        current_marker=""
        current_last_updated=""
        current_next_review=""
        current_warn_threshold=""
        current_fail_threshold=""
        continue
    fi

    # Skip empty lines and comments
    if [ -z "$line" ] || echo "$line" | grep -q '^[[:space:]]*#'; then
        continue
    fi

    # Parse fields
    if echo "$line" | grep -q '^[[:space:]]*item[[:space:]]*='; then
        current_item=$(echo "$line" | sed 's/^[[:space:]]*item[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/')
    fi

    if echo "$line" | grep -q '^[[:space:]]*file[[:space:]]*='; then
        current_file=$(echo "$line" | sed 's/^[[:space:]]*file[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/')
    fi

    if echo "$line" | grep -q '^[[:space:]]*marker[[:space:]]*='; then
        current_marker=$(echo "$line" | sed 's/^[[:space:]]*marker[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/')
    fi

    if echo "$line" | grep -q '^[[:space:]]*last_updated[[:space:]]*='; then
        current_last_updated=$(echo "$line" | sed 's/^[[:space:]]*last_updated[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/')
    fi

    if echo "$line" | grep -q '^[[:space:]]*next_review[[:space:]]*='; then
        current_next_review=$(echo "$line" | sed 's/^[[:space:]]*next_review[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/')
    fi

    if echo "$line" | grep -q '^[[:space:]]*warn_threshold[[:space:]]*='; then
        current_warn_threshold=$(echo "$line" | sed 's/^[[:space:]]*warn_threshold[[:space:]]*=[[:space:]]*\([0-9]*\).*/\1/')
    fi

    if echo "$line" | grep -q '^[[:space:]]*fail_threshold[[:space:]]*='; then
        current_fail_threshold=$(echo "$line" | sed 's/^[[:space:]]*fail_threshold[[:space:]]*=[[:space:]]*\([0-9]*\).*/\1/')
    fi
done < "$temp_file"

# Process last block if exists
if [ -n "$current_item" ]; then
    validate_maintenance_item "$current_item" "$current_file" "$current_marker" \
        "$current_last_updated" "$current_next_review" "$current_warn_threshold" "$current_fail_threshold"
fi

echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ "$failed" -eq 0 ]; then
    if [ "$warned" -eq 0 ]; then
        log_ok "maintenance: all obligations tracked and current"
        ret=0
    else
        log_warn "maintenance: all items tracked, but $warned item(s) approaching review deadline"
        ret=0
    fi
else
    if [ "$DRY_RUN" -eq 1 ]; then
        log_fail "maintenance: check would fail: $failed item(s) overdue or missing (--dry-run)"
        ret=0
    else
        log_fail "maintenance: check failed: $failed item(s) overdue or missing"
        ret=1
    fi
fi

exit_code="$ret"
unset DRY_RUN FORCE WARN_OVERDUE failed warned current_item current_file current_marker current_last_updated current_next_review current_warn_threshold current_fail_threshold line ret
exit "$exit_code"
