/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::StatusCode;

lazy_static! {
  static ref GITHUB_REGEX: Regex =
    Regex::new(r"(https://github.com/|git@github.com:)(?P<author>.*?)/(?P<name>.*?)(\.|$)")
      .unwrap();
}

#[derive(Debug, PartialEq, Eq)]
pub struct GitHubParts {
  author: String,
  name: String,
}

pub fn extract_github_url_parts(url: &str) -> Option<GitHubParts> {
  let captures = GITHUB_REGEX.captures(url)?;
  let author = captures.name("author")?;
  let name = captures.name("name")?;

  Some(GitHubParts {
    author: author.as_str().to_string(),
    name: name.as_str().to_string(),
  })
}

pub struct ResolvedRepoScheme {
  pub author: String,
  pub name: String,
  pub branch: String,
}

pub fn resolve_repo_scheme(
  parts: GitHubParts,
  force_branch: Option<&str>,
) -> Result<Option<ResolvedRepoScheme>> {
  if let Some(force_branch) = force_branch {
    if check_repo_with_branch(&parts, force_branch)? {
      return Ok(Some(ResolvedRepoScheme {
        author: parts.author,
        name: parts.name,
        branch: force_branch.to_string(),
      }));
    }
  } else {
    if check_repo_with_branch(&parts, "main")? {
      return Ok(Some(ResolvedRepoScheme {
        author: parts.author,
        name: parts.name,
        branch: "main".to_string(),
      }));
    }

    if check_repo_with_branch(&parts, "master")? {
      return Ok(Some(ResolvedRepoScheme {
        author: parts.author,
        name: parts.name,
        branch: "master".to_string(),
      }));
    }
  }

  Ok(None)
}

pub fn check_repo_with_branch(parts: &GitHubParts, branch: &str) -> Result<bool> {
  let client = reqwest::blocking::Client::new();

  let url = generate_github_download_url(parts, branch);
  let response = client.head(url).send()?;

  Ok(response.status() == StatusCode::FOUND || response.status() == StatusCode::OK)
}

fn generate_github_download_url(parts: &GitHubParts, branch: &str) -> String {
  format!(
    "https://github.com/{}/{}/archive/refs/heads/{}.zip",
    parts.author, parts.name, branch
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_extract_github_url_parts() {
    assert_eq!(
      extract_github_url_parts("https://github.com/espanso/espanso").unwrap(),
      GitHubParts {
        author: "espanso".to_string(),
        name: "espanso".to_string(),
      }
    );

    assert_eq!(
      extract_github_url_parts("https://github.com/espanso/espanso.git").unwrap(),
      GitHubParts {
        author: "espanso".to_string(),
        name: "espanso".to_string(),
      }
    );

    assert_eq!(
      extract_github_url_parts("git@github.com:espanso/espanso.git").unwrap(),
      GitHubParts {
        author: "espanso".to_string(),
        name: "espanso".to_string(),
      }
    );

    assert!(extract_github_url_parts("https://gitlab.com/espanso/espanso-test-package/").is_none());
  }
}
