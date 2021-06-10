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
use espanso_path::Paths;
use espanso_ui::{event::UIEvent, UIRemote};
use log::info;

use crate::{cli::worker::{engine::{dispatch::executor::{
        clipboard_injector::ClipboardInjectorAdapter, context_menu::ContextMenuHandlerAdapter,
        event_injector::EventInjectorAdapter, icon::IconHandlerAdapter,
        key_injector::KeyInjectorAdapter,
      }, process::middleware::{image_resolve::PathProviderAdapter, match_select::MatchSelectorAdapter, matcher::{convert::MatchConverter, regex::{RegexMatcherAdapter, RegexMatcherAdapterOptions}, rolling::{RollingMatcherAdapter, RollingMatcherAdapterOptions}}, multiplex::MultiplexAdapter, render::{
          extension::{clipboard::ClipboardAdapter, form::FormProviderAdapter},
          RendererAdapter,
        }}}, match_cache::MatchCache}, engine::event::ExitMode};

use super::secure_input::SecureInputEvent;

pub mod dispatch;
pub mod funnel;
pub mod process;

pub fn initialize_and_spawn(
  paths: Paths,
  config_store: Box<dyn ConfigStore>,
  match_store: Box<dyn MatchStore>,
  ui_remote: Box<dyn UIRemote>,
  exit_signal: Receiver<()>,
  ui_event_receiver: Receiver<UIEvent>,
  secure_input_receiver: Receiver<SecureInputEvent>,
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

      let (detect_source, modifier_state_store, sequencer) =
        super::engine::funnel::init_and_spawn().expect("failed to initialize detector module");
      let exit_source = super::engine::funnel::exit::ExitSource::new(exit_signal, &sequencer);
      let ui_source = super::engine::funnel::ui::UISource::new(ui_event_receiver, &sequencer);
      let secure_input_source = super::engine::funnel::secure_input::SecureInputSource::new(secure_input_receiver, &sequencer);
      let sources: Vec<&dyn crate::engine::funnel::Source> =
        vec![&detect_source, &exit_source, &ui_source, &secure_input_source];
      let funnel = crate::engine::funnel::default(&sources);

      let rolling_matcher = RollingMatcherAdapter::new(&match_converter.get_rolling_matches(), RollingMatcherAdapterOptions {
        char_word_separators: config_manager.default().word_separators(),
      });
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

      let injector = espanso_inject::get_injector(Default::default())
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

      let mut engine = crate::engine::Engine::new(&funnel, &mut processor, &dispatcher);
      let exit_mode = engine.run();

      info!("engine eventloop has terminated, propagating exit event...");
      ui_remote.exit();

      exit_mode
    })?;

  Ok(handle)
}
