//! T77: PhenoVCS hexagonal port — Vcs.
//!
//! 3 adapters: GitBackend, JjBackend, SaplingBackend.
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Commit {
    pub sha: String,
    pub author: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct Diff {
    pub from: String,
    pub to: String,
    pub patch: String,
}

#[async_trait]
pub trait Vcs: Send + Sync {
    fn backend(&self) -> &str;
    async fn log(&self, n: usize) -> Result<Vec<Commit>, Box<dyn std::error::Error + Send + Sync>>;
    async fn diff(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Diff, Box<dyn std::error::Error + Send + Sync>>;
    async fn commit(&self, msg: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
