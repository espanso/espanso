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

use std::{
  cmp::min,
  collections::HashMap,
  ffi::{c_void, CString},
  os::raw::c_char,
  sync::{
    atomic::{AtomicPtr, Ordering},
    Arc,
  },
  thread::ThreadId,
};

use lazycell::LazyCell;
use log::{error, trace};
use widestring::WideCString;

use crate::{event::UIEvent, icons::TrayIcon, menu::Menu};

// IMPORTANT: if you change these, also edit the native.h file.
const MAX_FILE_PATH: usize = 260;
const MAX_ICON_COUNT: usize = 3;

const UI_EVENT_TYPE_ICON_CLICK: i32 = 1;
const UI_EVENT_TYPE_CONTEXT_MENU_CLICK: i32 = 2;

// Take a look at the native.h header file for an explanation of the fields
#[repr(C)]
pub struct RawUIOptions {
  pub show_icon: i32,

  pub icon_paths: [[u16; MAX_FILE_PATH]; MAX_ICON_COUNT],
  pub icon_paths_count: i32,

  pub notification_icon_path: [u16; MAX_FILE_PATH],
}
// Take a look at the native.h header file for an explanation of the fields
#[repr(C)]
pub struct RawUIEvent {
  pub event_type: i32,
  pub context_menu_id: u32,
}

#[allow(improper_ctypes)]
#[link(name = "espansoui", kind = "static")]
extern "C" {
  pub fn ui_initialize(
    _self: *const Win32EventLoop,
    options: RawUIOptions,
    error_code: *mut i32,
  ) -> *mut c_void;
  pub fn ui_eventloop(
    window_handle: *const c_void,
    event_callback: extern "C" fn(_self: *mut Win32EventLoop, event: RawUIEvent),
  ) -> i32;
  pub fn ui_destroy(window_handle: *const c_void) -> i32;
  pub fn ui_update_tray_icon(window_handle: *const c_void, index: i32);
  pub fn ui_show_notification(window_handle: *const c_void, message: *const u16);
  pub fn ui_show_context_menu(window_handle: *const c_void, payload: *const c_char);
}

pub struct Win32UIOptions<'a> {
  pub show_icon: bool,
  pub icon_paths: &'a Vec<(TrayIcon, String)>,
  pub notification_icon_path: String,
}

pub fn create(options: Win32UIOptions) -> (Win32Remote, Win32EventLoop) {
  let handle: Arc<AtomicPtr<c_void>> = Arc::new(AtomicPtr::new(std::ptr::null_mut()));

  // Validate icons
  if options.icon_paths.len() > MAX_ICON_COUNT {
    panic!("Win32 UI received too many icon paths, please increase the MAX_ICON_COUNT constant to support more");
  }

  // Convert the icon paths to the internal representation
  let mut icon_indexes: HashMap<TrayIcon, usize> = HashMap::new();
  let mut icons = Vec::new();
  for (index, (tray_icon, path)) in options.icon_paths.iter().enumerate() {
    icon_indexes.insert(tray_icon.clone(), index);
    icons.push(path.clone());
  }

  let eventloop = Win32EventLoop::new(
    handle.clone(),
    icons,
    options.show_icon,
    options.notification_icon_path,
  );
  let remote = Win32Remote::new(handle, icon_indexes);

  (remote, eventloop)
}

pub struct Win32EventLoop {
  handle: Arc<AtomicPtr<c_void>>,

  show_icon: bool,
  icons: Vec<String>,
  notification_icon_path: String,

  // Internal
  _event_callback: LazyCell<Win32UIEventCallback>,
  _init_thread_id: LazyCell<ThreadId>,
}

impl Win32EventLoop {
  pub(crate) fn new(
    handle: Arc<AtomicPtr<c_void>>,
    icons: Vec<String>,
    show_icon: bool,
    notification_icon_path: String,
  ) -> Self {
    Self {
      handle,
      icons,
      show_icon,
      notification_icon_path,
      _event_callback: LazyCell::new(),
      _init_thread_id: LazyCell::new(),
    }
  }

  pub fn initialize(&mut self) {
    let window_handle = self.handle.load(Ordering::Acquire);
    if !window_handle.is_null() {
      panic!("Attempt to initialize Win32EventLoop on non-null window handle");
    }

    // Convert the icon paths to the raw representation
    let mut icon_paths: [[u16; MAX_FILE_PATH]; MAX_ICON_COUNT] =
      [[0; MAX_FILE_PATH]; MAX_ICON_COUNT];
    for (i, icon_path) in icon_paths.iter_mut().enumerate().take(self.icons.len()) {
      let wide_path =
        WideCString::from_str(&self.icons[i]).expect("Error while converting icon to wide string");
      let len = min(wide_path.len(), MAX_FILE_PATH - 1);
      icon_path[0..len].clone_from_slice(&wide_path.as_slice()[..len]);
      // TODO: test overflow, correct case
    }

    let wide_notification_icon_path =
      widestring::WideCString::from_str(&self.notification_icon_path)
        .expect("Error while converting notification icon to wide string");
    let mut wide_notification_icon_path_buffer: [u16; MAX_FILE_PATH] = [0; MAX_FILE_PATH];
    wide_notification_icon_path_buffer[..wide_notification_icon_path.as_slice().len()]
      .clone_from_slice(wide_notification_icon_path.as_slice());

    let options = RawUIOptions {
      show_icon: if self.show_icon { 1 } else { 0 },
      icon_paths,
      icon_paths_count: self.icons.len() as i32,
      notification_icon_path: wide_notification_icon_path_buffer,
    };

    let mut error_code = 0;
    let handle = unsafe { ui_initialize(self as *const Win32EventLoop, options, &mut error_code) };

    if handle.is_null() {
      match error_code {
        -1 => panic!("Unable to initialize Win32EventLoop, error registering window class"),
        -2 => panic!("Unable to initialize Win32EventLoop, error creating window"),
        -3 => panic!("Unable to initialize Win32EventLoop, initializing notifications"),
        _ => panic!("Unable to initialize Win32EventLoop, unknown error"),
      }
    }

    self.handle.store(handle, Ordering::Release);

    // Make sure the run() method is called in the same thread as initialize()
    self
      ._init_thread_id
      .fill(std::thread::current().id())
      .expect("Unable to set initialization thread id");
  }

  pub fn run(&self, event_callback: Win32UIEventCallback) {
    // Make sure the run() method is called in the same thread as initialize()
    if let Some(init_id) = self._init_thread_id.borrow() {
      if init_id != &std::thread::current().id() {
        panic!("Win32EventLoop run() and initialize() methods should be called in the same thread");
        // TODO: test
      }
    }

    let window_handle = self.handle.load(Ordering::Acquire);
    if window_handle.is_null() {
      panic!("Attempt to run Win32EventLoop on a null window handle");
      // TODO: test
    }

    if self._event_callback.fill(event_callback).is_err() {
      panic!("Unable to set Win32EventLoop callback");
    }

    extern "C" fn callback(_self: *mut Win32EventLoop, event: RawUIEvent) {
      if let Some(callback) = unsafe { (*_self)._event_callback.borrow() } {
        let event: Option<UIEvent> = event.into();
        if let Some(event) = event {
          callback(event)
        } else {
          trace!("Unable to convert raw event to input event");
        }
      }
    }

    let error_code = unsafe { ui_eventloop(window_handle, callback) };

    if error_code <= 0 {
      panic!("Win32EventLoop exited with <= 0 code")
    }
  }
}

impl Drop for Win32EventLoop {
  fn drop(&mut self) {
    let handle = self.handle.swap(std::ptr::null_mut(), Ordering::Acquire);
    if handle.is_null() {
      error!("Win32EventLoop destruction cannot be performed, handle is null");
      return;
    }

    let result = unsafe { ui_destroy(handle) };

    if result != 0 {
      error!("Win32EventLoop destruction returned non-zero code");
    }
  }
}

pub struct Win32Remote {
  handle: Arc<AtomicPtr<c_void>>,

  // Maps icon name to their index
  icon_indexes: HashMap<TrayIcon, usize>,
}

impl Win32Remote {
  pub(crate) fn new(
    handle: Arc<AtomicPtr<c_void>>,
    icon_indexes: HashMap<TrayIcon, usize>,
  ) -> Self {
    Self {
      handle,
      icon_indexes,
    }
  }

  pub fn update_tray_icon(&self, icon: TrayIcon) {
    let handle = self.handle.load(Ordering::Acquire);
    if handle.is_null() {
      error!("Unable to update tray icon, pointer is null");
      return;
    }

    if let Some(index) = self.icon_indexes.get(&icon) {
      unsafe { ui_update_tray_icon(handle, (*index) as i32) }
    } else {
      error!("Unable to update tray icon, invalid icon id");
    }
  }

  pub fn show_notification(&self, message: &str) {
    let handle = self.handle.load(Ordering::Acquire);
    if handle.is_null() {
      error!("Unable to show notification, pointer is null");
      return;
    }

    let wide_message = widestring::WideCString::from_str(message);
    match wide_message {
      Ok(wide_message) => unsafe { ui_show_notification(handle, wide_message.as_ptr()) },
      Err(error) => {
        error!(
          "Unable to show notification, invalid message encoding {}",
          error
        );
      }
    }
  }

  pub fn show_context_menu(&self, menu: &Menu) {
    let handle = self.handle.load(Ordering::Acquire);
    if handle.is_null() {
      error!("Unable to show context menu, pointer is null");
      return;
    }

    match menu.to_json() {
      Ok(payload) => {
        let c_string = CString::new(payload);
        match c_string {
          Ok(c_string) => unsafe { ui_show_context_menu(handle, c_string.as_ptr()) },
          Err(error) => error!(
            "Unable to show context menu, impossible to convert payload to c_string: {}",
            error
          ),
        }
      }
      Err(error) => {
        error!("Unable to show context menu, {}", error);
      }
    }
  }
}

#[allow(clippy::single_match)] // TODO: remove after another match is used
impl From<RawUIEvent> for Option<UIEvent> {
  fn from(raw: RawUIEvent) -> Option<UIEvent> {
    match raw.event_type {
      UI_EVENT_TYPE_ICON_CLICK => {
        return Some(UIEvent::TrayIconClick);
      }
      UI_EVENT_TYPE_CONTEXT_MENU_CLICK => {
        return Some(UIEvent::ContextMenuClick(raw.context_menu_id));
      }
      _ => {}
    }

    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn constants_are_not_changed_by_mistake() {
    assert_eq!(MAX_FILE_PATH, 260);
    assert_eq!(MAX_ICON_COUNT, 3);
  }
}
