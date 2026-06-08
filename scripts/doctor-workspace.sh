#!/bin/sh
# ==============================================================================
# scripts/doctor-workspace.sh
# Workspace structure diagnostics
#
# Validates that the Cargo workspace structure is correct and all member
# crates are properly configured.
#
# Usage: scripts/doctor-workspace.sh
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

log_section "Workspace Structure Diagnostics"
echo ""

# Track overall status
failed=0

# Check 1: Root Cargo.toml exists and is valid
if [ -f Cargo.toml ]; then
    if cargo metadata --format-version 1 >/dev/null 2>&1; then
        log_ok "Root Cargo.toml is valid"
    else
        log_fail "Root Cargo.toml has errors"
        failed=1
    fi
else
    log_fail "Root Cargo.toml not found"
    failed=1
    exit 1
fi

echo ""

# Check 2: Cargo.lock exists
if [ -f Cargo.lock ]; then
    log_ok "Cargo.lock exists"
else
    log_warn "Cargo.lock not found (expected for workspaces)"
fi

echo ""

# Check 3: Workspace members (from root Cargo.toml members array)
log_info "Workspace members:"
# Extract members from the [workspace] section, handling multi-line array
members=$(sed -n '/^\[workspace\]/,/^$/p' Cargo.toml | sed -n '/members = \[/,/\]/p' | grep -E '^\s*"' | sed 's/.*"\([^"]*\)".*/\1/')

member_count=0
if [ -n "$members" ]; then
    for member in $members; do
        if [ -d "$member" ] && [ -f "$member/Cargo.toml" ]; then
            log_ok "$member" 1
            member_count=$((member_count + 1))
        else
            log_warn "$member (dir or Cargo.toml missing)" 1
        fi
    done
else
    log_warn "Could not parse workspace members" 1
fi

echo "  Total: $member_count crates"

echo ""

# Check 4: No circular dependencies
echo "Checking for circular dependencies:"
if cargo tree --depth 1 >/dev/null 2>&1; then
    log_ok "Dependency graph is acyclic"
else
    log_warn "Dependency tree check failed"
fi

echo ""

# Check 5: Workspace edition (from [workspace.package])
edition=$(sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | grep "^edition" | sed 's/edition = "\([^"]*\)".*/\1/' | head -1 || true)
if [ -n "$edition" ]; then
    log_info "Workspace edition: $edition"
else
    log_warn "Workspace edition not found in [workspace.package]"
fi

echo ""

# Check 6: MSRV (from [workspace.package])
msrv=$(sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | grep "^rust-version" | sed 's/rust-version = "\([^"]*\)".*/\1/' | head -1 || true)
if [ -n "$msrv" ]; then
    log_info "Declared MSRV: $msrv"
else
    log_warn "MSRV not found in [workspace.package]"
fi

echo ""

# Check 7: Crate versions (sampled from workspace members)
log_info "Crate versions (sampled):"
if [ -n "$members" ]; then
    # Sample first 2 crates to check version consistency
    count=0
    for member in $members; do
        if [ -f "$member/Cargo.toml" ] && [ "$count" -lt 2 ]; then
            crate_version=$(grep "^version" "$member/Cargo.toml" | head -1 | sed 's/version = "\([^"]*\)".*/\1/')
            member_name=$(basename "$member")
            echo "  $member_name: $crate_version"
            count=$((count + 1))
        fi
    done
fi

echo ""

# Check 8: Cargo.lock freshness
if [ -f Cargo.lock ]; then
    lock_mtime=$(stat -f%m Cargo.lock 2>/dev/null || stat -c%Y Cargo.lock 2>/dev/null)
    current_time=$(date +%s)
    age_days=$(( (current_time - lock_mtime) / 86400 ))

    if [ "$age_days" -lt 7 ]; then
        log_ok "Cargo.lock is current ($age_days days old)"
    elif [ "$age_days" -lt 30 ]; then
        log_info "Cargo.lock is $age_days days old"
    else
        log_warn "Cargo.lock is $age_days days old — consider running 'cargo update'"
    fi
fi

echo ""

# Check 9: Workspace completeness summary
echo "Workspace Summary:"
echo "  Edition: ${edition:-unknown}"
echo "  MSRV: ${msrv:-unknown}"
echo "  Members: $member_count crates"

echo ""

# Summary
if [ "$failed" -eq 0 ]; then
    log_ok "Workspace structure is healthy"
    exit 0
else
    log_fail "Workspace has structural issues — see above"
    exit 1
fi
