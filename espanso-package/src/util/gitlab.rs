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

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
  static ref GITLAB_REGEX: Regex = Regex::new(r"(https://gitlab.com/|git@gitlab.com:)(?P<author>.*?)/(?P<name>.*?)(/|\.|$)").unwrap();
}

#[derive(Debug, PartialEq)]
pub struct GitLabParts {
  author: String,
  name: String,
}

pub fn extract_gitlab_url_parts(url: &str) -> Option<GitLabParts> {
  let captures = GITLAB_REGEX.captures(url)?;
  let author = captures.name("author")?;
  let name = captures.name("name")?;
  
  Some(GitLabParts {
    author: author.as_str().to_string(),
    name: name.as_str().to_string(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_extract_gitlab_url_parts() {
    assert_eq!(extract_gitlab_url_parts("https://gitlab.com/federicoterzi/espanso-test-package/").unwrap(), GitLabParts {
      author: "federicoterzi".to_string(),
      name: "espanso-test-package".to_string(),
    });

    assert_eq!(extract_gitlab_url_parts("git@gitlab.com:federicoterzi/espanso-test-package.git").unwrap(), GitLabParts {
      author: "federicoterzi".to_string(),
      name: "espanso-test-package".to_string(),
    });

    assert_eq!(extract_gitlab_url_parts("https://gitlab.com/federicoterzi/espanso-test-package.git").unwrap(), GitLabParts {
      author: "federicoterzi".to_string(),
      name: "espanso-test-package".to_string(),
    });

    assert_eq!(extract_gitlab_url_parts("https://github.com/federicoterzi/espanso-test-package/").is_none(), true);
  }
}
