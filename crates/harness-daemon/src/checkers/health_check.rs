//! Health check: worktree is on main, in sync, clean.

use std::process::Command;

use super::{Checker, CheckerKind, Ctx, Report, Severity};

pub struct HealthCheck;

impl Checker for HealthCheck {
    fn kind(&self) -> CheckerKind {
        CheckerKind::HealthCheck
    }

    fn run(&self, ctx: &Ctx) -> Report {
        let mut report = Report::new("health-check");

        // 1. current branch
        let branch = git(&ctx.repo_root, &["rev-parse", "--abbrev-ref", "HEAD"]);
        if branch.is_empty() {
            report.push(Severity::Error, "could not determine current branch".to_string());
            return report;
        }
        if branch != "main" && branch != "master" {
            report.push(Severity::Warn, format!("not on main: HEAD={branch}"));
        }

        // 2. clean working tree
        let status = git(&ctx.repo_root, &["status", "--porcelain"]);
        if !status.is_empty() {
            report.push(Severity::Warn, format!("dirty working tree ({} line(s))", status.lines().count()));
        }

        // 3. in sync with origin/main
        let ahead = git(&ctx.repo_root, &["rev-list", "--count", "origin/main..HEAD"]);
        let behind = git(&ctx.repo_root, &["rev-list", "--count", "HEAD..origin/main"]);
        if let (Ok(a), Ok(b)) = (ahead.parse::<i64>(), behind.parse::<i64>()) {
            if a > 0 {
                report.push(Severity::Warn, format!("ahead of origin/main by {a} commit(s)"));
            }
            if b > 0 {
                report.push(Severity::Warn, format!("behind origin/main by {b} commit(s)"));
            }
        }

        if report.findings.is_empty() {
            report.push(Severity::Info, format!("ok: {} healthy", ctx.repo_root.display()));
        }
        report
    }
}

fn git(repo: &std::path::Path, args: &[&str]) -> String {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo);
    for a in args {
        cmd.arg(a);
    }
    let out = cmd.output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => String::new(),
    }
}
