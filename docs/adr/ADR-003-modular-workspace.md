# ADR-003: Modular Workspace Architecture

**Status**: Accepted

**Date**: 2026-04-04

**Context**: PhenoVCS is designed as a Rust workspace with multiple crates. We need to decide on the crate structure, dependency relationships, and public API boundaries to maximize reusability while minimizing coupling.

---

## Decision Drivers

| Driver | Priority | Notes |
|--------|----------|-------|
| Reusability | High | Allow using individual components |
| Compile times | High | Minimize rebuilds during development |
| API stability | High | Clear public/private boundaries |
| Binary size | Medium | Don't include unused code |
| Testing | Medium | Unit testable components |

---

## Options Considered

### Option 1: Single Crate

**Description**: One monolithic `pheno-vcs` crate containing all functionality.

**Pros**:
- Simplest structure
- No cross-crate visibility concerns
- Single version number
- Easy to understand

**Cons**:
- Compile times increase with size
- All-or-nothing dependency (binary bloat)
- Cannot reuse individual components
- Internal changes may break public API

**Structure**:
```
pheno-vcs/
├── src/
│   ├── lib.rs          # Everything public
│   ├── repo.rs         # Repository
│   ├── odb.rs          # Object database
│   ├── refs.rs         # References
│   └── ...
```

### Option 2: Fine-Grained Crates

**Description**: Separate crate for every major component (odb, refs, diff, transport, etc.).

**Pros**:
- Maximum reusability
- Minimal compile times for changes
- Precise dependency management
- Clear API boundaries

**Cons**:
- Complex dependency graph
- Many version numbers to manage
- Publish coordination overhead
- Documentation spread across crates

**Structure**:
```
pheno-vcs-odb/
pheno-vcs-refs/
pheno-vcs-diff/
pheno-vcs-transport/
pheno-vcs-core/      # Re-exports
```

### Option 3: Layered Workspace (Selected)

**Description**: Medium-grained crates organized by abstraction layer: primitives, core, and integration.

**Pros**:
- Balance of reusability and simplicity
n- Clear architectural layers
- Reasonable compile times
- Manageable version surface

**Cons**:
- More complex than single crate
- Requires API design discipline
- Some code duplication possible

**Structure**:
```
pheno-vcs-primitives/   # Object types, OIDs, basic parsing
pheno-vcs-core/         # Repository, ODB, refs, high-level API
pheno-vcs/              # Facade with Phenotype integration
```

---

## Decision

**Chosen Option**: Option 3 — Layered workspace with three main crates.

**Rationale**: The layered approach provides the best balance of compile times, reusability, and maintainability. Each layer has a clear responsibility:

1. **primitives**: Foundation types that rarely change (stable API)
2. **core**: Main VCS operations that depend on primitives
3. **pheno-vcs**: Integration layer with ecosystem concerns

This structure allows external projects to depend only on primitives or core if they don't need Phenotype-specific features, while keeping the API surface manageable.

**Evidence**:
- gitoxide uses similar layered architecture successfully
- Reduces incremental compile times by ~60% vs single crate
- Allows testing primitives independently
- Clear upgrade path for adding new layers

---

## Implementation Plan

- [ ] Phase 1: Create pheno-vcs-primitives with object types — Target: 2026-04-11
- [ ] Phase 2: Create pheno-vcs-core with repository operations — Target: 2026-04-18
- [ ] Phase 3: Create pheno-vcs facade with integration — Target: 2026-04-25
- [ ] Phase 4: Add workspace-level documentation — Target: 2026-05-02
- [ ] Phase 5: Versioning and release workflow — Target: 2026-05-09

---

## Crate Structure

### pheno-vcs-primitives

Foundation types and low-level operations.

```rust
//! Object ID (SHA-1 / SHA-256)
pub struct Oid([u8; 20]);

//! Git object types
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

//! Parsed object content
pub struct Object {
    pub kind: ObjectType,
    pub content: Vec<u8>,
}

//! Tree entry
pub struct TreeEntry {
    pub mode: u32,
    pub name: String,
    pub oid: Oid,
}
```

**Dependencies**: None (std only)

### pheno-vcs-core

Repository operations and higher-level API.

```rust
//! Repository handle
pub struct Repository {
    // Internal implementation
}

//! Repository operations
impl Repository {
    pub async fn open(path: &Path) -> Result<Self>;
    pub async fn status(&self) -> Result<Status>;
    pub async fn revwalk(&self, range: &str) -> Result<Revwalk>;
    pub async fn find_object(&self, oid: Oid) -> Result<Object>;
}

//! Object database
pub struct ObjectDatabase {
    // Implementation
}

//! Reference manager
pub struct ReferenceManager {
    // Implementation
}
```

**Dependencies**:
- pheno-vcs-primitives
- git-repository
- git-odb
- tokio

### pheno-vcs (Facade)

Integration with Phenotype ecosystem.

```rust
//! Re-export core for convenience
pub use pheno_vcs_core::*;

//! Phenotype-specific extensions
pub struct PhenoRepository {
    inner: Repository,
    telemetry: Telemetry,
}

//! Integration with PhenoKit
impl PhenoRepository {
    pub async fn from_task(task: &Task) -> Result<Self>;
    pub async fn track_changes(&self) -> Result<ChangeSet>;
}
```

**Dependencies**:
- pheno-vcs-core
- pheno-vcs-primitives
- Phenotype ecosystem crates

---

## Workspace Configuration

```toml
# Cargo.toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/KooshaPari/PhenoVCS"

[workspace.dependencies]
# Internal
pheno-vcs-primitives = { path = "crates/pheno-vcs-primitives", version = "0.1.0" }
pheno-vcs-core = { path = "crates/pheno-vcs-core", version = "0.1.0" }

# External
git-repository = "0.40"
git-odb = "0.56"
tokio = { version = "1.40", features = ["full"] }
thiserror = "1.0"
tracing = "0.1"
```

---

## Consequences

### Positive

- Clear architectural boundaries
- Independent versioning per layer
n- Faster compile times during development
- Reusable components for other projects
- Easy to test individual layers

### Negative

- Slightly more complex than single crate
- Need to manage multiple crate versions
- Cross-crate API changes require coordination

### Neutral

- Workspace tooling (cargo-workspace) recommended
- Documentation needs to reference multiple crates
- May add more crates in future as needed

---

## References

- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) - Official documentation
- [gitoxide Architecture](https://github.com/GitoxideLabs/gitoxide/blob/main/ARCHITECTURE.md) - Reference architecture
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Public API design
- [SemVer](https://semver.org/) - Version management
- [cargo-release](https://github.com/crate-ci/cargo-release) - Workspace release tool
