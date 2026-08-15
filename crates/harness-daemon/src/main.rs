//! Binary entry: `hxd` (harness-daemon).
//!
//! Usage:
//!     hxd --repo /path/to/civis all
//!     hxd --repo /path/to/civis worktree-gc
//!     hxd --repo /path/to/civis run --interval-secs 60

use clap::Parser;

use harness_daemon::cli::{Cli, Commands};
use harness_daemon::daemon::run_loop;
use harness_daemon::reports::{run_aggregate, run_all};
use harness_daemon::resolve_repo_root;

fn main() {
    let cli = Cli::parse();
    let repo_root = resolve_repo_root(cli.repo);

    match cli.command {
        Commands::All => {
            let agg = run_aggregate(&repo_root, cli.dry_run);
            print!("{}", agg.render());
            std::process::exit(exit_code(agg.highest));
        }
        Commands::WorktreeGc | Commands::BranchLint | Commands::HealthCheck => {
            let reports = run_all(repo_root, cli.dry_run);
            for r in &reports {
                println!("\n[{}] {}", r.checker, "-".repeat(40));
                for f in &r.findings {
                    println!("  {:?}", f);
                }
            }
            std::process::exit(exit_code_for_findings(&reports));
        }
        Commands::Run { interval_secs } => {
            run_loop(repo_root, interval_secs, cli.dry_run);
        }
    }
}

fn exit_code(sev: harness_daemon::checkers::Severity) -> i32 {
    use harness_daemon::checkers::Severity::*;
    match sev {
        Error => 2,
        Warn => 1,
        Info => 0,
        _ => 0,
    }
}

fn exit_code_for_findings(reports: &[harness_daemon::Report]) -> i32 {
    let highest = reports
        .iter()
        .map(|r| r.highest())
        .max_by_key(|s| match s {
            harness_daemon::checkers::Severity::Error => 2,
            harness_daemon::checkers::Severity::Warn => 1,
            harness_daemon::checkers::Severity::Info => 0,
            _ => 0,
        })
        .unwrap_or(harness_daemon::checkers::Severity::Info);
    exit_code(highest)
}
