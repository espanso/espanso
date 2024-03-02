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
use espanso_modulo::form::*;

pub fn form_main(matches: &ArgMatches, _icon_paths: &IconPaths) -> i32 {
    let as_json: bool = matches.is_present("json");

    let input_file = matches
        .value_of("input_file")
        .expect("missing input, please specify the -i option");
    let data = if input_file == "-" {
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .expect("unable to obtain input from stdin");
        buffer
    } else {
        std::fs::read_to_string(input_file).expect("unable to read input file")
    };

    let mut config: config::FormConfig = if as_json {
        serde_json::from_str(&data).expect("unable to parse form configuration")
    } else {
        serde_yaml::from_str(&data).expect("unable to parse form configuration")
    };

    // Overwrite the icon
    config.icon = _icon_paths
        .form_icon
        .as_deref()
        .map(|path| path.to_string_lossy().to_string());

    let form = generator::generate(config);
    let values = show(form);

    let output = serde_json::to_string(&values).expect("unable to encode values as JSON");
    println!("{output}");

    0
}
