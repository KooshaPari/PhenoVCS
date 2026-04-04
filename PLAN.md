# PLAN.md - PhenoVCS

VCS (Version Control System) primitives for the Phenotype ecosystem.

## Phases

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| 1. VCS Primitives | 2 weeks | Core VCS abstractions in pheno-vcs-core |
| 2. Worktree Mgmt | 2 weeks | Worktree creation, management, removal |
| 3. Error Handling | 1 week | thiserror custom errors |
| 4. Testing | 1 week | Unit tests, clippy checks |

## Key Deliverables

- `pheno-vcs-core` - VCS primitives
- Worktree management commands
- Proper error handling with thiserror

## Resource Estimate

- **Dev time**: 6 person-weeks
- **Dependencies**: anyhow, thiserror
- **Testing**: cargo test, cargo clippy

---

Generated: 2026-04-03
