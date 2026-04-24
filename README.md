# PhenoVCS

Pure Rust, async-first Version Control System (VCS) primitives library for the Phenotype ecosystem. Provides high-performance Git operations with zero C dependencies, designed for CI/CD pipelines, automated repository management, and seamless integration with the broader Phenotype toolchain.

## Overview

**PhenoVCS** is the foundational VCS layer enabling automated repository operations across the Phenotype ecosystem. It leverages `gitoxide` for pure Rust Git implementations, delivering 2x performance over git2-rs while maintaining strict memory safety and async-first API design.

**Core Mission**: Enable seamless, performant version control operations for HeliosCLI, AgilePlus, workspace management, and CI/CD automation across all Phenotype projects.

## Technology Stack

- **Language**: Rust (Edition 2024)
- **Git Engine**: gitoxide (zero C dependencies, pure Rust)
- **Async Runtime**: Tokio
- **Key Crates**:
  - `pheno-vcs-core` — Core VCS primitives and Git repository operations
  - `pheno-vcs-integration` — Ecosystem integration helpers and telemetry
  - `pheno-vcs-cli` — Command-line tooling
- **Platform Support**: Linux, macOS, Windows

## Key Features

- **Pure Rust Implementation**: Zero C dependencies via gitoxide, passes Miri safety checks
- **Async-First Design**: 100% async/await API coverage with Tokio
- **High Performance**: 2x faster than git2-rs for core operations, <50ms cold repository open
- **Memory Safe**: Leverages Rust ownership model, zero CVEs
- **Ecosystem Integration**: Built for PhenoKit, HeliosCLI, AtomsBot, AgilePlus
- **Git Compatibility**: Full read/write support for Git repositories

## Quick Start

```bash
# Clone and explore
git clone <repo-url>
cd PhenoVCS

# Build all crates
cargo build --release

# Run tests and validation
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Review specification
cat SPEC.md
```

## Project Structure

```
PhenoVCS/
├── crates/
│   ├── pheno-vcs-core/        # Core VCS primitives & Git operations
│   ├── pheno-vcs-integration/  # Ecosystem integration & telemetry
│   └── pheno-vcs-cli/          # Command-line interface
├── docs/
│   ├── SPEC.md                 # Comprehensive specification
│   └── guides/                 # Integration and usage guides
├── examples/                   # Example usage patterns
├── tests/                      # Integration tests
└── CLAUDE.md, AGENTS.md        # Governance & agent contract
```

## Design Goals

| Goal | Status | Metric |
|------|--------|--------|
| Memory Safety | ✓ | Zero CVEs, Miri clean |
| Performance | ✓ | 1.5x git2-rs, <50ms cold open |
| Async API | ✓ | 100% async coverage |
| Binary Size | ✓ | <2MB core |
| Clean Build | ✓ | <30s |

## Related Phenotype Projects

- **[AgilePlus](../AgilePlus)** — Uses PhenoVCS for Git-backed workspace management
- **[PhenoDevOps](../PhenoDevOps)** — Integrates VCS operations in CI/CD pipelines
- **[thegent](../thegent)** — Git integration for dotfiles and workspace automation