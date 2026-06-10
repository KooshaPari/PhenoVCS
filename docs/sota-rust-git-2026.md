# State of the Art: Rust Git Libraries in 2026

**Document version:** 2026-06-10
**Audience:** PhenoVCS maintainers
**Scope:** Comparative survey of the two production-grade Rust git libraries
that matter for `worktree-manager` (`crates/worktree-manager/`) and
`pheno-vcs-core` (`crates/pheno-vcs-core/`).

## TL;DR

In 2026 the Rust ecosystem has converged on **two viable, in-tree git
stacks** plus the evergreen **subprocess `git`** shim. There is no longer
a meaningful "best library" answer — there is a *trade-off* answer.

| Library          | Backing impl     | Pure Rust | Maturity          | Worktree API | Best for                                                  |
|------------------|------------------|-----------|-------------------|--------------|-----------------------------------------------------------|
| `git2` (0.x/1.x) | `libgit2` (C)    | No (FFI)  | 1.0+ since 2014   | Mature       | Full porcelain parity, fastest path to shipping          |
| `gix` (0.84.x)   | native Rust      | Yes       | 0.x, ~1.0 in 2026 | Growing      | New builds, perf-critical tools, memory-safe pipelines    |
| `git-repository` | n/a (renamed)    | n/a       | deprecated        | n/a          | Use `gix` instead — same crate, new name                 |
| `Command::new("git")` | upstream CLI | n/a    | n/a               | Full         | Thin wrappers, low risk, `worktree-manager` today        |

`git-repository` is the **former name** of the `gix` crate (it was renamed
in the gitoxide workspace in 2024). Anything that imports
`git_repository::*` is talking to `gix`; treat them as one library going
forward.

## `git2` vs `gix` (the real decision)

`git2` is the official `rust-lang/git2-rs` binding around `libgit2`. It
inherits `libgit2`'s 1.0 maturity: full porcelain, rebase, merge,
submodules, worktree add/remove, signing, credential helpers, and an
official C library vendored in `libgit2-sys` (>= 1.9.0 today). The cost
is **a C toolchain and a ~1.7 MB statically linked `libgit2.a`** in
every binary. Memory safety across the FFI boundary is on the caller.

`gix` is the **GitoxideLabs/gitoxide** workspace — 60+ small crates
plumbed into a `Repository` facade. As of `gix 0.84.0` (Jan 2026) it
covers clone, fetch, blame, status, blob/tree diff, commit-graph
traversal, worktree checkout and worktree stream. **Push, full rebase,
full merge and checkout-orchestration are still flagged "in progress"** in
`crate-status.md`. The wins are pure-Rust safety, no FFI, parallel
object decode via `parking_lot` + `prodash`, a `ThreadSafeRepository`
escape hatch, and `gix-sec`-driven config trust that prevents invoking
untrusted `core.*` executables.

## `libgit2` vs `gix` (the underlying engine)

The apples-to-apples comparison is really `libgit2` (the C library that
`git2` wraps) vs `gix` (pure Rust). `libgit2` has the deepest test
coverage in the industry — it is what `libgit2-backed` Git LFS servers,
`git2`-using editors and a long tail of CI tooling rely on. `gix` is
younger, more modular, and the only one of the two that compiles to
WASM without a C cross-toolchain. The `gix` docs explicitly call out
gaps in `strict_hash_verification` and `strict_object_creation` that
`libgit2` provides by default — relevant if `pheno-vcs-core` ever
needs to *write* objects it didn't read.

## Recommendation for `worktree-manager` (current state)

`crates/worktree-manager/src/worktree_manager/infrastructure/git_adapter.rs`
shells out to `git` for every operation (`worktree list --porcelain`,
`worktree add -b …`, `worktree remove`, `worktree lock --reason …`,
`worktree unlock`, `worktree prune`, `rev-parse`, `branch -d`,
`checkout -b`). The header comment on the file already lists
`// Depends on external libraries (git2, std)` — i.e. the adapter
intends to be replaceable but is currently a `Command::new("git")`
adapter (see `git_adapter.rs:19-32` for `run_git`).

**This is the right choice for the *current* surface area.** Reasons:

1. **Coverage.** `git worktree add/remove/lock/unlock/prune --porcelain`
   is stable upstream porcelain. Re-implementing it in `gix` would
   re-open merge conflicts on the rebase/merge workstreams that
   `gix` has not yet completed.
2. **Hexagonal port.** `WorktreeRepository` and `BranchOperations`
   traits in `crates/worktree-manager/src/worktree_manager/ports/mod.rs`
   are already the seam: drop in a `GixWorktreeAdapter` later without
   touching `WorktreeService` or the CLI.
3. **Risk posture.** `wtm` is a developer-tooling utility, not a
   user-facing git GUI — a 5 ms subprocess fork is invisible next to
   the disk I/O of writing a worktree.

**When to revisit (trigger conditions):**

- We add a hot path that walks commits or blobs in-process (e.g. status
  diff, branch listing across 10k+ refs). `gix` parallel object decode
  plus commit-graph traversal wins here by a wide margin.
- We need to run inside a sandboxed/runtime where spawning `git` is
  blocked (e.g. a sandboxed MCP tool, WASM, seccomp-strict CI).
- We need SHA-256 / reftable support before upstream `git` ships it
  to the system `git` binary on every supported platform.
- `gix` hits 1.0 and the `push`/`rebase`/`merge` plumbing is
  `crate-status.md` complete.

At that point the migration plan is mechanical: implement a
`GixWorktreeAdapter : WorktreeRepository + BranchOperations`, swap the
`adapter =` line in `main.rs:29`, run `cargo test`. The hexagonal
shell we built pays off here.

## Recommendation for `pheno-vcs-core` (future)

`pheno-vcs-core` (`crates/pheno-vcs-core/`) is the higher-level API
crate. If it ever needs to **read** objects, walk history, or compute
diffs in-process, default to `gix`. It is pure Rust, has first-class
`commitgraph` and `traverse` crates, and the `gix-status` module is
feature-complete. Reserve `git2` for the narrow cases where we need
`libgit2`'s strict hash verification on **writes**.

## Sources

- <https://github.com/GitoxideLabs/gitoxide> (11.5k stars, README,
  `crate-status.md`)
- <https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md>
  (per-crate feature status, 0.84.0)
- <https://docs.rs/gix/latest/gix/> (gix 0.84.0 API reference, feature
  flags, `libgit2` API → `gix` mapping)
- <https://github.com/rust-lang/git2-rs> (rust-lang owned, libgit2 1.9.0+
  binding, `vendored-libgit2` feature)
- <https://github.com/GitoxideLabs/gitoxide/blob/main/STABILITY.md>
  (tier 1 / tier 2 / stabilization candidate model)
