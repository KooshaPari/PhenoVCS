# PhenoVCS Specification

## Overview

Rust workspace providing VCS (Version Control System) primitives for the Phenotype ecosystem.

## Workspace

- **members**: `crates/pheno-vcs-core`
- **Edition**: 2021
- **License**: MIT
- **Repository**: github.com/KooshaPari/PhenoVCS

## Crates

| Crate | Description |
|-------|-------------|
| `pheno-vcs-core` | VCS primitives |

## Dependencies

- anyhow: ^1.0
- thiserror: ^1.0

## Commands

```bash
cargo build
cargo test
cargo clippy --workspace -- -D warnings
```

## Related

- Part of Phenotype polyrepo shelf