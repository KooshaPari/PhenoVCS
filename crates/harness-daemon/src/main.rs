//! Binary entry: `hxd` (harness-daemon).
//!
//! Usage:
//!     hxd --repo /path/to/civis all
//!     hxd --repo /path/to/civis worktree-gc
//!     hxd --repo /path/to/civis run --interval-secs 60

use clap::Parser;

use harness_daemon::cli::{Cli, Commands};
use harness_daemon::checkers::Checker;
use harness_daemon::checkers::Ctx;
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
        Commands::WorktreeGc => {
            let r = harness_daemon::checkers::worktree_gc::WorktreeGc.run(&harness_daemon::checkers::Ctx { repo_root: repo_root.clone(), dry_run: cli.dry_run });
            print_report(&r);
            std::process::exit(exit_code(r.highest()));
        }
        Commands::BranchLint => {
            let r = harness_daemon::checkers::branch_lint::BranchLint.run(&harness_daemon::checkers::Ctx { repo_root: repo_root.clone(), dry_run: cli.dry_run });
            print_report(&r);
            std::process::exit(exit_code(r.highest()));
        }
        Commands::HealthCheck => {
            let r = harness_daemon::checkers::health_check::HealthCheck.run(&harness_daemon::checkers::Ctx { repo_root: repo_root.clone(), dry_run: cli.dry_run });
            print_report(&r);
            std::process::exit(exit_code(r.highest()));
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

fn print_report(r: &harness_daemon::Report) {
    println!("\n[{}] {}", r.checker, "-".repeat(40));
    if r.findings.is_empty() {
        println!("  (no findings)");
    }
    for f in &r.findings {
        let sev = match f.severity {
            harness_daemon::checkers::Severity::Info => "INFO ",
            harness_daemon::checkers::Severity::Warn => "WARN ",
            harness_daemon::checkers::Severity::Error => "ERROR",
            _ => "?    ",
        };
        let path = f.path.as_ref().map(|p| format!(" @{}", p.display())).unwrap_or_default();
        println!("  {sev} {}{path}", f.message);
    }
}
