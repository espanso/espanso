/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

use std::error::Error;
use std::fs::create_dir;
use std::path::Path;

pub fn copy_dir(source_dir: &Path, dest_dir: &Path) -> Result<(), Box<dyn Error>> {
    for entry in std::fs::read_dir(source_dir)? {
        let entry = entry?;
        let entry = entry.path();
        if entry.is_dir() {
            let name = entry.file_name().expect("Error obtaining the filename");
            let target_dir = dest_dir.join(name);
            create_dir(&target_dir)?;
            copy_dir(&entry, &target_dir)?;
        } else if entry.is_file() {
            let target_entry =
                dest_dir.join(entry.file_name().expect("Error obtaining the filename"));
            std::fs::copy(entry, target_entry)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::create_dir;
    use tempfile::TempDir;

    #[test]
    fn test_copy_dir_into() {
        let source_tmp_dir = TempDir::new().expect("Error creating temp directory");
        let dest_tmp_dir = TempDir::new().expect("Error creating temp directory");

        let source_dir = source_tmp_dir.path().join("source");
        create_dir(&source_dir);
        std::fs::write(source_dir.join("file1.txt"), "file1");
        std::fs::write(source_dir.join("file2.txt"), "file2");

        let target_dir = dest_tmp_dir.path().join("source");
        create_dir(&target_dir);

        copy_dir(&source_dir, &target_dir);

        assert!(dest_tmp_dir.path().join("source").exists());
        assert!(dest_tmp_dir.path().join("source/file1.txt").exists());
        assert!(dest_tmp_dir.path().join("source/file2.txt").exists());
    }

    #[test]
    fn test_copy_dir_into_recursive() {
        let source_tmp_dir = TempDir::new().expect("Error creating temp directory");
        let dest_tmp_dir = TempDir::new().expect("Error creating temp directory");

        let source_dir = source_tmp_dir.path().join("source");
        create_dir(&source_dir);
        std::fs::write(source_dir.join("file1.txt"), "file1");
        std::fs::write(source_dir.join("file2.txt"), "file2");
        let nested_dir = source_dir.join("nested");
        create_dir(&nested_dir);
        std::fs::write(nested_dir.join("nestedfile.txt"), "nestedfile1");

        let target_dir = dest_tmp_dir.path().join("source");
        create_dir(&target_dir);

        copy_dir(&source_dir, &target_dir);

        assert!(dest_tmp_dir.path().join("source").exists());
        assert!(dest_tmp_dir.path().join("source/file1.txt").exists());
        assert!(dest_tmp_dir.path().join("source/file2.txt").exists());

        assert!(dest_tmp_dir.path().join("source/nested").exists());
        assert!(dest_tmp_dir
            .path()
            .join("source/nested/nestedfile.txt")
            .exists());
    }
}
