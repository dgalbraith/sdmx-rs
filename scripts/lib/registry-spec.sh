#!/bin/sh
# ==============================================================================
# scripts/lib/registry-spec.sh
#
# Single source of truth for the DESIRED crates.io REGISTRY state — the
# Trusted Publishing (TP) configuration and enforcement posture that
# docs/project/registry-setup.md describes in prose. SOURCED-ONLY: this file
# defines functions and has NO top-level side effects (no probing, no output, no
# network/`gh`/`curl` calls). Consumers source it, then call the spec functions:
#
#   scripts/doctor-registry.sh — read-only assert (live crates.io == spec)
#   scripts/registry-tp.sh     — print-only register/enforce command helper
#
# PLANE BOUNDARY: crates.io is a REGISTRY (distributes built artifacts), NOT a
# forge (hosts source). This spec is deliberately separate from forge-spec.sh and
# uses registry_* names. The ONE sanctioned cross-plane reference is that the TP
# config's environment MUST equal the forge's gating `release` environment — so
# we BORROW that value from forge-spec rather than re-declaring it, binding the
# two planes at their single legitimate seam. forge-spec.sh must be sourced before
# this file (consumers source both); see the note on registry_spec_tp_environment.
#
# POSIX sh only — no bashisms.
# ==============================================================================

# --- Crates -------------------------------------------------------------------

# registry_spec_crates — emit the 5 publishable crate names, one per line, in
# topological (publish) order: a crate precedes every crate that depends on it.
# Authored here (matching forge-spec's authored-scalar convention) to keep the
# library pure — deriving via `cargo metadata` would add a side-effecting tool
# dependency to a sourced-only lib. Cross-checked against crates/.
registry_spec_crates() {
    printf '%s\n' "sdmx-types"
    printf '%s\n' "sdmx-parsers"
    printf '%s\n' "sdmx-writers"
    printf '%s\n' "sdmx-client"
    printf '%s\n' "sdmx-rs"
}

# --- Trusted Publishing config ------------------------------------------------

# registry_spec_tp_workflow — echo the workflow filename registered as the
# Trusted Publisher. crates.io validates this against the repo's default branch;
# it must match the actual publish workflow file name (not path).
registry_spec_tp_workflow() {
    printf '%s\n' "publish.yml"
}

# registry_spec_tp_repo — echo "OWNER/REPO" for the TP binding. DERIVED by reusing
# the forge helper (the repo identity already lives there); this is one of the two
# sanctioned forge->registry references. Returns 1 (no output) if unresolved, so
# callers degrade gracefully exactly as they do for the forge.
registry_spec_tp_repo() {
    forge_spec_owner_repo
}

# registry_spec_tp_environment — echo the GitHub environment the TP config is
# bound to. REUSES forge_spec_release_env_name: the TP environment and the forge
# gating environment are the SAME `release` env by invariant — binding them here
# means they cannot silently drift apart. The other sanctioned cross-plane
# reference. (Requires forge-spec.sh sourced first.)
registry_spec_tp_environment() {
    forge_spec_release_env_name
}

# --- Enforcement --------------------------------------------------------------

# registry_spec_enforcement — echo the desired value of the crate's `trustpub_only`
# flag (crates.io "Require Trusted Publishing"). `true` is the end state: once a TP
# publish is proven, API-token publishing is disabled. doctor-registry treats a
# live `false` as NOT-YET-ENFORCED (warn), not drift, until the caller opts in via
# REGISTRY_ENFORCEMENT_REQUIRED=1 — mirroring forge's FORGE_RELEASE_REQUIRED.
registry_spec_enforcement() {
    printf '%s\n' "true"
}
