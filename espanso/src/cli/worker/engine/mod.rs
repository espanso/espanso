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
use espanso_config::{config::ConfigStore, matches::store::MatchStore};
use espanso_detect::SourceCreationOptions;
use espanso_inject::InjectorCreationOptions;
use espanso_path::Paths;
use espanso_ui::{event::UIEvent, UIRemote};
use log::{debug, error, info, warn};

use crate::{
  cli::worker::{
    engine::{
      dispatch::executor::{
        clipboard_injector::ClipboardInjectorAdapter, context_menu::ContextMenuHandlerAdapter,
        event_injector::EventInjectorAdapter, icon::IconHandlerAdapter,
        key_injector::KeyInjectorAdapter,
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
        render::{
          extension::{clipboard::ClipboardAdapter, form::FormProviderAdapter},
          RendererAdapter,
        },
      },
    },
    match_cache::MatchCache,
  },
  engine::event::ExitMode,
};

use super::secure_input::SecureInputEvent;

pub mod dispatch;
pub mod funnel;
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
  run_count: i32,
  has_been_started_manually: bool,
) -> Result<JoinHandle<ExitMode>> {
  let handle = std::thread::Builder::new()
    .name("engine thread".to_string())
    .spawn(move || {
      // TODO: properly order the initializations if necessary

      let app_info_provider =
        espanso_info::get_provider().expect("unable to initialize app info provider");
      let config_manager =
        super::config::ConfigManager::new(&*config_store, &*match_store, &*app_info_provider);
      let match_converter = MatchConverter::new(&*config_store, &*match_store);
      let match_cache = MatchCache::load(&*config_store, &*match_store);

      let modulo_manager = crate::gui::modulo::manager::ModuloManager::new();
      let modulo_form_ui = crate::gui::modulo::form::ModuloFormUI::new(&modulo_manager);
      let modulo_search_ui = crate::gui::modulo::search::ModuloSearchUI::new(&modulo_manager);

      let has_granted_capabilities = grant_linux_capabilities(use_evdev_backend);

      // TODO: pass all the options
      let (detect_source, modifier_state_store, sequencer) =
        super::engine::funnel::init_and_spawn(SourceCreationOptions {
          use_evdev: use_evdev_backend,
          ..Default::default()
        })
        .expect("failed to initialize detector module");
      let exit_source = super::engine::funnel::exit::ExitSource::new(exit_signal, &sequencer);
      let ui_source = super::engine::funnel::ui::UISource::new(ui_event_receiver, &sequencer);
      let secure_input_source = super::engine::funnel::secure_input::SecureInputSource::new(
        secure_input_receiver,
        &sequencer,
      );
      let sources: Vec<&dyn crate::engine::funnel::Source> = vec![
        &detect_source,
        &exit_source,
        &ui_source,
        &secure_input_source,
      ];
      let funnel = crate::engine::funnel::default(&sources);

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
        &dyn crate::engine::process::Matcher<
          super::engine::process::middleware::matcher::MatcherState,
        >,
      > = vec![&rolling_matcher, &regex_matcher];
      let selector = MatchSelectorAdapter::new(&modulo_search_ui, &match_cache);
      let multiplexer = MultiplexAdapter::new(&match_cache);

      let injector = espanso_inject::get_injector(InjectorCreationOptions {
        use_evdev: use_evdev_backend,
        ..Default::default()
      })
      .expect("failed to initialize injector module"); // TODO: handle the options
      let clipboard = espanso_clipboard::get_clipboard(Default::default())
        .expect("failed to initialize clipboard module"); // TODO: handle options

      let clipboard_adapter = ClipboardAdapter::new(&*clipboard);
      let clipboard_extension =
        espanso_render::extension::clipboard::ClipboardExtension::new(&clipboard_adapter);
      let date_extension = espanso_render::extension::date::DateExtension::new();
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
      let renderer = espanso_render::create(vec![
        &clipboard_extension,
        &date_extension,
        &echo_extension,
        &dummy_extension,
        &random_extension,
        &script_extension,
        &shell_extension,
        &form_extension,
      ]);
      let renderer_adapter = RendererAdapter::new(&match_cache, &config_manager, &renderer);
      let path_provider = PathProviderAdapter::new(&paths);

      let disable_options =
        process::middleware::disable::extract_disable_options(config_manager.default());

      let mut processor = crate::engine::process::default(
        &matchers,
        &config_manager,
        &selector,
        &multiplexer,
        &renderer_adapter,
        &match_cache,
        &modifier_state_store,
        &sequencer,
        &path_provider,
        disable_options,
        &config_manager,
      );

      let event_injector = EventInjectorAdapter::new(&*injector, &config_manager);
      let clipboard_injector =
        ClipboardInjectorAdapter::new(&*injector, &*clipboard, &config_manager);
      let key_injector = KeyInjectorAdapter::new(&*injector, &config_manager);
      let context_menu_adapter = ContextMenuHandlerAdapter::new(&*ui_remote);
      let icon_adapter = IconHandlerAdapter::new(&*ui_remote);
      let dispatcher = crate::engine::dispatch::default(
        &event_injector,
        &clipboard_injector,
        &config_manager,
        &key_injector,
        &clipboard_injector,
        &clipboard_injector,
        &context_menu_adapter,
        &icon_adapter,
      );

      // Disable previously granted linux capabilities if not needed anymore
      if has_granted_capabilities {
        if let Err(err) = crate::capabilities::clear_capabilities() {
          error!("unable to revoke linux capabilities: {}", err);
        }
      }

      // TODO: check config
      match run_count {
        0 => ui_remote.show_notification("Espanso is running!"),
        n => {
          if has_been_started_manually {
            ui_remote.show_notification("Configuration reloaded!");
          } else if n == 1 {
            ui_remote.show_notification("Configuration reloaded! Espanso automatically loads new changes as soon as you save them.");
          }
        },
      }

      let mut engine = crate::engine::Engine::new(&funnel, &mut processor, &dispatcher);
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
