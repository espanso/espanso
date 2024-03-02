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

use std::{borrow::Cow, collections::HashMap};

use crate::{
  CasingStyle, Context, Extension, ExtensionOutput, ExtensionResult, RenderOptions, RenderResult,
  Renderer, Scope, Template, Value, Variable,
};
use lazy_static::lazy_static;
use log::{error, warn};
use regex::{Captures, Regex};
use thiserror::Error;

use self::util::{inject_variables_into_params, render_variables};

mod resolve;
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
          template.vars.iter().collect()
        };

      // Here we execute a graph dependency resolution algorithm to determine a valid
      // evaluation order for variables.
      let variables = match resolve::resolve_evaluation_order(
        &template.body,
        &local_variables,
        &context.global_vars,
      ) {
        Ok(variables) => variables,
        Err(err) => return RenderResult::Error(err),
      };

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
          let variable_params = if variable.inject_vars {
            match inject_variables_into_params(&variable.params, &scope) {
              Ok(augmented_params) => Cow::Owned(augmented_params),
              Err(err) => {
                error!(
                  "unable to inject variables into params of variable '{}': {}",
                  variable.name, err
                );

                if variable.var_type == "form" {
                  if let Some(RendererError::MissingVariable(_)) =
                    err.downcast_ref::<RendererError>()
                  {
                    log_new_form_syntax_tip();
                  }
                }

                return RenderResult::Error(err);
              }
            }
          } else {
            Cow::Borrowed(&variable.params)
          };

          match extension.calculate(context, &scope, &variable_params) {
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
          return RenderResult::Error(error);
        }
      }
    } else {
      template.body.clone()
    };

    let body = util::unescape_variable_inections(&body);

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
              String::new()
            }
          })
          .to_string()
      }
    };

    RenderResult::Success(body_with_casing)
  }
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

fn log_new_form_syntax_tip() {
  error!("");
  error!("TIP: This error might be happening because since version 2.1.0-alpha, Espanso changed");
  error!("the syntax to define form controls. Instead of `{{{{control}}}}` you need to use");
  error!("[[control]] (using square brackets instead of curly brackets).");
  error!("");
  error!("For example, if you have a form defined like the following:");
  error!("  - trigger: test");
  error!("    form: |");
  error!("      Hi {{{{name}}}}!");
  error!("");
  error!("You'll need to replace it with:");
  error!("  - trigger: test");
  error!("    form: |");
  error!("      Hi [[name]]!");
  error!("");
}

#[derive(Error, Debug)]
pub enum RendererError {
  #[error("missing variable: `{0}`")]
  MissingVariable(String),

  #[error("missing sub match")]
  MissingSubMatch,

  #[error("circular dependency: `{0}` -> `{1}`")]
  CircularDependency(String, String),
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
      if let Some(Value::String(string)) = params.get("echo") {
        return ExtensionResult::Success(ExtensionOutput::Single(string.clone()));
      }
      if let (Some(Value::String(name)), Some(Value::String(value))) =
        (params.get("name"), params.get("value"))
      {
        let mut map = HashMap::new();
        map.insert(name.to_string(), value.to_string());
        return ExtensionResult::Success(ExtensionOutput::Multiple(map));
      }
      // If the "read" param is present, echo the value of the corresponding result in the scope
      if let Some(Value::String(string)) = params.get("read") {
        if let Some(ExtensionOutput::Single(value)) = scope.get(string.as_str()) {
          return ExtensionResult::Success(ExtensionOutput::Single(value.to_string()));
        }
      }
      if params.get("abort").is_some() {
        return ExtensionResult::Aborted;
      }
      if params.get("error").is_some() {
        return ExtensionResult::Error(
          RendererError::MissingVariable("missing".to_string()).into(),
        );
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
        params: vec![("echo".to_string(), Value::String((*value).to_string()))]
          .into_iter()
          .collect::<Params>(),
        ..Default::default()
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
      &Context::default(),
      &RenderOptions::default(),
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "plain body"));
  }

  #[test]
  fn no_variable_capitalize() {
    let renderer = get_renderer();
    let res = renderer.render(
      &template_for_str("plain body"),
      &Context::default(),
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
      &Context::default(),
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
      &Context::default(),
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
    let res = renderer.render(&template, &Context::default(), &RenderOptions::default());
    assert!(matches!(res, RenderResult::Success(str) if str == "hello world"));
  }

  #[test]
  fn dict_variable_variable() {
    let renderer = get_renderer();
    let template = Template {
      body: "hello {{var.nested}}".to_string(),
      vars: vec![Variable {
        name: "var".to_string(),
        var_type: "mock".to_string(),
        params: vec![
          ("name".to_string(), Value::String("nested".to_string())),
          ("value".to_string(), Value::String("dict".to_string())),
        ]
        .into_iter()
        .collect::<Params>(),
        ..Default::default()
      }],
      ..Default::default()
    };
    let res = renderer.render(&template, &Context::default(), &RenderOptions::default());
    assert!(matches!(res, RenderResult::Success(str) if str == "hello dict"));
  }

  #[test]
  fn missing_variable() {
    let renderer = get_renderer();
    let template = template_for_str("hello {{var}}");
    let res = renderer.render(&template, &Context::default(), &RenderOptions::default());
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
          ..Default::default()
        }],
        ..Default::default()
      },
      &RenderOptions::default(),
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "hello world"));
  }

  #[test]
  fn global_dict_variable() {
    let renderer = get_renderer();
    let template = template("hello {{var.nested}}", &[]);
    let res = renderer.render(
      &template,
      &Context {
        global_vars: vec![&Variable {
          name: "var".to_string(),
          var_type: "mock".to_string(),
          params: vec![
            ("name".to_string(), Value::String("nested".to_string())),
            ("value".to_string(), Value::String("dict".to_string())),
          ]
          .into_iter()
          .collect::<Params>(),
          ..Default::default()
        }],
        ..Default::default()
      },
      &RenderOptions::default(),
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "hello dict"));
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
          params: vec![("echo".to_string(), Value::String("Bob".to_string()))]
            .into_iter()
            .collect::<Params>(),
          ..Default::default()
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
          ..Default::default()
        }],
        ..Default::default()
      },
      &RenderOptions::default(),
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "hello Bob Bob"));
  }

  #[test]
  fn nested_global_variable() {
    let renderer = get_renderer();
    let template = template("hello {{var2}}", &[]);
    let res = renderer.render(
      &template,
      &Context {
        global_vars: vec![
          &Variable {
            name: "var".to_string(),
            var_type: "mock".to_string(),
            params: Params::from_iter(vec![(
              "echo".to_string(),
              Value::String("world".to_string()),
            )]),
            ..Default::default()
          },
          &Variable {
            name: "var2".to_string(),
            var_type: "mock".to_string(),
            params: Params::from_iter(vec![(
              "echo".to_string(),
              Value::String("{{var}}".to_string()),
            )]),
            ..Default::default()
          },
        ],
        ..Default::default()
      },
      &RenderOptions::default(),
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "hello world"));
  }

  #[test]
  fn nested_global_variable_circular_dependency_should_fail() {
    let renderer = get_renderer();
    let template = template("hello {{var}}", &[]);
    let res = renderer.render(
      &template,
      &Context {
        global_vars: vec![
          &Variable {
            name: "var".to_string(),
            var_type: "mock".to_string(),
            params: Params::from_iter(vec![(
              "echo".to_string(),
              Value::String("{{var2}}".to_string()),
            )]),
            ..Default::default()
          },
          &Variable {
            name: "var2".to_string(),
            var_type: "mock".to_string(),
            params: Params::from_iter(vec![(
              "echo".to_string(),
              Value::String("{{var3}}".to_string()),
            )]),
            ..Default::default()
          },
          &Variable {
            name: "var3".to_string(),
            var_type: "mock".to_string(),
            params: Params::from_iter(vec![(
              "echo".to_string(),
              Value::String("{{var}}".to_string()),
            )]),
            ..Default::default()
          },
        ],
        ..Default::default()
      },
      &RenderOptions::default(),
    );
    assert!(matches!(res, RenderResult::Error(_)));
  }

  #[test]
  fn global_variable_depends_on() {
    let renderer = get_renderer();
    let template = template("hello {{var}}", &[]);
    let res = renderer.render(
      &template,
      &Context {
        global_vars: vec![
          &Variable {
            name: "var".to_string(),
            var_type: "mock".to_string(),
            params: Params::from_iter(vec![(
              "echo".to_string(),
              Value::String("world".to_string()),
            )]),
            depends_on: vec!["var2".to_string()],
            ..Default::default()
          },
          &Variable {
            name: "var2".to_string(),
            var_type: "mock".to_string(),
            params: Params::from_iter(vec![("abort".to_string(), Value::Null)]),
            ..Default::default()
          },
        ],
        ..Default::default()
      },
      &RenderOptions::default(),
    );
    assert!(matches!(res, RenderResult::Aborted));
  }

  #[test]
  fn local_variable_explicit_ordering() {
    let renderer = get_renderer();
    let template = Template {
      body: "hello {{var}}".to_string(),
      vars: vec![Variable {
        name: "var".to_string(),
        var_type: "mock".to_string(),
        params: vec![("echo".to_string(), Value::String("something".to_string()))]
          .into_iter()
          .collect::<Params>(),
        depends_on: vec!["global".to_string()],
        ..Default::default()
      }],
      ..Default::default()
    };
    let res = renderer.render(
      &template,
      &Context {
        global_vars: vec![&Variable {
          name: "global".to_string(),
          var_type: "mock".to_string(),
          params: Params::from_iter(vec![("abort".to_string(), Value::Null)]),
          ..Default::default()
        }],
        ..Default::default()
      },
      &RenderOptions::default(),
    );
    assert!(matches!(res, RenderResult::Aborted));
  }

  #[test]
  fn nested_match() {
    let renderer = get_renderer();
    let template = Template {
      body: "hello {{var}}".to_string(),
      vars: vec![Variable {
        name: "var".to_string(),
        var_type: "match".to_string(),
        params: vec![("trigger".to_string(), Value::String("nested".to_string()))]
          .into_iter()
          .collect::<Params>(),
        ..Default::default()
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
      &RenderOptions::default(),
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
        params: vec![("trigger".to_string(), Value::String("nested".to_string()))]
          .into_iter()
          .collect::<Params>(),
        ..Default::default()
      }],
      ..Default::default()
    };
    let res = renderer.render(
      &template,
      &Context {
        ..Default::default()
      },
      &RenderOptions::default(),
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
        params: vec![("abort".to_string(), Value::Null)]
          .into_iter()
          .collect::<Params>(),
        ..Default::default()
      }],
      ..Default::default()
    };
    let res = renderer.render(&template, &Context::default(), &RenderOptions::default());
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
        params: vec![("error".to_string(), Value::Null)]
          .into_iter()
          .collect::<Params>(),
        ..Default::default()
      }],
      ..Default::default()
    };
    let res = renderer.render(&template, &Context::default(), &RenderOptions::default());
    assert!(matches!(res, RenderResult::Error(_)));
  }

  #[test]
  fn variable_injection() {
    let renderer = get_renderer();
    let mut template = template_for_str("hello {{fullname}}");
    template.vars = vec![
      Variable {
        name: "firstname".to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(vec![(
          "echo".to_string(),
          Value::String("John".to_string()),
        )]),
        ..Default::default()
      },
      Variable {
        name: "lastname".to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(vec![(
          "echo".to_string(),
          Value::String("Snow".to_string()),
        )]),
        ..Default::default()
      },
      Variable {
        name: "fullname".to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(vec![(
          "echo".to_string(),
          Value::String("{{firstname}} {{lastname}}".to_string()),
        )]),
        inject_vars: true,
        ..Default::default()
      },
    ];

    let res = renderer.render(&template, &Context::default(), &RenderOptions::default());
    assert!(matches!(res, RenderResult::Success(str) if str == "hello John Snow"));
  }

  #[test]
  fn disable_variable_injection() {
    let renderer = get_renderer();
    let mut template = template_for_str("hello {{second}}");
    template.vars = vec![
      Variable {
        name: "first".to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(vec![("echo".to_string(), Value::String("one".to_string()))]),
        ..Default::default()
      },
      Variable {
        name: "second".to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(vec![(
          "echo".to_string(),
          Value::String("{{first}} two".to_string()),
        )]),
        inject_vars: false,
        ..Default::default()
      },
    ];

    let res = renderer.render(&template, &Context::default(), &RenderOptions::default());
    assert!(matches!(res, RenderResult::Success(str) if str == "hello {{first}} two"));
  }

  #[test]
  fn escaped_variable_injection() {
    let renderer = get_renderer();
    let mut template = template_for_str("hello {{second}}");
    template.vars = vec![
      Variable {
        name: "first".to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(vec![("echo".to_string(), Value::String("one".to_string()))]),
        ..Default::default()
      },
      Variable {
        name: "second".to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(vec![(
          "echo".to_string(),
          Value::String("\\{\\{first\\}\\} two".to_string()),
        )]),
        ..Default::default()
      },
    ];

    let res = renderer.render(&template, &Context::default(), &RenderOptions::default());
    assert!(matches!(res, RenderResult::Success(str) if str == "hello {{first}} two"));
  }

  #[test]
  fn variable_injection_missing_var() {
    let renderer = get_renderer();
    let mut template = template_for_str("hello {{second}}");
    template.vars = vec![Variable {
      name: "second".to_string(),
      var_type: "mock".to_string(),
      params: Params::from_iter(vec![(
        "echo".to_string(),
        Value::String("the next is {{missing}}".to_string()),
      )]),
      ..Default::default()
    }];

    let res = renderer.render(&template, &Context::default(), &RenderOptions::default());
    assert!(matches!(res, RenderResult::Error(_)));
  }

  #[test]
  fn variable_injection_with_global_variable() {
    let renderer = get_renderer();
    let mut template = template_for_str("hello {{output}}");
    template.vars = vec![
      Variable {
        name: "var".to_string(),
        var_type: "global".to_string(),
        ..Default::default()
      },
      Variable {
        name: "output".to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(vec![(
          "echo".to_string(),
          Value::String("{{var}}".to_string()),
        )]),
        ..Default::default()
      },
    ];

    let res = renderer.render(
      &template,
      &Context {
        global_vars: vec![&Variable {
          name: "var".to_string(),
          var_type: "mock".to_string(),
          params: Params::from_iter(vec![(
            "echo".to_string(),
            Value::String("global".to_string()),
          )]),
          ..Default::default()
        }],
        ..Default::default()
      },
      &RenderOptions::default(),
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "hello global"));
  }

  #[test]
  fn variable_injection_local_var_takes_precedence_over_global() {
    let renderer = get_renderer();
    let mut template = template_for_str("hello {{output}}");
    template.vars = vec![
      Variable {
        name: "var".to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(vec![(
          "echo".to_string(),
          Value::String("local".to_string()),
        )]),
        ..Default::default()
      },
      Variable {
        name: "output".to_string(),
        var_type: "mock".to_string(),
        params: Params::from_iter(vec![(
          "echo".to_string(),
          Value::String("{{var}}".to_string()),
        )]),
        ..Default::default()
      },
    ];

    let res = renderer.render(
      &template,
      &Context {
        global_vars: vec![&Variable {
          name: "var".to_string(),
          var_type: "mock".to_string(),
          params: Params::from_iter(vec![(
            "echo".to_string(),
            Value::String("global".to_string()),
          )]),
          ..Default::default()
        }],
        ..Default::default()
      },
      &RenderOptions::default(),
    );
    assert!(matches!(res, RenderResult::Success(str) if str == "hello local"));
  }

  #[test]
  fn variable_escape() {
    let renderer = get_renderer();
    let template = template("hello \\{\\{var\\}\\}", &[("var", "world")]);
    let res = renderer.render(&template, &Context::default(), &RenderOptions::default());
    assert!(matches!(res, RenderResult::Success(str) if str == "hello {{var}}"));
  }
}
