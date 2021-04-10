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

use std::{cell::RefCell, collections::HashMap, convert::TryInto};

use espanso_config::{
  config::Config,
  matches::{store::MatchSet, Match, MatchCause, MatchEffect},
};
use espanso_render::{CasingStyle, Context, RenderOptions, Template, Value, Variable};

use crate::{cli::worker::config::ConfigManager, engine::process::{Renderer, RendererError}};

pub trait MatchIterator<'a> {
  fn matches(&self) -> Vec<&'a Match>;
}

pub trait ConfigProvider<'a> {
  fn configs(&self) -> Vec<(&'a dyn Config, MatchSet)>;
  fn active(&self) -> (&'a dyn Config, MatchSet);
}

pub struct RendererAdapter<'a> {
  renderer: &'a dyn espanso_render::Renderer,
  config_provider: &'a dyn ConfigProvider<'a>,

  template_map: HashMap<i32, Option<Template>>,
  global_vars_map: HashMap<i32, Variable>,

  context_cache: RefCell<HashMap<i32, Context<'a>>>,
}

impl<'a> RendererAdapter<'a> {
  pub fn new(
    match_iterator: &'a dyn MatchIterator,
    config_provider: &'a dyn ConfigProvider<'a>,
    renderer: &'a dyn espanso_render::Renderer,
  ) -> Self {
    let template_map = generate_template_map(match_iterator);
    let global_vars_map = generate_global_vars_map(config_provider);

    Self {
      renderer,
      config_provider,
      template_map,
      global_vars_map,
      context_cache: RefCell::new(HashMap::new()),
    }
  }
}

// TODO: test
fn generate_template_map(match_iterator: &dyn MatchIterator) -> HashMap<i32, Option<Template>> {
  let mut template_map = HashMap::new();
  for m in match_iterator.matches() {
    let entry = convert_to_template(m);
    template_map.insert(m.id, entry);
  }
  template_map
}

// TODO: test
fn generate_global_vars_map(config_provider: &dyn ConfigProvider) -> HashMap<i32, Variable> {
  let mut global_vars_map = HashMap::new();

  for (_, match_set) in config_provider.configs() {
    for var in match_set.global_vars.iter() {
      if !global_vars_map.contains_key(&var.id) {
        global_vars_map.insert(var.id, convert_var((*var).clone()));
      }
    }
  }

  global_vars_map
}

// TODO: test
fn generate_context<'a>(
  match_set: &MatchSet,
  template_map: &'a HashMap<i32, Option<Template>>,
  global_vars_map: &'a HashMap<i32, Variable>,
) -> Context<'a> {
  let mut templates = Vec::new();
  let mut global_vars = Vec::new();

  for m in match_set.matches.iter() {
    if let Some(Some(template)) = template_map.get(&m.id) {
      templates.push(template);
    }
  }

  for var in match_set.global_vars.iter() {
    if let Some(var) = global_vars_map.get(&var.id) {
      global_vars.push(var);
    }
  }

  Context {
    templates,
    global_vars
  }
}

// TODO: move conversion methods to new file?

fn convert_to_template(m: &Match) -> Option<Template> {
  if let MatchEffect::Text(text_effect) = &m.effect {
    let ids = if let MatchCause::Trigger(cause) = &m.cause {
      cause.triggers.clone()
    } else {
      Vec::new()
    };

    Some(Template {
      ids,
      body: text_effect.replace.clone(),
      vars: convert_vars(text_effect.vars.clone()),
    })
  } else {
    None
  }
}

fn convert_vars(vars: Vec<espanso_config::matches::Variable>) -> Vec<espanso_render::Variable> {
  vars.into_iter().map(convert_var).collect()
}

fn convert_var(var: espanso_config::matches::Variable) -> espanso_render::Variable {
  Variable {
    name: var.name,
    var_type: var.var_type,
    params: convert_params(var.params),
  }
}

fn convert_params(params: espanso_config::matches::Params) -> espanso_render::Params {
  let mut new_params = espanso_render::Params::new();
  for (key, value) in params {
    new_params.insert(key, convert_value(value));
  }
  new_params
}

fn convert_value(value: espanso_config::matches::Value) -> espanso_render::Value {
  match value {
    espanso_config::matches::Value::Null => espanso_render::Value::Null,
    espanso_config::matches::Value::Bool(v) => espanso_render::Value::Bool(v),
    espanso_config::matches::Value::Number(n) => match n {
      espanso_config::matches::Number::Integer(i) => {
        espanso_render::Value::Number(espanso_render::Number::Integer(i))
      }
      espanso_config::matches::Number::Float(f) => {
        espanso_render::Value::Number(espanso_render::Number::Float(f.into_inner()))
      }
    },
    espanso_config::matches::Value::String(s) => espanso_render::Value::String(s),
    espanso_config::matches::Value::Array(v) => {
      espanso_render::Value::Array(v.into_iter().map(convert_value).collect())
    }
    espanso_config::matches::Value::Object(params) => {
      espanso_render::Value::Object(convert_params(params))
    }
  }
}

impl<'a> Renderer<'a> for RendererAdapter<'a> {
  fn render(&'a self, match_id: i32, trigger_vars: HashMap<String, String>) -> anyhow::Result<String> {
    if let Some(Some(template)) = self.template_map.get(&match_id) {
      let (config, match_set) = self.config_provider.active();

      let mut context_cache = self.context_cache.borrow_mut();
      let context = context_cache.entry(config.id()).or_insert(generate_context(&match_set, &self.template_map, &self.global_vars_map));

      // TODO: calculate the casing style instead of hardcoding it
      let options = RenderOptions {
        casing_style: CasingStyle::None,
      };

      // If some trigger vars are specified, augment the template with them
      let augmented_template = if !trigger_vars.is_empty() {
        let mut augmented = template.clone();
        for (name, value) in trigger_vars {
          let mut params = espanso_render::Params::new();
          params.insert("echo".to_string(), Value::String(value));
          augmented.vars.push(Variable {
              name, 
              var_type: "echo".to_string(), 
              params, 
          })
        }
        Some(augmented)
      } else {
        None
      };

      let template = if let Some(augmented) = augmented_template.as_ref() {
        augmented
      } else {
        template
      };

      match self.renderer.render(template, context, &options) {
        espanso_render::RenderResult::Success(body) => Ok(body),
        espanso_render::RenderResult::Aborted => Err(RendererError::Aborted.into()),
        espanso_render::RenderResult::Error(err) => Err(RendererError::RenderingError(err).into()), 
      }
    } else {
      Err(RendererError::NotFound.into())
    }
  }
}

