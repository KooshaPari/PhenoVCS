//! Domain models for worktree management
//!
//! These are pure domain concepts with zero external dependencies.
//! Following Hexagonal Architecture: Domain Layer.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use phenotype_secret::Secret;

/// Represents a git worktree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    /// Unique identifier (worktree path)
    pub id: WorktreeId,
    /// Associated branch name
    pub branch: BranchName,
    /// Path to the worktree directory
    pub path: PathBuf,
    /// Head commit SHA
    pub head: String,
    /// When the worktree was created
    pub created_at: DateTime<Utc>,
    /// Whether this is the main working directory (not a worktree)
    pub is_main: bool,
    /// Lock status
    pub locked: bool,
    /// Lock reason (if locked)
    pub lock_reason: Option<String>,
    /// Optional HTTPS git credentials (e.g. personal access token).
    ///
    /// Wrapped in [`phenotype_secret::Secret`] so accidental `format!`,
    /// `tracing::info!`, or `serde_json::to_string` calls cannot leak
    /// the value into logs, panic messages, or network payloads. The
    /// only way to read the inner value is the explicit
    /// [`phenotype_secret::Secret::expose`] accessor.
    ///
    /// `#[serde(skip, default)]` is required because
    /// `phenotype_secret::Secret` intentionally only implements
    /// `Serialize` (with a redacted payload) — there is no
    /// `Deserialize` impl, by design, to keep the audit surface
    /// explicit. Callers that need to round-trip a secret should use
    /// [`phenotype_secret::Secret::expose`] and re-wrap with
    /// [`phenotype_secret::Secret::from`].
    #[serde(skip, default)]
    pub git_credentials: Option<Secret<String>>,
}

impl Worktree {
    pub fn new(branch: BranchName, path: PathBuf, head: String) -> Self {
        Self {
            id: WorktreeId(path.clone()),
            branch,
            path,
            head,
            created_at: Utc::now(),
            is_main: false,
            locked: false,
            lock_reason: None,
            git_credentials: None,
        }
    }

    pub fn main(path: PathBuf, head: String) -> Self {
        Self {
            id: WorktreeId(path.clone()),
            branch: BranchName("main".to_string()),
            path,
            head,
            created_at: Utc::now(),
            is_main: true,
            locked: false,
            lock_reason: None,
            git_credentials: None,
        }
    }

    pub fn lock(&mut self, reason: String) {
        self.locked = true;
        self.lock_reason = Some(reason);
    }

    pub fn unlock(&mut self) {
        self.locked = false;
        self.lock_reason = None;
    }

    pub fn is_stale(&self, reference_head: &str) -> bool {
        self.head != reference_head && !self.is_main
    }
}

/// Value object for worktree ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorktreeId(pub PathBuf);

/// Value object for branch name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchName(pub String);

impl BranchName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for BranchName {
    fn default() -> Self {
        Self("main".to_string())
    }
}

/// Worktree listing with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeListing {
    pub worktrees: Vec<Worktree>,
    pub main: Worktree,
    pub total_count: usize,
}

/// Result of a worktree operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeResult {
    pub success: bool,
    pub worktree: Option<Worktree>,
    pub error: Option<String>,
    pub warnings: Vec<String>,
}

impl WorktreeResult {
    pub fn success(worktree: Worktree) -> Self {
        Self {
            success: true,
            worktree: Some(worktree),
            error: None,
            warnings: Vec::new(),
        }
    }

    pub fn failure(error: impl ToString) -> Self {
        Self {
            success: false,
            worktree: None,
            error: Some(error.to_string()),
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}

/// Policy for worktree cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPolicy {
    /// Remove worktrees with unmerged changes
    pub remove_unmerged: bool,
    /// Remove stale worktrees (diverged from main)
    pub remove_stale: bool,
    /// Remove worktrees older than duration
    pub max_age_days: Option<u32>,
    /// Remove worktrees on deleted branches
    pub remove_deleted_branches: bool,
}

impl Default for CleanupPolicy {
    fn default() -> Self {
        Self {
            remove_unmerged: false,
            remove_stale: true,
            max_age_days: Some(30),
            remove_deleted_branches: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_name() {
        let branch = BranchName::new("feature/test");
        assert_eq!(branch.as_str(), "feature/test");
    }

    #[test]
    fn test_worktree_lock() {
        let mut wt = Worktree::new(
            BranchName::new("feature/test"),
            PathBuf::from("/tmp/test"),
            "abc123".to_string(),
        );

        assert!(!wt.locked);
        wt.lock("in progress".to_string());
        assert!(wt.locked);
        assert_eq!(wt.lock_reason, Some("in progress".to_string()));

        wt.unlock();
        assert!(!wt.locked);
    }

    #[test]
    fn test_worktree_result() {
        let wt = Worktree::main(PathBuf::from("/repo"), "abc".to_string());
        let result = WorktreeResult::success(wt);

        assert!(result.success);
        assert!(result.worktree.is_some());

        let failure = WorktreeResult::failure("test error");
        assert!(!failure.success);
        assert_eq!(failure.error, Some("test error".to_string()));
    }

    /// The `git_credentials` field is wrapped in
    /// `phenotype_secret::Secret<String>` precisely so that the
    /// "innocent" leak paths (`format!`, `tracing::info!`,
    /// `serde_json::to_string`) can never emit the inner value.
    /// This test pins all three invariants at the Worktree level so
    /// a future refactor that drops the wrapper — or that swaps it
    /// for a `String` — fails CI rather than silently regressing
    /// the audit surface.
    #[test]
    fn test_worktree_git_credentials_are_redacted() {
        // A recognisable, never-real token string. If this leaks
        // into *any* of the formatted / serialised outputs below,
        // the redaction guarantee is broken.
        const LEAK: &str = "sk-live-supersecret-do-not-leak";

        let mut wt = Worktree::new(
            BranchName::new("feature/redaction-check"),
            PathBuf::from("/tmp/redaction"),
            "deadbeef".to_string(),
        );
        wt.git_credentials = Some(Secret::from(LEAK.to_string()));

        // --- Debug: the derived `Worktree` Debug delegates to
        //     `Secret`'s hand-rolled `Debug`, which writes
        //     `Secret("redacted")` — never the inner value.
        let dbg = format!("{wt:?}");
        assert!(
            !dbg.contains(LEAK),
            "Worktree::Debug leaked git_credentials inner value: {dbg}"
        );
        assert!(
            dbg.contains("Secret(\"redacted\")"),
            "Worktree::Debug should render the redaction placeholder, got: {dbg}"
        );

        // --- Display: `format!("{}", secret)` is the most common
        //     leak path (`println!("token = {token}")`,
        //     `panic!("got {secret}")`). Reach into the field and
        //     exercise the `Display` impl directly so a refactor
        //     that swaps `Secret` for a raw `String` fails this
        //     assertion.
        let creds = wt
            .git_credentials
            .as_ref()
            .expect("git_credentials must be Some for this test");
        assert_eq!(format!("{creds}"), "[REDACTED]");

        // --- Serialize: a struct-level `serde_json::to_string`
        //     must emit `"git_credentials":null` (the field is
        //     `#[serde(skip)]`) — and certainly must not contain
        //     the token anywhere else in the payload.
        let json = serde_json::to_string(&wt).expect("Worktree must serialize");
        assert!(
            !json.contains(LEAK),
            "Worktree::Serialize leaked git_credentials inner value: {json}"
        );
        assert!(
            !json.contains("git_credentials"),
            "git_credentials is #[serde(skip)] and must not appear in JSON, got: {json}"
        );

        // --- Escape hatch sanity check: the redaction wrapper is
        //     not a black hole; `Secret::expose` is the one named,
        //     auditable accessor. Lock its presence down so a
        //     future change that drops the accessor (and thus
        //     forces callers to reach for `&secret.0`) is caught.
        assert_eq!(creds.expose(), LEAK);
    }
}
