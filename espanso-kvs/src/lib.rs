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

use std::path::Path;

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

mod persistent;

#[allow(clippy::upper_case_acronyms)]
pub trait KVS: Send + Sync + Clone {
  fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
  fn set<T: Serialize>(&self, key: &str, value: T) -> Result<()>;
  fn delete(&self, key: &str) -> Result<()>;
}

pub fn get_persistent(base_dir: &Path) -> Result<impl KVS> {
  persistent::PersistentJsonKVS::new(base_dir)
}

#[cfg(test)]
mod tests {
  use super::*;

  use tempdir::TempDir;

  pub fn use_test_directory(callback: impl FnOnce(&Path)) {
    let dir = TempDir::new("kvstempconfig").unwrap();

    callback(
      dir.path(),
    );
  }

  #[test]
  fn test_base_types() {
    use_test_directory(|base_dir| {
      let kvs = get_persistent(base_dir).unwrap();

      assert!(kvs.get::<String>("my_key").unwrap().is_none());
      assert!(kvs.get::<bool>("another_key").unwrap().is_none());

      kvs.set("my_key", "test".to_string()).unwrap();
      kvs.set("another_key", false).unwrap();

      assert_eq!(kvs.get::<String>("my_key").unwrap().unwrap(), "test");
      assert!(!kvs.get::<bool>("another_key").unwrap().unwrap());

      kvs.delete("my_key").unwrap();

      assert!(kvs.get::<String>("my_key").unwrap().is_none());
      assert!(!kvs.get::<bool>("another_key").unwrap().unwrap());
    });
  }

  #[test]
  fn test_type_mismatch() {
    use_test_directory(|base_dir| {
      let kvs = get_persistent(base_dir).unwrap();

      assert!(kvs.get::<String>("my_key").unwrap().is_none());

      kvs.set("my_key", "test".to_string()).unwrap();

      assert!(kvs.get::<bool>("my_key").is_err());
      assert!(kvs.get::<String>("my_key").is_ok());
    });
  }

  #[test]
  fn test_delete_non_existing_key() {
    use_test_directory(|base_dir| {
      let kvs = get_persistent(base_dir).unwrap();

      kvs.delete("my_key").unwrap();
    });
  }

  #[test]
  fn test_invalid_key_name() {
    use_test_directory(|base_dir| {
      let kvs = get_persistent(base_dir).unwrap();

      assert!(kvs.get::<String>("invalid key name").is_err());
      assert!(kvs.get::<String>("").is_err());
    });
  }
}
