//! Git adapter using subprocess commands
//!
//! Following Hexagonal Architecture: Infrastructure (Driven Adapter).

use crate::domain::{
    BranchName, DomainResult, Worktree, WorktreeError, WorktreeId, WorktreeListing,
};
use crate::ports::{BranchOperations, WorktreeRepository};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git worktree adapter using git commands
#[derive(Clone)]
pub struct GitWorktreeAdapter;

impl GitWorktreeAdapter {
    pub fn new() -> Self {
        Self
    }

    fn run_git(&self, repo_path: &Path, args: &[&str]) -> Result<String, WorktreeError> {
        let output = Command::new("git")
            .args(["-C", repo_path.to_str().unwrap_or(".")])
            .args(args)
            .output()
            .map_err(|e| WorktreeError::GitError(format!("git command failed: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(WorktreeError::GitError(format!("git failed: {}", stderr)))
        }
    }

    fn ref_exists(&self, repo_path: &Path, reference: &str) -> DomainResult<bool> {
        let output = Command::new("git")
            .args(["-C", repo_path.to_str().unwrap_or(".")])
            .args(["show-ref", "--verify", "--quiet", reference])
            .output()
            .map_err(|e| WorktreeError::GitError(format!("git command failed: {}", e)))?;

        match output.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => Err(WorktreeError::GitError(format!(
                "git show-ref failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))),
        }
    }

    fn same_path(left: &Path, right: &Path) -> bool {
        match (left.canonicalize(), right.canonicalize()) {
            (Ok(left), Ok(right)) => left == right,
            _ => left == right,
        }
    }

    /// Route a parsed worktree into the main slot or the worktrees list.
    ///
    /// Extracted from `WorktreeRepository::list` to remove the duplicated
    /// "main vs linked" dispatch that previously appeared twice in the parser.
    fn push_record(wt: Worktree, main: &mut Option<Worktree>, worktrees: &mut Vec<Worktree>) {
        if wt.is_main {
            *main = Some(wt);
        } else {
            worktrees.push(wt);
        }
    }
}

impl Default for GitWorktreeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl WorktreeRepository for GitWorktreeAdapter {
    fn list(&self, repo_path: &Path) -> DomainResult<WorktreeListing> {
        let output = self.run_git(repo_path, &["worktree", "list", "--porcelain"])?;

        let mut worktrees: Vec<Worktree> = Vec::new();
        let mut main: Option<Worktree> = None;
        let mut current: Option<Worktree> = None;

        for line in output.lines() {
            if line.starts_with("worktree ") {
                if let Some(wt) = current.take() {
                    Self::push_record(wt, &mut main, &mut worktrees);
                }

                let path = line.trim_start_matches("worktree ");
                let is_main = Self::same_path(Path::new(path), repo_path);
                current = Some(Worktree {
                    id: WorktreeId(PathBuf::from(path)),
                    branch: BranchName::default(),
                    path: PathBuf::from(path),
                    head: String::new(),
                    created_at: chrono::Utc::now(),
                    is_main,
                    locked: false,
                    lock_reason: None,
                });
            } else if let Some(ref mut wt) = current {
                if line.starts_with("branch ") {
                    let branch = line
                        .trim_start_matches("branch ")
                        .trim()
                        .strip_prefix("refs/heads/")
                        .unwrap_or_else(|| line.trim_start_matches("branch ").trim());
                    wt.branch = BranchName::new(branch);
                } else if line.starts_with("HEAD ") {
                    wt.head = line.trim_start_matches("HEAD ").to_string();
                } else if line == "locked" || line.starts_with("locked ") {
                    wt.locked = true;
                    wt.lock_reason = line
                        .strip_prefix("locked ")
                        .filter(|reason| !reason.is_empty())
                        .map(str::to_string);
                }
            }
        }

        if let Some(wt) = current {
            Self::push_record(wt, &mut main, &mut worktrees);
        }

        let main = main.unwrap_or_else(|| Worktree::main(repo_path.to_path_buf(), String::new()));
        let total_count = worktrees.len();

        Ok(WorktreeListing {
            worktrees: worktrees.clone(),
            main,
            total_count,
        })
    }

    fn create(
        &self,
        repo_path: &Path,
        branch: &BranchName,
        worktree_path: &Path,
        start_point: Option<&str>,
    ) -> DomainResult<Worktree> {
        let path_str = worktree_path
            .to_str()
            .ok_or_else(|| WorktreeError::InvalidPath("Invalid worktree path".to_string()))?;

        self.run_git(
            repo_path,
            &["check-ref-format", "--branch", branch.as_str()],
        )
        .map_err(|_| WorktreeError::InvalidBranchName(branch.as_str().to_string()))?;

        let from_ref = start_point.unwrap_or("HEAD");
        self.run_git(
            repo_path,
            &["worktree", "add", "-b", branch.as_str(), path_str, from_ref],
        )?;
        let head = self.run_git(worktree_path, &["rev-parse", "HEAD"])?;

        Ok(Worktree::new(
            branch.clone(),
            worktree_path.to_path_buf(),
            head.trim().to_string(),
        ))
    }

    fn remove(&self, repo_path: &Path, worktree_path: &Path, force: bool) -> DomainResult<()> {
        let path_str = worktree_path
            .to_str()
            .ok_or_else(|| WorktreeError::InvalidPath("Invalid worktree path".to_string()))?;

        let mut args = vec!["worktree", "remove", path_str];
        if force {
            args.push("--force");
        }

        self.run_git(repo_path, &args)?;
        Ok(())
    }

    fn lock(&self, repo_path: &Path, worktree_path: &Path, reason: &str) -> DomainResult<()> {
        let path_str = worktree_path
            .to_str()
            .ok_or_else(|| WorktreeError::InvalidPath("Invalid worktree path".to_string()))?;

        self.run_git(
            repo_path,
            &["worktree", "lock", path_str, "--reason", reason],
        )?;
        Ok(())
    }

    fn unlock(&self, repo_path: &Path, worktree_path: &Path) -> DomainResult<()> {
        let path_str = worktree_path
            .to_str()
            .ok_or_else(|| WorktreeError::InvalidPath("Invalid worktree path".to_string()))?;

        self.run_git(repo_path, &["worktree", "unlock", path_str])?;
        Ok(())
    }

    fn prune(&self, repo_path: &Path) -> DomainResult<()> {
        self.run_git(repo_path, &["worktree", "prune"])?;
        Ok(())
    }
}

impl BranchOperations for GitWorktreeAdapter {
    fn exists(&self, repo_path: &Path, branch: &BranchName) -> DomainResult<bool> {
        let local = format!("refs/heads/{}", branch.as_str());
        let remote = format!("refs/remotes/origin/{}", branch.as_str());
        Ok(self.ref_exists(repo_path, &local)? || self.ref_exists(repo_path, &remote)?)
    }

    fn create(
        &self,
        repo_path: &Path,
        branch: &BranchName,
        from_ref: Option<&str>,
    ) -> DomainResult<()> {
        let mut args = vec!["checkout", "-b", branch.as_str()];
        if let Some(ref_name) = from_ref {
            args.push(ref_name);
        }

        self.run_git(repo_path, &args)?;
        Ok(())
    }

    fn delete(&self, repo_path: &Path, branch: &BranchName) -> DomainResult<()> {
        self.run_git(repo_path, &["branch", "-d", branch.as_str()])?;
        Ok(())
    }

    fn current(&self, repo_path: &Path) -> DomainResult<BranchName> {
        let output = self.run_git(repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        Ok(BranchName::new(output.trim()))
    }
}
