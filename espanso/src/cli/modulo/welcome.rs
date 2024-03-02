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

use crate::icon::IconPaths;
use clap::ArgMatches;
use espanso_modulo::welcome::*;
use espanso_path::Paths;

pub fn welcome_main(matches: &ArgMatches, _: &Paths, icon_paths: &IconPaths) -> i32 {
    let dont_show_again_handler = Box::new(move |_dont_show: bool| {
        //preferences.set_should_display_welcome(!dont_show);
        // TODO: this should probably be deleted if not used?
    });

    let is_already_running = matches.is_present("already-running");

    espanso_modulo::welcome::show(WelcomeOptions {
        window_icon_path: icon_paths
            .wizard_icon
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
        tray_image_path: icon_paths
            .tray_explain_image
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
        is_already_running,
        handlers: WelcomeHandlers {
            dont_show_again_changed: Some(dont_show_again_handler),
        },
    });

    0
}
