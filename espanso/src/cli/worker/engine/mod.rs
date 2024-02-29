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

use std::thread::JoinHandle;

use anyhow::Result;
use crossbeam::channel::Receiver;
use espanso_clipboard::ClipboardOptions;
use espanso_config::{config::ConfigStore, matches::store::MatchStore};
use espanso_detect::SourceCreationOptions;
use espanso_engine::event::{EventType, ExitMode};
use espanso_inject::{InjectorCreationOptions, KeyboardStateProvider};
use espanso_path::Paths;
use espanso_ui::{event::UIEvent, UIRemote};
use log::{debug, error, info, warn};

use crate::{
  cli::worker::{
    context::Context,
    engine::{
      dispatch::executor::{
        clipboard_injector::ClipboardInjectorAdapter, context_menu::ContextMenuHandlerAdapter,
        event_injector::EventInjectorAdapter, icon::IconHandlerAdapter,
        key_injector::KeyInjectorAdapter, secure_input::SecureInputManagerAdapter,
        text_ui::TextUIHandlerAdapter,
      },
      process::middleware::{
        image_resolve::PathProviderAdapter,
        match_select::MatchSelectorAdapter,
        matcher::{
          convert::MatchConverter,
          regex::{RegexMatcherAdapter, RegexMatcherAdapterOptions},
          rolling::{RollingMatcherAdapter, RollingMatcherAdapterOptions},
        },
        multiplex::MultiplexAdapter,
        open_config::ConfigPathProviderAdapter,
        render::{
          extension::{
            choice::ChoiceSelectorAdapter, clipboard::ClipboardAdapter, form::FormProviderAdapter,
          },
          RendererAdapter,
        },
      },
    },
    match_cache::{CombinedMatchCache, MatchCache},
    ui::notification::NotificationManager,
  },
  common_flags::{
    WORKER_START_REASON_CONFIG_CHANGED, WORKER_START_REASON_KEYBOARD_LAYOUT_CHANGED,
    WORKER_START_REASON_MANUAL,
  },
  preferences::Preferences,
};

use super::secure_input::SecureInputEvent;

mod caches;
pub mod dispatch;
pub mod funnel;
mod keyboard_layout_util;
pub mod process;

#[allow(clippy::too_many_arguments)]
pub fn initialize_and_spawn(
  paths: Paths,
  config_store: Box<dyn ConfigStore>,
  match_store: Box<dyn MatchStore>,
  ui_remote: Box<dyn UIRemote>,
  exit_signal: Receiver<ExitMode>,
  ui_event_receiver: Receiver<UIEvent>,
  secure_input_receiver: Receiver<SecureInputEvent>,
  use_evdev_backend: bool,
  start_reason: Option<String>,
  ipc_event_receiver: Receiver<EventType>,
) -> Result<JoinHandle<ExitMode>> {
  let handle = std::thread::Builder::new()
    .name("engine thread".to_string())
    .spawn(move || {
      // TODO: properly order the initializations if necessary
      let preferences =
        crate::preferences::get_default(&paths.runtime).expect("unable to load preferences");

      let app_info_provider =
        espanso_info::get_provider().expect("unable to initialize app info provider");
      // TODO: read interval from configs?
      let cached_app_info_provider = caches::app_info_provider::CachedAppInfoProvider::from(
        &*app_info_provider,
        std::time::Duration::from_millis(400),
      );
      let config_manager =
        super::config::ConfigManager::new(&*config_store, &*match_store, &cached_app_info_provider);
      let match_cache = MatchCache::load(&*config_store, &*match_store);
      let default_config = &*config_manager.default();

      let modulo_manager = crate::gui::modulo::manager::ModuloManager::new();
      let modulo_form_ui =
        crate::gui::modulo::form::ModuloFormUI::new(&modulo_manager, &config_manager);
      let modulo_search_ui =
        crate::gui::modulo::search::ModuloSearchUI::new(&modulo_manager, &config_manager);
      let modulo_text_ui = crate::gui::modulo::textview::ModuloTextUI::new(&modulo_manager);

      let context: Box<dyn Context> = Box::new(super::context::DefaultContext::new(
        &config_manager,
        &cached_app_info_provider,
      ));
      let builtin_matches = super::builtin::get_builtin_matches(&*config_manager.default());
      let combined_match_cache = CombinedMatchCache::load(&match_cache, &builtin_matches);

      let match_converter = MatchConverter::new(&*config_store, &*match_store, &builtin_matches);

      let has_granted_capabilities = grant_linux_capabilities(use_evdev_backend);

      // TODO: pass all the options
      let (detect_source, modifier_state_store, sequencer, key_state_store) =
        super::engine::funnel::init_and_spawn(SourceCreationOptions {
          use_evdev: use_evdev_backend,
          evdev_keyboard_rmlvo: keyboard_layout_util::generate_detect_rmlvo(
            &*config_manager.default(),
          ),
          hotkeys: match_converter.get_hotkeys(),
          win32_exclude_orphan_events: default_config.win32_exclude_orphan_events(),
          win32_keyboard_layout_cache_interval: default_config
            .win32_keyboard_layout_cache_interval(),
        })
        .expect("failed to initialize detector module");
      let exit_source = super::engine::funnel::exit::ExitSource::new(exit_signal, &sequencer);
      let ipc_event_source =
        super::engine::funnel::ipc::IpcEventSource::new(ipc_event_receiver, &sequencer);
      let ui_source = super::engine::funnel::ui::UISource::new(ui_event_receiver, &sequencer);
      let secure_input_source = super::engine::funnel::secure_input::SecureInputSource::new(
        secure_input_receiver,
        &sequencer,
      );
      let mut sources: Vec<&dyn espanso_engine::funnel::Source> =
        vec![&detect_source, &exit_source, &ui_source, &ipc_event_source];
      if cfg!(target_os = "macos") {
        sources.push(&secure_input_source);
      }
      let funnel = espanso_engine::funnel::default(&sources);

      let rolling_matcher = RollingMatcherAdapter::new(
        &match_converter.get_rolling_matches(),
        RollingMatcherAdapterOptions {
          char_word_separators: config_manager.default().word_separators(),
        },
      );
      let regex_matcher = RegexMatcherAdapter::new(
        &match_converter.get_regex_matches(),
        &RegexMatcherAdapterOptions {
          max_buffer_size: 30, // TODO: load from configs
        },
      );
      let matchers: Vec<
        &dyn espanso_engine::process::Matcher<
          super::engine::process::middleware::matcher::MatcherState,
        >,
      > = vec![&rolling_matcher, &regex_matcher];
      let selector = MatchSelectorAdapter::new(&modulo_search_ui, &combined_match_cache);
      let multiplexer = MultiplexAdapter::new(&combined_match_cache, &*context);

      let injector = espanso_inject::get_injector(InjectorCreationOptions {
        use_evdev: use_evdev_backend,
        keyboard_state_provider: key_state_store
          .map(|store| Box::new(store) as Box<dyn KeyboardStateProvider>),
        evdev_keyboard_rmlvo: keyboard_layout_util::generate_inject_rmlvo(
          &*config_manager.default(),
        ),
        ..Default::default()
      })
      .expect("failed to initialize injector module"); // TODO: handle the options
      let clipboard = espanso_clipboard::get_clipboard(ClipboardOptions::default())
        .expect("failed to initialize clipboard module"); // TODO: handle options

      let clipboard_adapter = ClipboardAdapter::new(&*clipboard, &config_manager);
      let clipboard_extension =
        espanso_render::extension::clipboard::ClipboardExtension::new(&clipboard_adapter);
      let locale_provider = espanso_render::extension::date::DefaultLocaleProvider::new();
      let date_extension = espanso_render::extension::date::DateExtension::new(&locale_provider);
      let echo_extension = espanso_render::extension::echo::EchoExtension::new();
      // For backwards compatiblity purposes, the echo extension can also be called with "dummy" type
      let dummy_extension = espanso_render::extension::echo::EchoExtension::new_with_alias("dummy");
      let random_extension = espanso_render::extension::random::RandomExtension::new();
      let home_path = dirs::home_dir().expect("unable to obtain home dir path");
      let script_extension = espanso_render::extension::script::ScriptExtension::new(
        &paths.config,
        &home_path,
        &paths.packages,
      );
      let shell_extension = espanso_render::extension::shell::ShellExtension::new(&paths.config);
      let form_adapter = FormProviderAdapter::new(&modulo_form_ui);
      let form_extension = espanso_render::extension::form::FormExtension::new(&form_adapter);
      let choice_adapter = ChoiceSelectorAdapter::new(&modulo_search_ui);
      let choice_extension =
        espanso_render::extension::choice::ChoiceExtension::new(&choice_adapter);
      let renderer = espanso_render::create(vec![
        &clipboard_extension,
        &date_extension,
        &echo_extension,
        &dummy_extension,
        &random_extension,
        &script_extension,
        &shell_extension,
        &form_extension,
        &choice_extension,
      ]);
      let renderer_adapter = RendererAdapter::new(&match_cache, &config_manager, &renderer);
      let path_provider = PathProviderAdapter::new(&paths);
      let config_path_provider = ConfigPathProviderAdapter::new(&paths);

      let disable_options =
        process::middleware::disable::extract_disable_options(&*config_manager.default());

      let notification_manager = NotificationManager::new(&*ui_remote, default_config);

      let mut processor = espanso_engine::process::default(
        &matchers,
        &config_manager,
        &selector,
        &multiplexer,
        &renderer_adapter,
        &match_cache,
        &modifier_state_store,
        &sequencer,
        &path_provider,
        &config_path_provider,
        disable_options,
        &config_manager,
        &combined_match_cache,
        &config_manager,
        &config_manager,
        &modifier_state_store,
        &combined_match_cache,
        &notification_manager,
        &config_manager,
      );

      let event_injector = EventInjectorAdapter::new(&*injector, &config_manager);
      let clipboard_injector =
        ClipboardInjectorAdapter::new(&*injector, &*clipboard, &config_manager);
      let key_injector = KeyInjectorAdapter::new(&*injector, &config_manager);
      let context_menu_adapter = ContextMenuHandlerAdapter::new(&*ui_remote);
      let icon_adapter = IconHandlerAdapter::new(&*ui_remote);
      let secure_input_adapter = SecureInputManagerAdapter::new();
      let text_ui_adapter = TextUIHandlerAdapter::new(&modulo_text_ui, &paths);
      let dispatcher = espanso_engine::dispatch::default(
        &event_injector,
        &clipboard_injector,
        &config_manager,
        &key_injector,
        &clipboard_injector,
        &clipboard_injector,
        &context_menu_adapter,
        &icon_adapter,
        &secure_input_adapter,
        &text_ui_adapter,
      );

      // Disable previously granted linux capabilities if not needed anymore
      if has_granted_capabilities {
        if let Err(err) = crate::capabilities::clear_capabilities() {
          error!("unable to revoke linux capabilities: {}", err);
        }
      }

      match start_reason.as_deref() {
        Some(flag) if flag == WORKER_START_REASON_CONFIG_CHANGED => {
          notification_manager.notify_config_reloaded(false);
        }
        Some(flag) if flag == WORKER_START_REASON_MANUAL => {
          notification_manager.notify_config_reloaded(true);
        }
        Some(flag) if flag == WORKER_START_REASON_KEYBOARD_LAYOUT_CHANGED => {
          notification_manager.notify_keyboard_layout_reloaded();
        }
        _ => {
          notification_manager.notify_start();

          if !preferences.has_displayed_welcome() {
            super::ui::welcome::show_welcome_screen();
            preferences.set_has_displayed_welcome(true);
          }
        }
      }

      let mut engine = espanso_engine::Engine::new(&funnel, &mut processor, &dispatcher);
      let exit_mode = engine.run();

      info!("engine eventloop has terminated, propagating exit event...");
      ui_remote.exit();

      exit_mode
    })?;

  Ok(handle)
}

fn grant_linux_capabilities(use_evdev_backend: bool) -> bool {
  if use_evdev_backend {
    if crate::capabilities::can_use_capabilities() {
      debug!("using linux capabilities to grant permissions needed by EVDEV backend");
      if let Err(err) = crate::capabilities::grant_capabilities() {
        error!("unable to grant CAP_DAC_OVERRIDE capability: {}", err);
        false
      } else {
        debug!("successfully granted permissions using capabilities");
        true
      }
    } else {
      warn!("EVDEV backend is being used, but without enabling linux capabilities.");
      warn!("  Although you CAN run espanso EVDEV backend as root, it's not recommended due");
      warn!(
        "  to security reasons. Espanso supports linux capabilities to limit the attack surface"
      );
      warn!("  area by only leveraging on the CAP_DAC_OVERRIDE capability (needed to work with");
      warn!("  /dev/input/* devices to detect and inject text) and disabling it as soon as the");
      warn!("  initial setup is completed.");
      false
    }
  } else {
    false
  }
}
