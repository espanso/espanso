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

#[macro_use]
extern crate lazy_static;

use enum_as_inner::EnumAsInner;
use std::collections::HashMap;

pub mod extension;
mod renderer;

pub trait Renderer {
  fn render(&self, template: &Template, context: &Context, options: &RenderOptions)
    -> RenderResult;
}

pub fn create(extensions: Vec<Box<dyn Extension>>) -> impl Renderer {
  renderer::DefaultRenderer::new(extensions)
}

#[derive(Debug)]
pub enum RenderResult {
  Success(String),
  Aborted,
  Error(anyhow::Error),
}

pub struct Context<'a> {
  pub global_vars: Vec<&'a Variable>,
  pub templates: Vec<&'a Template>,
}

impl<'a> Default for Context<'a> {
  fn default() -> Self {
    Self {
      global_vars: Vec::new(),
      templates: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderOptions {
  casing_style: CasingStyle,
}

impl Default for RenderOptions {
  fn default() -> Self {
    Self {
      casing_style: CasingStyle::None,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CasingStyle {
  None,
  Capitalize,
  CapitalizeWords,
  Uppercase,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Template {
  ids: Vec<String>,
  body: String,
  vars: Vec<Variable>,
}

impl Default for Template {
  fn default() -> Self {
    Self {
      ids: Vec::new(),
      body: "".to_string(),
      vars: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
  name: String,
  var_type: String,
  params: Params,
}

impl Default for Variable {
  fn default() -> Self {
    Self {
      name: "".to_string(),
      var_type: "".to_string(),
      params: Params::new(),
    }
  }
}

pub type Params = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq, EnumAsInner)]
pub enum Value {
  Null,
  Bool(bool),
  Number(Number),
  String(String),
  Array(Vec<Value>),
  Object(HashMap<String, Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
  Integer(i64),
  Float(f64),
}

pub trait Extension {
  fn name(&self) -> &str;
  fn calculate(&self, context: &Context, scope: &Scope, params: &Params) -> ExtensionResult;
}

pub type Scope<'a> = HashMap<&'a str, ExtensionOutput>;

#[derive(Debug, PartialEq)]
pub enum ExtensionOutput {
  Single(String),
  Multiple(HashMap<String, String>),
}

#[derive(Debug, EnumAsInner)]
pub enum ExtensionResult {
  Success(ExtensionOutput),
  Aborted,
  Error(anyhow::Error),
}
