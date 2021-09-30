/*
 * This file is part of esp name: (), var_type: (), params: ()anso.
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

use std::collections::{HashMap, HashSet};

use crate::{
  CasingStyle, Context, Extension, ExtensionOutput, ExtensionResult, RenderOptions, RenderResult,
  Renderer, Scope, Template, Value, Variable,
};
use anyhow::Result;
use log::{error, warn};
use regex::{Captures, Regex};
use thiserror::Error;
use util::get_body_variable_names;

mod util;

lazy_static! {
  pub(crate) static ref VAR_REGEX: Regex =
    Regex::new(r"\{\{\s*((?P<name>\w+)(\.(?P<subname>(\w+)))?)\s*\}\}").unwrap();
  static ref WORD_REGEX: Regex = Regex::new(r"(\w+)").unwrap();
}

pub(crate) struct DefaultRenderer<'a> {
  extensions: HashMap<String, &'a dyn Extension>,
}

impl<'a> DefaultRenderer<'a> {
  pub fn new(extensions: Vec<&'a dyn Extension>) -> Self {
    let extensions = extensions
      .into_iter()
      .map(|ext| (ext.name().to_string(), ext))
      .collect();
    Self { extensions }
  }
}

impl<'a> Renderer for DefaultRenderer<'a> {
  fn render(
    &self,
    template: &Template,
    context: &Context,
    options: &RenderOptions,
  ) -> RenderResult {
    let body = if VAR_REGEX.is_match(&template.body) {
      // In order to define a variable evaluation order, we first need to find
      // the global variables that are being used but for which an explicit order
      // is not defined.
      let body_variable_names = get_body_variable_names(&template.body);
      let local_variable_names: HashSet<&str> =
        template.vars.iter().map(|var| var.name.as_str()).collect();
      let missing_global_variable_names: HashSet<&str> = body_variable_names
        .difference(&local_variable_names)
        .copied()
        .collect();
      let missing_global_variables: Vec<&Variable> = context
        .global_vars
        .iter()
        .copied()
        .filter(|global_var| missing_global_variable_names.contains(&*global_var.name))
        .collect();

      // Convert "global" variable type aliases when needed
      let local_variables: Vec<&Variable> =
        if template.vars.iter().any(|var| var.var_type == "global") {
          let global_vars: HashMap<&str, &Variable> = context
            .global_vars
            .iter()
            .map(|var| (&*var.name, *var))
            .collect();
          template
            .vars
            .iter()
            .filter_map(|var| {
              if var.var_type == "global" {
                global_vars.get(&*var.name).copied()
              } else {
                Some(var)
              }
            })
            .collect()
        } else {
          template.vars.iter().map(|var| var).collect()
        };

      // The implicit global variables will be evaluated first, followed by the local vars
      let mut variables: Vec<&Variable> = Vec::new();
      variables.extend(missing_global_variables);
      variables.extend(local_variables.iter());

      // Compute the variable outputs
      let mut scope = Scope::new();
      for variable in variables {
        if variable.var_type == "match" {
          // Recursive call
          // Call render recursively
          if let Some(sub_template) = get_matching_template(variable, context.templates.as_slice())
          {
            match self.render(sub_template, context, options) {
              RenderResult::Success(output) => {
                scope.insert(&variable.name, ExtensionOutput::Single(output));
              }
              result => return result,
            }
          } else {
            error!("unable to find sub-match: {}", variable.name);
            return RenderResult::Error(RendererError::MissingSubMatch.into());
          }
        } else if let Some(extension) = self.extensions.get(&variable.var_type) {
          match extension.calculate(context, &scope, &variable.params) {
            ExtensionResult::Success(output) => {
              scope.insert(&variable.name, output);
            }
            ExtensionResult::Aborted => {
              warn!(
                "rendering was aborted by extension: {}, on var: {}",
                variable.var_type, variable.name
              );
              return RenderResult::Aborted;
            }
            ExtensionResult::Error(err) => {
              warn!(
                "extension '{}' on var: '{}' reported an error: {}",
                variable.var_type, variable.name, err
              );
              return RenderResult::Error(err);
            }
          }
        } else {
          error!(
            "no extension found for variable type: {}",
            variable.var_type
          );
        }
      }

      // Replace the variables
      match render_variables(&template.body, &scope) {
        Ok(output) => output,
        Err(error) => {
          return RenderResult::Error(error.into());
        }
      }
    } else {
      template.body.clone()
    };

    // Process the casing style
    let body_with_casing = match options.casing_style {
      CasingStyle::None => body,
      CasingStyle::Uppercase => body.to_uppercase(),
      CasingStyle::Capitalize => {
        // Capitalize the first letter
        let mut v: Vec<char> = body.chars().collect();
        v[0] = v[0].to_uppercase().next().unwrap();
        v.into_iter().collect()
      }
      CasingStyle::CapitalizeWords => {
        // Capitalize the first letter of each word
        WORD_REGEX
          .replace_all(&body, |caps: &Captures| {
            if let Some(word_match) = caps.get(0) {
              let mut v: Vec<char> = word_match.as_str().chars().collect();
              v[0] = v[0].to_uppercase().next().unwrap();
              let capitalized_word: String = v.into_iter().collect();
              capitalized_word
            } else {
              "".to_string()
            }
          })
          .to_string()
      }
    };

    RenderResult::Success(body_with_casing)
  }
}

// TODO: test
pub(crate) fn render_variables(body: &str, scope: &Scope) -> Result<String> {
  let mut replacing_error = None;
  let output = VAR_REGEX
    .replace_all(&body, |caps: &Captures| {
      let var_name = caps.name("name").unwrap().as_str();
      let var_subname = caps.name("subname");
      match scope.get(var_name) {
        Some(output) => match output {
          ExtensionOutput::Single(output) => output,
          ExtensionOutput::Multiple(results) => match var_subname {
            Some(var_subname) => {
              let var_subname = var_subname.as_str();
              results.get(var_subname).map_or("", |value| &*value)
            }
            None => {
              error!(
                "nested name missing from multi-value variable: {}",
                var_name
              );
              replacing_error = Some(RendererError::MissingVariable(format!(
                "nested name missing from multi-value variable: {}",
                var_name
              )));
              ""
            }
          },
        },
        None => {
          replacing_error = Some(RendererError::MissingVariable(format!(
            "variable '{}' is missing",
            var_name
          )));
          ""
        }
      }
    })
    .to_string();
  
  if let Some(error) = replacing_error {
    return Err(error.into());
  }

  Ok(output)
}

fn get_matching_template<'a>(
  variable: &Variable,
  templates: &'a [&Template],
) -> Option<&'a Template> {
  // Find matching template
  let id = variable.params.get("trigger")?;
  if let Value::String(id) = id {
    templates
      .iter()
      .find(|template| template.ids.contains(id))
      .copied()
  } else {
    None
  }
}

#[derive(Error, Debug)]
pub enum RendererError {
  #[error("missing variable: `{0}`")]
  MissingVariable(String),

  #[error("missing sub match")]
  MissingSubMatch,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Params;
  use std::iter::FromIterator;

  struct MockExtension {}

  impl Extension for MockExtension {
    fn name(&self) -> &str {
      "mock"
    }

    fn calculate(
      &self,
      _context: &Context,
      scope: &Scope,
      params: &crate::Params,
    ) -> ExtensionResult {
      if let Some(value) = params.get("echo") {
        if let Value::String(string) = value {
          return ExtensionResult::Success(ExtensionOutput::Single(string.clone()));
        }
      }
      // If the "read" param is present, echo the value of the corresponding result in the scope
      if let Some(value) = params.get("read") {
        if let Value::String(string) = value {
          if let Some(ExtensionOutput::Single(value)) = scope.get(string.as_str()) {
            return ExtensionResult::Success(ExtensionOutput::Single(value.to_string()));
          }
        }
      }
      if params.get("abort").is_some() {
        return ExtensionResult::Aborted;
      }
      if params.get("error").is_some() {
        return ExtensionResult::Error(RendererError::MissingVariable("missing".to_string()).into());
      }
      ExtensionResult::Aborted
    }
  }

  pub fn get_renderer() -> impl Renderer {
    DefaultRenderer::new(vec![&MockExtension {}])
  }

  pub fn template_for_str(str: &str) -> Template {
    Template {
      ids: vec!["id".to_string()],
      body: str.to_string(),
      vars: Vec::new(),
    }
  }

  pub fn template(body: &str, vars: &[(&str, &str)]) -> Template {
    let vars = vars
      .iter()
      .map(|(name, value)| Variable {
        name: (*name).to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(
          vec![("echo".to_string(), Value::String((*value).to_string()))].into_iter(),
        ),
      })
      .collect();
    Template {
      ids: vec!["id".to_string()],
      body: body.to_string(),
      vars,
    }
  }

  #[test]
  fn no_variable_no_styling() {
    let renderer = get_renderer();
    let res = renderer.render(
      &template_for_str("plain body"),
      &Default::default(),
      &Default::default(),
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "plain body"));
  }

  #[test]
  fn no_variable_capitalize() {
    let renderer = get_renderer();
    let res = renderer.render(
      &template_for_str("plain body"),
      &Default::default(),
      &RenderOptions {
        casing_style: CasingStyle::Capitalize,
      },
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "Plain body"));
  }

  #[test]
  fn no_variable_capitalize_words() {
    let renderer = get_renderer();
    let res = renderer.render(
      &template_for_str("ordinary least squares, with other.punctuation !Marks"),
      &Default::default(),
      &RenderOptions {
        casing_style: CasingStyle::CapitalizeWords,
      },
    );
    assert!(
      matches!(res, RenderResult::Success(str) if str == "Ordinary Least Squares, With Other.Punctuation !Marks")
    );
  }

  #[test]
  fn no_variable_uppercase() {
    let renderer = get_renderer();
    let res = renderer.render(
      &template_for_str("plain body"),
      &Default::default(),
      &RenderOptions {
        casing_style: CasingStyle::Uppercase,
      },
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "PLAIN BODY"));
  }

  #[test]
  fn basic_variable() {
    let renderer = get_renderer();
    let template = template("hello {{var}}", &[("var", "world")]);
    let res = renderer.render(&template, &Default::default(), &Default::default());
    assert!(matches!(res, RenderResult::Success(str) if str == "hello world"));
  }

  #[test]
  fn missing_variable() {
    let renderer = get_renderer();
    let template = template_for_str("hello {{var}}");
    let res = renderer.render(&template, &Default::default(), &Default::default());
    assert!(matches!(res, RenderResult::Error(_)));
  }

  #[test]
  fn global_variable() {
    let renderer = get_renderer();
    let template = template("hello {{var}}", &[]);
    let res = renderer.render(
      &template,
      &Context {
        global_vars: vec![&Variable {
          name: "var".to_string(),
          var_type: "mock".to_string(),
          params: Params::from_iter(vec![(
            "echo".to_string(),
            Value::String("world".to_string()),
          )]),
        }],
        ..Default::default()
      },
      &Default::default(),
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "hello world"));
  }

  #[test]
  fn global_variable_explicit_ordering() {
    let renderer = get_renderer();
    let template = Template {
      body: "hello {{var}} {{local}}".to_string(),
      vars: vec![
        Variable {
          name: "local".to_string(),
          var_type: "mock".to_string(),
          params: Params::from_iter(
            vec![("echo".to_string(), Value::String("Bob".to_string()))].into_iter(),
          ),
        },
        Variable {
          name: "var".to_string(),
          var_type: "global".to_string(),
          ..Default::default()
        },
      ],
      ..Default::default()
    };
    let res = renderer.render(
      &template,
      &Context {
        global_vars: vec![&Variable {
          name: "var".to_string(),
          var_type: "mock".to_string(),
          params: Params::from_iter(vec![(
            "read".to_string(),
            Value::String("local".to_string()),
          )]),
        }],
        ..Default::default()
      },
      &Default::default(),
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "hello Bob Bob"));
  }

  #[test]
  fn nested_match() {
    let renderer = get_renderer();
    let template = Template {
      body: "hello {{var}}".to_string(),
      vars: vec![Variable {
        name: "var".to_string(),
        var_type: "match".to_string(),
        params: Params::from_iter(
          vec![("trigger".to_string(), Value::String("nested".to_string()))].into_iter(),
        ),
      }],
      ..Default::default()
    };
    let nested_template = Template {
      ids: vec!["nested".to_string()],
      body: "world".to_string(),
      ..Default::default()
    };
    let res = renderer.render(
      &template,
      &Context {
        templates: vec![&nested_template],
        ..Default::default()
      },
      &Default::default(),
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "hello world"));
  }

  #[test]
  fn missing_nested_match() {
    let renderer = get_renderer();
    let template = Template {
      body: "hello {{var}}".to_string(),
      vars: vec![Variable {
        name: "var".to_string(),
        var_type: "match".to_string(),
        params: Params::from_iter(
          vec![("trigger".to_string(), Value::String("nested".to_string()))].into_iter(),
        ),
      }],
      ..Default::default()
    };
    let res = renderer.render(
      &template,
      &Context {
        ..Default::default()
      },
      &Default::default(),
    );
    assert!(matches!(res, RenderResult::Error(_)));
  }

  #[test]
  fn extension_aborting_propagates() {
    let renderer = get_renderer();
    let template = Template {
      body: "hello {{var}}".to_string(),
      vars: vec![Variable {
        name: "var".to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(vec![("abort".to_string(), Value::Null)].into_iter()),
      }],
      ..Default::default()
    };
    let res = renderer.render(&template, &Default::default(), &Default::default());
    assert!(matches!(res, RenderResult::Aborted));
  }

  #[test]
  fn extension_error_propagates() {
    let renderer = get_renderer();
    let template = Template {
      body: "hello {{var}}".to_string(),
      vars: vec![Variable {
        name: "var".to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(vec![("error".to_string(), Value::Null)].into_iter()),
      }],
      ..Default::default()
    };
    let res = renderer.render(&template, &Default::default(), &Default::default());
    assert!(matches!(res, RenderResult::Error(_)));
  }
}
