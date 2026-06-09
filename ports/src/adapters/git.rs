use super::vcs::{Commit, Diff, Vcs};
use async_trait::async_trait;

pub struct GitBackend;

#[async_trait]
impl Vcs for GitBackend {
    fn backend(&self) -> &str {
        "git"
    }
    async fn log(&self, _n: usize) -> Result<Vec<Commit>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }
    async fn diff(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Diff, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Diff {
            from: from.into(),
            to: to.into(),
            patch: "".into(),
        })
    }
    async fn commit(
        &self,
        _msg: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("0000000".into())
    }
}
