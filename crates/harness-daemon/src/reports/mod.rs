//! Aggregated report across all checkers in one pass.

use std::path::{Path, PathBuf};

use crate::checkers::{
    checker,
    CheckerKind, Report, Severity,
};

#[derive(Debug, Default)]
pub struct AggregateReport {
    pub reports: Vec<Report>,
    pub highest: Severity,
}

impl AggregateReport {
    pub fn from(reports: Vec<Report>) -> Self {
        let highest = reports
            .iter()
            .map(|r| r.highest())
            .max_by_key(|s| match s {
                Severity::Info => 0,
                Severity::Warn => 1,
                Severity::Error => 2,
            })
            .unwrap_or(Severity::Info);
        Self { reports, highest }
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "harness-daemon: highest={:?}\n",
            self.highest
        ));
        for r in &self.reports {
            out.push_str(&format!("\n[{}] {}\n", r.checker, "-".repeat(40)));
            if r.findings.is_empty() {
                out.push_str("  (no findings)\n");
            }
            for f in &r.findings {
                let sev = match f.severity {
                    Severity::Info => "INFO ",
                    Severity::Warn => "WARN ",
                    Severity::Error => "ERROR",
                };
                let path = f
                    .path
                    .as_ref()
                    .map(|p| format!(" @{}", p.display()))
                    .unwrap_or_default();
                out.push_str(&format!("  {sev} {}{path}\n", f.message));
            }
        }
        out
    }
}

/// Run every checker against `repo_root` and aggregate.
pub fn run_all(repo_root: PathBuf, dry_run: bool) -> Vec<Report> {
    [
        CheckerKind::WorktreeGc,
        CheckerKind::BranchLint,
        CheckerKind::HealthCheck,
    ]
    .iter()
    .map(|k| {
        let c = checker(*k);
        c.run(&crate::checkers::Ctx {
            repo_root: repo_root.clone(),
            dry_run,
        })
    })
    .collect()
}

pub fn run_aggregate(repo_root: &Path, dry_run: bool) -> AggregateReport {
    let reports = run_all(repo_root.to_path_buf(), dry_run);
    AggregateReport::from(reports)
}
