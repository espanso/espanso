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

use chrono::{DateTime, Duration, Local};

use crate::{Extension, ExtensionOutput, ExtensionResult, Number, Params, Value};

pub struct DateExtension {
  fixed_date: Option<DateTime<Local>>,
}

#[allow(clippy::new_without_default)]
impl DateExtension {
  pub fn new() -> Self {
    Self { fixed_date: None }
  }
}

impl Extension for DateExtension {
  fn name(&self) -> &str {
    "date"
  }

  fn calculate(
    &self,
    _: &crate::Context,
    _: &crate::Scope,
    params: &Params,
  ) -> crate::ExtensionResult {
    let mut now = self.get_date();

    // Compute the given offset
    let offset = params.get("offset");
    if let Some(Value::Number(Number::Integer(offset))) = offset {
      let offset = Duration::seconds(*offset);
      now = now + offset;
    }

    let format = params.get("format");

    let date = if let Some(Value::String(format)) = format {
      now.format(format).to_string()
    } else {
      now.to_rfc2822()
    };

    ExtensionResult::Success(ExtensionOutput::Single(date))
  }
}

impl DateExtension {
  fn get_date(&self) -> DateTime<Local> {
    if let Some(fixed_date) = self.fixed_date {
      fixed_date
    } else {
      Local::now()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::offset::TimeZone;
  use std::iter::FromIterator;

  #[test]
  fn date_formatted_correctly() {
    let mut extension = DateExtension::new();
    extension.fixed_date = Some(Local.ymd(2014, 7, 8).and_hms(9, 10, 11));

    let param = Params::from_iter(
      vec![("format".to_string(), Value::String("%H:%M:%S".to_string()))].into_iter(),
    );
    assert_eq!(
      extension
        .calculate(&Default::default(), &Default::default(), &param)
        .into_success()
        .unwrap(),
      ExtensionOutput::Single("09:10:11".to_string())
    );
  }

  #[test]
  fn offset_works_correctly() {
    let mut extension = DateExtension::new();
    extension.fixed_date = Some(Local.ymd(2014, 7, 8).and_hms(9, 10, 11));

    let param = Params::from_iter(
      vec![
        ("format".to_string(), Value::String("%H:%M:%S".to_string())),
        ("offset".to_string(), Value::Number(Number::Integer(3600))),
      ]
      .into_iter(),
    );
    assert_eq!(
      extension
        .calculate(&Default::default(), &Default::default(), &param)
        .into_success()
        .unwrap(),
      ExtensionOutput::Single("10:10:11".to_string())
    );
  }
}
