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

use crate::engine::{Engine, funnel, process, dispatch};
use super::{CliModule, CliModuleArgs};

mod source;
mod matcher;
mod executor;

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
  let detect_source = source::detect::init_and_spawn().unwrap(); // TODO: handle error
  let sources: Vec<&dyn Source> = vec![&detect_source];
  let funnel = funnel::default(&sources);

  let matcher = matcher::rolling::RollingMatcherAdapter::new();
  let matchers: Vec<&dyn Matcher<matcher::MatcherState>> = vec![&matcher];
  let mut processor = process::default(&matchers);

  let text_injector = executor::text_injector::TextInjectorAdapter::new();
  let dispatcher = dispatch::default(&text_injector);

  let mut engine = Engine::new(&funnel, &mut processor, &dispatcher);
  engine.run();
}

