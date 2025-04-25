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

use anyhow::Result;
use serde_json::Value;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use thiserror::Error;

use super::KVS;

const DEFAULT_KVS_DIR_NAME: &str = "kvs";

#[derive(Clone)]
pub struct PersistentJsonKVS {
    kvs_dir: PathBuf,
    store: Arc<Mutex<HashMap<String, Value>>>,
}

impl PersistentJsonKVS {
    pub fn new(base_dir: &Path) -> Result<Self> {
        let kvs_dir = base_dir.join(DEFAULT_KVS_DIR_NAME);
        if !kvs_dir.is_dir() {
            std::fs::create_dir_all(&kvs_dir)?;
        }

        Ok(Self {
            kvs_dir,
            store: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

impl KVS for PersistentJsonKVS {
    fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        if !is_valid_key_name(key) {
            return Err(PersistentJsonKVSError::InvalidKey(key.to_string()).into());
        }

        let mut lock = self.store.lock().expect("unable to obtain KVS read lock");

        if let Some(cached_value) = lock.get(key) {
            let converted_value = serde_json::from_value(cached_value.clone())?;
            return Ok(Some(converted_value));
        }

        // Not found in the cache, read from the file
        let target_file = self.kvs_dir.join(key);
        if target_file.is_file() {
            let content = std::fs::read_to_string(&target_file)?;
            let deserialized_value: Value = serde_json::from_str(&content)?;
            let converted_value = serde_json::from_value(deserialized_value.clone())?;

            lock.insert(key.to_string(), deserialized_value);

            return Ok(Some(converted_value));
        }

        Ok(None)
    }

    fn set<T: serde::Serialize>(&self, key: &str, value: T) -> Result<()> {
        if !is_valid_key_name(key) {
            return Err(PersistentJsonKVSError::InvalidKey(key.to_string()).into());
        }

        let mut lock = self.store.lock().expect("unable to obtain KVS write lock");

        let serialized_value = serde_json::to_value(value)?;
        let serialized_string = serde_json::to_string(&serialized_value)?;

        lock.insert(key.to_string(), serialized_value);

        let target_file = self.kvs_dir.join(key);
        std::fs::write(target_file, serialized_string)?;

        Ok(())
    }

    fn delete(&self, key: &str) -> Result<()> {
        if !is_valid_key_name(key) {
            return Err(PersistentJsonKVSError::InvalidKey(key.to_string()).into());
        }

        let mut lock = self.store.lock().expect("unable to obtain KVS delete lock");

        lock.remove(key);

        let target_file = self.kvs_dir.join(key);
        if target_file.is_file() {
            std::fs::remove_file(target_file)?;
        }

        Ok(())
    }
}

fn is_valid_key_name(key: &str) -> bool {
    if key.is_empty() || key.len() > 200 {
        return false;
    }

    if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return false;
    }

    true
}

#[derive(Error, Debug)]
pub enum PersistentJsonKVSError {
    #[error("The provided key `{0}` is is invalid. Keys must only be composed of ascii letters, numbers and underscores.")]
    InvalidKey(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_key_names() {
        assert!(is_valid_key_name("key"));
        assert!(is_valid_key_name("key_name"));
        assert!(is_valid_key_name("Another_long_key_name_2"));
    }

    #[test]
    fn test_invalid_key_names() {
        assert!(!is_valid_key_name(""));
        assert!(!is_valid_key_name("with space"));
        assert!(!is_valid_key_name("with/special"));
        assert!(!is_valid_key_name("with\\special"));
    }
}
