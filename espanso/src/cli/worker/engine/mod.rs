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
use espanso_config::{config::ConfigStore, matches::store::MatchStore};
use espanso_path::Paths;
use ui::selector::MatchSelectorAdapter;

use crate::cli::worker::engine::path::PathProviderAdapter;

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
) -> Result<()> {
  std::thread::Builder::new()
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

      let modulo_manager = ui::modulo::ModuloManager::new();

      let (detect_source, modifier_state_store, sequencer) =
        super::engine::source::init_and_spawn().expect("failed to initialize detector module");
      let sources: Vec<&dyn crate::engine::funnel::Source> = vec![&detect_source];
      let funnel = crate::engine::funnel::default(&sources);

      let matcher = super::engine::matcher::rolling::RollingMatcherAdapter::new(
        &match_converter.get_rolling_matches(),
      );
      let matchers: Vec<
        &dyn crate::engine::process::Matcher<super::engine::matcher::MatcherState>,
      > = vec![&matcher];
      let selector = MatchSelectorAdapter::new();
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
      let form_adapter =
        ui::modulo::form::ModuloFormProviderAdapter::new(&modulo_manager, icon_paths.form_icon);
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
        );
      let key_injector = super::engine::executor::key_injector::KeyInjectorAdapter::new(&*injector);
      let dispatcher = crate::engine::dispatch::default(
        &event_injector,
        &clipboard_injector,
        &config_manager,
        &key_injector,
        &clipboard_injector,
        &clipboard_injector,
      );

      let mut engine = crate::engine::Engine::new(&funnel, &mut processor, &dispatcher);
      engine.run();
    })?;

  Ok(())
}
