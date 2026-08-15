//! Branch lint: detect stale local branches (merged to main or upstream-gone).

use std::process::Command;

use super::{Checker, CheckerKind, Ctx, Report, Severity};

pub struct BranchLint;

impl Checker for BranchLint {
    fn kind(&self) -> CheckerKind {
        CheckerKind::BranchLint
    }

    fn run(&self, ctx: &Ctx) -> Report {
        let mut report = Report::new("branch-lint");

        // 1. branches merged into main (per `git branch --merged`)
        let merged = git(&ctx.repo_root, &["branch", "--format=%(refname:short)", "--merged", "main"]);
        for line in merged.lines() {
            let n = line.trim();
            if n.is_empty() || n == "main" || n == "master" {
                continue;
            }
            report.push(Severity::Info, format!("merged into main: {n}"));
        }

        // 2. branches whose upstream is gone (track = "[gone]")
        let all = git(&ctx.repo_root, &["for-each-ref", "--format=%(refname:short) %(upstream:track)", "refs/heads/"]);
        for line in all.lines() {
            let l = line.trim();
            if l.is_empty() {
                continue;
            }
            let (branch, track) = match l.split_once(' ') {
                Some(p) => p,
                None => continue,
            };
            if branch == "main" || branch == "master" {
                continue;
            }
            if track == "gone" {
                report.push(Severity::Warn, format!("upstream is gone: {branch}"));
            }
        }

        if report.findings.is_empty() {
            report.push(Severity::Info, format!("no branch issues at {}", ctx.repo_root.display()));
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
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => String::new(),
    }
}
