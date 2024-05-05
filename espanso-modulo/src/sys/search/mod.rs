/*
 * This file is part of modulo.
 *
 * Copyright (C) 2020-2021 Federico Terzi
 *
 * modulo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * modulo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with modulo.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};

pub mod types {
  #[derive(Debug)]
  pub struct SearchItem {
    pub id: String,
    pub label: String,
    pub trigger: Option<String>,
    pub search_terms: Vec<String>,
    pub is_builtin: bool,
  }

  #[derive(Debug)]
  pub struct Search {
    pub title: String,
    pub icon: Option<String>,
    pub hint: Option<String>,
    pub items: Vec<SearchItem>,
  }
}

#[allow(dead_code)]
mod interop {
  use crate::sys;

  use super::super::interop::*;
  use super::types;
  use std::ffi::{c_void, CString};

  pub(crate) struct OwnedSearch {
    title: CString,
    icon_path: CString,
    hint: CString,
    items: Vec<OwnedSearchItem>,
    pub(crate) interop_items: Vec<SearchItem>,
    interop: Box<SearchMetadata>,
  }

  impl Interoperable for OwnedSearch {
    fn as_ptr(&self) -> *const c_void {
      std::ptr::from_ref::<SearchMetadata>(&(*self.interop)) as *const c_void
    }
  }

  impl From<&types::Search> for OwnedSearch {
    fn from(search: &types::Search) -> Self {
      let title =
        CString::new(search.title.clone()).expect("unable to convert search title to CString");

      let items: Vec<OwnedSearchItem> = search.items.iter().map(Into::into).collect();

      let interop_items: Vec<SearchItem> = items
        .iter()
        .map(sys::search::interop::OwnedSearchItem::to_search_item)
        .collect();

      let icon_path = if let Some(icon_path) = search.icon.as_ref() {
        icon_path.clone()
      } else {
        String::new()
      };

      let icon_path = CString::new(icon_path).expect("unable to convert search icon to CString");

      let icon_path_ptr = if search.icon.is_some() {
        icon_path.as_ptr()
      } else {
        std::ptr::null()
      };

      let hint = if let Some(hint) = search.hint.as_ref() {
        hint.clone()
      } else {
        String::new()
      };

      let hint = CString::new(hint).expect("unable to convert search icon to CString");

      let hint_ptr = if search.hint.is_some() {
        hint.as_ptr()
      } else {
        std::ptr::null()
      };

      let interop = Box::new(SearchMetadata {
        iconPath: icon_path_ptr,
        windowTitle: title.as_ptr(),
        hintText: hint_ptr,
      });

      Self {
        title,
        icon_path,
        hint,
        items,
        interop_items,
        interop,
      }
    }
  }

  pub(crate) struct OwnedSearchItem {
    id: CString,
    label: CString,
    trigger: CString,
  }

  impl OwnedSearchItem {
    fn to_search_item(&self) -> SearchItem {
      SearchItem {
        id: self.id.as_ptr(),
        label: self.label.as_ptr(),
        trigger: self.trigger.as_ptr(),
      }
    }
  }

  impl From<&types::SearchItem> for OwnedSearchItem {
    fn from(item: &types::SearchItem) -> Self {
      let id = CString::new(item.id.clone()).expect("unable to convert item id to CString");
      let label =
        CString::new(item.label.clone()).expect("unable to convert item label to CString");

      let trigger = if let Some(trigger) = item.trigger.as_deref() {
        CString::new(trigger.to_string()).expect("unable to convert item trigger to CString")
      } else {
        CString::new(String::new()).expect("unable to convert item trigger to CString")
      };

      Self { id, label, trigger }
    }
  }
}

type SearchAlgorithmCallback = dyn Fn(&str, &[types::SearchItem]) -> Vec<usize>;

struct SearchData {
  owned_search: interop::OwnedSearch,
  items: Vec<types::SearchItem>,
  algorithm: Box<SearchAlgorithmCallback>,
}

pub fn show(search: types::Search, algorithm: Box<SearchAlgorithmCallback>) -> Option<String> {
  use super::interop::*;

  let owned_search: interop::OwnedSearch = (&search).into();
  let metadata: *const SearchMetadata = owned_search.as_ptr() as *const SearchMetadata;

  let search_data = SearchData {
    owned_search,
    items: search.items,
    algorithm,
  };

  extern "C" fn search_callback(query: *const c_char, app: *const c_void, data: *const c_void) {
    let query = unsafe { CStr::from_ptr(query) };
    let query = query.to_string_lossy().to_string();

    let search_data = data as *const SearchData;
    let search_data = unsafe { &*search_data };

    let indexes = (*search_data.algorithm)(&query, &search_data.items);
    let items: Vec<SearchItem> = indexes
      .into_iter()
      .map(|index| search_data.owned_search.interop_items[index])
      .collect();

    unsafe {
      update_items(app, items.as_ptr(), items.len() as c_int);
    }
  }

  let mut result: Option<String> = None;

  extern "C" fn result_callback(id: *const c_char, result: *mut c_void) {
    let id = unsafe { CStr::from_ptr(id) };
    let id = id.to_string_lossy().to_string();
    let result: *mut Option<String> = result as *mut Option<String>;
    unsafe {
      *result = Some(id);
    }
  }

  unsafe {
    interop_show_search(
      metadata,
      search_callback,
      std::ptr::from_ref::<SearchData>(&search_data) as *const c_void,
      result_callback,
      std::ptr::from_mut::<Option<String>>(&mut result) as *mut c_void,
    );
  }

  result
}
