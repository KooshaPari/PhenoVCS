# worktree-manager — Technical Specification

## Architecture (Hexagonal)

```
┌─────────────────────────────────────────────────────┐
│                  CLI (clap derive)                    │
│        list | create | remove | lock | unlock         │
│              cleanup | prune | branch                 │
├─────────────────────────────────────────────────────┤
│              Application Layer                        │
│            WorktreeService (use cases)                │
├────────────┬────────────────────────────────────────┤
│   Ports    │          Domain (pure)                  │
│  (traits)  │   Worktree, BranchName, CleanupPolicy   │
│            │   errors (PoLA pattern)                 │
│            │   ZERO external dependencies            │
├────────────┴────────────────────────────────────────┤
│              Infrastructure (Adapters)                │
│   GitWorktreeAdapter   │  SimpleFilesystemAdapter    │
│   GitBranchAdapter     │  GitCleanupAdapter          │
│   (subprocess git)     │  (std::fs)                  │
└─────────────────────────────────────────────────────┘
```

## Components

| Component | Location | Responsibility |
|-----------|----------|---------------|
| Domain | `src/worktree_manager/domain/` | Pure business logic, zero deps |
| Models | `src/worktree_manager/domain/models.rs` | Worktree, BranchName, CleanupPolicy |
| Errors | `src/worktree_manager/domain/errors.rs` | Descriptive error types |
| Ports | `src/worktree_manager/ports/mod.rs` | Repository, BranchOperations traits |
| Service | `src/worktree_manager/application/service.rs` | Use case orchestration |
| Git Adapter | `src/worktree_manager/infrastructure/git_adapter.rs` | git subprocess adapter |
| FS Adapter | `src/worktree_manager/infrastructure/filesystem_adapter.rs` | std::fs adapter |
| CLI | `src/worktree_manager/cli/mod.rs` | Command implementations |

## Domain Model

```rust
struct Worktree {
    path: PathBuf,
    branch: BranchName,
    head: CommitHash,
    locked: bool,
    prune_expiry: Option<DateTime>,
}

struct BranchName(String); // validated: no spaces, no double-dots

struct CleanupPolicy {
    remove_stale: bool,
    max_age: Option<Duration>,
    dry_run: bool,
}

struct WorktreeListing {
    worktrees: Vec<Worktree>,
    total_count: usize,
    locked_count: usize,
}
```

## Port Interfaces

```rust
trait WorktreeRepository {
    fn list(&self, repo_path: &Path) -> Result<WorktreeListing>;
    fn create(&self, repo_path: &Path, branch: &BranchName, path: &Path, start_point: Option<&str>) -> Result<Worktree>;
    fn remove(&self, repo_path: &Path, path: &Path, force: bool) -> Result<()>;
    fn lock(&self, repo_path: &Path, path: &Path, reason: &str) -> Result<()>;
    fn unlock(&self, repo_path: &Path, path: &Path) -> Result<()>;
    fn prune(&self, repo_path: &Path) -> Result<()>;
}

trait BranchOperations {
    fn current(&self, repo_path: &Path) -> Result<BranchName>;
    fn exists(&self, repo_path: &Path, branch: &BranchName) -> Result<bool>;
}

trait CleanupOperations {
    fn cleanup(&self, repo_path: &Path, policy: &CleanupPolicy) -> Result<CleanupResult>;
}
```

## CLI Commands

| Command | Flags | Purpose |
|---------|-------|---------|
| `list` | | List all worktrees |
| `create` | `--branch`, `--path` | Create worktree with branch |
| `remove` | `<path>` | Remove worktree |
| `lock` | `<path> --reason` | Lock worktree |
| `unlock` | `<path>` | Unlock worktree |
| `cleanup` | `--remove-stale --dry-run` | Clean stale worktrees |
| `prune` | | Prune stale references |
| `branch` | | Show current branch |

## Design Principles

| Principle | Implementation |
|-----------|---------------|
| SOLID | Dependency Inversion via trait ports |
| DRY | Shared port interfaces |
| KISS | Simple, focused interfaces |
| GRASP | Application Service pattern |
| PoLA | Descriptive error types with context |

## Performance Targets

| Metric | Target |
|--------|--------|
| List worktrees | <200ms |
| Create worktree | <2s |
| Remove worktree | <1s |
| Cleanup scan | <500ms |
| Lock/unlock | <100ms |
| Binary size | <5MB |

## Verified lifecycle requirements

The quality-loop coverage denominator is the nine externally observable
requirements below. A requirement is covered only when a real-git integration
test executes it through `WorktreeService` and asserts the resulting repository
state.

| ID | Requirement |
|---|---|
| QL-FR-001 | Listing identifies the canonical worktree and linked worktrees with short branch names and commit IDs. |
| QL-FR-002 | Creating a worktree succeeds when the requested branch does not exist. |
| QL-FR-003 | An explicit start point becomes the linked worktree's checked-out commit. |
| QL-FR-004 | Existing and malformed branch names are rejected without creating a directory. |
| QL-FR-005 | Locking a linked worktree records its lock and reason. |
| QL-FR-006 | Unlocking clears both lock state and reason. |
| QL-FR-007 | Removal is executed from the canonical repository and removes the linked directory. |
| QL-FR-008 | Current-branch reporting returns the short branch name. |
| QL-FR-009 | Invalid repository paths return a typed git error. |

## Dependencies

| Crate | Purpose |
|-------|---------|
| `anyhow` | Error handling |
| `thiserror` | Error derive macros |
| `serde` / `serde_json` | Serialization |
| `chrono` | Date/time |
| `clap` | CLI parsing (derive) |
| `tracing` | Structured logging |
| `tempfile` | Test fixtures |
| `assert_cmd` | CLI testing |
