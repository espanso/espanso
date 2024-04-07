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

use std::{cmp::min, collections::HashMap, ffi::CString, os::raw::c_char, thread::ThreadId};

use anyhow::Result;
use lazycell::LazyCell;
use log::{error, trace};
use thiserror::Error;

use crate::{event::UIEvent, icons::TrayIcon, menu::Menu, UIEventLoop, UIRemote};

// IMPORTANT: if you change these, also edit the native.h file.
const MAX_FILE_PATH: usize = 1024;
const MAX_ICON_COUNT: usize = 3;

const UI_EVENT_TYPE_ICON_CLICK: i32 = 1;
const UI_EVENT_TYPE_CONTEXT_MENU_CLICK: i32 = 2;
const UI_EVENT_TYPE_HEARTBEAT: i32 = 3;

// Take a look at the native.h header file for an explanation of the fields
#[repr(C)]
pub struct RawUIOptions {
  pub show_icon: i32,

  pub icon_paths: [[u8; MAX_FILE_PATH]; MAX_ICON_COUNT],
  pub icon_paths_count: i32,
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
  pub fn ui_initialize(_self: *const MacEventLoop, options: RawUIOptions);
  pub fn ui_eventloop(
    event_callback: extern "C" fn(_self: *mut MacEventLoop, event: RawUIEvent),
  ) -> i32;
  pub fn ui_exit();
  pub fn ui_update_tray_icon(index: i32);
  pub fn ui_show_notification(message: *const c_char, delay: f64);
  pub fn ui_show_context_menu(payload: *const c_char);
}

pub struct MacUIOptions<'a> {
  pub show_icon: bool,
  pub icon_paths: &'a Vec<(TrayIcon, String)>,
}

pub fn create(options: MacUIOptions) -> Result<(MacRemote, MacEventLoop)> {
  // Validate icons
  assert!(options.icon_paths.len() <= MAX_ICON_COUNT, "MacOS UI received too many icon paths, please increase the MAX_ICON_COUNT constant to support more");
  // Convert the icon paths to the internal representation
  let mut icon_indexes: HashMap<TrayIcon, usize> = HashMap::new();
  let mut icons = Vec::new();
  for (index, (tray_icon, path)) in options.icon_paths.iter().enumerate() {
    icon_indexes.insert(tray_icon.clone(), index);
    icons.push(path.clone());
  }

  let eventloop = MacEventLoop::new(icons, options.show_icon);
  let remote = MacRemote::new(icon_indexes);

  Ok((remote, eventloop))
}

pub type MacUIEventCallback = Box<dyn Fn(UIEvent)>;

pub struct MacEventLoop {
  show_icon: bool,
  icons: Vec<String>,

  // Internal
  _event_callback: LazyCell<MacUIEventCallback>,
  _init_thread_id: LazyCell<ThreadId>,
}

impl MacEventLoop {
  pub(crate) fn new(icons: Vec<String>, show_icon: bool) -> Self {
    Self {
      icons,
      show_icon,
      _event_callback: LazyCell::new(),
      _init_thread_id: LazyCell::new(),
    }
  }
}

impl UIEventLoop for MacEventLoop {
  fn initialize(&mut self) -> Result<()> {
    // Convert the icon paths to the raw representation
    let mut icon_paths: [[u8; MAX_FILE_PATH]; MAX_ICON_COUNT] =
      [[0; MAX_FILE_PATH]; MAX_ICON_COUNT];
    for (i, icon_path) in icon_paths.iter_mut().enumerate().take(self.icons.len()) {
      let c_path = CString::new(self.icons[i].clone())?;
      let len = min(c_path.as_bytes().len(), MAX_FILE_PATH - 1);
      icon_path[0..len].clone_from_slice(&c_path.as_bytes()[..len]);
    }

    let options = RawUIOptions {
      show_icon: i32::from(self.show_icon),
      icon_paths,
      icon_paths_count: self.icons.len() as i32,
    };

    unsafe { ui_initialize(self as *const MacEventLoop, options) };

    // Make sure the run() method is called in the same thread as initialize()
    self
      ._init_thread_id
      .fill(std::thread::current().id())
      .expect("Unable to set initialization thread id");

    Ok(())
  }

  fn run(&self, event_callback: MacUIEventCallback) -> Result<()> {
    // Make sure the run() method is called in the same thread as initialize()
    if let Some(init_id) = self._init_thread_id.borrow() {
      assert!(
        !(init_id != &std::thread::current().id()),
        "MacEventLoop run() and initialize() methods should be called in the same thread"
      );
    }

    if self._event_callback.fill(event_callback).is_err() {
      error!("Unable to set MacEventLoop callback");
      return Err(MacUIError::InternalError().into());
    }

    extern "C" fn callback(_self: *mut MacEventLoop, event: RawUIEvent) {
      if let Some(callback) = unsafe { (*_self)._event_callback.borrow() } {
        let event: Option<UIEvent> = event.into();
        if let Some(event) = event {
          callback(event);
        } else {
          trace!("Unable to convert raw event to input event");
        }
      }
    }

    let error_code = unsafe { ui_eventloop(callback) };

    if error_code <= 0 {
      error!("MacEventLoop exited with <= 0 code");
      return Err(MacUIError::InternalError().into());
    }

    Ok(())
  }
}

pub struct MacRemote {
  // Maps icon name to their index
  icon_indexes: HashMap<TrayIcon, usize>,
}

impl MacRemote {
  pub(crate) fn new(icon_indexes: HashMap<TrayIcon, usize>) -> Self {
    Self { icon_indexes }
  }
}

impl UIRemote for MacRemote {
  fn update_tray_icon(&self, icon: TrayIcon) {
    if let Some(index) = self.icon_indexes.get(&icon) {
      unsafe { ui_update_tray_icon((*index) as i32) }
    } else {
      error!("Unable to update tray icon, invalid icon id");
    }
  }

  fn show_notification(&self, message: &str) {
    let c_string = CString::new(message);
    match c_string {
      Ok(message) => unsafe { ui_show_notification(message.as_ptr(), 3.0) },
      Err(error) => {
        error!("Unable to show notification {}", error);
      }
    }
  }

  fn show_context_menu(&self, menu: &Menu) {
    match menu.to_json() {
      Ok(payload) => {
        let c_string = CString::new(payload);
        match c_string {
          Ok(c_string) => unsafe { ui_show_context_menu(c_string.as_ptr()) },
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

  fn exit(&self) {
    unsafe { ui_exit() };
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
      UI_EVENT_TYPE_HEARTBEAT => {
        return Some(UIEvent::Heartbeat);
      }
      _ => {}
    }

    None
  }
}

#[derive(Error, Debug)]
pub enum MacUIError {
  #[error("internal error")]
  InternalError(),
}
