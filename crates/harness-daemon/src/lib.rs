//! harness-daemon (binary: `hxd`) — unified cross-repo hygiene daemon.
//!
//! Replaces the 7 prior launchd agents (worktree-gc, lint-branches, health-repo,
//! airlock-cleanup, snapshot, thegent-autoloop, thegent-mcp) with a single Rust
//! binary. All checkers are independent and idempotent.

pub mod checkers;
pub mod cli;
pub mod daemon;
pub mod reports;

pub use checkers::{
    branch_lint, health_check, worktree_gc, Checker, CheckerKind, Ctx, Finding, Report,
    Severity,
};
pub use cli::{Cli, Commands};
pub use reports::{run_aggregate, AggregateReport};

use std::path::PathBuf;

/// Resolve the repo root to operate on: CLI flag → REPO_ROOT env → cwd.
pub fn resolve_repo_root(cli_value: Option<PathBuf>) -> PathBuf {
    cli_value
        .or_else(|| std::env::var_os("REPO_ROOT").map(PathBuf::from))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// Run a single pass of `checker` against `repo_root`. Public so tests + manual
/// invocations can use the same code path as the daemon loop.
pub fn run_check(checker: &dyn Checker, repo_root: PathBuf, dry_run: bool) -> Report {
    let ctx = Ctx {
        repo_root,
        dry_run,
    };
    checker.run(&ctx)
}
