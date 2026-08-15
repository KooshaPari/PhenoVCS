//! Worktree GC: garbage-collect stale git worktrees (paths where the dir is gone).
//!
//! Conservative: only flags worktrees whose git worktree dir is missing. The
//! actual `git worktree prune` is left to the user (or the worktree-hygiene
//! hook) so we never silently delete state.

use std::process::Command;

use super::{Checker, CheckerKind, Ctx, Report, Severity};

pub struct WorktreeGc;

impl Checker for WorktreeGc {
    fn kind(&self) -> CheckerKind {
        CheckerKind::WorktreeGc
    }

    fn run(&self, ctx: &Ctx) -> Report {
        let mut report = Report::new("worktree-gc");

        let out = Command::new("git")
            .args(["-C", &ctx.repo_root.to_string_lossy(), "worktree", "list", "--porcelain"])
            .output();
        let out = match out {
            Ok(o) if o.status.success() => o,
            Ok(o) => {
                report.push(Severity::Error, format!("git worktree list failed: {}", String::from_utf8_lossy(&o.stderr)));
                return report;
            }
            Err(e) => {
                report.push(Severity::Error, format!("git not found: {e}"));
                return report;
            }
        };
        let listing = String::from_utf8_lossy(&out.stdout);
        let mut idx = 0usize;
        let mut stale = 0usize;
        while let Some(block) = parse_block(&listing, &mut idx) {
            let path = match block.iter().find(|l| l.starts_with("worktree ")) {
                Some(s) => s.trim_start_matches("worktree ").trim().to_string(),
                None => continue,
            };
            let head = match block.iter().find(|l| l.starts_with("HEAD ")) {
                Some(s) => s.trim_start_matches("HEAD ").trim().to_string(),
                None => String::new(),
            };
            if std::path::Path::new(&path).exists() {
                report.push_path(Severity::Info, format!("active worktree HEAD={head}"), std::path::PathBuf::from(&path));
                continue;
            }
            stale += 1;
            report.push_path(Severity::Warn, format!("stale worktree pointer (dir missing), HEAD={head}"), std::path::PathBuf::from(&path));
        }
        if stale == 0 && !report.findings.iter().any(|f| f.severity == Severity::Error) {
            report.push(Severity::Info, format!("no stale worktrees at {}", ctx.repo_root.display()));
        }
        report
    }
}

fn parse_block(s: &str, idx: &mut usize) -> Option<Vec<String>> {
    let mut lines = Vec::new();
    while *idx < s.len() {
        let nl = s[*idx..].find('\n').map(|o| *idx + o).unwrap_or(s.len());
        let line = s[*idx..nl].trim_end_matches('\r').to_string();
        if line.is_empty() {
            if lines.is_empty() {
                *idx = nl + 1;
                continue;
            }
            *idx = nl + 1;
            return Some(lines);
        }
        lines.push(line);
        *idx = nl + 1;
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}
