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

#[macro_export]
macro_rules! generate_patchable_config {
  ( $struct_name:ident, $( $config_field:ident ->$config_type:ty ),* ) => {
      use std::sync::Arc;
      use espanso_config::config::{AppProperties, Config};

      pub struct $struct_name {
        base: Arc<dyn Config>,
        patch: Patches,
      }

      impl $struct_name {
        #[allow(dead_code)]
        pub fn patch(base: Arc<dyn Config>, patch: Patches) -> Self {
          Self {
            base,
            patch,
          }
        }
      }

      #[derive(Default)]
      pub struct Patches {
        $(
          pub $config_field: Option<$config_type>,
        )*
      }

      impl Config for $struct_name {
        $(
          fn $config_field(&self) -> $config_type {
            if let Some(patched) = self.patch.$config_field.clone() {
              return patched;
            }

            self.base.$config_field()
          }
        )*

        fn id(&self) -> i32 {
          self.base.id()
        }

        fn label(&self) -> &str {
          self.base.label()
        }

        fn match_paths(&self) -> &[String] {
          self.base.match_paths()
        }

        fn is_match<'b>(&self, app: &AppProperties<'b>) -> bool {
          self.base.is_match(app)
        }
      }
  };
}