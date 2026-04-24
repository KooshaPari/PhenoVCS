# PhenoVCS

Version control primitives registry for the Phenotype ecosystem — git worktree management, VCS abstractions, and the concurrency-safe primitives Phenotype agents rely on when they check out, branch, commit, and integrate code.

**Part of the [Phenotype org](https://github.com/KooshaPari) ecosystem.** Shares CI reusables and conventions with [phenoShared](https://github.com/KooshaPari/phenoShared). Follows org conventions: conventional commits, `<type>/<topic>` branching, Apache-2.0 + MIT dual license.

## What it does

Phenotype runs many concurrent agents that each need their own safe, reproducible working tree. PhenoVCS encapsulates the rules — worktree creation and teardown, branch discipline, integration-mode detection, lock coordination — so every agent runtime enforces the same VCS behaviors instead of reimplementing `git` shell-outs.

The crates here are consumed by agent dispatchers, the Phenotype daemon, and higher-level spec-driven workflows that spawn per-feature worktrees (`repos/<repo>-wtrees/<topic>`).

## Status

**Active — scaffolding phase.** Core crates (`pheno-vcs-core`, `worktree-manager`) are stabilizing; see [SPEC.md](./SPEC.md), [PLAN.md](./PLAN.md), and [ADR.md](./ADR.md) for current design.

## Requirements

- Rust stable (edition 2021)
- `git` 2.40+ on `$PATH` (required for worktree semantics used by `worktree-manager`)

## Quick start

```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Format
cargo fmt --all -- --check
```

Add a crate as a git dependency:

```toml
[dependencies]
pheno-vcs-core     = { git = "https://github.com/KooshaPari/PhenoVCS" }
worktree-manager   = { git = "https://github.com/KooshaPari/PhenoVCS" }
```

## Structure

```
crates/
  pheno-vcs-core/      # Core VCS traits, types, and git-agnostic primitives
  worktree-manager/    # Git worktree lifecycle — create, lock, integrate, prune
ADR.md                 # Architecture decision records
SPEC.md                # Formal spec for worktree semantics and invariants
PRD.md                 # Product requirements
PLAN.md                # Roadmap and sequencing
AGENTS.md              # Agent-facing workflow contract
CLAUDE.md              # Claude-agent operating notes
```

## Design principles

- **Worktrees, not branches-in-place.** Every concurrent agent gets an isolated checkout; no shared working trees.
- **Canonical checkouts stay on `main`.** Feature work happens in `<repo>-wtrees/<topic>`, never in the canonical folder.
- **Fail loudly on dirty integration.** Mixed-provenance commits are a policy violation; VCS primitives surface them early.
- **Non-destructive by default.** `git reset --hard`, `git clean -f`, and friends are gated behind explicit opt-in.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md), [AGENTS.md](./AGENTS.md), and [CLAUDE.md](./CLAUDE.md) for the agent-driven workflow. Report security issues per [SECURITY.md](./SECURITY.md).

## License

Dual-licensed under Apache-2.0 OR MIT. See [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT).
