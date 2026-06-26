{
  description = "Deterministic development environment for sdmx-rs";

  inputs = {
    # Nix inputs are cryptographically pinned to the May 2026 milestone via commit SHAs
    # to guarantee absolute reproducible environments and freeze compiler-floating tool behaviours
    # while ensuring stable 1.91.0 is fully indexed and available in the rust-overlay.
    nixpkgs.url = "github:NixOS/nixpkgs/d233902339c02a9c334e7e593de68855ad26c4cb";
    rust-overlay = {
      url = "github:oxalica/rust-overlay/6f44d8874ac29806c8d5cae42bf8e19ebb5ce0d3";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane/10e6e3cb966f7cfcc789fe5eee7a85f3188ce08b";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, crane }:
    let
      # Surface area for cross-platform development and CI validation
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      # MAINTENANCE: rustfmt-date-bump
      # Last updated: 2026-05-30
      # Next review: 2026-07-29
      # Explicit pin for nightly rustfmt to ensure deterministic formatting
      # This date must be periodically updated in sync with team toolchain reviews
      toolchainDate = "2026-05-01";

      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          commonArgs = {
            inherit src;
            strictDeps = true;
            pname = "sdmx-rs-workspace";
            # Placeholder for Crane/Nix derivation metadata ONLY — this is not
            # the source of truth for published versions (those come from each
            # crate's Cargo.toml via `cargo metadata` in publish.yml). Pinned to
            # "0.0.0" so it never drifts from, or is mistaken for, a real
            # release version.
            version = "0.0.0";
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        in
        f { inherit pkgs rustToolchain craneLib src commonArgs cargoArtifacts; }
      );
    in
    {
      packages = forEachSupportedSystem ({ craneLib, commonArgs, cargoArtifacts, ... }:
        let
          # Build the workspace members
          sdmx-rs-workspace = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
        in
        {
          default = sdmx-rs-workspace;
          # The crane dependency-closure derivation, exposed so CI can
          # `nix build` it with `--out-link` to register a GC root. Without a
          # root, cache-nix-action's pre-save garbage collection removes it and
          # the dependency tree recompiles every run (#92).
          inherit cargoArtifacts;
        }
      );

      # Fetch-on-demand SDMX schemas (see specs/README.md). The pins live
      # in specs/sources.toml — the SINGLE source of truth for BOTH this
      # fixed-output derivation (per-edition commit + NAR hash) and the shell
      # verify gate (per-file sha256, scripts/fetch-specs.sh). The SDMX schema
      # files are fetched at build time, never committed; a repo-scoped Actions
      # cache is usage only, never redistribution (must never reach a public
      # substituter).
      #
      # Deliberately exposed under legacyPackages, NOT packages/checks, so that
      # `nix flake check` never evaluates or builds it (legacyPackages is exempt
      # from flake check): the schema closure is materialised only by an explicit
      # `nix build .#sdmxSpecs`, run by scripts/fetch-specs.sh locally and by the
      # fetch-gated CI jobs. `nix build .#sdmxSpecs` resolves here.
      legacyPackages = forEachSupportedSystem ({ pkgs, ... }:
        let
          specsCfg = builtins.fromTOML (builtins.readFile ./specs/sources.toml);
          # fetchFromGitHub NAR-hashes the UNPACKED tree at the pinned commit:
          # robust to GitHub re-compressing the .tar.gz over time (whose bytes
          # are not stable), unlike a fetchurl of the archive. The rev is the
          # full 40-char commit SHA (immutable), the hash is trust-on-first-use,
          # both recorded by scripts/update-specs.sh.
          fetchEdition = ed: pkgs.fetchFromGitHub {
            owner = specsCfg.upstream.owner;
            repo = specsCfg.upstream.repo;
            rev = specsCfg.edition.${ed}.rev;
            hash = specsCfg.edition.${ed}.narHash;
          };
          # One $out/<ed>/schemas/ per pinned edition, mirroring the in-tree
          # specs/ layout the generator and parsers expect (xs:include /
          # xs:import relative paths resolve unchanged).
          editions = builtins.attrNames specsCfg.edition;
        in
        {
          sdmxSpecs = pkgs.runCommand "sdmx-specs" { } (
            ''
              mkdir -p "$out"
            ''
            + pkgs.lib.concatMapStrings (ed: ''
              mkdir -p "$out/${ed}/schemas"
              cp -R ${fetchEdition ed}/schemas/. "$out/${ed}/schemas/"
            '') editions
          );
        }
      );

      # --- Relationship between `nix flake check` and `just verify` ---
      #
      # `nix flake check` evaluates the derivations below inside a pure Nix sandbox.
      # It is NOT a direct alias for `just verify`; the two are complementary:
      #
      #   Nix check attribute    Equivalent `just verify` step
      #   ─────────────────────  ────────────────────────────────────────────────
      #   sdmx-rs-fmt            fmt-check  (nightly rustfmt + taplo)
      #   sdmx-rs-clippy         clippy
      #   sdmx-rs-tests          test  (standard cargoTest; no llvm-cov instrumentation)
      #   sdmx-rs-wasm           check-wasm  (WASM + parsers-only feature combination)
      #   sdmx-rs-doc            doc
      #   sdmx-rs-toml           toml-check  (subset of fmt-check)
      #   sdmx-rs-markdown       md-check
      #
      # The following `just verify` steps have NO Nix check equivalent and are
      # instead covered by dedicated CI jobs that run `nix develop --command ...`:
      #
      #   `deny` / `machete`     → CI: security job        (just deny machete)
      #   `shellcheck`           → CI: check-scripts job   (just shellcheck)
      #   `actionlint`           → CI: check-scripts job   (just actionlint)
      #   `link-check`           → CI: check-docs job      (just link-check)
      #   `verify-adr`           → CI: check-docs job      (just verify-adr)
      #   `verify-design`        → CI: check-docs job      (just verify-design)
      #   `semver-check`         → CI: semver-check job    (just semver-check)
      #   `release-dry-run`      → CI: release-check job   (just release-dry-run)
      #   `test-coverage-headless` → CI: coverage job      (just test-coverage-headless)
      #   `nix-check`            → CI: nix-check job       (nix flake check)
      #
      # Together all CI jobs cover every step in `just verify`. The `nix-check`
      # CI job additionally validates that the Nix derivations themselves evaluate
      # and build correctly inside the Nix sandbox — something `just verify` cannot test.
      checks = forEachSupportedSystem ({ pkgs, craneLib, src, commonArgs, cargoArtifacts, ... }:
        {
          # Standard Package build check
          sdmx-rs-workspace = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });

          # Enforce nightly rustfmt imports-sorting checks inside the sandbox.
          # Explicit pname and version silence Crane's fallback root Cargo.toml parsing warnings.
          sdmx-rs-fmt = craneLib.cargoFmt {
            inherit src;
            pname = "sdmx-rs-fmt";
            version = "0.0.0";
            RUSTFMT = "${pkgs.rust-bin.nightly.${toolchainDate}.rustfmt}/bin/rustfmt";
          };

          # Clippy Warnings check
          sdmx-rs-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--workspace --all-targets -- -D warnings";
          });

          # Test suite check
          sdmx-rs-tests = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--workspace";
          });

          # WASM compilation check
          sdmx-rs-wasm = craneLib.cargoBuild (commonArgs // {
            inherit cargoArtifacts;
            cargoBuildCommand = "cargo check -p sdmx-types --target wasm32-unknown-unknown && cargo check -p sdmx-parsers --target wasm32-unknown-unknown && cargo check -p sdmx-writers --target wasm32-unknown-unknown && cargo check -p sdmx-rs --target wasm32-unknown-unknown --no-default-features && cargo check -p sdmx-rs --no-default-features --features parsers";
          });

          # Cargo doc warnings-as-errors check
          sdmx-rs-doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
            cargoDocExtraArgs = "--workspace --no-deps --all-features";
            RUSTDOCFLAGS = "-D warnings";
          });

          # Validate formatting of all TOML manifests in the workspace
          sdmx-rs-toml = craneLib.cargoBuild (commonArgs // {
            inherit cargoArtifacts;
            cargoBuildCommand = "taplo fmt --check";
            nativeBuildInputs = [ pkgs.taplo ];
          });

          # Validate markdown files structure and layout style rules
          sdmx-rs-markdown = pkgs.stdenv.mkDerivation {
            pname = "sdmx-rs-markdown-check";
            # Derivation-metadata placeholder only (see commonArgs.version).
            version = "0.0.0";
            src = ./.;
            nativeBuildInputs = [ pkgs.markdownlint-cli2 ];
            dontBuild = true;
            installPhase = ''
              # Glob + path ignores passed explicitly as CLI negated globs (cli2
              # does not read .markdownlintignore in this mode). Rule config is read
              # from .markdownlint.yaml, merged per-directory via `extends` — those
              # config files must be git-tracked to be present in the flake src.
              markdownlint-cli2 "**/*.md" "#target" "#.direnv" "#.git" "#node_modules"
              mkdir -p $out
            '';
          };
        }
      );

      devShells = forEachSupportedSystem ({ pkgs, ... }: {
        default = pkgs.mkShell {
          nativeBuildInputs = [
            # ==================================================================
            # 1. Core Toolchain & Orchestration
            # ==================================================================
            # Parses rust-toolchain.toml to provision the exact stable toolchain compiler
            (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
            pkgs.just                           # Task runner / script coordinator
            pkgs.sccache                        # Compilation caching compiler wrapper

            # ==================================================================
            # 2. Rust Verification & Audit Utilities
            # ==================================================================
            pkgs.cargo-bloat                    # Profiles size footprint of compiled binaries
            pkgs.cargo-deny                     # Audits licenses, sources, and RustSec advisories
            pkgs.cargo-fuzz                     # LibFuzzer-based fuzz testing orchestration harness
            pkgs.cargo-geiger                   # Scans compilation dependency tree for unsafe Rust code
            pkgs.cargo-llvm-cov                 # Instrument-free cargo code coverage generator
            pkgs.cargo-machete                  # Fast detector of unused dependencies in Cargo.toml
            pkgs.cargo-nextest                  # Concurrent test runner optimised for Rust workspaces
            pkgs.cargo-outdated                 # Scans dependency tree for outdated package versions
            pkgs.cargo-semver-checks            # Validates public API changes for SemVer compliance

            # ==================================================================
            # 3. Multi-Language Linters & Quality Gates
            # ==================================================================
            pkgs.actionlint                     # GitHub Actions CI workflow validator
            pkgs.gitleaks                       # Secret leak scanner; secrets-scan recipe + verify gate
            pkgs.lychee                         # Offline document link checking engine
            pkgs.markdownlint-cli2              # Markdown style/structure linter (per-dir config via extends)
            pkgs.shellcheck                     # Shell script static analysis tool
            pkgs.taplo                          # TOML syntax formatter and linter

            # ==================================================================
            # 4. Git Integration & Release Automation
            # ==================================================================
            pkgs.cargo-release                  # Automated workspace publishing tool
            pkgs.commitlint                     # Ensures commit messages follow Conventional Commits
            pkgs.curl                           # crates.io sparse-index probe in scripts/ci/check-published.sh
            pkgs.gh                             # GitHub CLI for automated maintenance issues
            pkgs.git                            # Distributed version control system
            pkgs.git-cliff                      # Automatically generates changelogs from commits
            pkgs.gnupg                          # gpg: signs release commits/tags (prep-release, release-commit-changelogs); pin it so the signing tool is hermetic, not host-supplied. (pinentry is NOT pinned — it is spawned by the user's gpg-agent, a session daemon outside this shell; pinentry/agent setup is a documented user responsibility, see docs/project/releasing.md.)
            pkgs.jq                             # JSON processor; parses `cargo metadata` in release/CI scripts
            pkgs.pre-commit                     # Git hooks manager running verification gates

            # ==================================================================
            # 5. Specialised Test Runtimes
            # ==================================================================
            # Shell script testing framework for maintenance and diagnostic scripts
            pkgs.bash
            pkgs.bats
            # Scaffold for WASM execution testing (Phase 1). wasm-pack drives
            # `wasm-pack test --node`; nodejs provides the V8 WASM runtime.
            pkgs.wasm-pack
            pkgs.nodejs

            # ==================================================================
            # 6. SBOM & Attestation Tooling
            # ==================================================================
            pkgs.cargo-cyclonedx                # Generates CycloneDX SBOMs from Cargo workspaces
            pkgs.cyclonedx-cli                  # Converts CycloneDX SBOMs to SPDX and other formats
          ];

          # Routes `cargo fmt` to the nightly rustfmt for unstable features.
          # We pin to a specific date to prevent silent toolchain drift.
          # Note: Nightly rustfmt is excluded from nativeBuildInputs to avoid
          # PATH collisions with the stable rustfmt in the toolchain.
          RUSTFMT = "${pkgs.rust-bin.nightly.${toolchainDate}.rustfmt}/bin/rustfmt";

          # Enable sccache for local builds within the Nix shell environment
          RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";

          # Diagnostics go to stderr, not stdout. `nix develop --command <cmd>`
          # runs this hook before <cmd>, so anything echoed to stdout here would
          # corrupt the stdout of that command — e.g. `cargo metadata | jq` in
          # publish.yml would receive the banner ahead of the JSON and fail.
          shellHook = ''
            {
              echo "--- SDMX-RS DEVELOPMENT ENVIRONMENT ACTIVE ---"
              cargo --version
              rustc --version
              echo "rustfmt (nightly): $($RUSTFMT --version)"
            } >&2
          '';
        };
      });
    };
}
