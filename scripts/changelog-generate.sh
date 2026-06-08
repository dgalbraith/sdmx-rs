#!/bin/sh
# ==============================================================================
# scripts/changelog-generate.sh
#
# Regenerate every crate's CHANGELOG.md from git history with git-cliff, WITHOUT
# committing — the review step a maintainer runs before cutting a release. Each
# crate's changelog is generated independently and written in place.
#
# CRITICAL — gate parity: this generator MUST mirror scripts/check-changelog.sh's
# git-cliff invocation EXACTLY (same --config, --tag-pattern, --include-path,
# --output target). check-changelog regenerates each changelog and diffs it
# byte-for-byte against the committed file; any divergence between the two
# invocations makes the gate fail even when nothing is wrong. The two call sites
# are a matched pair — change one, change the other.
#
# Scoping is per-crate and is the whole point of the loop:
#   --include-path "crates/<crate>/**"  filters history to THIS crate's commits.
#       The trailing /** is load-bearing: a bare directory matches the dir entry
#       but NOT its contents, filtering out every commit -> an empty changelog
#       that the gate still passes (empty == empty) but that detonates later at
#       the GitHub-release step. Keep the /**.
#   --tag-pattern "^<crate>/v..."  restricts release boundaries to THIS crate's
#       own tags, so a sibling's tag (e.g. sdmx-types/v0.3.0) cannot inject a
#       phantom version section into this crate's changelog under decoupled
#       versioning. This overrides cliff.toml's default tag_pattern (the facade).
# No --tag here: no new version is being cut, so generation yields an
# [Unreleased] section, not a concrete one (a literal --tag glob would become a
# malformed "## [*]" header — that is the release.toml hook's job, not this one).
#
# Extracted from the `changelog-generate` Justfile recipe so the per-crate loop
# is a testable unit and its result is framed by log.sh rather than a raw echo.
# The recipe now delegates here.
#
# POSIX sh only.
#
# Environment:
#   GIT_CLIFF  git-cliff invocation to use (default: git-cliff) — indirection for
#              tests, which point it at a stub so no real generation runs.
#
# Exit codes:
#   0 = generation completed for all crates (individual crate failures are warned
#       but do not abort — see the per-crate `|| status` note below)
# ==============================================================================

set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/common.sh"

GIT_CLIFF="${GIT_CLIFF:-git-cliff}"

log_section "Generating per-crate changelogs"

CRATES_TO_GENERATE=$(get_crates "$@")

for crate in $CRATES_TO_GENERATE; do
    log_item "$crate" 1

    # `|| crate_status=$?` so a single crate's git-cliff failure is surfaced as a
    # warning and the loop continues to the remaining crates, rather than `set -e`
    # aborting the whole batch part-way through (the original recipe used a bare
    # `|| true`, which swallowed the failure entirely — this keeps the
    # don't-abort behaviour but no longer hides it).
    crate_status=0
    "$GIT_CLIFF" --config cliff.toml \
        --tag-pattern "^${crate}/v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$" \
        --include-path "crates/${crate}/**" \
        --output "crates/${crate}/CHANGELOG.md" || crate_status=$?

    if [ "$crate_status" -ne 0 ]; then
        log_warn "git-cliff exited ${crate_status} for ${crate}; its CHANGELOG.md may be unchanged. Review before committing." 1
    fi
done

log_ok "changelog-generate: changelogs written to crates/*/CHANGELOG.md"
# NB: do NOT curate any CHANGELOG.md — they are the machine record, gated
# byte-for-byte by check-changelog. Facade curation lives in the separate
# release-notes/<version>.md file (just check-release-notes), not here.
log_hint "Review crates/*/CHANGELOG.md (machine record — fix commit messages, not the file), curate crates/sdmx-rs/release-notes/<version>.md, then run: just release-commit-changelogs"
