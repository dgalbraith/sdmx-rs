#!/bin/sh
# ===================================================================
# check-shebangs.sh
#
# Validates that all shell scripts in scripts/ declare #!/bin/sh
# (POSIX-portable shebang) rather than #!/bin/bash.
#
# Exit codes:
#   0 — all shebangs valid
#   1 — one or more scripts have invalid shebangs
#
# ===================================================================

set -u

# Source shared configuration and loggers
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/log.sh"


tmp_fail=$(mktemp)
trap 'rm -f "$tmp_fail"' EXIT

log_section "Checking script shebangs..."

# Recurse the whole tree (not a fixed list of subdirs) so a newly added nested
# directory cannot silently bypass the check. `find … | while read` is the
# POSIX-portable recursion idiom here — /bin/sh (dash) has no `globstar`.
find scripts -type f -name '*.sh' | while IFS= read -r script; do
    shebang=$(head -n 1 "$script")
    if [ "$shebang" != "#!/bin/sh" ]; then
        log_err_file "$script" "Invalid shebang: $shebang (expected #!/bin/sh)"
        echo "1" > "$tmp_fail"
    fi
done

if [ -s "$tmp_fail" ]; then
    log_fail "check-shebangs: one or more scripts have invalid shebangs (expected #!/bin/sh)"
    exit 1
fi

log_ok "check-shebangs: all script shebangs are #!/bin/sh"
