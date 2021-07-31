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

pub mod win;



// #[macro_export]
// macro_rules! create_patch {
//   // TODO: add function body
//   ( $should_be_activated:expr, $( $config_field:ident ->$config_type:ty ),* ) => {
//     {
//       $(
//         if $child.$x.is_none() {
//           $child.$x = $parent.$x.clone();
//         }
//       )*

//       // Build a temporary object to verify that all fields
//       // are being used at compile time
//       $t {
//         $(
//           $x: None,
//         )*
//       };
//     }
//     {
//       use crate::patch::{ConfigProxy, Patch};

//       pub struct PatchImpl<'a> {
//         patched_config: PatchedConfig<'a>,
//       }

//       impl<'a> PatchImpl<'a> {
//         pub fn new(default: &'a dyn Config) -> Self {
//           Self {
//             patched_config: PatchedConfig::new(default),
//           }
//         }
//       }

//       impl<'a> Patch<'a> for PatchImpl<'a> {
//         fn should_be_activated(&self) -> bool {
//           $should_be_activated
//         }

//         fn patch_name(&self) -> &'static str {
//           todo!()
//         }

//         fn config(&self) -> &'a dyn Config {
//           &self.patched_config
//         }
//       }

//       pub struct PatchedConfig<'a> {
//         default: &'a dyn Config,
//       }

//       impl<'a> PatchedConfig<'a> {
//         pub fn new(default: &'a dyn Config) -> Self {
//           Self { default }
//         }
//       }

//       impl<'a> ConfigProxy<'a> for PatchedConfig<'a> {
//         fn get_default(&self) -> &'a dyn espanso_config::config::Config {
//           self.default
//         }

//         $(
//           fn $config_field(&self) -> $config_type {
//             todo!()
//           }
//         )*
//       }

//     }
//   };
// }
