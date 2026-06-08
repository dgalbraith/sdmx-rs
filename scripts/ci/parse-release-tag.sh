#!/bin/sh
set -eu

# ==============================================================================
# scripts/ci/parse-release-tag.sh
# Parses a release tag of the form "<crate>/v<version>" and asserts that the
# version it names matches the crate's actual Cargo.toml version. This binds
# the publish run to the signed tag that triggered it: the tag is the trust
# anchor, so the artifact that gets published must be exactly the version the
# tag claims — never whatever happens to sit in the working tree.
#
# Usage: scripts/ci/parse-release-tag.sh <tag>
#   e.g. scripts/ci/parse-release-tag.sh sdmx-types/v0.1.0
#
# Output (to $GITHUB_OUTPUT, or stdout when run outside Actions):
#   crate=<name>
#   version=<x.y.z>
#   crate_file=target/package/<name>-<x.y.z>.crate
#
# Exit codes:
#   0 = parsed and version-consistent
#   1 = malformed tag, unknown crate, or tag/Cargo.toml version mismatch
# ==============================================================================

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/../lib/log.sh"

TAG="${1:?usage: parse-release-tag.sh <crate>/v<version>}"

# Split "<crate>/v<version>". The crate segment must not contain a slash; the
# version segment must start with 'v'.
CRATE="${TAG%/*}"
VTAIL="${TAG##*/}"

case "$TAG" in
    */v*) ;;
    *)
        log_err_ci "Malformed release tag '${TAG}' — expected '<crate>/v<version>'."
        exit 1
        ;;
esac

case "$VTAIL" in
    v*) TAG_VERSION="${VTAIL#v}" ;;
    *)
        log_err_ci "Malformed release tag '${TAG}' — version segment must start with 'v'."
        exit 1
        ;;
esac

# Confirm the crate is a real workspace member and read its declared version.
CARGO_TOML="crates/${CRATE}/Cargo.toml"
if [ ! -f "$CARGO_TOML" ]; then
    log_err_ci "Tag names crate '${CRATE}', but ${CARGO_TOML} does not exist."
    exit 1
fi

# Authoritative version from cargo metadata (stdout is clean — the devShell
# banner goes to stderr; see flake.nix shellHook).
META_VERSION=$(cargo metadata --no-deps --format-version 1 \
    | jq -r --arg n "$CRATE" '.packages[] | select(.name == $n) | .version')

if [ -z "$META_VERSION" ] || [ "$META_VERSION" = "null" ]; then
    log_err_ci "Could not read a version for '${CRATE}' from cargo metadata."
    exit 1
fi

if [ "$TAG_VERSION" != "$META_VERSION" ]; then
    log_err_ci "Version mismatch: tag '${TAG}' claims ${TAG_VERSION}, but ${CRATE}'s"
    echo "   Cargo.toml is ${META_VERSION}. The signed tag and the artifact must agree." >&2
    exit 1
fi

emit() {
    key="$1"
    val="$2"
    if [ -n "${GITHUB_OUTPUT:-}" ]; then
        echo "${key}=${val}" >> "$GITHUB_OUTPUT"
    fi
    echo "${key}=${val}"
}

log_ok "${TAG} is consistent with ${CRATE} ${META_VERSION}."
emit crate "$CRATE"
emit version "$META_VERSION"
emit crate_file "target/package/${CRATE}-${META_VERSION}.crate"
