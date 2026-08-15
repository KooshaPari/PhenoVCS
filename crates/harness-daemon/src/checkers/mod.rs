//! Checker trait + shared types. Each checker is independent.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Severity {
    #[default]
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub severity: Severity,
    pub message: String,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct Report {
    pub checker: &'static str,
    pub findings: Vec<Finding>,
}

impl Report {
    pub fn new(checker: &'static str) -> Self {
        Self {
            checker,
            findings: Vec::new(),
        }
    }
    pub fn push(&mut self, severity: Severity, message: impl Into<String>) -> &mut Self {
        self.findings.push(Finding {
            severity,
            message: message.into(),
            path: None,
        });
        self
    }
    pub fn push_path(
        &mut self,
        severity: Severity,
        message: impl Into<String>,
        path: PathBuf,
    ) -> &mut Self {
        self.findings.push(Finding {
            severity,
            message: message.into(),
            path: Some(path),
        });
        self
    }
    pub fn highest(&self) -> Severity {
        self.findings
            .iter()
            .map(|f| f.severity)
            .max_by_key(|s| match s {
                Severity::Info => 0,
                Severity::Warn => 1,
                Severity::Error => 2,
            })
            .unwrap_or(Severity::Info)
    }
}

#[derive(Debug, Clone)]
pub struct Ctx {
    pub repo_root: PathBuf,
    pub dry_run: bool,
}

pub trait Checker: Sync {
    fn kind(&self) -> CheckerKind;
    fn run(&self, ctx: &Ctx) -> Report;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckerKind {
    WorktreeGc,
    BranchLint,
    HealthCheck,
}

/// Dispatch table — used by both CLI and daemon.
pub fn checker(kind: CheckerKind) -> Box<dyn Checker> {
    match kind {
        CheckerKind::WorktreeGc => Box::new(worktree_gc::WorktreeGc),
        CheckerKind::BranchLint => Box::new(branch_lint::BranchLint),
        CheckerKind::HealthCheck => Box::new(health_check::HealthCheck),
    }
}

pub mod worktree_gc;
pub mod branch_lint;
pub mod health_check;
