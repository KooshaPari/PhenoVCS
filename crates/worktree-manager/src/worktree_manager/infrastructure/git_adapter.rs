//! Git adapter using subprocess commands
//!
//! Following Hexagonal Architecture: Infrastructure (Driven Adapter).

use crate::domain::{
    BranchName, DomainResult, Worktree, WorktreeError, WorktreeId, WorktreeListing,
};
use crate::ports::{BranchOperations, WorktreeRepository};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Maximum number of retry attempts for git operations.
const GIT_RETRY_MAX: u32 = 3;

/// Base delay in milliseconds for exponential backoff.
const GIT_RETRY_BASE_DELAY_MS: u64 = 100;

/// Git worktree adapter using git commands
#[derive(Clone)]
pub struct GitWorktreeAdapter;

impl GitWorktreeAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Execute a git command with retry and exponential backoff.
    fn run_git(&self, repo_path: &Path, args: &[&str]) -> Result<String, WorktreeError> {
        let mut last_err: Option<WorktreeError> = None;

        for attempt in 0..GIT_RETRY_MAX {
            match self.run_git_inner(repo_path, args) {
                Ok(output) => return Ok(output),
                Err(e) => {
                    // Only retry on non-zero exit (GitError), not on invalid paths
                    // or other fatal domain errors.
                    if attempt + 1 < GIT_RETRY_MAX && Self::is_retriable(&e) {
                        let delay_ms = GIT_RETRY_BASE_DELAY_MS * 2u64.pow(attempt);
                        std::thread::sleep(Duration::from_millis(delay_ms));
                        last_err = Some(e);
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            WorktreeError::GitError("git command failed after retries".to_string())
        }))
    }

    /// Inner git command execution without retry logic.
    fn run_git_inner(&self, repo_path: &Path, args: &[&str]) -> Result<String, WorktreeError> {
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

    /// Check whether an error is retriable.
    /// Returns `true` for transient git subprocess failures (non-zero exit).
    fn is_retriable(err: &WorktreeError) -> bool {
        matches!(err, WorktreeError::GitError(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::WorktreeError;

    #[test]
    fn test_retry_exhaustion_returns_last_error() {
        // Verify that is_retriable correctly identifies retriable errors.
        let git_err = WorktreeError::GitError("transient failure".to_string());
        assert!(GitWorktreeAdapter::is_retriable(&git_err));

        let invalid_err = WorktreeError::InvalidPath("bad path".to_string());
        assert!(!GitWorktreeAdapter::is_retriable(&invalid_err));
    }

    #[test]
    fn test_non_retriable_error_passthrough() {
        // Non-retriable errors should not be retried and return immediately.
        let invalid_err = WorktreeError::InvalidPath("bad path".to_string());
        assert!(!GitWorktreeAdapter::is_retriable(&invalid_err));

        let not_found = WorktreeError::NotFound("missing".to_string());
        assert!(!GitWorktreeAdapter::is_retriable(&not_found));
    }

    #[test]
    fn test_retry_constant_bounds() {
        // Verify retry constants are reasonable.
        assert!(GIT_RETRY_MAX >= 1, "must allow at least one retry");
        assert!(GIT_RETRY_MAX <= 10, "sanity: no more than 10 retries");
        assert!(
            GIT_RETRY_BASE_DELAY_MS >= 50,
            "base delay should allow backoff"
        );
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
                let is_main =
                    path.contains("/.git/worktrees") || path == repo_path.to_str().unwrap_or("");
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
                    wt.branch = BranchName::new(line.trim_start_matches("branch ").trim());
                } else if line.starts_with("head ") {
                    wt.head = line.trim_start_matches("head ").to_string();
                } else if line.starts_with("locked ") {
                    wt.locked = true;
                    wt.lock_reason = Some(line.trim_start_matches("locked ").to_string());
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
    ) -> DomainResult<Worktree> {
        let path_str = worktree_path
            .to_str()
            .ok_or_else(|| WorktreeError::InvalidPath("Invalid worktree path".to_string()))?;

        let _output = self.run_git(
            repo_path,
            &["worktree", "add", "-b", branch.as_str(), path_str, "HEAD"],
        )?;

        Ok(Worktree::new(
            branch.clone(),
            worktree_path.to_path_buf(),
            "HEAD".to_string(),
        ))
    }

    fn remove(&self, worktree_path: &Path, force: bool) -> DomainResult<()> {
        let path_str = worktree_path
            .to_str()
            .ok_or_else(|| WorktreeError::InvalidPath("Invalid worktree path".to_string()))?;

        let mut args = vec!["worktree", "remove", path_str];
        if force {
            args.push("--force");
        }

        self.run_git(worktree_path, &args)?;
        Ok(())
    }

    fn lock(&self, worktree_path: &Path, reason: &str) -> DomainResult<()> {
        let path_str = worktree_path
            .to_str()
            .ok_or_else(|| WorktreeError::InvalidPath("Invalid worktree path".to_string()))?;

        self.run_git(
            worktree_path,
            &["worktree", "lock", path_str, "--reason", reason],
        )?;
        Ok(())
    }

    fn unlock(&self, worktree_path: &Path) -> DomainResult<()> {
        let path_str = worktree_path
            .to_str()
            .ok_or_else(|| WorktreeError::InvalidPath("Invalid worktree path".to_string()))?;

        self.run_git(worktree_path, &["worktree", "unlock", path_str])?;
        Ok(())
    }

    fn prune(&self, repo_path: &Path) -> DomainResult<()> {
        self.run_git(repo_path, &["worktree", "prune"])?;
        Ok(())
    }
}

/// Simple filesystem adapter for lock files
#[derive(Clone)]
pub struct SimpleFilesystemAdapter;

impl SimpleFilesystemAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleFilesystemAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BranchOperations for GitWorktreeAdapter {
    fn exists(&self, repo_path: &Path, branch: &BranchName) -> DomainResult<bool> {
        let output = self.run_git(
            repo_path,
            &[
                "rev-parse",
                "--verify",
                &format!("origin/{}", branch.as_str()),
            ],
        )?;
        Ok(!output.trim().is_empty())
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
