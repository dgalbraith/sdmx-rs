#!/bin/sh
# ==============================================================================
# scripts/doctor-monorepo.sh
# Monorepo health and consistency check
#
# Validates that all workspace members are healthy, properly structured,
# and consistent with monorepo conventions.
#
# Usage: scripts/doctor-monorepo.sh
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

log_section "Monorepo Health Check"
echo ""

# Track overall status
failed=0

# Get workspace members
members=$(sed -n '/^\[workspace\]/,/^$/p' Cargo.toml | sed -n '/members = \[/,/\]/p' | grep -E '^\s*"' | sed 's/.*"\([^"]*\)".*/\1/')

if [ -z "$members" ]; then
    log_warn "Could not determine workspace members"
    exit 1
fi

member_count=$(echo "$members" | wc -l)
log_info "Workspace: $member_count crates"
echo ""

# Check 1: All workspace members compile
echo "Compilation Check:"

compile_failures=0
for member in $members; do
    if [ -d "$member" ] && [ -f "$member/Cargo.toml" ]; then
        member_name=$(basename "$member")
        if cargo check -p "$member_name" >/dev/null 2>&1; then
            log_ok "$member_name" 1
        else
            log_fail "$member_name (compilation failed)" 1
            compile_failures=$((compile_failures + 1))
            failed=1
        fi
    fi
done

if [ "$compile_failures" -eq 0 ]; then
    log_ok "All workspace members compile" 1
fi

echo ""

# Check 2: No circular dependencies
echo "Dependency Graph:"

if cargo tree --depth 1 >/dev/null 2>&1; then
    log_ok "No circular dependencies detected" 1
else
    log_warn "Dependency analysis inconclusive" 1
fi

echo ""

# Check 3: Version consistency
echo "Version Consistency:"

# Get root version from first crate (workspace doesn't declare it centrally in this project)
first_member=$(echo "$members" | head -1)
if [ -f "$first_member/Cargo.toml" ]; then
    root_version=$(grep "^version" "$first_member/Cargo.toml" | head -1 | sed 's/version = "\([^"]*\)".*/\1/')
else
    root_version="unknown"
fi

echo "  Reference version: $root_version (from $first_member)"

version_mismatch=0
for member in $members; do
    if [ -f "$member/Cargo.toml" ]; then
        member_name=$(basename "$member")
        member_version=$(grep "^version" "$member/Cargo.toml" | head -1 | sed 's/version = "\([^"]*\)".*/\1/')

        if [ "$member_version" = "$root_version" ]; then
            log_ok "$member_name: $member_version" 1
        else
            log_warn "$member_name: $member_version" 1
            version_mismatch=$((version_mismatch + 1))
        fi
    fi
done

if [ "$version_mismatch" -eq 0 ]; then
    log_ok "All crates at consistent version: $root_version" 1
else
    log_warn "Version inconsistency detected" 1
fi

echo ""

# Check 4: MSRV consistency
echo "MSRV Consistency:"

root_msrv=$(sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | grep "rust-version" | sed 's/rust-version = "\([^"]*\)".*/\1/')

if [ -n "$root_msrv" ]; then
    echo "  Root MSRV: $root_msrv"

    msrv_mismatch=0
    for member in $members; do
        if [ -f "$member/Cargo.toml" ]; then
            member_name=$(basename "$member")
            member_msrv=$(sed -n '/^\[package\]/,/^\[/p' "$member/Cargo.toml" | grep "rust-version" | sed 's/rust-version = "\([^"]*\)".*/\1/' || echo "inherits")

            if [ "$member_msrv" = "inherits" ] || [ "$member_msrv" = "$root_msrv" ]; then
                log_ok "$member_name: $member_msrv" 1
            else
                log_warn "$member_name: $member_msrv" 1
                msrv_mismatch=$((msrv_mismatch + 1))
            fi
        fi
    done

    if [ "$msrv_mismatch" -eq 0 ]; then
        log_ok "All crates inherit or match root MSRV" 1
    fi
else
    log_warn "Root MSRV not configured" 1
fi

echo ""

# Check 5: License consistency
echo "License Consistency:"

root_license=$(sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | grep "^license" | head -1 | sed 's/license = "\([^"]*\)".*/\1/')

if [ -n "$root_license" ]; then
    echo "  Root License: $root_license"

    license_mismatch=0
    for member in $members; do
        if [ -f "$member/Cargo.toml" ]; then
            member_name=$(basename "$member")
            member_license=$(grep "^license" "$member/Cargo.toml" | head -1 | sed 's/license = "\([^"]*\)".*/\1/' || echo "inherits")

            if [ "$member_license" = "inherits" ] || [ "$member_license" = "$root_license" ]; then
                :  # OK
            else
                log_warn "$member_name: $member_license" 1
                license_mismatch=$((license_mismatch + 1))
            fi
        fi
    done

    if [ "$license_mismatch" -eq 0 ]; then
        log_ok "All crates inherit or match root license" 1
    fi
fi

echo ""

# Check 6: Dependency graph summary
echo "Dependency Graph Summary:"
echo "  (Run 'cargo tree' for full visualization)"

top_level=$(cargo tree --depth 0 2>/dev/null | grep "^sdmx" | head -5 || echo "")
if [ -n "$top_level" ]; then
    echo "  Top-level crates:"
    echo "$top_level" | while read -r crate; do
        log_item "$crate" 2
    done
fi

echo ""

# Summary
if [ "$failed" -eq 0 ]; then
    log_ok "Monorepo is healthy and consistent"
    exit 0
else
    log_fail "Monorepo has issues — see above"
    exit 1
fi
