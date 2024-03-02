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

use espanso_config::config::{Config, RMLVOConfig};
use log::{info, warn};

fn generate_rmlvo_config(config: &dyn Config) -> Option<RMLVOConfig> {
  // Not needed on Windows and macOS
  if !cfg!(target_os = "linux") {
    return None;
  }

  // Not needed on X11
  if !cfg!(feature = "wayland") {
    return None;
  }

  if let Some(keyboard_config) = config.keyboard_layout() {
    Some(keyboard_config)
  } else if let Some(active_layout) = espanso_detect::get_active_layout() {
    Some(RMLVOConfig {
      layout: Some(active_layout),
      ..Default::default()
    })
  } else {
    warn!("unable to determine keyboard layout automatically, please explicitly specify it in the configuration.");
    None
  }
}

pub fn generate_detect_rmlvo(config: &dyn Config) -> Option<espanso_detect::KeyboardConfig> {
  generate_rmlvo_config(config)
    .map(|config| {
      info!("detection module will use this keyboard layout: {}", config);
      config
    })
    .map(|config| espanso_detect::KeyboardConfig {
      rules: config.rules,
      model: config.model,
      layout: config.layout,
      variant: config.variant,
      options: config.options,
    })
}

pub fn generate_inject_rmlvo(config: &dyn Config) -> Option<espanso_inject::KeyboardConfig> {
  generate_rmlvo_config(config)
    .map(|config| {
      info!("inject module will use this keyboard layout: {}", config);
      config
    })
    .map(|config| espanso_inject::KeyboardConfig {
      rules: config.rules,
      model: config.model,
      layout: config.layout,
      variant: config.variant,
      options: config.options,
    })
}
