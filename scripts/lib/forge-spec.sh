#!/bin/sh
# ==============================================================================
# scripts/lib/forge-spec.sh
#
# Single source of truth for the DESIRED forge configuration — the declarative
# "what" that docs/project/forge-setup.md describes in prose. SOURCED-ONLY: this
# file defines functions and has NO top-level side effects (no probing, no
# output, no `gh` calls). Consumers source it, then call the spec functions:
#
#   scripts/doctor-forge.sh — read-only assert (live == spec)
#   scripts/forge-apply.sh  — guarded one-shot setup (push spec -> live)
#
# Two values are DERIVED from the repo (machine-readable, single source already
# in-tree); everything else is HARDCODED here (authored scalars with no natural
# in-repo export unit). See the hybrid-sourcing note in each function.
#
# Record format: functions that emit a SET of records print one record per line,
# fields separated by a literal TAB. Read them with:
#
#   forge_spec_labels | while IFS="$(printf '\t')" read -r name color desc; do ...
#
# Ruleset BODIES are NOT emitted here — they live as committed JSON under
# forge/github/ruleset-*.json (file-based: structured, security-critical, round-
# trips losslessly). This library only names them + their derivations.
#
# POSIX sh only — no bashisms.
# ==============================================================================

# A literal TAB, computed once, for both emit (printf) and read (IFS) sides.
# shellcheck disable=SC2034  # consumed by sourcing scripts (doctor-forge/forge-apply), not here
FORGE_TAB="$(printf '\t')"

# Hardcoded subkey fingerprint — the signing [S] subkey. Unlike the primary
# fingerprint (which has an in-repo source in verify-signature.yml), the subkey
# has no committed source to derive from, so it is authored here. Cross-checked
# against docs/project/forge-setup.md's maintainer key register.
# shellcheck disable=SC2034  # consumed by sourcing scripts, not within this library
FORGE_SUBKEY_FPR="B43D054479B0A9374BC35C167D4A0D2EE2E2ECD7"     # gitleaks:allow

# --- Derived values -----------------------------------------------------------

# forge_spec_owner_repo — echo "OWNER/REPO" derived from the origin remote URL.
# Handles both git@github.com:O/R.git and https://github.com/O/R.git, strips the
# trailing .git. Returns 1 (no output) if origin is absent/unparseable so the
# caller can degrade gracefully.
#
# DELIBERATELY GitHub-scoped: it requires a github.com host before parsing.
# Without that guard, a non-github origin (e.g. codeberg.org) would fall through
# the `*github.com` strip unchanged and the `*/*` case would match the `//` in
# `https://…`, returning a plausible-looking but WRONG slug that then drives
# every `gh api` call. The whole online tier is `gh` (GitHub) only, so refusing
# is correct; a host-agnostic parse waits for a real second forge to exist.
forge_spec_owner_repo() {
    _url="$(git remote get-url origin 2>/dev/null)" || { unset _url; return 1; }
    # Require a github.com host (ssh `git@github.com:` or https `//github.com/`).
    case "$_url" in
        *github.com[:/]*) ;;
        *) unset _url; return 1 ;;
    esac
    # Strip protocol/host prefix: everything up to and including github.com[:/].
    _slug="${_url#*github.com}"
    _slug="${_slug#:}"
    _slug="${_slug#/}"
    # Strip trailing .git if present.
    _slug="${_slug%.git}"
    case "$_slug" in
        */*) printf '%s\n' "$_slug"; unset _url _slug; return 0 ;;
        *)   unset _url _slug; return 1 ;;
    esac
}

# forge_spec_primary_fpr — echo the maintainer PRIMARY GPG fingerprint, derived
# from the CI trust root (.github/workflows/verify-signature.yml). Takes the
# first 40-hex-char token so a line move does not break it (do NOT hardcode the
# line number). Returns 1 (no output) if the file/fingerprint is absent.
forge_spec_primary_fpr() {
    _wf=".github/workflows/verify-signature.yml"
    [ -f "$_wf" ] || { unset _wf; return 1; }
    _fpr="$(grep -oE '[0-9A-F]{40}' "$_wf" 2>/dev/null | head -n1)"
    if [ -n "$_fpr" ]; then
        printf '%s\n' "$_fpr"
        unset _wf _fpr
        return 0
    fi
    unset _wf _fpr
    return 1
}

# forge_spec_signing_email — echo the maintainer signing identity. Authored
# scalar (verified against forge-setup.md).
forge_spec_signing_email() {
    printf '%s\n' "dg@lbraith.io"
}

# --- Labels -------------------------------------------------------------------

# forge_spec_labels — emit the 14 desired labels, one per line:
#   <name>\t<color>\t<description>
# Colors are lowercased to match the GitHub API return form (so a string compare
# in doctor needs no normalisation). Source of truth = forge-setup.md §4.
forge_spec_labels() {
    printf '%s\t%s\t%s\n' "feat" "0e8a16" "New feature implementations and structural enhancements"
    printf '%s\t%s\t%s\n' "fix" "d93f0b" "Targeted bug fixes and code corrections"
    printf '%s\t%s\t%s\n' "docs" "0075ca" "Documentation-only modifications and additions"
    printf '%s\t%s\t%s\n' "refactor" "0052cc" "Code changes that neither fix a bug nor add a feature"
    printf '%s\t%s\t%s\n' "perf" "006b75" "Code changes that explicitly improve execution performance"
    printf '%s\t%s\t%s\n' "test" "fef2c0" "Adding missing tests or correcting existing test suites"
    printf '%s\t%s\t%s\n' "ci" "0369a1" "Changes to continuous integration configurations, scripts, and automation workflows"
    printf '%s\t%s\t%s\n' "build" "93c5fd" "Changes affecting the build system, workspace layouts, or external dependencies"
    printf '%s\t%s\t%s\n' "chore" "5319e7" "Repository maintenance, toolchain shifts, and meta-configuration updates"
    printf '%s\t%s\t%s\n' "maintenance" "ffa500" "Scheduled maintenance obligations, dependency reviews, and toolchain upkeep"
    printf '%s\t%s\t%s\n' "breaking" "9b1c1c" "Introduces a breaking change; requires migration notes and semver major bump"
    printf '%s\t%s\t%s\n' "good first issue" "7057ff" "Well-scoped issue suitable for first-time contributors"
    printf '%s\t%s\t%s\n' "help wanted" "ec4899" "Maintainer is requesting outside input or contributions"
    printf '%s\t%s\t%s\n' "duplicate" "cfd3d7" "Tracks a problem already reported in another issue"
}

# forge_spec_default_labels_to_delete — emit the 6 GitHub default labels that are
# NOT in our spec, one per line. (The other 3 defaults — duplicate / good first
# issue / help wanted — ARE in our spec and are upserted, not deleted.) Consumed
# by forge-apply.sh; doctor only asserts the spec labels exist + match.
forge_spec_default_labels_to_delete() {
    printf '%s\n' "bug"
    printf '%s\n' "documentation"
    printf '%s\n' "enhancement"
    printf '%s\n' "invalid"
    printf '%s\n' "question"
    printf '%s\n' "wontfix"
}

# --- Merge flags --------------------------------------------------------------

# forge_spec_merge_flags — emit the 5 repo merge-method flags, one per line:
#   <key>\t<value>
# Only standard merge commits are permitted; squash/rebase create commits not
# signed by the maintainer. PATCH /repos/{o}/{r}.
forge_spec_merge_flags() {
    printf '%s\t%s\n' "allow_squash_merge" "false"
    printf '%s\t%s\n' "allow_rebase_merge" "false"
    printf '%s\t%s\n' "allow_merge_commit" "true"
    printf '%s\t%s\n' "allow_auto_merge" "false"
    printf '%s\t%s\n' "delete_branch_on_merge" "true"
}

# --- Repo settings (gap-analysis additions) -----------------------------------

# forge_spec_repo_settings — emit non-merge repo settings, one per line:
#   <key>\t<value>
# All booleans on the repo object, applied via PATCH /repos/{o}/{r}:
#   has_projects=false               we use issues+labels, not Projects
#   web_commit_signoff_required=true require sign-off on web-UI commits
#   has_wiki=false / has_pages=false reduce unmanaged-content attack surface
#   allow_update_branch=false        no GitHub-side branch updates (they would
#                                    create commits outside the signed local-merge
#                                    flow, breaking the signed-history invariant)
forge_spec_repo_settings() {
    printf '%s\t%s\n' "has_projects" "false"
    printf '%s\t%s\n' "web_commit_signoff_required" "true"
    printf '%s\t%s\n' "has_wiki" "false"
    printf '%s\t%s\n' "has_pages" "false"
    printf '%s\t%s\n' "allow_update_branch" "false"
}

# --- Security & analysis settings ---------------------------------------------
# GitHub's repository security toggles do NOT all live on the same endpoint, so
# they are modelled in two sets by endpoint shape (see each function). The DESIRED
# values encode this repo's stated posture (SECURITY.md): vulnerability alerts ON
# (monitor mode), Dependabot automated-fix PRs OFF (they would introduce
# unsigned/bot-signed commits, violating the signed-history invariant), private
# vulnerability reporting ON, secret scanning + push protection ON.

# forge_spec_security_toggles — emit the "boolean toggle" security endpoints, one
# per line:
#   <key>\t<want>\t<endpoint>\t<probe>
# These are NOT fields on the repo object; each is its own sub-resource:
#   <probe>=presence — GET returns 204 (enabled) / 404 (disabled), no body. Apply:
#                      PUT to enable, DELETE to disable.
#   <probe>=enabled  — GET returns a JSON body with `.enabled`. Apply: PUT/DELETE.
# All are available on private repos (unlike secret scanning), so they are
# asserted unconditionally.
forge_spec_security_toggles() {
    printf '%s\t%s\t%s\t%s\n' "vulnerability-alerts"          "true"  "vulnerability-alerts"          "presence"
    printf '%s\t%s\t%s\t%s\n' "automated-security-fixes"      "false" "automated-security-fixes"      "enabled"
    printf '%s\t%s\t%s\t%s\n' "private-vulnerability-reporting" "true" "private-vulnerability-reporting" "presence"
}

# forge_spec_security_analysis — emit the security_and_analysis.* settings, one
# per line:
#   <key>\t<want>
# These are nested under `.security_and_analysis.<key>.status` on the repo object
# (values "enabled"/"disabled"); applied via PATCH /repos with a nested object.
# PUBLIC-ONLY: secret scanning + push protection are free only on public repos
# (GitHub Advanced Security otherwise). doctor-forge treats these as deferred-warn
# until the repo is public (see FORGE_SECURITY_REQUIRED), mirroring the release-env
# deferral. forge-setup.md times their enablement to the go-live window.
forge_spec_security_analysis() {
    printf '%s\t%s\n' "secret_scanning" "enabled"
    printf '%s\t%s\n' "secret_scanning_push_protection" "enabled"
}

# --- Actions permissions ------------------------------------------------------

# forge_spec_actions — emit the desired actions-permission mode.
# allowed_actions=selected   restricts which actions may run at all. The committed
#                            allowlist file is the other half; flip this mode before
#                            PUT-ing the allowlist.
# sha_pinning_required is read-only on standard GitHub plans — not set here.
# PUT /repos/{o}/{r}/actions/permissions (JSON body; not PATCH).
forge_spec_actions() {
    printf '%s\t%s\n' "allowed_actions" "selected"
}

# forge_spec_actions_allowlist_file — echo the path (repo-root-relative) of the
# committed actions allowlist artifact. This file IS the body of the GitHub
# PUT /repos/{o}/{r}/actions/permissions/selected-actions request — fed with
# `gh api --input` verbatim. It is also the compare target for doctor-forge's
# selected-actions diff.
forge_spec_actions_allowlist_file() {
    printf '%s\n' "forge/github/actions-allowlist.json"
}

# forge_spec_actions_allowlist_projection_jq — echo the jq program that
# canonicalises both the live selected-actions body and the committed file into
# the same shape for comparison (sort patterns_allowed so list order is not
# spurious drift).
forge_spec_actions_allowlist_projection_jq() {
    printf '%s\n' '{github_owned_allowed, verified_allowed, patterns_allowed: (.patterns_allowed | sort)}'
}

# forge_spec_workflow_permissions — emit the default GITHUB_TOKEN permissions for
# workflow runs, one per line:
#   <key>\t<value>
# Least-privilege for the Actions token: default_workflow_permissions=read (a
# workflow that needs write must request it explicitly via a job `permissions:`
# block — publish.yml already does), and the Actions bot may NOT approve PRs.
# Distinct endpoint: GET/PUT /repos/{o}/{r}/actions/permissions/workflow (NOT the
# /actions/permissions root that forge_spec_actions targets).
forge_spec_workflow_permissions() {
    printf '%s\t%s\n' "default_workflow_permissions" "read"
    printf '%s\t%s\n' "can_approve_pull_request_reviews" "false"
}

# --- Rulesets -----------------------------------------------------------------

# forge_spec_rulesets — emit the desired rulesets, one per line:
#   <name>\t<target>\t<file>
# The committed JSON file IS the request body. <name> matches the "name" field
# inside the file (used to find the live ruleset by name); <target> is recorded
# for display. Files are resolved relative to the repo root.
forge_spec_rulesets() {
    printf '%s\t%s\t%s\n' "Enforce High Integrity Development" "branch" "forge/github/ruleset-signing.json"
    printf '%s\t%s\t%s\n' "Zero Trust Gate" "branch" "forge/github/ruleset-push-restriction.json"
    printf '%s\t%s\t%s\n' "Protect Release Tags" "tag" "forge/github/ruleset-tag-protection.json"
}

# forge_spec_ruleset_projection_jq — echo the jq program that strips server-owned
# read-only fields, normalising both the live ruleset and (idempotently) the
# committed file into the same canonical shape for comparison.
#
# Two server-side NORMALISATIONS must be absorbed here or every apply re-reports
# false drift (GitHub rewrites the body it stores, so a round-tripped ruleset no
# longer byte-matches the authored file):
#   1. `rules` is a SET — GitHub may reorder it. Sort by `.type` so order is
#      irrelevant. (Rule types are unique within a ruleset, so `.type` is a
#      total, stable key.)
#   2. GitHub injects default parameter fields the author omits — notably
#      `required_status_checks.parameters.do_not_enforce_on_create: false`.
#      Drop that specific server default so an omitted-but-defaulted field does
#      not read as drift. (false == the strict default: enforce on create.)
# Both sides run through the same program, so the committed file is normalised
# identically and the comparison stays a pure intent diff.
forge_spec_ruleset_projection_jq() {
    printf '%s\n' '
      {
        name, target, enforcement,
        bypass_actors: [.bypass_actors[]? | {actor_id, actor_type, bypass_mode}],
        conditions,
        rules: (
          [ .rules[]?
            | if .parameters then
                .parameters |= del(.do_not_enforce_on_create)
              else . end
          ] | sort_by(.type)
        )
      }'
}

# --- Release environment ------------------------------------------------------

# forge_spec_release_env_name — echo the gating environment name. The reviewer id
# is account-specific (derived at apply time via `gh api user --jq .id`), so it
# is NOT part of the spec; only the name + the prevent_self_review invariant are.
forge_spec_release_env_name() {
    printf '%s\n' "release"
}

# forge_spec_release_env_prevent_self_review — echo the required value of
# prevent_self_review. MUST be false for a solo maintainer (else publish
# deadlocks: the maintainer cannot approve their own deployment).
forge_spec_release_env_prevent_self_review() {
    printf '%s\n' "false"
}
