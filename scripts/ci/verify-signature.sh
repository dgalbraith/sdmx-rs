#!/bin/sh
# shellcheck disable=SC2310,SC2311
set -eu

# ==============================================================================
# scripts/ci/verify-signature.sh
# Enforces the maintainer-signed-only invariant: every commit that lands on
# main, and every tag object PLUS the commit that tag points at, must carry a
# valid GPG signature whose PRIMARY key fingerprint appears in
# ALLOWED_PRIMARY_FINGERPRINTS. (The tag commit is checked because the publish
# chain keys off it, not off the merge commit — see the tag-push branch below.)
#
# Trust model:
#   - The committed public key (.github/maintainer-keys/*.asc) is the trust
#     source. It is imported into a clean keyring at job start (done in the
#     calling workflow step, not here — this script only verifies).
#   - Verification uses `git verify-{commit,tag} --raw`, which emits a VALIDSIG
#     status line ONLY for a good, non-expired, non-revoked signature.
#   - Match the LAST field of VALIDSIG (the PRIMARY key fingerprint), not the
#     signing subkey. Rotation-proof: rotating the signing subkey needs no
#     change here, only a refreshed .asc commit.
#   - The extracted fingerprint is validated for cardinality (exactly one
#     VALIDSIG line) and shape (^[0-9A-F]{40}$) before the allowlist check.
#     This ensures format drift or unexpected gpg output fails closed and
#     legibly rather than silently passing or misclassifying.
#
# Runner trust:
#   This script runs with the GitHub-hosted runner's git and gpg — deliberately
#   not Nix-pinned. That is an accepted trade-off: a compromised runner is
#   outside our threat model (SLSA L2 designates the runner as the trusted
#   authority), and keeping the trust anchor a simple, dependency-light gate
#   with no Nix bootstrap on the critical path is worth more than toolchain
#   pinning here. The fail-closed parse design (content over exit codes, shape
#   and cardinality guards) is the primary robustness mechanism — not the
#   toolchain version.
#
# Usage (called by verify-signature.yml):
#   ALLOWED_PRIMARY_FINGERPRINTS="..." \
#   GITHUB_REF_TYPE="tag|branch" \
#   GITHUB_REF_NAME="..." \
#   GITHUB_SHA="..." \
#   GITHUB_EVENT_BEFORE="..." \   # push pre-image; optional but SHOULD be set on
#                                 # branch pushes — without it the walk falls back
#                                 # to the single target commit (--no-walk) instead
#                                 # of the BEFORE..SHA delta (see dispatch below).
#   scripts/ci/verify-signature.sh
#
# Exit codes:
#   0 = all checked refs satisfy the maintainer-signed-only invariant
#   1 = violation detected, configuration error, or missing required input
#
# Logging convention — DELIBERATE EXCEPTION: this gate does NOT route output
# through scripts/lib/log.sh. It uses its own domain severity vocabulary
# (`❌ CONFIGURATION ERROR:`, `❌ PROTOCOL VIOLATION:`) that is semantically
# meaningful for a security gate and asserted verbatim by tests/bats/
# verify-signature.bats (including the ABSENCE of PROTOCOL VIOLATION in a
# pure-config-error case). The verify_ref function also `return`s rather than
# `exit`s so it can accumulate and report ALL violations before failing once.
# Folding these into the generic logger would erase the distinction and the
# report-all design, so this script is intentionally left outside the
# logger-by-construction rule.
# ==============================================================================

# ------------------------------------------------------------------------------
# Input contract — assert all required env vars are present and non-empty.
# Every variable the script consumes is declared here. A missing var means the
# YAML→script wiring is broken; fail loudly rather than silently reading an
# unset variable as empty and producing a misleading result.
# ------------------------------------------------------------------------------
_require_var() {
    _var_name="$1"
    # Use eval to dereference the variable by name in POSIX sh (no nameref).
    # shellcheck disable=SC2016
    _var_val=$(eval 'echo "${'"$_var_name"':-}"')
    if [ -z "$_var_val" ]; then
        echo "❌ CONFIGURATION ERROR: required environment variable ${_var_name} is not set." >&2
        exit 1
    fi
}

_require_var ALLOWED_PRIMARY_FINGERPRINTS
_require_var GITHUB_REF_TYPE
_require_var GITHUB_REF_NAME
_require_var GITHUB_SHA
# GITHUB_EVENT_BEFORE is optional (absent on first push / new branch); handled
# with a :- default below.

# ------------------------------------------------------------------------------
# Build the allowlist: normalise to uppercase, strip spaces, keep only lines
# that are exactly 40 hex characters. An empty result means the env var was
# set but contained no valid fingerprints — a configuration error, not a
# "no maintainers" state.
# ------------------------------------------------------------------------------
ALLOWED=$(printf '%s' "${ALLOWED_PRIMARY_FINGERPRINTS}" \
    | tr '[:lower:]' '[:upper:]' \
    | tr -d ' ' \
    | grep -E '^[0-9A-F]{40}$' || true)

if [ -z "${ALLOWED}" ]; then
    echo "❌ CONFIGURATION ERROR: ALLOWED_PRIMARY_FINGERPRINTS is empty or malformed." >&2
    echo "   Expected one or more 40-character uppercase hex fingerprints." >&2
    exit 1
fi

# ------------------------------------------------------------------------------
# verify_ref VERB REF LABEL
#
# Verifies a single git ref (commit or tag object).
#
# Design: git's exit code is intentionally NOT used as the pass/fail signal —
# it varies by gpg version and trust-model configuration in ways that create
# fail-open edge cases (e.g. exit 0 for a valid-but-not-in-our-allowlist key).
# Instead, content-based verification: extract the PRIMARY fingerprint from the
# VALIDSIG status line emitted by --raw, then match against the allowlist.
#
# Hardening against output format drift (cardinality + shape):
#   1. Count VALIDSIG lines — exactly one expected. Zero → no valid sig. More
#      than one → unexpected format or multiple signatures; fail closed.
#   2. Validate the extracted fingerprint is ^[0-9A-F]{40}$ before the
#      allowlist check. A format shift that moves the fingerprint off $NF
#      becomes a clean failure rather than a silent allowlist miss.
#
# All non-happy paths return 1 (fail closed). The sole return 0 path requires:
#   - exactly one VALIDSIG line
#   - fingerprint matches ^[0-9A-F]{40}$
#   - fingerprint exact-matches an entry in ALLOWED
# ------------------------------------------------------------------------------
verify_ref() {
    _verb="$1"
    _ref="$2"
    _label="$3"

    # Capture raw gpg status output. || true: git exits non-zero for unsigned/
    # invalid objects; we parse content rather than trusting the exit code (see
    # design note above). Stderr redirected to stdout so GNUPG status lines are
    # captured regardless of gpg's fd routing.
    _raw=$(git "${_verb}" --raw "${_ref}" 2>&1 || true)

    # Count VALIDSIG lines. awk prints a count of matching lines.
    _validsig_count=$(printf '%s\n' "${_raw}" | awk '/^\[GNUPG:\] VALIDSIG/{n++} END{print n+0}')

    if [ "${_validsig_count}" -eq 0 ]; then
        echo "❌ PROTOCOL VIOLATION: ${_label} (${_ref}) has no valid signature." >&2
        echo "   A web-flow (GitHub UI) merge or an unsigned/expired/revoked" >&2
        echo "   signature produces no VALIDSIG and fails here by design." >&2
        return 1
    fi

    if [ "${_validsig_count}" -gt 1 ]; then
        echo "❌ PROTOCOL VIOLATION: ${_label} (${_ref}) produced ${_validsig_count} VALIDSIG lines." >&2
        echo "   Expected exactly one. This may indicate unexpected gpg output format." >&2
        return 1
    fi

    # Extract the PRIMARY key fingerprint from the single VALIDSIG line.
    # VALIDSIG format: [GNUPG:] VALIDSIG <fpr> <date> <ts> ... <primary-fpr>
    # $NF (last field) is the primary fingerprint.
    _primary=$(printf '%s\n' "${_raw}" | awk '/^\[GNUPG:\] VALIDSIG/{print $NF}')

    # Shape guard: primary fingerprint must be exactly 40 uppercase hex chars.
    case "${_primary}" in
        [0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F]\
[0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F]\
[0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F]\
[0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F]\
[0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F])
            ;;
        *)
            echo "❌ PROTOCOL VIOLATION: ${_label} (${_ref}) — extracted fingerprint is malformed." >&2
            echo "   Got: '${_primary}' (expected 40 uppercase hex characters)." >&2
            return 1
            ;;
    esac

    # Allowlist check: exact match against normalised ALLOWED.
    if ! printf '%s\n' "${ALLOWED}" | grep -qx "${_primary}"; then
        echo "❌ PROTOCOL VIOLATION: ${_label} (${_ref}) signed by an unauthorised key." >&2
        echo "   Primary fingerprint: ${_primary}" >&2
        return 1
    fi

    echo "✅ ${_label} (${_ref}) verified — primary ${_primary}"
    return 0
}

# ------------------------------------------------------------------------------
# Dispatch: tag push verifies the tag object AND the commit it points at; branch
# push verifies every commit newly introduced in the push range.
# ------------------------------------------------------------------------------
FAILED=0

if [ "${GITHUB_REF_TYPE}" = "tag" ]; then
    # Tag push: the canonical published ref. Verify BOTH the tag object AND the
    # commit it points at.
    #
    # Why the commit too, not just the tag wrapper: the publish chain keys off
    # the TAG COMMIT (publish.yml passes "$GITHUB_SHA" — the commit the tag
    # names — to verify-tag-on-main and packages that tree), not off the merge
    # commit. Verifying only the tag object would leave the published source
    # commit signature-unchecked on this path. The companion verify-tag-on-main
    # gate proves that commit is REACHABLE FROM main (an ancestor), which is a
    # weaker property than "a maintainer signed it": an ancestor of main is not
    # necessarily a maintainer-signed commit (e.g. pre-ruleset history, or a
    # commit that entered main by a path that bypassed the branch-push gate).
    # Checking the peeled commit here makes the published artifact's source
    # provenance hold on the tag path itself, independent of how it reached main.
    verify_ref verify-tag "${GITHUB_REF_NAME}" "tag" || FAILED=1

    # Peel the tag to its commit. Resolve before verifying so a malformed/missing
    # ref fails legibly rather than handing verify-commit an empty argument.
    TAG_COMMIT=$(git rev-parse --verify --quiet "${GITHUB_REF_NAME}^{commit}" || true)
    if [ -z "${TAG_COMMIT}" ]; then
        echo "❌ PROTOCOL VIOLATION: tag (${GITHUB_REF_NAME}) does not resolve to a commit." >&2
        FAILED=1
    else
        verify_ref verify-commit "${TAG_COMMIT}" "tag commit" || FAILED=1
    fi
else
    # Branch push: verify every commit newly introduced in the push.
    # GITHUB_EVENT_BEFORE is all-zeros on the first push / new branch.
    BEFORE="${GITHUB_EVENT_BEFORE:-}"
    if [ -z "${BEFORE}" ] || [ "${BEFORE}" = "0000000000000000000000000000000000000000" ]; then
        # No usable pre-image (first push / new branch / unset BEFORE): just the
        # target commit. NOT `git rev-list ${GITHUB_SHA}` — a bare revision means
        # "everything reachable", i.e. the whole ancestry, which would gpg-verify
        # all of history on one push. The normal path is the BEFORE..SHA delta
        # below; with GITHUB_EVENT_BEFORE wired in verify-signature.yml this
        # fallback is only the genuine first-commit case.
        COMMITS="${GITHUB_SHA}"
    else
        COMMITS=$(git rev-list "${BEFORE}..${GITHUB_SHA}")
    fi

    if [ -z "${COMMITS}" ]; then
        COMMITS="${GITHUB_SHA}"
    fi

    for _sha in ${COMMITS}; do
        verify_ref verify-commit "${_sha}" "commit" || FAILED=1
    done
fi

if [ "${FAILED}" -ne 0 ]; then
    echo "::error::Signature enforcement failed — see violations above."
    exit 1
fi

echo "✅ All checked refs satisfy the maintainer-signed-only invariant."
