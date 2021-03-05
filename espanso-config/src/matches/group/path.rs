use std::path::{Path, PathBuf};
use anyhow::Result;
use log::error;
use thiserror::Error;

// TODO: test
pub fn resolve_imports(group_path: &Path, imports: &[String]) -> Result<Vec<String>> {
  let mut paths = Vec::new();

  // Get the containing directory
  let current_dir = if group_path.is_file() {
    if let Some(parent) = group_path.parent() {
      parent
    } else {
      return Err(
        ResolveImportError::Failed(format!(
          "unable to resolve imports for match group starting from current path: {:?}",
          group_path
        ))
        .into(),
      );
    }
  } else {
    group_path
  }; 
  
  for import in imports.iter() {
    let import_path = PathBuf::from(import);

    // Absolute or relative import
    let full_path = if import_path.is_relative() {
      current_dir.join(import_path)
    } else {
      import_path
    };

    if full_path.exists() && full_path.is_file() {
      paths.push(full_path)
    } else {
      // Best effort imports
      error!("unable to resolve import at path: {:?}", full_path);
    }
  }

  let string_paths = paths
    .into_iter()
    .map(|path| path.to_string_lossy().to_string())
    .collect();
  
  Ok(string_paths)
}

#[derive(Error, Debug)]
pub enum ResolveImportError {
  #[error("resolve import failed: `{0}`")]
  Failed(String),
}