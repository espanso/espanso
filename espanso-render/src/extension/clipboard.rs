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

use crate::{Extension, ExtensionOutput, ExtensionResult, Params};
use thiserror::Error;

pub trait ClipboardProvider {
  fn get_text(&self) -> Option<String>;
}

pub struct ClipboardExtension<'a> {
  provider: &'a dyn ClipboardProvider,
}

#[allow(clippy::new_without_default)]
impl<'a> ClipboardExtension<'a> {
  pub fn new(provider: &'a dyn ClipboardProvider) -> Self {
    Self { provider }
  }
}

impl<'a> Extension for ClipboardExtension<'a> {
  fn name(&self) -> &str {
    "clipboard"
  }

  fn calculate(&self, _: &crate::Context, _: &crate::Scope, _: &Params) -> crate::ExtensionResult {
    if let Some(clipboard) = self.provider.get_text() {
      ExtensionResult::Success(ExtensionOutput::Single(clipboard))
    } else {
      ExtensionResult::Error(ClipboardExtensionError::MissingClipboard.into())
    }
  }
}

#[derive(Error, Debug)]
pub enum ClipboardExtensionError {
  #[error("clipboard provider returned error")]
  MissingClipboard,
}

#[cfg(test)]
mod tests {
  use super::*;

  struct MockClipboardProvider {
    return_none: bool,
  }

  impl super::ClipboardProvider for MockClipboardProvider {
    fn get_text(&self) -> Option<String> {
      if self.return_none {
        None
      } else {
        Some("test".to_string())
      }
    }
  }

  #[test]
  fn clipboard_works_correctly() {
    let provider = MockClipboardProvider { return_none: false };
    let extension = ClipboardExtension::new(&provider);

    assert_eq!(
      extension
        .calculate(&Default::default(), &Default::default(), &Params::new())
        .into_success()
        .unwrap(),
      ExtensionOutput::Single("test".to_string())
    );
  }

  #[test]
  fn none_clipboard_produces_error() {
    let provider = MockClipboardProvider { return_none: true };
    let extension = ClipboardExtension::new(&provider);

    assert!(matches!(
      extension.calculate(&Default::default(), &Default::default(), &Params::new()),
      ExtensionResult::Error(_)
    ));
  }
}
