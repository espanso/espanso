use tempfile::TempDir;
use std::error::Error;
use git2::Repository;
use super::PackageResolver;

pub struct GitPackageResolver;

impl GitPackageResolver {
    pub fn new() -> GitPackageResolver {
        return GitPackageResolver{};
    }
}

impl super::PackageResolver for GitPackageResolver {
    fn clone_repo_to_temp(&self, repo_url: &str) -> Result<TempDir, Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let _repo = Repository::clone(repo_url, temp_dir.path())?;
        Ok(temp_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, NamedTempFile};

    #[test]
    fn test_clone_temp_repository() {
        let resolver = GitPackageResolver::new();
        let cloned_dir = resolver.clone_repo_to_temp("https://github.com/federico-terzi/espanso-hub-core").unwrap();
        assert!(cloned_dir.path().join("LICENSE").exists());
    }
}