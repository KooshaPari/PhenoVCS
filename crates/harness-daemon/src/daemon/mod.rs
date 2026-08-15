//! Daemon loop: run all checkers at a fixed interval.

use std::path::PathBuf;
use std::time::Duration;

use crate::reports::{run_aggregate, AggregateReport};

/// Run the daemon loop until SIGINT/SIGTERM. Returns the final aggregate
/// from the last iteration (for testing).
pub fn run_loop(repo_root: PathBuf, interval_secs: u64, dry_run: bool) -> AggregateReport {
    let interval = Duration::from_secs(interval_secs);
    let mut last = AggregateReport::default();
    loop {
        last = run_aggregate(&repo_root, dry_run);
        eprintln!("{}", last.render());
        std::thread::sleep(interval);
    }
}
