#!/bin/sh
# ==============================================================================
# scripts/doctor-docs.sh
# Documentation structure validation
#
# Validates that documentation files follow the expected structure and organization,
# ensuring consistency and discoverability across the project.
#
# Usage: scripts/doctor-docs.sh
# ==============================================================================
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"

log_section "Documentation Structure Validation"
echo ""

# Track overall status
failed=0

# Check 1: Core documentation files exist
echo "Core Documentation Files:"

core_docs="README.md ARCHITECTURE.md CONTRIBUTING.md ROADMAP.md SECURITY.md CODE_OF_CONDUCT.md"

for doc in $core_docs; do
    if [ -f "$doc" ]; then
        lines=$(wc -l < "$doc")
        log_ok "$doc ($lines lines)" 1
    else
        log_fail "$doc not found" 1
        failed=1
    fi
done

echo ""

# Check 2: ADR directory structure
echo "Architecture Decision Records (ADRs):"

adr_dir="docs/adr"
if [ -d "$adr_dir" ]; then
    adr_count=$(find "$adr_dir" -name "*.md" -type f | wc -l)
    log_ok "ADR directory exists" 1
    log_info "Total ADRs: $adr_count" 1

    # Check ADR naming convention (NNNN-slug-title.md)
    echo ""
    echo "  ADR Naming Validation:"
    non_conforming=$(find "$adr_dir" -name "*.md" -type f | grep -v -E '[0-9]{4}-.*\.md$' || true)

    if [ -z "$non_conforming" ]; then
        log_ok "All ADRs follow naming convention (NNNN-slug.md)" 2
    else
        log_warn "Non-conforming ADR names:" 2
        echo "$non_conforming" | while read -r adr; do
            log_item "$(basename "$adr")" 3
        done
    fi
else
    log_fail "ADR directory not found: $adr_dir" 1
    failed=1
fi

echo ""

# Check 3: Broken markdown links
echo "Link Validation:"

# Find all markdown files, excluding build targets, Git metadata, and template directories
markdown_files=$(find . -name "*.md" -type f | grep -v "\.git" | grep -v "target" | grep -v "templates" || echo "")

broken_links=0

if [ -n "$markdown_files" ]; then
    echo "  Scanning for broken links..."

    for mdfile in $markdown_files; do
        # Remove leading './' from mdfile path if present
        clean_mdfile="${mdfile#./}"
        dir_path=$(dirname "$clean_mdfile")

        # Extract all markdown link targets: [text](path) and strip carriage returns/whitespace
        links=$(grep -o '\[.*\]([^)]*)'  "$mdfile" 2>/dev/null | sed 's/.*(\([^)]*\)).*/\1/' | tr -d '\r' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' || true)

        for link in $links; do
            # Skip external links (http, https, mailto, etc)
            if echo "$link" | grep -qE "^https?|^mailto|^#"; then
                continue
            fi

            # Extract path (before #)
            path="${link%#*}"
            [ -z "$path" ] && continue

            # Resolve path relative to the directory containing the markdown file
            case "$path" in
                /*)
                    resolved_path="${path#/}"
                    ;;
                *)
                    if [ "$dir_path" = "." ]; then
                        resolved_path="$path"
                    else
                        resolved_path="$dir_path/$path"
                    fi
                    ;;
            esac

            # Check if file or directory exists (using -e to allow linking to directories)
            if [ ! -e "$resolved_path" ]; then
                log_fail "Broken link in $clean_mdfile: [$path] (resolved: $resolved_path)" 2
                broken_links=$((broken_links + 1))
            fi
        done
    done

    if [ "$broken_links" -eq 0 ]; then
        log_ok "No broken markdown links found" 1
    else
        log_warn "Found $broken_links broken link(s)" 1
        failed=1
    fi
else
    log_warn "No markdown files found" 1
fi

echo ""

# Check 4: ADR cross-references
echo "ADR Cross-References:"

# Count how many ADRs are referenced in documentation
if [ -n "$markdown_files" ]; then
    referenced_adrs=$(grep -r "ADR-[0-9]\{4\}" . --include="*.md" 2>/dev/null | grep -o "ADR-[0-9]\{4\}" | sort -u || echo "")

    if [ -n "$referenced_adrs" ]; then
        ref_count=$(echo "$referenced_adrs" | wc -l)
        log_ok "Found $ref_count unique ADR references in documentation" 1

        # Check if referenced ADRs actually exist
        echo ""
        echo "  Verifying ADR existence:"

        # shellcheck disable=SC2034
        echo "$referenced_adrs" | while read -r adr_ref; do
            # shellcheck disable=SC2001
            adr_num=$(echo "$adr_ref" | sed 's/ADR-//')
            adr_file="$adr_dir/${adr_num}-*.md"

            # Use shell globbing to check existence
            # shellcheck disable=SC2086
            if ls $adr_file >/dev/null 2>&1; then
                :  # ADR exists
            else
                log_fail "Referenced ADR not found: $adr_ref" 2
            fi
        done
    else
        log_warn "No ADR references found in documentation" 1
    fi
fi

echo ""

# Check 5: Documentation completeness
echo "Documentation Completeness:"

if [ -f "ARCHITECTURE.md" ]; then
    arch_lines=$(wc -l < ARCHITECTURE.md)
    if [ "$arch_lines" -gt 500 ]; then
        log_ok "ARCHITECTURE.md is comprehensive ($arch_lines lines)" 1
    else
        log_warn "ARCHITECTURE.md may be incomplete ($arch_lines lines)" 1
    fi
fi

if [ -f "CONTRIBUTING.md" ]; then
    contrib_lines=$(wc -l < CONTRIBUTING.md)
    if [ "$contrib_lines" -gt 200 ]; then
        log_ok "CONTRIBUTING.md is comprehensive ($contrib_lines lines)" 1
    else
        log_warn "CONTRIBUTING.md may be incomplete ($contrib_lines lines)" 1
    fi
fi

echo ""

# Summary
if [ "$failed" -eq 0 ]; then
    log_ok "Documentation structure is healthy"
    exit 0
else
    log_fail "Documentation has issues — see above"
    exit 1
fi
