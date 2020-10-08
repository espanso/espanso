/*
 * This file is part of espanso.
 *
 * Copyright (C) 2020 Federico Terzi
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

use log::info;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

const MODULO_APP_BUNDLE_NAME: &str = "Modulo.app";
const MODULO_APP_BUNDLE_PLIST_CONTENT: &'static str = include_str!("../../res/mac/modulo.plist");
const MODULO_APP_BUNDLE_ICON: &[u8] = include_bytes!("../../res/mac/AppIcon.icns");

pub fn generate_modulo_app_bundle(modulo_path: &str) -> Result<PathBuf, std::io::Error> {
    let modulo_pathbuf = PathBuf::from(modulo_path);
    let modulo_path: String = if !modulo_pathbuf.exists() {
        // If modulo was taken from the PATH, we need to calculate the absolute path
        // To do so, we use the `which` command
        let output = std::process::Command::new("which")
            .arg("modulo")
            .output()
            .expect("unable to call 'which' command to determine modulo's full path");
        let path = String::from_utf8_lossy(output.stdout.as_slice());
        let path = path.trim();

        info!("Detected modulo's full path: {:?}", &path);
        path.to_string()
    } else {
        modulo_path.to_owned()
    };

    let data_dir = crate::context::get_data_dir();

    let modulo_app_dir = data_dir.join(MODULO_APP_BUNDLE_NAME);

    // Remove previous bundle if present
    if modulo_app_dir.exists() {
        std::fs::remove_dir_all(&modulo_app_dir)?;
    }

    // Recreate the App bundle stub
    std::fs::create_dir(&modulo_app_dir)?;

    let contents_dir = modulo_app_dir.join("Contents");
    std::fs::create_dir(&contents_dir)?;

    let macos_dir = contents_dir.join("MacOS");
    std::fs::create_dir(&macos_dir)?;

    let resources_dir = contents_dir.join("Resources");
    std::fs::create_dir(&resources_dir)?;

    // Generate the Plist file
    let plist_content = MODULO_APP_BUNDLE_PLIST_CONTENT.replace("{{{modulo_path}}}", &modulo_path);
    let plist_file = contents_dir.join("Info.plist");
    std::fs::write(plist_file, plist_content)?;

    // Copy the icon file
    let icon_file = resources_dir.join("AppIcon.icns");
    std::fs::write(icon_file, MODULO_APP_BUNDLE_ICON)?;

    // Generate the symbolic link to the modulo binary
    let target_link = macos_dir.join("modulo");
    symlink(modulo_path, &target_link)?;

    info!("Created Modulo APP stub at: {:?}", &target_link);

    Ok(target_link)
}
