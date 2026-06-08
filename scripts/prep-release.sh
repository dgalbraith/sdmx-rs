#!/bin/sh
# ==============================================================================
# scripts/prep-release.sh
#
# PRE-1.0 ONLY. Bump EVERY workspace crate to one chosen <version> (lockstep),
# regenerate Cargo.lock, and record it as ONE signed `chore(release): prepare
# release batch <version>` commit — the path-touching prep commit that the
# pre-1.0 release pipeline depends on. Run BEFORE the per-crate
# `cargo release -p <crate> <version> --execute` steps (see docs/project/
# releasing.md §0); cargo-release then runs on the pre-bumped tree and does NOT
# re-bump.
#
# WHY lockstep needs this (the non-obvious part): pre-1.0 every crate goes to the
# SAME version every batch, so a crate with no real changes has only chore commits
# in range. git-cliff's per-crate --include-path would see ZERO path-touching
# commits for that crate and omit its release section entirely -> the release
# would later hard-fail at the GitHub-release step ("no release notes"). This
# single commit touches EVERY crate's Cargo.toml, giving each no-op crate the
# path-touching, virtual-tag-eligible commit it needs (see cliff.toml's
# capture-and-suppress parser + "No user-facing changes" body). Retires at 1.0,
# when decoupled versioning means no-op releases don't happen.
#
# Two rewrites per manifest (both anchored at column 0 so only the intended lines
# move):
#   1. `^version = "..."`  -> the package's own version.
#   2. `^sdmx-x = { version = "=..."` -> the exact inter-crate pin (ADR-0003).
#      Without this, sibling manifests would still require the OLD exact version
#      and `cargo update` would fail to resolve. The `=` (exact pin) is preserved.
# `rust-version = "..."` is deliberately NOT matched (the ^version anchor excludes
# the `rust-` prefix) — MSRV is governed separately by update-msrv.
#
# Extracted from the `prep-release` Justfile recipe so the manifest-mutation +
# commit logic is a testable unit (this is the highest-blast-radius recipe in the
# repo — it rewrites every manifest) and its result is framed by log.sh. The
# recipe now delegates here.
#
# POSIX sh only.
#
# Usage: scripts/prep-release.sh <version>     (e.g. 0.2.0  or  0.2.0-alpha.1)
#
# Environment:
#   CARGO  cargo invocation to use (default: cargo) — indirection for tests,
#          which stub it so no real registry resolve runs.
#   GIT    git invocation to use (default: git) — indirection for tests, which
#          stub it so no real (signed) commit is created.
#
# Exit codes:
#   0 = all manifests bumped, lockfile updated, signed batch commit created
#   1 = no <version> argument given (refusing to rewrite manifests with an empty
#       version, which would silently corrupt every Cargo.toml), OR an exact
#       inter-crate pin was not rewritten to <version> (the post-loop guard
#       caught a silent sed miss before it could reach the signed commit)
#   N = a delegated cargo/git step failed (its own exit code)
# ==============================================================================

set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/common.sh"

CARGO="${CARGO:-cargo}"
GIT="${GIT:-git}"

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    log_err "prep-release: no version given."
    log_err_detail "Usage: prep-release.sh <version>   (e.g. 0.2.0 or 0.2.0-alpha.1)"
    exit 1
fi

log_section "Preparing pre-1.0 release batch ${VERSION}"

for crate in $(get_crates); do
    manifest="crates/${crate}/Cargo.toml"
    log_item "$crate: bumping ${manifest} to ${VERSION}" 1
    # 1. The package's own version (column-0 anchored: excludes rust-version).
    sed -i -E 's/^version = "[^"]*"/version = "'"$VERSION"'"/' "$manifest"
    # 2. The exact inter-crate pins (ADR-0003): rewrite the pinned version while
    #    KEEPING the leading `=` exact-pin marker.
    sed -i -E 's/^(sdmx-[a-z]+ = \{ version = ")=[^"]*(")/\1='"$VERSION"'\2/' "$manifest"
done

# Post-condition guard — fail loud on a SILENT pin miss.
#
# Both rewrites above are formatting-fragile by nature (column-0 anchor, exact
# single-line `{ version = "=..." }` shape, `[a-z]+` crate-name class). A pin that
# doesn't match — a multi-line reformat, a digit/uppercase in a future crate name,
# or any spacing drift — is sed silently doing nothing: the script still exits 0,
# but that manifest now carries a STALE exact pin against a sibling that no longer
# publishes at the old version. `cargo update` may then fail to resolve, or worse,
# a crate gets published pinned to the wrong version. This is the highest-blast-
# radius script in the repo; a missed pin must abort, not slip through to a signed
# commit. So after the loop, assert that EVERY exact `sdmx-* = { version = "=..." }`
# pin across all manifests now reads exactly `=${VERSION}`. Any other value is a
# pin the rewrite failed to reach.
STALE_PINS=$(grep -rEn '^sdmx-[a-zA-Z0-9_-]+ = \{ version = "=' crates/*/Cargo.toml \
    | grep -vF "version = \"=${VERSION}\"" || true)
if [ -n "$STALE_PINS" ]; then
    log_err "prep-release: exact inter-crate pin(s) not rewritten to ${VERSION}."
    log_err_detail "The sed pin-rewrite did not reach these lines (formatting drift, a"
    log_err_detail "crate name outside [a-z]+, or a multi-line pin). Fix the manifest"
    log_err_detail "formatting or the rewrite, then re-run — NOT committing a stale pin:"
    printf '%s\n' "$STALE_PINS" | sed 's/^/    /' >&2
    exit 1
fi

log_item "Regenerating Cargo.lock" 1
"$CARGO" update --workspace

log_item "Committing signed release-batch checkpoint" 1
"$GIT" add crates/*/Cargo.toml Cargo.lock
"$GIT" commit --gpg-sign -m "chore(release): prepare release batch ${VERSION}"

log_ok "prep-release: bumped all crates to ${VERSION} and committed signed release-batch checkpoint"
