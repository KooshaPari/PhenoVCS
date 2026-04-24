# AGENTS.md — PhenoVCS

## Project Overview

- **Name**: PhenoVCS (Version Control System)
- **Description**: Distributed version control system with content-addressable storage, advanced branching, and merge capabilities
- **Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/PhenoVCS`
- **Language Stack**: Rust (Edition 2024), libgit2
- **Published**: Private (Phenotype org)

## Quick Start Commands

```bash
# Clone and setup
git clone https://github.com/KooshaPari/PhenoVCS.git
cd PhenoVCS

# Install Rust toolchain
rustup update nightly
rustup default nightly

# Build
cargo build --release

# Run tests
cargo test
cargo nextest run

# Initialize repo
pvcs init my-repo
```

## Architecture

### VCS Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Client Interface                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   CLI             │  │   Library         │  │   Web             │         │
│  │   (Terminal)        │  │   (API)           │  │   (UI)            │         │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘         │
└───────────┼────────────────────┼────────────────────┼────────────────┘
            │                    │                    │
            ▼                    ▼                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      PhenoVCS Core (Rust)                                │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    Version Control Engine                        │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐            │   │
│  │  │   Object   │  │   Branch   │  │   Merge    │            │   │
│  │  │   Store    │  │   Manager  │  │   Engine   │            │   │
│  │  └────────────┘  └────────────┘  └────────────┘            │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐            │   │
│  │  │   Diff     │  │   Index    │  │   Rebase   │            │   │
│  │  │   Engine   │  │   Manager  │  │   Engine   │            │   │
│  │  └────────────┘  └────────────┘  └────────────┘            │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Storage Layer                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   Content-Addr    │  │   Refs            │  │   Config          │         │
│  │   (Objects)         │  │   (Branches/Tags) │  │   (Settings)      │         │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
└─────────────────────────────────────────────────────────────────────┘
```

### Object Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Content-Addressable Storage                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Commit ──▶ Tree ──▶ Blob                                           │
│                                                                      │
│   ┌──────┐    ┌──────┐    ┌──────┐                                 │
│   │Commit│───▶│ Tree │───▶│ Blob │                                 │
│   │      │    │      │    │      │                                 │
│   │parent│    │file_a│    │content│                                 │
│   │tree  │    │file_b│    └──────┘                                 │
│   │author│    │dir/  │                                              │
│   └──────┘    └──────┘                                              │
│                                                                      │
│   All objects SHA-256 hashed                                         │
│   Deduplication automatic                                             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Quality Standards

### Rust Code Quality

- **Formatter**: `rustfmt` (nightly)
- **Linter**: `clippy -- -D warnings`
- **Tests**: `cargo nextest run` with coverage >80%

### VCS Standards

- Git compatibility where possible
- Cryptographic integrity (SHA-256)
- Efficient delta compression
- Fast large-file handling

### Test Requirements

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# Compatibility tests
cargo test --test git-compat
```

## Git Workflow

### Branch Naming

Format: `<type>/<component>/<description>`

Types: `feat`, `fix`, `docs`, `refactor`, `perf`

Examples:
- `feat/merge/add-three-way`
- `fix/delta/handle-binary`
- `perf/index/use-mmap`

## CLI Commands

```bash
# Initialize repository
pvcs init

# Stage files
pvcs add file.txt

# Commit
pvcs commit -m "Initial commit"

# Create branch
pvcs branch feature-x

# Merge
pvcs merge feature-x

# Push
pvcs push origin main
```

## Environment Variables

```bash
# Core
PVCS_AUTHOR_NAME="User Name"
PVCS_AUTHOR_EMAIL="user@example.com"

# Storage
PVCS_OBJECT_CACHE_SIZE=100MB
PVCS_DELTA_CACHE_SIZE=50MB
```

## Governance Rules

### Mandatory Checks

1. **FR Traceability**: All tests MUST reference FR-XXX-NNN
2. **AI Attribution**: .phenotype/ai-traceability.yaml MUST exist
3. **CI/CD Compliance**: .github/workflows/traceability.yml MUST pass
4. **Code Quality**: Minimum 80% coverage for new code

### Prohibited Actions

- ❌ Delete without read first
- ❌ Modify without FR reference
- ❌ Skip validation on merge

### Validation

Run before any commit:
```bash
python3 validate_governance.py
```

---

Last Updated: 2026-04-05
Version: 1.0.0
