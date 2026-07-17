//! Safe, non-invasive capture of a repository's dirty state.
use anyhow::{anyhow, Context, Result};
use std::{fs, path::{Path, PathBuf}, process::Command};

const DENYLIST: &[&str] = &[".env", ".env.", "secrets", ".ssh", ".airlock", "target", "node_modules", ".venv"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot { pub commit: String, pub files: Vec<String> }

/// Captures tracked modifications and untracked files in a temporary index.
/// The caller's index and working tree are never modified.
pub fn capture(repo: impl AsRef<Path>, message: &str) -> Result<Snapshot> {
    let repo = repo.as_ref();
    let root = git(repo, &["rev-parse", "--show-toplevel"])?;
    let root = PathBuf::from(root.trim());
    let status = git_bytes(&root, &["status", "--porcelain=v1", "-z", "--untracked-files=all"])?;
    let files = parse_status(&status)?;
    let denied: Vec<_> = files.iter().filter(|p| denied(p)).cloned().collect();
    if !denied.is_empty() { return Err(anyhow!("snapshot denied paths: {}", denied.join(", "))); }

    let index = tempfile::NamedTempFile::new().context("create temporary git index")?;
    let index_path = index.path().to_path_buf();
    let _ = fs::remove_file(&index_path);
    git_env(&root, &["read-tree", "HEAD"], &index_path)?;
    git_env(&root, &["add", "-A", "--", "."], &index_path)?;
    let tree = git_env(&root, &["write-tree"], &index_path)?.trim().to_owned();
    let parent = git(&root, &["rev-parse", "HEAD"])?;
    let commit = git_env_input(&root, &["commit-tree", &tree, "-p", parent.trim()], message, &index_path)?;
    let verified = git(&root, &["diff-tree", "--no-commit-id", "--name-only", "-r", commit.trim()])?;
    let verified_files: Vec<String> = verified.lines().filter(|s| !s.is_empty()).map(str::to_owned).collect();
    if verified_files != files { return Err(anyhow!("snapshot verification mismatch")); }
    Ok(Snapshot { commit: commit.trim().to_owned(), files })
}

fn denied(path: &str) -> bool {
    path.split('/').any(|part| DENYLIST.iter().any(|d| part == *d || part.starts_with(d) && d.ends_with('.')))
}

fn parse_status(bytes: &[u8]) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for record in bytes.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let text = std::str::from_utf8(record)?;
        let path = if text.len() > 3 && text.as_bytes()[1] == b' ' { &text[3..] } else { text };
        out.push(path.rsplit(" -> ").next().unwrap_or(path).to_owned());
    }
    out.sort(); out.dedup(); Ok(out)
}

fn git(repo: &Path, args: &[&str]) -> Result<String> { Ok(String::from_utf8(git_bytes(repo, args)?)?) }
fn git_bytes(repo: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new("git").args(args).current_dir(repo).output()?;
    if !out.status.success() { return Err(anyhow!("git {:?} failed: {}", args, String::from_utf8_lossy(&out.stderr))); }
    Ok(out.stdout)
}
fn git_env(repo: &Path, args: &[&str], index: &Path) -> Result<String> {
    let out = Command::new("git").args(args).current_dir(repo).env("GIT_INDEX_FILE", index).output()?;
    if !out.status.success() { return Err(anyhow!("git {:?} failed: {}", args, String::from_utf8_lossy(&out.stderr))); }
    Ok(String::from_utf8(out.stdout)?)
}
fn git_env_input(repo: &Path, args: &[&str], input: &str, index: &Path) -> Result<String> {
    let mut child = Command::new("git").args(args).current_dir(repo).env("GIT_INDEX_FILE", index).env("GIT_AUTHOR_NAME", "PhenoVCS").env("GIT_AUTHOR_EMAIL", "airlock@localhost").env("GIT_COMMITTER_NAME", "PhenoVCS").env("GIT_COMMITTER_EMAIL", "airlock@localhost").arg("-F").arg("-").stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn()?;
    use std::io::Write;
    child.stdin.take().unwrap().write_all(input.as_bytes())?;
    let out = child.wait_with_output()?;
    if !out.status.success() { return Err(anyhow!("git commit-tree failed: {}", String::from_utf8_lossy(&out.stderr))); }
    Ok(String::from_utf8(out.stdout)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn repo() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        run(dir.path(), &["init", "-q"]);
        run(dir.path(), &["config", "user.name", "Test"]);
        run(dir.path(), &["config", "user.email", "test@example.invalid"]);
        fs::write(dir.path().join("tracked.txt"), "before\n").unwrap();
        run(dir.path(), &["add", "."]);
        run(dir.path(), &["commit", "-qm", "initial"]);
        dir
    }

    fn run(dir: &Path, args: &[&str]) {
        let out = Command::new("git").args(args).current_dir(dir).output().unwrap();
        assert!(out.status.success(), "git {:?}: {}", args, String::from_utf8_lossy(&out.stderr));
    }

    #[test]
    fn captures_tracked_and_untracked_without_changing_index() {
        let dir = repo();
        fs::write(dir.path().join("tracked.txt"), "after\n").unwrap();
        fs::write(dir.path().join("new.txt"), "new\n").unwrap();
        let before = git(dir.path(), &["write-tree"]).unwrap();
        let snap = capture(dir.path(), "capture").unwrap();
        let after = git(dir.path(), &["write-tree"]).unwrap();
        assert_eq!(before, after);
        assert_eq!(snap.files, vec!["new.txt", "tracked.txt"]);
        assert_eq!(git(dir.path(), &["status", "--porcelain"]).unwrap().lines().count(), 2);
    }

    #[test]
    fn rejects_secret_paths_before_staging() {
        let dir = repo();
        fs::write(dir.path().join(".env"), "TOKEN=redacted\n").unwrap();
        let error = capture(dir.path(), "capture").unwrap_err().to_string();
        assert!(error.contains("snapshot denied paths: .env"));
        assert!(!dir.path().join(".git/index.lock").exists());
    }
}
