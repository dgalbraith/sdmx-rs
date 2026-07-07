# shellcheck shell=bash
# Shared test helpers and utilities for BATS tests
# (BATS executes tests and sourced helpers under bash; see ADR-0020.)



# Create a sample maintenance.toml for testing
create_test_maintenance_toml() {
    cat > "$BATS_TEST_TMPDIR/maintenance.toml" << 'TOML_EOF'
[[maintenance]]
item = "test-item"
file = "test.txt"
marker = "# MAINTENANCE: test-item"
review_cadence = 30
warn_threshold = 45
fail_threshold = 60
last_updated = "2026-07-15"
next_review = "2026-08-15"
TOML_EOF
}

# Create a sample maintenance.toml with custom dates
create_test_maintenance_toml_with_dates() {
    local last_updated="$1"
    local next_review="$2"
    cat > "$BATS_TEST_TMPDIR/maintenance.toml" << TOML_EOF
[[maintenance]]
item = "test-item"
file = "test.txt"
marker = "# MAINTENANCE: test-item"
review_cadence = 30
warn_threshold = 45
fail_threshold = 60
last_updated = "$last_updated"
next_review = "$next_review"
TOML_EOF
}

# Create a sample source file with maintenance comment
create_test_source_file() {
    local file="$1"
    local last_updated="${2:-2026-07-15}"
    local next_review="${3:-2026-08-15}"
    local item="${4:-test-item}"

    mkdir -p "$(dirname "$file")"
    cat > "$file" << EOF
# Sample source file

# MAINTENANCE: $item
# Last updated: $last_updated
# Next review: $next_review
# Description: Test maintenance item

echo "test"
EOF
}

# Create a source file WITHOUT maintenance comment
create_test_source_file_no_comment() {
    local file="$1"

    mkdir -p "$(dirname "$file")"
    cat > "$file" << 'EOF'
# Sample source file without comment
echo "test"
EOF
}

# ============================================================================
# ADR and Design Document Test Helpers
# ============================================================================

# Initialise test environment for ADR tests
# Creates git repo, directories, and copies templates/scripts
setup_adr_test() {
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"

    mkdir -p docs/adr/templates
    cp "$BATS_TEST_DIRNAME/../../docs/adr/templates/template.md" docs/adr/templates/
    cp "$BATS_TEST_DIRNAME/../../scripts/doc-engine.sh" .
    cp "$BATS_TEST_DIRNAME/../../scripts/common.sh" .
    # doc-engine.sh sources lib/log.sh and lib/doc-engine-helpers.sh directly.
    mkdir -p lib
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/doc-engine-helpers.sh" lib/

    create_adr_gitignore
}

# Initialise test environment for design document tests
setup_design_test() {
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"

    mkdir -p docs/design/templates
    cp "$BATS_TEST_DIRNAME/../../docs/design/templates/template.md" docs/design/templates/
    cp "$BATS_TEST_DIRNAME/../../scripts/doc-engine.sh" .
    cp "$BATS_TEST_DIRNAME/../../scripts/common.sh" .
    # doc-engine.sh sources lib/log.sh and lib/doc-engine-helpers.sh directly.
    mkdir -p lib
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/doc-engine-helpers.sh" lib/

    create_design_gitignore
}

# Initialise test environment for guide tests
setup_guide_test() {
    git init --initial-branch=main -q
    git config user.email "test@example.com"
    git config user.name "Test User"

    mkdir -p docs/guides/templates
    cp "$BATS_TEST_DIRNAME/../../docs/guides/templates/template.md" docs/guides/templates/
    cp "$BATS_TEST_DIRNAME/../../scripts/doc-engine.sh" .
    cp "$BATS_TEST_DIRNAME/../../scripts/common.sh" .
    # doc-engine.sh sources lib/log.sh and lib/doc-engine-helpers.sh directly.
    mkdir -p lib
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/log.sh" lib/
    cp "$BATS_TEST_DIRNAME/../../scripts/lib/doc-engine-helpers.sh" lib/

    create_guide_gitignore

    # Create a basic README.md for guides index
    cat > docs/guides/README.md << 'EOF'
# User Guides

## Guide Index

(Guides will be listed here)
EOF
}

# Create a proper .gitignore with ADR section
create_adr_gitignore() {
    cat > .gitignore <<'EOF'
# ==============================================================================
# X. Architecture Decision Records (ADRs)
# ==============================================================================

# ==============================================================================
# XI. Guides
# ==============================================================================
EOF
}

# Create a proper .gitignore with design section
create_design_gitignore() {
    cat > .gitignore <<'EOF'
# ==============================================================================
# IX. Design Documentation
# ==============================================================================

# ==============================================================================
# X. Architecture Decision Records (ADRs)
# ==============================================================================
EOF
}

# Create a proper .gitignore with guides section
create_guide_gitignore() {
    cat > .gitignore <<'EOF'
# ==============================================================================
# XI. Guides
# ==============================================================================

# ==============================================================================
# XII. Source Code & Workspace Packages
# ==============================================================================
EOF
}

# Add an ADR entry to .gitignore in the correct section.
# Inserts before the NEXT section header (Guides); anchored on the title to avoid
# Roman-numeral prefix collisions (e.g. /^# X/ would also match XI, XII).
add_adr_to_gitignore() {
    local file="$1"
    sed -i "/^# XI\\. Guides/i\\!/docs/adr/${file}" .gitignore
}

# Add a design document entry to .gitignore in the correct section.
# Inserts before the NEXT section header (Architecture Decision Records).
add_design_to_gitignore() {
    local file="$1"
    sed -i "/^# X\\. Architecture Decision Records/i\\!/docs/design/${file}" .gitignore
}

# Add a guide document entry to .gitignore in the correct section.
# Inserts before the NEXT section header (Source Code & Workspace Packages).
add_guide_to_gitignore() {
    local file="$1"
    sed -i "/^# XII\\. Source Code/i\\!/docs/guides/${file}" .gitignore
}

# Helper to add guide entry to README.md index
add_guide_to_readme() {
    local file="$1"
    local title="${2:-$file}"
    echo "- [$title]($file)" >> docs/guides/README.md
}

# Assert that an ADR file exists with required sections
assert_adr_file_exists() {
    local file="$1"
    [ -f "docs/adr/$file" ]
    grep -q "^## Status$" "docs/adr/$file"
    grep -q "^## Context$" "docs/adr/$file"
    grep -q "^## Decision$" "docs/adr/$file"
    grep -q "^## Consequences$" "docs/adr/$file"
}

# Assert that a design document file exists with required sections
assert_design_file_exists() {
    local file="$1"
    [ -f "docs/design/$file" ]
    grep -q "^## Status$" "docs/design/$file"
    grep -q "^## Summary$" "docs/design/$file"
    grep -q "^## Problem / Motivation$" "docs/design/$file"
    grep -q "^## Proposed Design$" "docs/design/$file"
}

# Assert that an ADR is registered in .gitignore
assert_adr_in_gitignore() {
    local file="$1"
    grep -q "!/docs/adr/$file" .gitignore
}

# Assert that a design document is registered in .gitignore
assert_design_in_gitignore() {
    local file="$1"
    grep -q "!/docs/design/$file" .gitignore
}

# Assert that a guide file exists with required sections
assert_guide_file_exists() {
    local file="$1"
    [ -f "docs/guides/$file" ]
    grep -q "^## Overview$" "docs/guides/$file"
    grep -q "^## Prerequisites$" "docs/guides/$file"
    [ "$(grep -c "^## " "docs/guides/$file" || true)" -ge 3 ]
}

# Assert that a guide is registered in .gitignore
assert_guide_in_gitignore() {
    local file="$1"
    grep -q "!/docs/guides/$file" .gitignore
}

# Assert that a file does NOT exist
assert_file_not_exists() {
    local file="$1"
    [ ! -f "$file" ]
}

# Assert that a file is tracked in git; prints diagnostic on failure
assert_file_exists_in_git() {
    local file="$1"
    git ls-files --error-unmatch "$file" >/dev/null 2>&1 || {
        echo "ERROR: Expected file not in git: $file"
        echo "Files in git:"
        git ls-files
        return 1
    }
}

# Run script with isolated environment (no CI or tooling-config variable leakage)
# Unsets problematic ambient variables while preserving test and system variables. See ADR-0020.
#
# Optional leading `--ci` flag: after scrubbing the ambient environment, RE-assert
# a canonical CI environment (GITHUB_ACTIONS=true, CI=true) for the run. This is
# the single, defensible delta for testing scripts that branch on CI — e.g. a
# secrets scan that runs full-history `detect` in CI but working-tree `protect`
# locally. Expressed as a flag (not a sibling function) so there is ONE isolation
# code path: any future change to the scrub/run logic is inherited by both modes
# automatically, and the only difference between local and CI runs stays exactly
# the two exported vars. "CI" is modelled as the whole marker set, not a single
# var, so a script may branch on either GITHUB_ACTIONS or CI and still see truth.
run_isolated() {
    local ci=0
    if [ "${1:-}" = "--ci" ]; then
        ci=1
        shift
    fi

    local script="$1"
    shift

    # Unset specific CI variables that cause test pollution, but preserve others
    unset GITHUB_EVENT_NAME
    unset GITHUB_ACTIONS
    unset CI

    # Unset tooling-config overrides so scripts use their built-in defaults
    # (fixtures assume the default `origin` remote, not a maintainer's override).
    unset SDMX_MAIN_REMOTE

    # --ci: re-establish the canonical CI environment AFTER the scrub above, so
    # the run sees exactly the markers CI sets and nothing else from the host.
    if [ "$ci" = 1 ]; then
        export GITHUB_ACTIONS=true
        export CI=true
    fi

    # Handle bash -c commands (from BATS test context) and normal scripts (sh)
    if [ "$script" = "bash" ]; then
        run bash "$@"
    else
        # Normal script invocation with sh (POSIX shell, as scripts are written for sh)
        run sh "$script" "$@"
    fi
}

# Inject a zero-exit just shim into PATH, allowing full (non-dry-run) execution
# of scripts that call `just verify` without requiring the Nix/cargo environment.
# Writes to $BATS_TEST_TMPDIR/bin — gitignored by setup() in update-msrv.bats
# so the working tree stays clean for pre-flight checks.
mock_just() {
    mkdir -p "$BATS_TEST_TMPDIR/bin"
    cat > "$BATS_TEST_TMPDIR/bin/just" << 'EOF'
#!/bin/sh
echo "just (mock): $*"
exit 0
EOF
    chmod +x "$BATS_TEST_TMPDIR/bin/just"
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

# Inject a `gh` shim into PATH so forge scripts can be exercised offline — NO
# network, NO real auth, NO mutation of any live forge. Mirrors mock_just's
# PATH-shim approach. The shim dispatches on its argument vector and serves
# canned JSON from tests/bats/fixtures/forge/ via a per-endpoint dispatch file.
#
# Auth state is controlled by the first argument:
#   mock_gh            → `gh auth status` succeeds (authenticated)
#   mock_gh --no-auth  → `gh auth status` exits 1 (unauthenticated); used to
#                        prove the online tier degrades to warn + exit 0.
#
# The shim reads fixtures from $FORGE_FIXTURES (exported here); a test points
# that at a prepared directory of *.json response bodies. Endpoint → file
# mapping lives in the shim's case statement below; an unmapped endpoint exits 1
# so an unexpected call fails loudly rather than silently returning empty.
mock_gh() {
    local authed=1
    if [ "${1:-}" = "--no-auth" ]; then
        authed=0
        shift
    fi

    : "${FORGE_FIXTURES:?mock_gh requires FORGE_FIXTURES to point at a fixture dir}"
    mkdir -p "$BATS_TEST_TMPDIR/bin"

    cat > "$BATS_TEST_TMPDIR/bin/gh" << EOF
#!/bin/sh
# Test shim — serves canned forge responses, never touches a network/forge.
FORGE_FIXTURES="$FORGE_FIXTURES"
AUTHED="$authed"
GH_MOCK_VULN_ALERTS="${GH_MOCK_VULN_ALERTS:-on}"
GH_MOCK_PRIVATE_VULN="${GH_MOCK_PRIVATE_VULN:-on}"
GH_MOCK_AUTO_FIXES="${GH_MOCK_AUTO_FIXES:-false}"
GH_MOCK_ALLOWED_ACTIONS="${GH_MOCK_ALLOWED_ACTIONS:-all}"
GH_MOCK_STAGED_SHAS="${GH_MOCK_STAGED_SHAS:-}"
EOF
    cat >> "$BATS_TEST_TMPDIR/bin/gh" << 'EOF'

# gh applies a jq filter when -q/--jq is passed (to `--json` output or `api`).
# Capture it once and route response bodies through emit() so the mock matches
# real gh; previously --jq/-q was ignored and raw JSON was returned.
_jq=""; _want=0
for _a in "$@"; do
    [ "$_want" = 1 ] && { _jq="$_a"; _want=0; continue; }
    case "$_a" in --jq|-q) _want=1 ;; esac
done
emit() { if [ -n "$_jq" ]; then printf '%s' "$1" | jq -r "$_jq"; else printf '%s\n' "$1"; fi; }

# gh auth status — gate for the online tier.
if [ "$1" = "auth" ] && [ "$2" = "status" ]; then
    [ "$AUTHED" = "1" ] && exit 0
    echo "not logged in" >&2
    exit 1
fi

# gh repo view --json <fields> [-q <jq>] — a canned repo object.
if [ "$1" = "repo" ] && [ "$2" = "view" ]; then
    emit '{"nameWithOwner":"owner/repo"}'
    exit 0
fi

# gh api <endpoint> [flags...] — strip the leading 'api' and find the endpoint
# (the first non-flag argument after 'api'). Map it to a fixture file.
if [ "$1" = "api" ]; then
    shift
    endpoint=""
    for arg in "$@"; do
        case "$arg" in
            -*) ;;                       # skip flags (--paginate, --jq, etc.)
            *) endpoint="$arg"; break ;; # first positional = endpoint
        esac
    done

    # Security toggle sub-resources are handled inline (they are presence/enabled
    # probes, not fixture-file reads). State is env-controlled with spec-compliant
    # defaults (alerts ON, auto-fixes OFF, private reporting ON); a test overrides
    # one to exercise a drift path.
    case "$endpoint" in
        repos/*/vulnerability-alerts)
            # presence probe: exit 0 (204, enabled) unless overridden to off.
            [ "${GH_MOCK_VULN_ALERTS:-on}" = "on" ] && exit 0
            echo "mock gh: 404 vulnerability-alerts (disabled)" >&2; exit 1 ;;
        repos/*/private-vulnerability-reporting)
            [ "${GH_MOCK_PRIVATE_VULN:-on}" = "on" ] && exit 0
            echo "mock gh: 404 private-vulnerability-reporting (disabled)" >&2; exit 1 ;;
        repos/*/automated-security-fixes)
            # enabled probe: JSON body with .enabled. Spec wants false (disabled).
            printf '{"enabled":%s}\n' "${GH_MOCK_AUTO_FIXES:-false}"; exit 0 ;;
        repos/*/actions/runs*)
            # CI workflow runs for a head_sha. Dynamic (not a fixture): a SHA in
            # GH_MOCK_STAGED_SHAS gets a staging-* run plus a main run, otherwise
            # only a main run. What a staging-* branch MEANS is the caller's
            # concern, not the mock's.
            _sha=$(printf '%s' "$endpoint" | sed -n 's/.*head_sha=\([0-9a-f]*\).*/\1/p')
            case " ${GH_MOCK_STAGED_SHAS:-} " in
                *" $_sha "*) emit '{"workflow_runs":[{"name":"Continuous Integration","head_branch":"staging-x"},{"name":"Continuous Integration","head_branch":"main"}]}' ;;
                *)           emit '{"workflow_runs":[{"name":"Continuous Integration","head_branch":"main"}]}' ;;
            esac
            exit 0 ;;
    esac

    case "$endpoint" in
        repos/*/rulesets)                           fixture="rulesets.json" ;;
        repos/*/rulesets/*)                         fixture="ruleset-$(basename "$endpoint").json" ;;
        repos/*/labels*)                            fixture="labels.json" ;;
        repos/*/actions/permissions/workflow)       fixture="actions-permissions-workflow.json" ;;
        repos/*/actions/permissions/selected-actions) fixture="actions-permissions-selected.json" ;;
        repos/*/actions/permissions)
            # GH_MOCK_ALLOWED_ACTIONS controls whether the live setting reports
            # "all" (pre-apply default) or "selected" (post-apply). Tests that
            # exercise the selected-actions diff set this to "selected".
            if [ "$GH_MOCK_ALLOWED_ACTIONS" = "selected" ]; then
                fixture="actions-permissions-selected-mode.json"
            else
                fixture="actions-permissions.json"
            fi
            ;;
        repos/*/environments/*)                     fixture="environment-$(basename "$endpoint").json" ;;
        repos/*)                                    fixture="repo.json" ;;
        *)
            echo "mock gh: unmapped endpoint: $endpoint" >&2
            exit 1
            ;;
    esac

    if [ -f "$FORGE_FIXTURES/$fixture" ]; then
        emit "$(cat "$FORGE_FIXTURES/$fixture")"
        exit 0
    fi
    # An absent fixture models a 404 (e.g. release environment not created).
    echo "mock gh: no fixture for endpoint $endpoint ($fixture)" >&2
    exit 1
fi

echo "mock gh: unhandled invocation: $*" >&2
exit 1
EOF

    chmod +x "$BATS_TEST_TMPDIR/bin/gh"
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

# Inject a recording `gh` shim for forge-apply tests. NEVER touches the network
# or real forge state. Every outbound gh call is appended to $GH_CALL_LOG
# (one line per call: the full argument vector). Tests assert against that log.
#
# Guard-state control via environment variables (set before calling mock_gh_apply):
#   GH_MOCK_HAS_RULESETS=1     /rulesets returns a non-empty list (guard fires)
#   GH_MOCK_HAS_ISSUES=1       /issues returns a non-empty list (guard fires)
#   GH_MOCK_RULESETS_FAIL=1    /rulesets GET exits non-zero (models a transient
#                              forge error) — proves the guard FAILS CLOSED rather
#                              than reading the empty body as "no rulesets".
#
# Default (clean state): both return empty arrays → guard passes.
mock_gh_apply() {
    : "${GH_CALL_LOG:?mock_gh_apply requires GH_CALL_LOG to be set}"
    mkdir -p "$BATS_TEST_TMPDIR/bin"

    cat > "$BATS_TEST_TMPDIR/bin/gh" << EOF
#!/bin/sh
GH_CALL_LOG="$GH_CALL_LOG"
GH_MOCK_HAS_RULESETS="${GH_MOCK_HAS_RULESETS:-0}"
GH_MOCK_HAS_ISSUES="${GH_MOCK_HAS_ISSUES:-0}"
GH_MOCK_RULESETS_FAIL="${GH_MOCK_RULESETS_FAIL:-0}"
FORGE_FIXTURES="${FORGE_FIXTURES:-}"
EOF
    cat >> "$BATS_TEST_TMPDIR/bin/gh" << 'EOF'

# Record every invocation for test assertions.
printf '%s\n' "$*" >> "$GH_CALL_LOG"

# gh auth status — always succeeds in apply tests.
if [ "$1" = "auth" ] && [ "$2" = "status" ]; then
    exit 0
fi

if [ "$1" = "api" ]; then
    shift

    # Identify method and endpoint from arguments.
    method="GET"
    endpoint=""
    next_is_endpoint=0
    for arg in "$@"; do
        case "$arg" in
            --method|-X) next_is_endpoint=0; method="" ;;
            --input|-f|-F|--jq) ;;
            *)
                if [ -z "$method" ]; then
                    method="$arg"
                elif [ "$next_is_endpoint" = "1" ] || [ -z "$endpoint" ]; then
                    case "$arg" in
                        -*) ;;
                        *) endpoint="$arg"; next_is_endpoint=0 ;;
                    esac
                fi
                ;;
        esac
    done

    # For mutating calls (POST/PATCH/PUT/DELETE), succeed silently — we only
    # care that they were called (asserted via GH_CALL_LOG).
    case "$method" in
        POST|PATCH|PUT|DELETE) echo '{}'; exit 0 ;;
    esac

    # GET responses.
    case "$endpoint" in
        user)
            printf '{"id":42,"login":"dgalbraith"}\n'
            exit 0
            ;;
        repos/*/rulesets)
            # Model a transient forge error: exit non-zero with no body. The
            # guard must treat this as "unknown -> block", not "empty -> apply".
            if [ "$GH_MOCK_RULESETS_FAIL" = "1" ]; then
                echo "mock gh (apply): simulated rulesets failure" >&2
                exit 1
            fi
            if [ "$GH_MOCK_HAS_RULESETS" = "1" ]; then
                printf '[{"id":1,"name":"existing"}]\n'
            else
                printf '[]\n'
            fi
            exit 0
            ;;
        repos/*/issues*)
            if [ "$GH_MOCK_HAS_ISSUES" = "1" ]; then
                printf '[{"number":1,"title":"existing issue"}]\n'
            else
                printf '[]\n'
            fi
            exit 0
            ;;
        repos/*/environments/*)
            printf '{"name":"release","prevent_self_review":false}\n'
            exit 0
            ;;
        repos/*)
            printf '{"full_name":"dgalbraith/sdmx-rs"}\n'
            exit 0
            ;;
        *)
            echo "mock gh (apply): unmapped GET endpoint: $endpoint" >&2
            exit 1
            ;;
    esac
fi

echo "mock gh (apply): unhandled invocation: $*" >&2
exit 1
EOF

    chmod +x "$BATS_TEST_TMPDIR/bin/gh"
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

# Inject a `curl` shim so the registry scripts (doctor-registry, registry-tp) can
# be exercised offline — NO network, NO real crates.io. Mirrors mock_gh's PATH
# shim. The shim dispatches on the URL in the argument vector and serves canned
# JSON / HTTP statuses from $FORGE_FIXTURES/crates/ (reuse FORGE_FIXTURES as the
# fixtures root; the registry fixtures live under a crates/ subdir).
#
# Two URL families are served:
#   index.crates.io/...                 → sparse-index reservation probe. The shim
#                                         honours the -o/-w form: writes nothing to
#                                         the -o target and echoes the status that
#                                         -w '%{http_code}' would. 200 = reserved.
#   crates.io/api/v1/...github_configs  → TP config list JSON (configs.json).
#   crates.io/api/v1/crates/<name>      → crate object JSON (crate-<name>.json),
#                                         carrying .crate.trustpub_only.
#
# Reservation is controlled per crate by the presence of a marker file
# $FORGE_FIXTURES/crates/reserved/<name>; absent → the index probe 404s (the
# crate is not yet reserved). A test seeds/removes these to model the bootstrap.
#
# An unmapped URL fails loudly (exit 1) so an unexpected call surfaces.
mock_crates() {
    : "${FORGE_FIXTURES:?mock_crates requires FORGE_FIXTURES to point at a fixture dir}"
    mkdir -p "$BATS_TEST_TMPDIR/bin"

    cat > "$BATS_TEST_TMPDIR/bin/curl" << EOF
#!/bin/sh
FORGE_FIXTURES="$FORGE_FIXTURES"
EOF
    cat >> "$BATS_TEST_TMPDIR/bin/curl" << 'EOF'

# Parse the curl arg vector: capture -o target, whether -w wants the http_code,
# and the URL (the last non-flag-ish token / the http(s) argument).
out=""
want_status=0
url=""
prev=""
for arg in "$@"; do
    case "$prev" in
        -o) out="$arg"; prev=""; continue ;;
        -H|-w|-X|-d|--data) prev=""; continue ;;
    esac
    case "$arg" in
        -o) prev="-o" ;;
        -w) prev="-w"; case " $* " in *'%{http_code}'*) want_status=1 ;; esac ;;
        -H|-X|-d|--data) prev="$arg" ;;
        -*) ;;
        http://*|https://*) url="$arg" ;;
    esac
done

crates_dir="$FORGE_FIXTURES/crates"

emit_status() { [ "$want_status" = "1" ] && printf '%s' "$1"; }

case "$url" in
    *index.crates.io/*)
        # Reservation probe. crate name = last path segment.
        name="${url##*/}"
        if [ -f "$crates_dir/reserved/$name" ]; then
            [ -n "$out" ] && cat "$crates_dir/reserved/$name" > "$out"
            emit_status 200
        else
            [ -n "$out" ] && : > "$out"
            emit_status 404
        fi
        exit 0
        ;;
    *trusted_publishing/github_configs*)
        # Emulate crates.io's ?crate=<name> filter: return only that crate's
        # configs (the real API scopes by crate). Extract the crate param.
        qcrate="${url##*crate=}"
        qcrate="${qcrate%%&*}"
        f="$crates_dir/configs.json"
        if [ -f "$f" ]; then
            jq --arg c "$qcrate" '{github_configs: [.github_configs[] | select(.crate == $c)]}' "$f"
        else
            echo '{"github_configs":[]}'
        fi
        emit_status 200
        exit 0
        ;;
    *api/v1/crates/*)
        name="${url##*/crates/}"
        name="${name%%\?*}"
        f="$crates_dir/crate-$name.json"
        if [ -f "$f" ]; then cat "$f"; emit_status 200; else emit_status 404; fi
        exit 0
        ;;
esac

echo "mock curl: unmapped URL: $url" >&2
exit 1
EOF

    chmod +x "$BATS_TEST_TMPDIR/bin/curl"
    export PATH="$BATS_TEST_TMPDIR/bin:$PATH"
}

# Export test utilities
export -f create_test_maintenance_toml
export -f create_test_maintenance_toml_with_dates
export -f create_test_source_file
export -f create_test_source_file_no_comment
export -f setup_adr_test
export -f setup_design_test
export -f setup_guide_test
export -f create_adr_gitignore
export -f create_design_gitignore
export -f create_guide_gitignore
export -f add_adr_to_gitignore
export -f add_design_to_gitignore
export -f add_guide_to_gitignore
export -f assert_adr_file_exists
export -f assert_design_file_exists
export -f assert_guide_file_exists
export -f assert_adr_in_gitignore
export -f assert_design_in_gitignore
export -f assert_guide_in_gitignore
export -f assert_file_not_exists
export -f assert_file_exists_in_git
export -f run_isolated
export -f mock_just
export -f mock_gh
export -f mock_crates
export -f mock_gh_apply
