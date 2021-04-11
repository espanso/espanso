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

use funnel::Source;
use process::Matcher;
use engine::ui::selector::MatchSelectorAdapter;

use crate::engine::{Engine, funnel, process, dispatch};
use super::{CliModule, CliModuleArgs};

mod engine;
mod config;

pub fn new() -> CliModule {
  #[allow(clippy::needless_update)]
  CliModule {
    requires_paths: true,
    requires_config: true,
    enable_logs: true,
    subcommand: "worker".to_string(),
    entry: worker_main,
    ..Default::default()
  }
}

fn worker_main(args: CliModuleArgs) {
  let config_store = args.config_store.expect("missing config store in worker main");
  let match_store = args.match_store.expect("missing match store in worker main");

  let app_info_provider = espanso_info::get_provider().expect("unable to initialize app info provider");
  let config_manager = config::ConfigManager::new(&*config_store, &*match_store, &*app_info_provider);
  let match_converter = engine::matcher::convert::MatchConverter::new(&*config_store, &*match_store);
  let match_cache = engine::match_cache::MatchCache::load(&*config_store, &*match_store);

  let detect_source = engine::source::detect::init_and_spawn().expect("failed to initialize detector module");
  let sources: Vec<&dyn Source> = vec![&detect_source];
  let funnel = funnel::default(&sources);

  let matcher = engine::matcher::rolling::RollingMatcherAdapter::new(&match_converter.get_rolling_matches());
  let matchers: Vec<&dyn Matcher<engine::matcher::MatcherState>> = vec![&matcher];
  let selector = MatchSelectorAdapter::new();
  let multiplexer = engine::multiplex::MultiplexAdapter::new(&match_cache);

  // TODO: add extensions
  let renderer = espanso_render::create(Vec::new());
  let renderer_adapter = engine::render::RendererAdapter::new(&match_cache, &config_manager, &renderer);

  let mut processor = process::default(&matchers, &config_manager, &selector, &multiplexer, &renderer_adapter);

  let injector = espanso_inject::get_injector(Default::default()).expect("failed to initialize injector module"); // TODO: handle the options

  let text_injector = engine::executor::text_injector::TextInjectorAdapter::new(&*injector);
  let key_injector = engine::executor::key_injector::KeyInjectorAdapter::new(&*injector);
  let dispatcher = dispatch::default(&text_injector, &key_injector);

  let mut engine = Engine::new(&funnel, &mut processor, &dispatcher);
  engine.run();
}