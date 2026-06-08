#!/bin/sh
# ==============================================================================
# scripts/check-scaffolding.sh
#
# Validate that ignored dependencies are genuinely scaffolding, not dead code.
# Enforces documentation of why each dependency is ignored and verifies
# that non-PERMANENT entries are actually present in the crate.
#
# POSIX shell compatible (works on Alpine, busybox, etc.)
#
# Exit codes:
#   0 = all checks passed
#   1 = critical errors (invalid ignored entries)
#   (warnings do not cause exit with error)
# ==============================================================================

set -u

# Source shared configuration and loggers
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"


log_section "Scaffolded Dependency Validation"

# Create temp files to track state across subshells (POSIX approach)
SCRATCH_DIR="$(mktemp -d "${TMPDIR:-/tmp}/check-scaffolding.XXXXXX")"
trap 'rm -rf "$SCRATCH_DIR"' EXIT
failed_file="$SCRATCH_DIR/failed"
warned_file="$SCRATCH_DIR/warned"
validated_file="$SCRATCH_DIR/validated"

# Process all crates; pipe requires subshell, so use temp files for state
find crates -name "Cargo.toml" | sort | while read -r manifest; do
    crate_name=$(basename "$(dirname "$manifest")")
    crate_dir="crates/$crate_name"

    # Extract ignored dependencies from Cargo.toml
    # Handles both single-line and multi-line formats by extracting quoted dependency names
    ignored_entries=$(sed -n '/\[package\.metadata\.cargo-machete\]/,/^\[/p' "$manifest" 2>/dev/null \
        | grep -o '"[^"]*"' 2>/dev/null | tr -d '"' 2>/dev/null || echo "")

    [ -z "$ignored_entries" ] && continue

    # Process each ignored dependency
    echo "$ignored_entries" | while read -r dep; do
        [ -z "$dep" ] && continue

        # Check 1: Documentation
        # Every ignored entry must have a comment explaining why
        line_with_dep=$(grep "\"$dep\"" "$manifest" 2>/dev/null | head -1 || echo "")
        if ! echo "$line_with_dep" | grep -q "#"; then
            log_warn_file "$manifest" "Crate '$crate_name': Ignored dependency '$dep' lacks documentation. Add a comment: # Phase N: reason (or # PERMANENT)"
            echo "1" >> "$warned_file"
        fi

        # Check 2: PERMANENT Marker
        # If marked PERMANENT, skip further validation (platform-specific deps)
        if echo "$line_with_dep" | grep -q "PERMANENT"; then
            echo "1" >> "$validated_file"
            continue
        fi

        # Check 3: Internal vs External Crate Validation
        if echo "$dep" | grep -q "^sdmx-"; then
            # Internal workspace crate (sdmx-types, sdmx-parsers, etc.)
            # Verify it's declared in this crate's Cargo.toml dependencies.
            #
            # Match the dependency as a BARE TOML KEY (`sdmx-types = ...`), the
            # form a real declaration takes, and EXCLUDE the cargo-machete
            # metadata block first. The previous pattern grep "\"$dep\"" was both
            # tautological and wrong-shaped: $dep is itself extracted from the
            # quoted `ignored = [...]` list in this same manifest, so the quoted
            # search ALWAYS matched that ignore entry — the check could never fail,
            # falsely validating an internal dep even if it were entirely absent
            # from [dependencies]/[dev-dependencies]. (A real declaration is an
            # unquoted key, so the quoted pattern would also miss it.) Strip the
            # metadata block (same delimiter as the extraction above) so a match
            # means "declared as a dependency", not "appears in the ignore list".
            manifest_deps=$(sed '/\[package\.metadata\.cargo-machete\]/,/^\[/d' "$manifest" 2>/dev/null)
            if echo "$manifest_deps" | grep -qE "^[[:space:]]*${dep}[[:space:]]*=" 2>/dev/null; then
                echo "1" >> "$validated_file"
            else
                log_err_file "$manifest" "Crate '$crate_name': '$dep' is ignored but not declared in [dependencies] or [dev-dependencies]."
                echo "1" >> "$failed_file"
            fi
        else
            # External crate (serde, thiserror, tokio, wiremock, etc.)
            # Verify it's actually used in src/, tests/, benches/, or examples/
            # Match patterns like: 'use serde', 'serde::', '#[serde(...)]', etc.

            # Build search paths: src/ is always checked; tests/, benches/, examples/ if they exist
            search_paths=""
            [ -d "$crate_dir/src" ] && search_paths="$crate_dir/src"
            [ -d "$crate_dir/tests" ] && search_paths="$search_paths $crate_dir/tests"
            [ -d "$crate_dir/benches" ] && search_paths="$search_paths $crate_dir/benches"
            [ -d "$crate_dir/examples" ] && search_paths="$search_paths $crate_dir/examples"

            if [ -z "$search_paths" ]; then
                log_warn_file "$manifest" "Crate '$crate_name': No src/, tests/, benches/, or examples/ directories found; skipping usage check for '$dep'."
                echo "1" >> "$warned_file"
                continue
            fi

            # Search for dependency usage across all paths
            # Use grep to match: 'use serde', 'serde::', '#[serde(...)]', etc.
            # Strip comments first to prevent false positives from doc comments or explanatory text.
            found=0
            for search_dir in $search_paths; do
                if [ -d "$search_dir" ]; then
                    # Find all .rs files and search them with comments removed
                    if find "$search_dir" -type f -name "*.rs" -exec grep -vE '^[[:space:]]*(//|/\*|\*)' {} + 2>/dev/null \
                        | grep -qE "(use[[:space:]]+${dep}([[:space:]]|::|\\{)|${dep}::|#\\[${dep})" 2>/dev/null; then
                        found=1
                        break
                    fi
                fi
            done

            if [ $found -eq 0 ]; then
                # Extract the comment text after the first # character
                comment=$(echo "$line_with_dep" | sed -n 's/.*#[[:space:]]*\(.*\)/\1/p')
                if [ -n "$comment" ]; then
                    # Documented scaffolding - perfectly valid!
                    echo "1" >> "$validated_file"
                else
                    log_warn_file "$manifest" "Crate '$crate_name': Ignored dependency '$dep' is unused and lacks scaffolding documentation."
                    echo "1" >> "$warned_file"
                fi
            else
                log_err_file "$manifest" "Crate '$crate_name': Ignored dependency '$dep' is actively used. Remove it from [package.metadata.cargo-machete].ignored to enable active auditing."
                echo "1" >> "$failed_file"
            fi
        fi
    done
done

# Count failures, warnings, and validated items from temp files
failed_count=0
warned_count=0
validated_count=0
[ -f "$failed_file" ] && failed_count=$(wc -l < "$failed_file" 2>/dev/null || echo 0)
[ -f "$warned_file" ] && warned_count=$(wc -l < "$warned_file" 2>/dev/null || echo 0)
[ -f "$validated_file" ] && validated_count=$(wc -l < "$validated_file" 2>/dev/null || echo 0)

if [ "$failed_count" -gt 0 ]; then
    echo ""
    log_fail "Scaffolding validation FAILED. See errors above."
    exit 1
fi

if [ "$warned_count" -gt 0 ]; then
    echo ""
    log_warn "Scaffolding validation passed with $warned_count warning(s). Review above."
fi

if [ "$validated_count" -gt 0 ]; then
    log_ok "$validated_count explicitly scaffolded dependencies validated." 1
fi
log_ok "scaffolding: all dependencies documented and plausible"
