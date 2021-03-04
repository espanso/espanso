use anyhow::Result;
use log::error;
use std::{
  cell::RefCell,
  convert::TryInto,
  path::{Path, PathBuf},
};
use thiserror::Error;

use super::{Match, Variable};

mod yaml;
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MatchGroup {
  imports: Vec<String>,
  pub global_vars: Vec<Variable>,
  pub matches: Vec<Match>,

  pub resolved_imports: Vec<String>,
}

impl Default for MatchGroup {
  fn default() -> Self {
    Self {
      imports: Vec::new(),
      global_vars: Vec::new(),
      matches: Vec::new(),
      resolved_imports: Vec::new(),
    }
  }
}

impl MatchGroup {
  // TODO: test
  pub fn load(group_path: &Path) -> Result<Self> {
    if let Some(extension) = group_path.extension() {
      let extension = extension.to_string_lossy().to_lowercase();

      if extension == "yml" || extension == "yaml" {
        match yaml::YAMLMatchGroup::parse_from_file(group_path) {
          Ok(yaml_group) => {
            let match_group: Result<MatchGroup, _> = yaml_group.try_into();
            match match_group {
              Ok(mut group) => {
                group.resolve_imports(group_path)?;
                Ok(group)
              }
              Err(err) => Err(MatchGroupError::ParsingError(err).into()),
            }
          }
          Err(err) => Err(MatchGroupError::ParsingError(err).into()),
        }
      } else {
        Err(MatchGroupError::InvalidFormat().into())
      }
    } else {
      Err(MatchGroupError::MissingExtension().into())
    }
  }

  // TODO: test
  fn resolve_imports(&mut self, group_path: &Path) -> Result<()> {
    let mut paths = Vec::new();

    if !group_path.exists() {
      return Err(
        MatchGroupError::ResolveImportFailed(format!(
          "unable to resolve imports for match group at path: {:?}",
          group_path
        ))
        .into(),
      );
    }

    // Get the containing directory
    let current_dir = if group_path.is_file() {
      if let Some(parent) = group_path.parent() {
        parent
      } else {
        return Err(
          MatchGroupError::ResolveImportFailed(format!(
            "unable to resolve imports for match group starting from current path: {:?}",
            group_path
          ))
          .into(),
        );
      }
    } else {
      group_path
    };

    for import in self.imports.iter() {
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
    self.resolved_imports = string_paths;

    Ok(())
  }
}

#[derive(Error, Debug)]
pub enum MatchGroupError {
  #[error("missing extension in match group file")]
  MissingExtension(),

  #[error("invalid match group format")]
  InvalidFormat(),

  #[error("parser reported an error: `{0}`")]
  ParsingError(anyhow::Error),

  #[error("resolve import failed: `{0}`")]
  ResolveImportFailed(String),
}
