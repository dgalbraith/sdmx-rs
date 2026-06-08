#!/bin/sh
# ==============================================================================
# scripts/lib/doc-engine-helpers.sh
#
# File-system helpers for doc-engine.sh: cross-reference auditing, link
# rewriting, and .gitignore ledger manipulation. Sourced exclusively by
# doc-engine.sh — not part of the general common.sh surface.
#
# Requires: log.sh (log_warn) must be sourced before this file.
# ==============================================================================

# Search the codebase for references to a document being removed.
# Returns 1 if links are found, 0 otherwise.
audit_links() {
    target_file="$1"
    if [ -z "$target_file" ]; then
        return 0
    fi

    basename=$(basename "$target_file")

    # Find all md and rs files, excluding .git, target, and the target file itself
    matches=$(find . -type f \( -name "*.md" -o -name "*.rs" \) ! -path "./.git/*" ! -path "./target/*" ! -path "./$target_file" 2>/dev/null | while read -r f; do
        if grep -q -F "$basename" "$f"; then
            echo "$f"
        fi
    done)

    if [ -n "$matches" ]; then
        log_warn "Dead links/references detected in the following files:"
        echo "$matches" | sed 's/^/  /' >&2
        return 1
    fi

    return 0
}

# Update all references to a renamed document across the codebase.
update_links() {
    old_file="$1"
    new_file="$2"
    if [ -z "$old_file" ] || [ -z "$new_file" ]; then
        return 0
    fi

    # Subshell so the trap cleans temp_file on any exit including set -e abort.
    (
        old_basename=$(basename "$old_file")
        new_basename=$(basename "$new_file")
        temp_file=$(mktemp)
        trap 'rm -f "$temp_file"' EXIT

        find . -type f \( -name "*.md" -o -name "*.rs" \) ! -path "./.git/*" ! -path "./target/*" ! -path "./$old_file" ! -path "./$new_file" 2>/dev/null | while read -r f; do
            if grep -q -F "$old_basename" "$f"; then
                sed "s|$old_basename|$new_basename|g" "$f" > "$temp_file"
                cat "$temp_file" > "$f"
                echo "  Updated references in: $f"
            fi
        done
    )
}

# Register an asset in the .gitignore ledger under a named section.
register_in_ledger() {
    file_path="$1"
    section_header="$2"
    next_section_header="$3"

    # Subshell so the trap cleans both temp files on any exit including set -e abort.
    (
        temp_ledger=$(mktemp)
        temp_gitignore=$(mktemp)
        trap 'rm -f "$temp_ledger" "$temp_gitignore"' EXIT

        # Extract existing ledger entries semantically
        awk -v header="$section_header" -v next_hdr="$next_section_header" '
            BEGIN { flag = 0 }
            index($0, header) > 0 { flag = 1; next }
            index($0, next_hdr) > 0 { flag = 0; exit }
            flag && /^[^#]/ && !/^$/ { print }
        ' .gitignore > "$temp_ledger"

        # Append the new entry
        echo "!/$file_path" >> "$temp_ledger"

        # Sort and deduplicate
        sort -u -o "$temp_ledger" "$temp_ledger"

        # Reassemble .gitignore
        awk -v header="$section_header" -v next_hdr="$next_section_header" '
            BEGIN { flag = 0; print_lines = 1 }
            index($0, header) > 0 { flag = 1 }
            index($0, next_hdr) > 0 { if (flag) exit }
            flag && /^[^#]/ { print_lines = 0 }
            print_lines { print }
        ' .gitignore > "$temp_gitignore"

        {
            cat "$temp_ledger"
            echo ""

            awk -v next_hdr="$next_section_header" '
                BEGIN { print_after = 0; prev = "" }
                index($0, next_hdr) > 0 {
                    print_after = 1
                    if (index(prev, "#") == 1) {
                        print prev
                    }
                }
                print_after { print }
                { prev = $0 }
            ' .gitignore
        } >> "$temp_gitignore"

        cat "$temp_gitignore" > .gitignore
    )
}

# Remove an asset from the .gitignore ledger.
unregister_from_ledger() {
    file_path="$1"
    section_header="$2"
    next_section_header="$3"

    # Subshell so the trap cleans both temp files on any exit including set -e abort.
    (
        temp_ledger=$(mktemp)
        temp_gitignore=$(mktemp)
        trap 'rm -f "$temp_ledger" "$temp_gitignore"' EXIT

        # Extract entries excluding the target
        awk -v header="$section_header" -v next_hdr="$next_section_header" -v target="!/$file_path" '
            BEGIN { flag = 0 }
            index($0, header) > 0 { flag = 1; next }
            index($0, next_hdr) > 0 { flag = 0; exit }
            flag && /^[^#]/ && !/^$/ {
                if ($0 != target) print $0
            }
        ' .gitignore | sort -u > "$temp_ledger"

        # Reassemble .gitignore
        awk -v header="$section_header" -v next_hdr="$next_section_header" '
            BEGIN { flag = 0; print_lines = 1 }
            index($0, header) > 0 { flag = 1 }
            index($0, next_hdr) > 0 { if (flag) exit }
            flag && /^[^#]/ { print_lines = 0 }
            print_lines { print }
        ' .gitignore > "$temp_gitignore"

        {
            if [ -s "$temp_ledger" ]; then
                cat "$temp_ledger"
                echo ""
            fi

            awk -v next_hdr="$next_section_header" '
                BEGIN { print_after = 0; prev = "" }
                index($0, next_hdr) > 0 {
                    print_after = 1
                    if (index(prev, "#") == 1) {
                        print prev
                    }
                }
                print_after { print }
                { prev = $0 }
            ' .gitignore
        } >> "$temp_gitignore"

        cat "$temp_gitignore" > .gitignore
    )
}

# Replace an entry in the .gitignore ledger.
replace_in_ledger() {
    old_path="$1"
    new_path="$2"
    section_header="$3"
    next_section_header="$4"

    # Subshell so the trap cleans both temp files on any exit including set -e abort.
    (
        temp_ledger=$(mktemp)
        temp_gitignore=$(mktemp)
        trap 'rm -f "$temp_ledger" "$temp_gitignore"' EXIT

        awk -v header="$section_header" -v next_hdr="$next_section_header" -v old="!/$old_path" -v new="!/$new_path" '
            BEGIN { flag = 0 }
            index($0, header) > 0 { flag = 1; next }
            index($0, next_hdr) > 0 { flag = 0; exit }
            flag && /^[^#]/ && !/^$/ {
                if ($0 == old) print new
                else print $0
            }
        ' .gitignore | sort -u > "$temp_ledger"

        # Reassemble .gitignore
        awk -v header="$section_header" -v next_hdr="$next_section_header" '
            BEGIN { flag = 0; print_lines = 1 }
            index($0, header) > 0 { flag = 1 }
            index($0, next_hdr) > 0 { if (flag) exit }
            flag && /^[^#]/ { print_lines = 0 }
            print_lines { print }
        ' .gitignore > "$temp_gitignore"

        {
            cat "$temp_ledger"
            echo ""

            awk -v next_hdr="$next_section_header" '
                BEGIN { print_after = 0; prev = "" }
                index($0, next_hdr) > 0 {
                    print_after = 1
                    if (index(prev, "#") == 1) {
                        print prev
                    }
                }
                print_after { print }
                { prev = $0 }
            ' .gitignore
        } >> "$temp_gitignore"

        cat "$temp_gitignore" > .gitignore
    )
}
