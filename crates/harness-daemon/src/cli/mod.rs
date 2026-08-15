//! CLI: dispatch one checker or run all via `hxd all`.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "hxd", about = "harness-daemon: cross-repo hygiene checks")]
pub struct Cli {
    /// Path to the git repo to operate on (defaults to $REPO_ROOT or cwd).
    #[arg(long)]
    pub repo: Option<PathBuf>,

    /// Run checkers but don't write/delete anything.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run all checkers in one pass.
    All,
    /// Garbage-collect stale git worktree pointers.
    WorktreeGc,
    /// Lint local branches (merged or upstream-gone).
    BranchLint,
    /// Health-check the worktree (on main, in sync, clean).
    HealthCheck,
    /// Run the daemon loop (default: every 60s).
    Run {
        /// Loop interval in seconds.
        #[arg(long, default_value_t = 60)]
        interval_secs: u64,
    },
}
