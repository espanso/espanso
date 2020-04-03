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
use crate::package::{PackageIndex, UpdateResult, Package, InstallResult, RemoveResult, PackageResolver};
use std::error::Error;
use std::fs::{File, create_dir};
use std::io::{BufReader, BufRead};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::package::UpdateResult::{NotOutdated, Updated};
use crate::package::InstallResult::{NotFoundInIndex, AlreadyInstalled, BlockedExternalPackage};
use std::fs;
use tempfile::TempDir;
use regex::Regex;
use crate::package::RemoveResult::Removed;
use std::collections::HashMap;

const DEFAULT_PACKAGE_INDEX_FILE : &str = "package_index.json";

pub struct DefaultPackageManager {
    package_dir: PathBuf,
    data_dir: PathBuf,

    package_resolver: Option<Box<dyn PackageResolver>>,

    local_index: Option<PackageIndex>,
}

impl DefaultPackageManager {
    pub fn new(package_dir: PathBuf, data_dir: PathBuf, package_resolver: Option<Box<dyn PackageResolver>>) -> DefaultPackageManager {
        let local_index = Self::load_local_index(&data_dir);

        DefaultPackageManager{
            package_dir,
            data_dir,
            package_resolver,
            local_index
        }
    }

    pub fn new_default(package_resolver: Option<Box<dyn PackageResolver>>) -> DefaultPackageManager {
        DefaultPackageManager::new(
            crate::context::get_package_dir(),
            crate::context::get_data_dir(),
            package_resolver,
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
        let client = reqwest::Client::new();
        let request = client.get("https://hub.espanso.org/json/")
            .header("User-Agent", format!("espanso/{}", crate::VERSION));

        let mut res = request.send()?;
        let body = res.text()?;
        let index : PackageIndex = serde_json::from_str(&body)?;

        Ok(index)
    }

    fn parse_package_from_readme(readme_path: &Path) -> Option<Package> {
        lazy_static! {
            static ref FIELD_REGEX: Regex = Regex::new(r###"^\s*(.*?)\s*:\s*"?(.*?)"?$"###).unwrap();
        }

        // Read readme line by line
        let file = File::open(readme_path);
        if let Ok(file) = file {
            let reader = BufReader::new(file);

            let mut fields :HashMap<String, String> = HashMap::new();

            let mut started = false;

            for (_index, line) in reader.lines().enumerate() {
                let line = line.unwrap();
                if line.contains("---") {
                    if started {
                        break
                    }else{
                        started = true;
                    }
                }else if started {
                    let caps = FIELD_REGEX.captures(&line);
                    if let Some(caps) = caps {
                        let property = caps.get(1);
                        let value = caps.get(2);
                        if property.is_some() && value.is_some() {
                            fields.insert(property.unwrap().as_str().to_owned(),
                                          value.unwrap().as_str().to_owned());
                        }
                    }
                }
            }

            if !fields.contains_key("package_name") ||
               !fields.contains_key("package_title") ||
               !fields.contains_key("package_version") ||
               !fields.contains_key("package_repo") ||
               !fields.contains_key("package_desc") ||
               !fields.contains_key("package_author") {
                return None
            }

            let original_repo = if fields.contains_key("package_original_repo") {
                fields.get("package_original_repo").unwrap().clone()
            }else{
                fields.get("package_repo").unwrap().clone()
            };

            let is_core = if fields.contains_key("is_core") {
                match fields.get("is_core").unwrap().clone().as_ref() {
                    "true" => true,
                    "false" => false,
                    _ => false,
                }
            }else{
                false
            };

            let package = Package {
                name: fields.get("package_name").unwrap().clone(),
                title: fields.get("package_title").unwrap().clone(),
                version: fields.get("package_version").unwrap().clone(),
                repo: fields.get("package_repo").unwrap().clone(),
                desc: fields.get("package_desc").unwrap().clone(),
                author: fields.get("package_author").unwrap().clone(),
                is_core,
                original_repo
            };

            Some(package)
        }else{
            None
        }
    }

    fn local_index_timestamp(&self) -> u64 {
        if let Some(local_index) = &self.local_index {
            return local_index.last_update
        }

        0
    }

    fn list_local_packages_names(&self) -> Vec<String> {
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

    fn cache_local_index(&self) {
        if let Some(local_index) = &self.local_index {
            let serialized = serde_json::to_string(local_index).expect("Unable to serialize local index");
            let local_index_file = self.data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(local_index_file, serialized).expect("Unable to cache local index");
        }
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

            // Save the index to file
            self.cache_local_index();

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

    fn install_package(&self, name: &str, allow_external: bool) -> Result<InstallResult, Box<dyn Error>> {
        let package = self.get_package(name);
        match package {
            Some(package) => {
                if package.is_core || allow_external {
                    self.install_package_from_repo(name, &package.repo)
                }else{
                    Ok(BlockedExternalPackage(package.original_repo))
                }
            },
            None => {
                Ok(NotFoundInIndex)
            },
        }
    }

    fn install_package_from_repo(&self, name: &str, repo_url: &str) -> Result<InstallResult, Box<dyn Error>> {
        // Check if package is already installed
        let packages = self.list_local_packages_names();
        if packages.iter().any(|p| p == name) {  // Package already installed
            return Ok(AlreadyInstalled);
        }

        let temp_dir = self.package_resolver.as_ref().unwrap().clone_repo_to_temp(repo_url)?;

        let temp_package_dir = temp_dir.path().join(name);
        if !temp_package_dir.exists() {
            return Ok(InstallResult::NotFoundInRepo);
        }

        let readme_path = temp_package_dir.join("README.md");

        let package = Self::parse_package_from_readme(&readme_path);
        if package.is_none() {
            return Ok(InstallResult::UnableToParsePackageInfo);
        }
        let package = package.unwrap();

        let source_dir = temp_package_dir.join(package.version);
        if !source_dir.exists() {
            return Ok(InstallResult::MissingPackageVersion);
        }

        let target_dir = &self.package_dir.join(name);
        create_dir(&target_dir)?;

        crate::utils::copy_dir(&source_dir, target_dir)?;

        let readme_dest = target_dir.join("README.md");
        std::fs::copy(readme_path, readme_dest)?;

        Ok(InstallResult::Installed)
    }

    fn remove_package(&self, name: &str) -> Result<RemoveResult, Box<dyn Error>> {
        let package_dir = self.package_dir.join(name);
        if !package_dir.exists() {
            return Ok(RemoveResult::NotFound);
        }

        std::fs::remove_dir_all(package_dir)?;

        Ok(Removed)
    }

    fn list_local_packages(&self) -> Vec<Package> {
        let mut output = Vec::new();

        let package_names = self.list_local_packages_names();

        for name in package_names.iter() {
            let package_dir = &self.package_dir.join(name);
            let readme_file = package_dir.join("README.md");
            let package = Self::parse_package_from_readme(&readme_file);
            if let Some(package) = package {
                output.push(package);
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, NamedTempFile};
    use std::path::Path;
    use crate::package::PackageManager;
    use std::fs::{create_dir, create_dir_all};
    use crate::package::InstallResult::*;
    use std::io::Write;
    use crate::package::zip::ZipPackageResolver;

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
            data_dir.path().clone().to_path_buf(),
            Some(Box::new(ZipPackageResolver::new())),
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
    fn test_update_index_should_create_file() {
        let mut temp = create_temp_package_manager(|_, _| {});

        assert_eq!(temp.package_manager.update_index(false).unwrap(), UpdateResult::Updated);
        assert!(temp.data_dir.path().join(DEFAULT_PACKAGE_INDEX_FILE).exists())
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
    fn test_list_local_packages_names() {
        let mut temp = create_temp_package_manager(|package_dir, _| {
            create_dir(package_dir.join("package-1"));
            create_dir(package_dir.join("package2"));
            std::fs::write(package_dir.join("dummyfile.txt"), "test");
        });

        let packages = temp.package_manager.list_local_packages_names();
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

        assert_eq!(temp.package_manager.install_package("doesnotexist", false).unwrap(), NotFoundInIndex);
    }

    #[test]
    fn test_install_package_already_installed() {
        let mut temp = create_temp_package_manager(|package_dir, data_dir| {
            create_dir(package_dir.join("italian-accents"));
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, INSTALL_PACKAGE_INDEX);
        });

        assert_eq!(temp.package_manager.install_package("italian-accents", false).unwrap(), AlreadyInstalled);
    }

    #[test]
    fn test_install_package() {
        let mut temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, INSTALL_PACKAGE_INDEX);
        });

        assert_eq!(temp.package_manager.install_package("dummy-package", false).unwrap(), Installed);
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

        assert_eq!(temp.package_manager.install_package("not-existing", false).unwrap(), NotFoundInRepo);
    }

    #[test]
    fn test_install_package_missing_version() {
        let mut temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, INSTALL_PACKAGE_INDEX);
        });

        assert_eq!(temp.package_manager.install_package("dummy-package2", false).unwrap(), MissingPackageVersion);
    }

    #[test]
    fn test_install_package_missing_readme_unable_to_parse_package_info() {
        let mut temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, INSTALL_PACKAGE_INDEX);
        });

        assert_eq!(temp.package_manager.install_package("dummy-package3", false).unwrap(), UnableToParsePackageInfo);
    }

    #[test]
    fn test_install_package_bad_readme_unable_to_parse_package_info() {
        let mut temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, INSTALL_PACKAGE_INDEX);
        });

        assert_eq!(temp.package_manager.install_package("dummy-package4", false).unwrap(), UnableToParsePackageInfo);
    }

    #[test]
    fn test_list_local_packages() {
        let mut temp = create_temp_package_manager(|_, data_dir| {
            let index_file = data_dir.join(DEFAULT_PACKAGE_INDEX_FILE);
            std::fs::write(index_file, INSTALL_PACKAGE_INDEX);
        });

        assert_eq!(temp.package_manager.install_package("dummy-package", false).unwrap(), Installed);
        assert!(temp.package_dir.path().join("dummy-package").exists());
        assert!(temp.package_dir.path().join("dummy-package/README.md").exists());
        assert!(temp.package_dir.path().join("dummy-package/package.yml").exists());

        let list = temp.package_manager.list_local_packages();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "dummy-package");
    }

    #[test]
    fn test_remove_package() {
        let mut temp = create_temp_package_manager(|package_dir, _| {
            let dummy_package_dir = package_dir.join("dummy-package");
            create_dir_all(&dummy_package_dir);
            std::fs::write(dummy_package_dir.join("README.md"), "readme");
            std::fs::write(dummy_package_dir.join("package.yml"), "name: package");
        });

        assert!(temp.package_dir.path().join("dummy-package").exists());
        assert!(temp.package_dir.path().join("dummy-package/README.md").exists());
        assert!(temp.package_dir.path().join("dummy-package/package.yml").exists());
        assert_eq!(temp.package_manager.remove_package("dummy-package").unwrap(), RemoveResult::Removed);
        assert!(!temp.package_dir.path().join("dummy-package").exists());
        assert!(!temp.package_dir.path().join("dummy-package/README.md").exists());
        assert!(!temp.package_dir.path().join("dummy-package/package.yml").exists());
    }

    #[test]
    fn test_remove_package_not_found() {
        let mut temp = create_temp_package_manager(|_, _| {});

        assert_eq!(temp.package_manager.remove_package("not-existing").unwrap(), RemoveResult::NotFound);
    }

    #[test]
    fn test_parse_package_from_readme() {
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), r###"
        ---
        package_name: "italian-accents"
        package_title: "Italian Accents"
        package_desc: "Include Italian accents substitutions to espanso."
        package_version: "0.1.0"
        package_author: "Federico Terzi"
        package_repo: "https://github.com/federico-terzi/espanso-hub-core"
        is_core: true
        ---
        "###);

        let package = DefaultPackageManager::parse_package_from_readme(file.path()).unwrap();

        let target_package = Package {
            name: "italian-accents".to_string(),
            title: "Italian Accents".to_string(),
            version: "0.1.0".to_string(),
            repo: "https://github.com/federico-terzi/espanso-hub-core".to_string(),
            desc: "Include Italian accents substitutions to espanso.".to_string(),
            author: "Federico Terzi".to_string(),
            original_repo: "https://github.com/federico-terzi/espanso-hub-core".to_string(),
            is_core: true,
        };

        assert_eq!(package, target_package);
    }

    #[test]
    fn test_parse_package_from_readme_with_bad_metadata() {
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), r###"
        ---
        package_name: italian-accents
        package_title: "Italian Accents"
        package_desc: "Include Italian accents substitutions to espanso."
        package_version:"0.1.0"
        package_author:Federico Terzi
        package_repo: "https://github.com/federico-terzi/espanso-hub-core"
        is_core: true
        ---
        Readme text
        "###);

        let package = DefaultPackageManager::parse_package_from_readme(file.path()).unwrap();

        let target_package = Package {
            name: "italian-accents".to_string(),
            title: "Italian Accents".to_string(),
            version: "0.1.0".to_string(),
            repo: "https://github.com/federico-terzi/espanso-hub-core".to_string(),
            desc: "Include Italian accents substitutions to espanso.".to_string(),
            author: "Federico Terzi".to_string(),
            original_repo: "https://github.com/federico-terzi/espanso-hub-core".to_string(),
            is_core: true,
        };

        assert_eq!(package, target_package);
    }
}