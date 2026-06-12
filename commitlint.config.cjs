/**
 * sdmx-rs Conventional Commit Linting Configuration
 *
 * Enforces git commit message standards to ensure clean history and
 * automated changelog generation (via git-cliff).
 */
const allowedScopes = () => [
  // Crates
  'client',      // sdmx-client crate
  'parsers',     // sdmx-parsers crate
  'facade',      // sdmx-rs facade crate
  'types',       // sdmx-types crate
  'writers',     // sdmx-writers crate

  // Documentation
  'docs',        // General documentation (README, SECURITY, etc.)
  'adr',         // Architecture Decision Records
  'design',      // Design documents (docs/design/ and just design)
  'guide',       // User-facing usage guides (docs/guides/) — consumer audience
  'arch',        // Repository architecture (ARCHITECTURE.md)
  'project',     // Project planning & process docs (ROADMAP.md, docs/project/)
  'dev',         // Developer/contributor docs (docs/dev/) — not for consumers

  // Testing & Infrastructure
  'test',        // BATS tests and test infrastructure
  'fuzz',        // Fuzz targets and fuzzing environment
  'bench',       // Benchmarks and performance baselines
  'ci',          // GitHub Actions workflows

  // Configuration & Tooling
  'repo',        // Cross-cutting changes (repo config, multiple crates affected)
  'config',      // Configuration files (commitlint, pre-commit, deny.toml, etc.)
  'keys',        // GPG maintainer keys (.github/maintainer-keys/)
  'deps',        // Dependency updates
  'nix',         // Nix flake and development environment
  'tooling',     // Justfile and repository scripts
  'maintenance', // maintenance.toml and obligation tracking

  // Release & Performance
  'release',     // Version bumps, changelog, release process
  'msrv',        // Minimum Supported Rust Version changes
  'perf',        // Performance improvements and profiling
  'security',    // Security policies and vulnerability responses
];

module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'type-enum': [
      2,
      'always',
      [
        'feat',
        'fix',
        'refactor',
        'perf',
        'style',
        'test',
        'chore',
        'docs',
        'ci',
        'build',
        'revert',
      ],
    ],
    'scope-enum': () => [
      2,
      'always',
      allowedScopes(),
    ],
  },
};
