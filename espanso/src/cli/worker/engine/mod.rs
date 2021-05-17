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
use espanso_ui::{UIRemote, event::UIEvent};
use log::info;
use ui::selector::MatchSelectorAdapter;

use crate::cli::worker::engine::{matcher::regex::RegexMatcherAdapterOptions, path::PathProviderAdapter};

use super::ui::icon::IconPaths;

pub mod executor;
pub mod match_cache;
pub mod matcher;
pub mod multiplex;
pub mod path;
pub mod render;
pub mod source;
pub mod ui;

pub fn initialize_and_spawn(
  paths: Paths,
  config_store: Box<dyn ConfigStore>,
  match_store: Box<dyn MatchStore>,
  icon_paths: IconPaths,
  ui_remote: Box<dyn UIRemote>,
  exit_signal: Receiver<()>,
  ui_event_receiver: Receiver<UIEvent>,
) -> Result<JoinHandle<()>> {
  let handle = std::thread::Builder::new()
    .name("engine thread".to_string())
    .spawn(move || {
      // TODO: properly order the initializations if necessary

      let app_info_provider =
        espanso_info::get_provider().expect("unable to initialize app info provider");
      let config_manager =
        super::config::ConfigManager::new(&*config_store, &*match_store, &*app_info_provider);
      let match_converter =
        super::engine::matcher::convert::MatchConverter::new(&*config_store, &*match_store);
      let match_cache = super::engine::match_cache::MatchCache::load(&*config_store, &*match_store);

      let modulo_manager = crate::gui::modulo::manager::ModuloManager::new();
      let modulo_form_ui = crate::gui::modulo::form::ModuloFormUI::new(&modulo_manager, &icon_paths.form_icon);
      let modulo_search_ui = crate::gui::modulo::search::ModuloSearchUI::new(&modulo_manager, &icon_paths.search_icon);

      let (detect_source, modifier_state_store, sequencer) =
        super::engine::source::init_and_spawn().expect("failed to initialize detector module");
      let exit_source = super::engine::source::exit::ExitSource::new(exit_signal, &sequencer);
      let ui_source = super::engine::source::ui::UISource::new(ui_event_receiver, &sequencer);
      let sources: Vec<&dyn crate::engine::funnel::Source> = vec![&detect_source, &exit_source, &ui_source];
      let funnel = crate::engine::funnel::default(&sources);

      let rolling_matcher = super::engine::matcher::rolling::RollingMatcherAdapter::new(
        &match_converter.get_rolling_matches(),
      );
      let regex_matcher = super::engine::matcher::regex::RegexMatcherAdapter::new(
        &match_converter.get_regex_matches(),
        &RegexMatcherAdapterOptions {
          max_buffer_size: 30,  // TODO: load from configs
        }
      );
      let matchers: Vec<
        &dyn crate::engine::process::Matcher<super::engine::matcher::MatcherState>,
      > = vec![&rolling_matcher, &regex_matcher];
      let selector = MatchSelectorAdapter::new(&modulo_search_ui, &match_cache);
      let multiplexer = super::engine::multiplex::MultiplexAdapter::new(&match_cache);

      let injector = espanso_inject::get_injector(Default::default())
        .expect("failed to initialize injector module"); // TODO: handle the options
      let clipboard = espanso_clipboard::get_clipboard(Default::default())
        .expect("failed to initialize clipboard module"); // TODO: handle options

      let clipboard_adapter = super::engine::render::clipboard::ClipboardAdapter::new(&*clipboard);
      let clipboard_extension =
        espanso_render::extension::clipboard::ClipboardExtension::new(&clipboard_adapter);
      let date_extension = espanso_render::extension::date::DateExtension::new();
      let echo_extension = espanso_render::extension::echo::EchoExtension::new();
      let random_extension = espanso_render::extension::random::RandomExtension::new();
      let home_path = dirs::home_dir().expect("unable to obtain home dir path");
      let script_extension = espanso_render::extension::script::ScriptExtension::new(
        &paths.config,
        &home_path,
        &paths.packages,
      );
      let shell_extension = espanso_render::extension::shell::ShellExtension::new(&paths.config);
      let form_adapter = ui::form::FormProviderAdapter::new(&modulo_form_ui);
      let form_extension = espanso_render::extension::form::FormExtension::new(&form_adapter);
      let renderer = espanso_render::create(vec![
        &clipboard_extension,
        &date_extension,
        &echo_extension,
        &random_extension,
        &script_extension,
        &shell_extension,
        &form_extension,
      ]);
      let renderer_adapter =
        super::engine::render::RendererAdapter::new(&match_cache, &config_manager, &renderer);
      let path_provider = PathProviderAdapter::new(&paths);

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
      );

      let event_injector =
        super::engine::executor::event_injector::EventInjectorAdapter::new(&*injector);
      let clipboard_injector =
        super::engine::executor::clipboard_injector::ClipboardInjectorAdapter::new(
          &*injector,
          &*clipboard,
          &config_manager,
        );
      let key_injector = super::engine::executor::key_injector::KeyInjectorAdapter::new(&*injector);
      let context_menu_adapter = super::engine::executor::context_menu::ContextMenuHandlerAdapter::new(&*ui_remote);
      let dispatcher = crate::engine::dispatch::default(
        &event_injector,
        &clipboard_injector,
        &config_manager,
        &key_injector,
        &clipboard_injector,
        &clipboard_injector,
        &context_menu_adapter,
      );

      let mut engine = crate::engine::Engine::new(&funnel, &mut processor, &dispatcher);
      engine.run();

      info!("engine eventloop has terminated, propagating exit event...");
      ui_remote.exit();
    })?;

  Ok(handle)
}
