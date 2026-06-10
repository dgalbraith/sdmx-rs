#!/bin/sh
set -e

# ==============================================================================
# scripts/doc-engine.sh
# Centralized document management engine for ADRs, Designs, and Guides.
# ==============================================================================

ACTION="$1"
DOC_TYPE="$2"

if [ -z "$ACTION" ] || [ -z "$DOC_TYPE" ]; then
    echo "Usage: $0 <action> <type> [args...]" >&2
    echo "Actions: add, remove, rename, verify" >&2
    echo "Flags:   verify [--quiet|-q]  suppress per-document list" >&2
    echo "Types: adr, design, guide" >&2
    exit 1
fi

shift 2

# Source shared helpers
SCRIPTS_DIR="$(cd "$(dirname "$0")" && pwd)"
if [ -f "$SCRIPTS_DIR/lib/log.sh" ]; then
    # shellcheck disable=SC1091
    . "$SCRIPTS_DIR/lib/log.sh"
else
    echo "Error: lib/log.sh not found." >&2
    exit 1
fi
# shellcheck disable=SC1091
. "$SCRIPTS_DIR/lib/doc-engine-helpers.sh"

# Define mappings based on document type
case "$DOC_TYPE" in
    adr)
        DIR="docs/adr"
        TEMPLATE="docs/adr/templates/template.md"
        GITIGNORE_HEADER="X. Architecture Decision Records"
        GITIGNORE_NEXT="XI. Guides"
        NUMBERED=1
        LABEL="Architecture Decision Record"
        TYPE_LABEL="ADR"
        EMPTY_TITLE_LABEL="ADR"
        DIR_LABEL="ADR"
        DEFAULT_STATUS="Accepted"
        GIT_MOVE=1
        GIT_REMOVE=1
        REQUIRED_SECTIONS="## Status
## Context
## Decision
## Consequences"
        ;;
    design)
        DIR="docs/design"
        TEMPLATE="docs/design/templates/template.md"
        GITIGNORE_HEADER="IX. Design Documentation"
        GITIGNORE_NEXT="X. Architecture Decision Records"
        NUMBERED=1
        LABEL="Design Document"
        TYPE_LABEL="Design Document"
        EMPTY_TITLE_LABEL="Design Document"
        DIR_LABEL="Design"
        DEFAULT_STATUS="Proposed"
        GIT_MOVE=1
        GIT_REMOVE=1
        REQUIRED_SECTIONS="## Status
## Summary
## Problem / Motivation
## Proposed Design"
        ;;
    guide)
        DIR="docs/guides"
        TEMPLATE="docs/guides/templates/template.md"
        GITIGNORE_HEADER="XI. Guides"
        GITIGNORE_NEXT="XII. Source Code & Workspace Packages"
        NUMBERED=0
        LABEL="User Guide"
        TYPE_LABEL="Guide Document"
        EMPTY_TITLE_LABEL="Guide"
        DIR_LABEL="Guide"
        GIT_MOVE=0
        GIT_REMOVE=0
        ;;
    *)
        log_fatal "Unknown document type '$DOC_TYPE'."
        ;;
esac

# Execute action
case "$ACTION" in
    add)
        TITLE="$1"
        if [ -z "$TITLE" ]; then
            log_err "$EMPTY_TITLE_LABEL title cannot be empty."
            echo "Usage: $0 add $DOC_TYPE \"<Title>\"" >&2
            exit 1
        fi

        # Sanitise title
        TITLE_SANITISED=$(echo "$TITLE" | tr '/' '-' | tr "\\\\" "-")

        # Create slug
        SLUG=$(echo "$TITLE_SANITISED" | tr ' ' '-' | tr '[:upper:]' '[:lower:]' | \
               tr -Cs '[:alnum:]-' '-' | sed 's/-*$//' | sed 's/^-*//')

        if [ "$NUMBERED" -eq 1 ]; then
            LAST=$(find "$DIR" -maxdepth 1 -name '[0-9]*.md' | sort | tail -n 1)
            if [ -z "$LAST" ]; then
                NEWNUM=1
            else
                MAXID=$(basename "$LAST" | grep -Eo '^[0-9]+' | sed 's/^0*//; s/^$/0/')
                NEWNUM=$((MAXID + 1))
            fi
            NEWID=$(printf "%04d" "$NEWNUM")
            NEW_FILE="$DIR/${NEWID}-${SLUG}.md"
        else
            NEW_FILE="$DIR/${SLUG}.md"
        fi

        if [ -f "$NEW_FILE" ]; then
            log_fatal "File $NEW_FILE already exists."
        fi

        # Render template
        DATE=$(date +%Y-%m-%d)
        if [ "$NUMBERED" -eq 1 ]; then
            sed \
                -e "s|NUMBER|$NEWNUM|" \
                -e "s|TITLE|$TITLE_SANITISED|" \
                -e "s|DATE|$DATE|" \
                -e "s|STATUS|$DEFAULT_STATUS|" \
                "$TEMPLATE" > "$NEW_FILE"
        else
            sed \
                -e "s|TITLE|$TITLE_SANITISED|" \
                -e "s|DATE|$DATE|" \
                "$TEMPLATE" > "$NEW_FILE"
        fi

        echo "Created: $NEW_FILE"

        # Register in .gitignore ledger
        register_in_ledger "$NEW_FILE" "$GITIGNORE_HEADER" "$GITIGNORE_NEXT"
        echo "Semantically registered $NEW_FILE in .gitignore"
        ;;

    remove)
        FORCE=0
        if [ "$1" = "-f" ]; then
            FORCE=1
            shift
        fi

        TARGET="$1"
        if [ -z "$TARGET" ]; then
            log_err "$EMPTY_TITLE_LABEL identifier cannot be empty."
            echo "Usage: $0 remove $DOC_TYPE [-f] <identifier>" >&2
            exit 1
        fi

        # Find file
        if [ -f "$DIR/$TARGET" ]; then
            DOC_FILE="$DIR/$TARGET"
        elif [ -f "$DIR/$TARGET.md" ]; then
            DOC_FILE="$DIR/$TARGET.md"
        else
            MATCHES=$(find "$DIR" -maxdepth 1 -name "$TARGET*.md" 2>/dev/null)
            COUNT=$(echo "$MATCHES" | wc -l)
            if [ "$COUNT" -eq 0 ] || [ -z "$MATCHES" ]; then
                log_fatal "Could not find $TYPE_LABEL matching '$TARGET' in $DIR/"
            elif [ "$COUNT" -gt 1 ]; then
                log_err "Ambiguous target '$TARGET'. Found multiple matches:"
                echo "$MATCHES" | sed 's/^/  /' >&2
                exit 1
            fi
            DOC_FILE=$(echo "$MATCHES" | head -1)
        fi

        # Run audits
        if ! audit_links "$DOC_FILE"; then
            if [ "$FORCE" -eq 0 ]; then
                if [ "$GIT_REMOVE" -eq 0 ]; then
                    printf "Dead links detected. Do you still want to proceed with deletion? (yes/no): "
                    read -r RESPONSE
                    if [ "$RESPONSE" != "yes" ]; then
                        echo "Cancelled."
                        exit 0
                    fi
                else
                    printf "Dead links detected. Do you still want to proceed with deletion? [y/N]: "
                    read -r RESPONSE
                    if [ "$RESPONSE" != "y" ] && [ "$RESPONSE" != "Y" ]; then
                        echo "Aborted."
                        exit 0
                    fi
                fi
            else
                log_warn "Proceeding with deletion despite dead links due to force (-f) flag."
            fi
        fi

        # Confirmation
        if [ "$FORCE" -eq 0 ]; then
            if [ "$GIT_REMOVE" -eq 0 ]; then
                echo "Found: $DOC_FILE"
                echo ""
                printf "Remove this %s and update .gitignore? (yes/no): " "$LABEL"
                read -r RESPONSE
                if [ "$RESPONSE" != "yes" ]; then
                    echo "Cancelled."
                    exit 0
                fi
            else
                echo "This will execute the following actions:"
                echo "  1. git rm $DOC_FILE"
                echo "  2. Remove '!/$DOC_FILE' from .gitignore"
                printf "Proceed? [y/N]: "
                read -r RESPONSE
                if [ "$RESPONSE" != "y" ] && [ "$RESPONSE" != "Y" ]; then
                    echo "Aborted."
                    exit 0
                fi
            fi
        fi

        # Delete file
        if [ "$GIT_REMOVE" -eq 1 ]; then
            git rm "$DOC_FILE"
        else
            rm -f "$DOC_FILE"
        fi
        echo "Successfully removed $DOC_FILE and updated .gitignore."

        # Unregister from .gitignore
        unregister_from_ledger "$DOC_FILE" "$GITIGNORE_HEADER" "$GITIGNORE_NEXT"
        echo "Updated .gitignore ledger"
        ;;

    rename)
        FORCE=0
        if [ "$1" = "-f" ]; then
            FORCE=1
            shift
        fi

        TARGET="$1"
        NEW_TITLE="$2"
        if [ -z "$TARGET" ] || [ -z "$NEW_TITLE" ]; then
            echo "Usage: $0 rename $DOC_TYPE [-f] <identifier> \"<new-title>\"" >&2
            exit 1
        fi

        # Find file
        if [ -f "$DIR/$TARGET" ]; then
            OLD_FILE="$DIR/$TARGET"
        elif [ -f "$DIR/$TARGET.md" ]; then
            OLD_FILE="$DIR/$TARGET.md"
        else
            MATCHES=$(find "$DIR" -maxdepth 1 -name "$TARGET*.md" 2>/dev/null)
            COUNT=$(echo "$MATCHES" | wc -l)
            if [ "$COUNT" -eq 0 ] || [ -z "$MATCHES" ]; then
                log_fatal "Could not find $TYPE_LABEL matching '$TARGET' in $DIR/"
            elif [ "$COUNT" -gt 1 ]; then
                log_err "Ambiguous target '$TARGET'. Found multiple matches:"
                echo "$MATCHES" >&2
                exit 1
            fi
            OLD_FILE="$MATCHES"
        fi

        BASENAME=$(basename "$OLD_FILE")
        NEW_TITLE_SANITISED=$(echo "$NEW_TITLE" | tr '/' '-' | tr "\\\\" "-" | tr ' ' '-' | tr '[:upper:]' '[:lower:]' | tr -Cs '[:alnum:]-' '-' | sed 's/-*$//' | sed 's/^-*//')

        if [ "$NUMBERED" -eq 1 ]; then
            PREFIX=$(echo "$BASENAME" | cut -d'-' -f1)
            if ! echo "$PREFIX" | grep -Eq '^[0-9]+$'; then
                log_fatal "Could not extract a valid numeric prefix from '$BASENAME'."
            fi
            NEW_FILE="$DIR/${PREFIX}-${NEW_TITLE_SANITISED}.md"
        else
            NEW_FILE="$DIR/${NEW_TITLE_SANITISED}.md"
        fi

        if [ "$OLD_FILE" = "$NEW_FILE" ]; then
            log_fatal "The new filename would be identical to the old one."
        fi

        if [ -f "$NEW_FILE" ]; then
            log_fatal "Destination file $NEW_FILE already exists."
        fi

        if [ "$FORCE" -eq 0 ]; then
            if [ "$GIT_MOVE" -eq 0 ]; then
                echo "Old: $OLD_FILE"
                echo "New: $NEW_FILE"
                echo ""
                printf "Rename this guide and update .gitignore? (yes/no): "
                read -r CONFIRM
                if [ "$CONFIRM" != "yes" ]; then
                    echo "Cancelled."
                    exit 0
                fi
            else
                echo "This will execute the following actions:"
                echo "  1. git mv $OLD_FILE $NEW_FILE"
                echo "  2. Replace '!/$OLD_FILE' with '!/$NEW_FILE' in .gitignore"
                printf "Proceed? [y/N]: "
                read -r CONFIRM
                if [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "Y" ]; then
                    echo "Aborted."
                    exit 0
                fi
            fi
        fi

        if [ "$GIT_MOVE" -eq 1 ]; then
            git mv "$OLD_FILE" "$NEW_FILE"
        else
            mv "$OLD_FILE" "$NEW_FILE"
        fi

        replace_in_ledger "$OLD_FILE" "$NEW_FILE" "$GITIGNORE_HEADER" "$GITIGNORE_NEXT"

        update_links "$OLD_FILE" "$NEW_FILE"

        echo "Successfully renamed to $NEW_FILE and updated .gitignore."
        ;;

    verify)
        # --quiet/-q: suppress per-document list; emit count summary only.
        # Parsed here rather than globally so flag cannot corrupt positional
        # args consumed by add/remove/rename.
        QUIET=0
        for _arg in "$@"; do
            case "$_arg" in --quiet|-q) QUIET=1 ;; esac
        done
        unset _arg

        # Check if dir exists and has files
        if [ ! -d "$DIR" ] || [ -z "$(find "$DIR" -maxdepth 1 -name '*.md' ! -name 'README.md' ! -name 'templates' 2>/dev/null)" ]; then
            if [ "$DOC_TYPE" = "guide" ]; then
                echo "No guides found in docs/guides/"
                exit 0
            else
                echo "No files found in $DIR/"
                exit 0
            fi
        fi

        TEMP_REGISTERED=$(mktemp)
        TEMP_PHYSICAL=$(mktemp)
        trap 'rm -f "$TEMP_REGISTERED" "$TEMP_PHYSICAL"' EXIT

        # 1. Extract registered from .gitignore
        awk -v header="$GITIGNORE_HEADER" -v next_hdr="$GITIGNORE_NEXT" '
            BEGIN { flag = 0 }
            index($0, header) > 0 { flag = 1; next }
            index($0, next_hdr) > 0 { flag = 0; exit }
            flag && /^[^#]/ && !/^$/ { print }
        ' .gitignore | sed 's|^!/||' | sort > "$TEMP_REGISTERED"

        # 2. Find physical files
        find "$DIR" -maxdepth 1 -name "*.md" ! -name "README.md" 2>/dev/null | sort > "$TEMP_PHYSICAL"

        # 3. Compare lists
        if ! DIFFS=$(diff -u "$TEMP_PHYSICAL" "$TEMP_REGISTERED"); then
            log_err "$DIR_LABEL directory and .gitignore ledger are out of sync!"
            echo "Please ensure all documents are registered in .gitignore under the correct section." >&2
            echo "Discrepancy details (- physical, + registered):" >&2
            echo "$DIFFS" >&2
            exit 1
        fi

        # 4. Index verification (README.md)
        INDEX_ERRORS=0
        while IFS= read -r doc_file; do
            doc_basename=$(basename "$doc_file")
            if ! grep -q "$doc_basename" "$DIR/README.md"; then
                log_err "$doc_file is not listed in $DIR/README.md"
                INDEX_ERRORS=$((INDEX_ERRORS + 1))
            fi
        done < "$TEMP_PHYSICAL"

        if [ $INDEX_ERRORS -gt 0 ]; then
            log_fatal "$INDEX_ERRORS $LABEL(s) are missing from README.md. All documents must be manually indexed."
        fi

        # 5. Template conformance
        TEMPLATE_ERRORS=0
        if [ "$NUMBERED" -eq 1 ]; then
            # ADR / Design
            while IFS= read -r doc_file; do
                while IFS= read -r section; do
                    if ! grep -q "^${section}$" "$doc_file"; then
                        log_err "$doc_file missing required section: $section"
                        TEMPLATE_ERRORS=$((TEMPLATE_ERRORS + 1))
                    fi
                done <<EOF
$REQUIRED_SECTIONS
EOF
            done < "$TEMP_PHYSICAL"
        else
            # Guide
            while IFS= read -r doc_file; do
                if ! grep -q "^## Overview$" "$doc_file"; then
                    log_err "$doc_file missing required section: ## Overview"
                    TEMPLATE_ERRORS=$((TEMPLATE_ERRORS + 1))
                fi
                if ! grep -q "^## Prerequisites$" "$doc_file"; then
                    log_err "$doc_file missing required section: ## Prerequisites"
                    TEMPLATE_ERRORS=$((TEMPLATE_ERRORS + 1))
                fi
                header_count=$(grep -c "^## " "$doc_file" || true)
                if [ "$header_count" -lt 3 ]; then
                    log_err "$doc_file must include at least one additional section beyond ## Overview and ## Prerequisites"
                    TEMPLATE_ERRORS=$((TEMPLATE_ERRORS + 1))
                fi
            done < "$TEMP_PHYSICAL"
        fi

        if [ $TEMPLATE_ERRORS -gt 0 ]; then
            if [ "$DOC_TYPE" = "adr" ]; then
                log_err "$TEMPLATE_ERRORS template conformance issues found. ADRs must include all required MADR sections."
            elif [ "$DOC_TYPE" = "design" ]; then
                log_err "$TEMPLATE_ERRORS template conformance issues found. Design Documents must include all required sections."
            else
                log_err "$TEMPLATE_ERRORS template conformance issues found."
                log_err_detail "Guides must include: ## Overview, ## Prerequisites, and at least one custom content header."
            fi
            exit 1
        fi

        # Output: verbose by default, summary-only with --quiet
        DOC_COUNT=$(wc -l < "$TEMP_PHYSICAL")
        if [ "$QUIET" -eq 0 ]; then
            if [ "$DOC_TYPE" = "guide" ]; then
                echo "Registered guides:"
            else
                echo "Registered ${DOC_TYPE}s:"
            fi
            while IFS= read -r doc_file; do
                log_ok "$(basename "$doc_file")" 1
            done < "$TEMP_PHYSICAL"
            echo ""
        fi
        if [ "$DOC_TYPE" = "guide" ]; then
            if [ "$DOC_COUNT" -eq 1 ]; then
                log_ok "$DOC_COUNT guide verified"
            else
                log_ok "$DOC_COUNT guides verified"
            fi
        else
            if [ "$DOC_COUNT" -eq 1 ]; then
                log_ok "$DOC_COUNT $LABEL verified"
            else
                log_ok "$DOC_COUNT ${LABEL}s verified"
            fi
        fi
        ;;

    *)
        log_fatal "Unknown action '$ACTION'."
        ;;
esac
