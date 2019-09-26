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

use std::path::{PathBuf, Path};
use crate::package::{PackageIndex, UpdateResult, Package, InstallResult};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use chrono::{NaiveDateTime, Timelike};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::package::UpdateResult::{NotOutdated, Updated};
use crate::package::InstallResult::{NotFoundInIndex, AlreadyInstalled};
use std::fs;
use tempfile::TempDir;
use git2::Repository;

const DEFAULT_PACKAGE_INDEX_FILE : &str = "package_index.json";

pub struct DefaultPackageManager {
    package_dir: PathBuf,
    data_dir: PathBuf,

    local_index: Option<PackageIndex>,
}

impl DefaultPackageManager {
    pub fn new(package_dir: PathBuf, data_dir: PathBuf) -> DefaultPackageManager {
        let local_index = Self::load_local_index(&data_dir);

        DefaultPackageManager{
            package_dir,
            data_dir,
            local_index
        }
    }

    pub fn new_default() -> DefaultPackageManager {
        DefaultPackageManager::new(
            crate::config::ConfigSet::get_default_packages_dir(),
            crate::context::get_data_dir()
        )
    }

    fn get_package_index_path(data_dir: &Path) -> PathBuf {
        data_dir.join(DEFAULT_PACKAGE_INDEX_FILE)
    }

    fn load_local_index(data_dir: &Path) -> Option<super::PackageIndex> {
        let local_index_file = File::open(Self::get_package_index_path(data_dir));
        if let Ok(local_index_file) = local_index_file {
            let reader = BufReader::new(local_index_file);
            let local_index = serde_json::from_reader(reader);

            if let Ok(local_index) = local_index {
                return local_index
            }
        }

        None
    }

    fn request_index() -> Result<super::PackageIndex, Box<dyn Error>> {
        let mut client = reqwest::Client::new();
        let request = client.get("https://hub.espanso.org/json/")
            .header("User-Agent", format!("espanso/{}", crate::VERSION));

        let mut res = request.send()?;
        let body = res.text()?;
        let index : PackageIndex = serde_json::from_str(&body)?;

        Ok(index)
    }

    fn clone_repo_to_temp(repo_url: &str) -> Result<TempDir, Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let repo = Repository::clone(repo_url, temp_dir.path())?;
        Ok(temp_dir)
    }


    fn local_index_timestamp(&self) -> u64 {
        if let Some(local_index) = &self.local_index {
            return local_index.last_update
        }

        return 0;
    }

    fn list_local_packages(&self) -> Vec<String> {
        let dir = fs::read_dir(&self.package_dir);
        let mut output = Vec::new();
        if let Ok(dir) = dir {
            for entry in dir {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name();
                        if let Some(name) = name {
                            output.push(name.to_str().unwrap().to_owned())
                        }
                    }
                }
            }
        }

        output
    }
}

impl super::PackageManager for DefaultPackageManager {
    fn is_index_outdated(&self) -> bool {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
        let current_timestamp = current_time.as_secs();

        let local_index_timestamp = self.local_index_timestamp();

        // Local index is outdated if older than a day
        local_index_timestamp + 60*60*24 < current_timestamp
    }

    fn update_index(&mut self, force: bool) -> Result<UpdateResult, Box<dyn Error>> {
        if force || self.is_index_outdated() {
            let updated_index = DefaultPackageManager::request_index()?;
            self.local_index = Some(updated_index);

            Ok(Updated)
        }else{
            Ok(NotOutdated)
        }
    }

    fn get_package(&self, name: &str) -> Option<Package> {
        if let Some(local_index) = &self.local_index {
            let result = local_index.packages.iter().find(|package| {
                package.name == name
            });
            if let Some(package) = result {
                return Some(package.clone())
            }
        }

        None
    }

    fn install_package(&self, name: &str) -> Result<InstallResult, Box<dyn Error>> {
        let package = self.get_package(name);
        match package {
            Some(package) => {
                self.install_package_from_repo(name, &package.repo)
            },
            None => {
                Ok(NotFoundInIndex)
            },
        }
    }

    fn install_package_from_repo(&self, name: &str, repo_url: &str) -> Result<InstallResult, Box<dyn Error>> {
        // Check if package is already installed
        let packages = self.list_local_packages();
        if packages.iter().any(|p| p == name) {  // Package already installed
            return Ok(AlreadyInstalled);
        }

        let temp_dir = Self::clone_repo_to_temp(repo_url)?;

        let temp_package_dir = temp_dir.path().join(name);
        if !temp_package_dir.exists() {
            return Ok(InstallResult::NotFoundInRepo);
        }

        crate::utils::copy_dir_into(&temp_package_dir, &self.package_dir)?;

        Ok(InstallResult::Installed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::Path;
    use crate::package::PackageManager;
    use std::fs::create_dir;
    use crate::package::InstallResult::{Installed, NotFoundInRepo};

    const OUTDATED_INDEX_CONTENT : &str = include_str!("../res/test/outdated_index.json");
    const INDEX_CONTENT_WITHOUT_UPDATE: &str = include_str!("../res/test/index_without_update.json");
    const GET_PACKAGE_INDEX: &str = include_str!("../res/test/get_package_index.json");
    const INSTALL_PACKAGE_INDEX: &str = include_str!("../res/test/install_package_index.json");

    struct TempPackageManager {
        package_dir: TempDir,
        data_dir: TempDir,
        package_manager: DefaultPackageManager,
    }

    fn create_temp_package_manager<F>(setup: F) -> TempPackageManager where F: Fn(&Path, &Path) -> (){
        let package_dir = TempDir::new().expect("unable to create temp directory");
        let data_dir = TempDir::new().expect("unable to create temp directory");

        setup(package_dir.path(), data_dir.path());

        let package_manager = DefaultPackageManager::new(
            package_dir.path().clone().to_path_buf(),
            data_dir.path().clone().to_path_buf()
        );

        TempPackageManager {
            package_dir,
            data_dir,
            package_manager
        }
    }

    #[test]
    fn test_download_index() {
        let temp = create_temp_package_manager(|_, _| {});
        let index = DefaultPackageManager::request_index();

        assert!(index.is_ok());
        assert!(index.unwrap().packages.len() > 0);
    }

    #[test]
    fn test_outdated_index() {
        let temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, OUTDATED_INDEX_CONTENT);
        });

        assert!(temp.package_manager.is_index_outdated());
    }

    #[test]
    fn test_up_to_date_index_should_not_be_updated() {
        let mut temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
            let current_timestamp = current_time.as_secs();
            let new_contents = INDEX_CONTENT_WITHOUT_UPDATE.replace("XXXX", &format!("{}", current_timestamp));
            std::fs::write(index_file, new_contents);
        });

        assert_eq!(temp.package_manager.update_index(false).unwrap(), UpdateResult::NotOutdated);
    }

    #[test]
    fn test_up_to_date_index_with_force_should_be_updated() {
        let mut temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
            let current_timestamp = current_time.as_secs();
            let new_contents = INDEX_CONTENT_WITHOUT_UPDATE.replace("XXXX", &format!("{}", current_timestamp));
            std::fs::write(index_file, new_contents);
        });

        assert_eq!(temp.package_manager.update_index(true).unwrap(), UpdateResult::Updated);
    }

    #[test]
    fn test_outdated_index_should_be_updated() {
        let mut temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, OUTDATED_INDEX_CONTENT);
        });

        assert_eq!(temp.package_manager.update_index(false).unwrap(), UpdateResult::Updated);
    }

    #[test]
    fn test_get_package_should_be_found() {
        let mut temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, GET_PACKAGE_INDEX);
        });

        assert_eq!(temp.package_manager.get_package("italian-accents").unwrap().title, "Italian Accents");
    }

    #[test]
    fn test_get_package_should_not_be_found() {
        let mut temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, GET_PACKAGE_INDEX);
        });

        assert!(temp.package_manager.get_package("not-existing").is_none());
    }

    #[test]
    fn test_list_local_packages() {
        let mut temp = create_temp_package_manager(|package_dir, _| {
            create_dir(package_dir.join("package-1"));
            create_dir(package_dir.join("package2"));
            std::fs::write(package_dir.join("dummyfile.txt"), "test");
        });

        let packages = temp.package_manager.list_local_packages();
        assert_eq!(packages.len(), 2);
        assert!(packages.iter().any(|p| p == "package-1"));
        assert!(packages.iter().any(|p| p == "package2"));
    }

    #[test]
    fn test_install_package_not_found() {
        let mut temp = create_temp_package_manager(|package_dir, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, INSTALL_PACKAGE_INDEX);
        });

        assert_eq!(temp.package_manager.install_package("doesnotexist").unwrap(), NotFoundInIndex);
    }

    #[test]
    fn test_install_package_already_installed() {
        let mut temp = create_temp_package_manager(|package_dir, data_dir| {
            create_dir(package_dir.join("italian-accents"));
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, INSTALL_PACKAGE_INDEX);
        });

        assert_eq!(temp.package_manager.install_package("italian-accents").unwrap(), AlreadyInstalled);
    }

    #[test]
    fn test_clone_temp_repository() {
        let cloned_dir = DefaultPackageManager::clone_repo_to_temp("https://github.com/federico-terzi/espanso-hub-core").unwrap();
        assert!(cloned_dir.path().join("LICENSE").exists());
    }

    #[test]
    fn test_install_package() {
        let mut temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, INSTALL_PACKAGE_INDEX);
        });

        assert_eq!(temp.package_manager.install_package("dummy-package").unwrap(), Installed);
        assert!(temp.package_dir.path().join("dummy-package").exists());
        assert!(temp.package_dir.path().join("dummy-package/README.md").exists());
        assert!(temp.package_dir.path().join("dummy-package/package.yml").exists());
    }

    #[test]
    fn test_install_package_does_not_exist_in_repo() {
        let mut temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, INSTALL_PACKAGE_INDEX);
        });

        assert_eq!(temp.package_manager.install_package("not-existing").unwrap(), NotFoundInRepo);
    }
}