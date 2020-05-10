/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

use serde_yaml::{Value};
use std::collections::{HashMap, HashSet};
use regex::{Regex, Captures};
use log::{warn, error};
use super::*;
use crate::matcher::{Match, MatchContentType};
use crate::config::Configs;
use crate::extension::Extension;

lazy_static! {
    static ref VAR_REGEX: Regex = Regex::new("\\{\\{\\s*(?P<name>\\w+)\\s*\\}\\}").unwrap();
    static ref UNKNOWN_VARIABLE : String = "".to_string();
}

pub struct DefaultRenderer {
    extension_map: HashMap<String, Box<dyn Extension>>,

    // Regex used to identify matches (and arguments) in passive expansions
    passive_match_regex: Regex,
}

impl DefaultRenderer {
    pub fn new(extensions: Vec<Box<dyn Extension>>, config: Configs) -> DefaultRenderer {
        // Register all the extensions
        let mut extension_map = HashMap::new();
        for extension in extensions.into_iter() {
            extension_map.insert(extension.name(), extension);
        }

        // Compile the regexes
        let passive_match_regex = Regex::new(&config.passive_match_regex)
                                        .unwrap_or_else(|e| {
                                            panic!("Invalid passive match regex: {:?}", e);
                                        });

        DefaultRenderer{
            extension_map,
            passive_match_regex,
        }
    }

    fn find_match(config: &Configs, trigger: &str) -> Option<(Match, usize)> {
        let mut result = None;

        // TODO: if performances become a problem, implement a more efficient lookup
        for m in config.matches.iter() {
            for (trigger_offset, m_trigger) in m.triggers.iter().enumerate() {
                if m_trigger == trigger {
                    result = Some((m.clone(), trigger_offset));
                    break;
                }
            }

        }

        result
    }
}

impl super::Renderer for DefaultRenderer {
    fn render_match(&self, m: &Match, trigger_offset: usize, config: &Configs, args: Vec<String>) -> RenderResult {
        // Manage the different types of matches
        match &m.content {
            // Text Match
            MatchContentType::Text(content) => {
                // Find all the variables that are required by the current match
                let mut target_vars = HashSet::new();

                for caps in VAR_REGEX.captures_iter(&content.replace) {
                    let var_name = caps.name("name").unwrap().as_str();
                    target_vars.insert(var_name.to_owned());
                };

                let target_string = if target_vars.len() > 0 {
                    let mut output_map = HashMap::new();

                    // Cycle through both the local and global variables
                    for variable in config.global_vars.iter().chain(&content.vars) {
                        // Skip all non-required variables
                        if !target_vars.contains(&variable.name) {
                            continue;
                        }

                        // In case of variables of type match, we need to recursively call
                        // the render function
                        if variable.var_type == "match" {
                            // Extract the match trigger from the variable params
                            let trigger = variable.params.get(&Value::from("trigger"));
                            if trigger.is_none() {
                                warn!("Missing param 'trigger' in match variable: {}", variable.name);
                                continue;
                            }
                            let trigger = trigger.unwrap();

                            // Find the given match from the active configs
                            let inner_match = DefaultRenderer::find_match(config, trigger.as_str().unwrap_or(""));

                            if inner_match.is_none() {
                                warn!("Could not find inner match with trigger: '{}'", trigger.as_str().unwrap_or("undefined"));
                                continue
                            }

                            let (inner_match, trigger_offset) = inner_match.unwrap();

                            // Render the inner match
                            // TODO: inner arguments
                            let result = self.render_match(&inner_match, trigger_offset, config, vec![]);

                            // Inner matches are only supported for text-expansions, warn the user otherwise
                            match result {
                                RenderResult::Text(inner_content) => {
                                    output_map.insert(variable.name.clone(), inner_content);
                                },
                                _ => {
                                    warn!("Inner matches must be of TEXT type. Mixing images is not supported yet.")
                                },
                            }
                        }else{  // Normal extension variables
                            let extension = self.extension_map.get(&variable.var_type);
                            if let Some(extension) = extension {
                                let ext_out = extension.calculate(&variable.params, &args);
                                if let Some(output) = ext_out {
                                    output_map.insert(variable.name.clone(), output);
                                }else{
                                    output_map.insert(variable.name.clone(), "".to_owned());
                                    warn!("Could not generate output for variable: {}", variable.name);
                                }
                            }else{
                                error!("No extension found for variable type: {}", variable.var_type);
                            }
                        }
                    }

                    // Replace the variables
                    let result = VAR_REGEX.replace_all(&content.replace, |caps: &Captures| {
                        let var_name = caps.name("name").unwrap().as_str();
                        let output = output_map.get(var_name);
                        output.unwrap_or(&UNKNOWN_VARIABLE)
                    });

                    result.to_string()
                }else{  // No variables, simple text substitution
                    content.replace.clone()
                };

                // Unescape any brackets (needed to be able to insert double brackets in replacement
                // text, without triggering the variable system). See issue #187
                let target_string = target_string.replace("\\{", "{")
                                                        .replace("\\}", "}");

                // Render any argument that may be present
                let target_string = utils::render_args(&target_string, &args);

                // Handle case propagation
                let target_string = if m.propagate_case {
                    let trigger = &m.triggers[trigger_offset];

                    // The check should be carried out from the position of the first
                    // alphabetic letter
                    // See issue #244
                    let first_alphabetic = trigger.chars().position(|c| {
                        c.is_alphabetic()
                    }).unwrap_or(0);

                    let first_char = trigger.chars().nth(first_alphabetic);
                    let second_char  = trigger.chars().nth(first_alphabetic + 1);
                    let mode: i32 = if let Some(first_char) = first_char {
                        if first_char.is_uppercase() {
                            if let Some(second_char) = second_char {
                                if second_char.is_uppercase() {
                                    2 // Full CAPITALIZATION
                                }else{
                                    1 // Only first letter capitalized: Capitalization
                                }
                            }else{
                                2  // Single char, defaults to full CAPITALIZATION
                            }
                        }else{
                            0  // Lowercase, no action
                        }
                    }else{
                        0
                    };

                    match mode {
                        1 => {
                            // Capitalize the first letter
                            let mut v: Vec<char> = target_string.chars().collect();
                            v[0] = v[0].to_uppercase().nth(0).unwrap();
                            v.into_iter().collect()
                        },
                        2 => { // Full capitalization
                            target_string.to_uppercase()
                        },
                        _ => { // Noop
                            target_string
                        }
                    }
                }else{
                    target_string
                };

                RenderResult::Text(target_string)
            },

            // Image Match
            MatchContentType::Image(content) => {
                // Make sure the image exist beforehand
                if content.path.exists() {
                    RenderResult::Image(content.path.clone())
                }else{
                    error!("Image not found in path: {:?}", content.path);
                    RenderResult::Error
                }
            },
        }
    }

    fn render_passive(&self, text: &str, config: &Configs) -> RenderResult {
        // Render the matches
        let result = self.passive_match_regex.replace_all(&text, |caps: &Captures| {
            let match_name = if let Some(name) = caps.name("name") {
                name.as_str()
            }else{
                ""
            };


            // Get the original matching string, useful to return the match untouched
            let original_match = caps.get(0).unwrap().as_str();

            // Find the corresponding match
            let m = DefaultRenderer::find_match(config, match_name);

            // If no match is found, leave the match without modifications
            if m.is_none() {
                return original_match.to_owned();
            }

            // Compute the args by separating them
            let match_args = if let Some(args) = caps.name("args") {
                args.as_str()
            }else{
                ""
            };
            let args : Vec<String> = utils::split_args(match_args,
                                                       config.passive_arg_delimiter,
                                                       config.passive_arg_escape);

            let (m, trigger_offset) = m.unwrap();
            // Render the actual match
            let result = self.render_match(&m, trigger_offset, &config, args);

            match result {
                RenderResult::Text(out) => {
                    out
                },
                _ => {
                    original_match.to_owned()
                }
            }
        });

        RenderResult::Text(result.into_owned())
    }
}

// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    fn get_renderer(config: Configs) -> DefaultRenderer {
        DefaultRenderer::new(vec![Box::new(crate::extension::dummy::DummyExtension::new())], config)
    }

    fn get_config_for(s: &str) -> Configs {
        let config : Configs = serde_yaml::from_str(s).unwrap();
        config
    }

    fn verify_render(rendered: RenderResult, target: &str) {
        match rendered {
            RenderResult::Text(rendered) => {
                assert_eq!(rendered, target);
            },
            _ => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_render_passive_no_matches() {
        let text = r###"
        this text contains no matches
        "###;

        let config = get_config_for(r###"
        matches:
            - trigger: test
              replace: result
        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, text);
    }

    #[test]
    fn test_render_passive_simple_match_no_args() {
        let text = "this is a :test";

        let config = get_config_for(r###"
        matches:
            - trigger: ':test'
              replace: result
        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "this is a result");
    }

    #[test]
    fn test_render_passive_multiple_match_no_args() {
        let text = "this is a :test and then another :test";

        let config = get_config_for(r###"
        matches:
            - trigger: ':test'
              replace: result
        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "this is a result and then another result");
    }

    #[test]
    fn test_render_passive_simple_match_multiline_no_args() {
        let text = r###"this is a
        :test
        "###;

        let result= r###"this is a
        result
        "###;

        let config = get_config_for(r###"
        matches:
            - trigger: ':test'
              replace: result
        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, result);
    }

    #[test]
    fn test_render_passive_nested_matches_no_args() {
        let text = ":greet";

        let config = get_config_for(r###"
        matches:
            - trigger: ':greet'
              replace: "hi {{name}}"
              vars:
                - name: name
                  type: match
                  params:
                    trigger: ":name"

            - trigger: ':name'
              replace: john
        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "hi john");
    }

    #[test]
    fn test_render_passive_simple_match_with_args() {
        let text = ":greet/Jon/";

        let config = get_config_for(r###"
        matches:
            - trigger: ':greet'
              replace: "Hi $0$"
        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "Hi Jon");
    }

    #[test]
    fn test_render_passive_simple_match_with_multiple_args() {
        let text = ":greet/Jon/Snow/";

        let config = get_config_for(r###"
        matches:
            - trigger: ':greet'
              replace: "Hi $0$, there is $1$ outside"
        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "Hi Jon, there is Snow outside");
    }

    #[test]
    fn test_render_passive_simple_match_with_escaped_args() {
        let text = ":greet/Jon/10\\/12/";

        let config = get_config_for(r###"
        matches:
            - trigger: ':greet'
              replace: "Hi $0$, today is $1$"
        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "Hi Jon, today is 10/12");
    }

    #[test]
    fn test_render_passive_simple_match_with_args_not_closed() {
        let text = ":greet/Jon/Snow";

        let config = get_config_for(r###"
        matches:
            - trigger: ':greet'
              replace: "Hi $0$"
        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "Hi JonSnow");
    }

    #[test]
    fn test_render_passive_local_var() {
        let text = "this is :test";

        let config = get_config_for(r###"
        matches:
            - trigger: ':test'
              replace: "my {{output}}"
              vars:
                - name: output
                  type: dummy
                  params:
                    echo: "result"
        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "this is my result");
    }

    #[test]
    fn test_render_passive_global_var() {
        let text = "this is :test";

        let config = get_config_for(r###"
        global_vars:
            - name: output
              type: dummy
              params:
                echo: "result"
        matches:
            - trigger: ':test'
              replace: "my {{output}}"

        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "this is my result");
    }

    #[test]
    fn test_render_passive_global_var_is_overridden_by_local() {
        let text = "this is :test";

        let config = get_config_for(r###"
        global_vars:
            - name: output
              type: dummy
              params:
                echo: "result"
        matches:
            - trigger: ':test'
              replace: "my {{output}}"
              vars:
                - name: "output"
                  type: dummy
                  params:
                    echo: "local"

        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "this is my local");
    }

    #[test]
    fn test_render_match_with_unknown_variable_does_not_crash() {
        let text = "this is :test";

        let config = get_config_for(r###"
            matches:
                - trigger: ':test'
                  replace: "my {{unknown}}"
            "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "this is my ");
    }

    #[test]
    fn test_render_escaped_double_brackets_should_not_consider_them_variable() {
        let text = "this is :test";

        let config = get_config_for(r###"
            matches:
                - trigger: ':test'
                  replace: "my \\{\\{unknown\\}\\}"
            "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "this is my {{unknown}}");
    }

    #[test]
    fn test_render_passive_simple_match_multi_trigger_no_args() {
        let text = "this is a :yolo and :test";

        let config = get_config_for(r###"
        matches:
            - triggers: [':test', ':yolo']
              replace: result
        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "this is a result and result");
    }

    #[test]
    fn test_render_passive_simple_match_multi_trigger_with_args() {
        let text = ":yolo/Jon/";

        let config = get_config_for(r###"
        matches:
            - triggers: [':greet', ':yolo']
              replace: "Hi $0$"
        "###);

        let renderer = get_renderer(config.clone());

        let rendered = renderer.render_passive(text, &config);

        verify_render(rendered, "Hi Jon");
    }

    #[test]
    fn test_render_match_case_propagation_no_case() {
        let config = get_config_for(r###"
        matches:
            - trigger: 'test'
              replace: result
              propagate_case: true
        "###);

        let renderer = get_renderer(config.clone());

        let m = config.matches[0].clone();

        let trigger_offset = m.triggers.iter().position(|x| x== "test").unwrap();

        let rendered = renderer.render_match(&m, trigger_offset, &config, vec![]);

        verify_render(rendered, "result");
    }

    #[test]
    fn test_render_match_case_propagation_first_capital() {
        let config = get_config_for(r###"
        matches:
            - trigger: 'test'
              replace: result
              propagate_case: true
        "###);

        let renderer = get_renderer(config.clone());

        let m = config.matches[0].clone();

        let trigger_offset = m.triggers.iter().position(|x| x== "Test").unwrap();

        let rendered = renderer.render_match(&m, trigger_offset, &config, vec![]);

        verify_render(rendered, "Result");
    }

    #[test]
    fn test_render_match_case_propagation_all_capital() {
        let config = get_config_for(r###"
        matches:
            - trigger: 'test'
              replace: result
              propagate_case: true
        "###);

        let renderer = get_renderer(config.clone());

        let m = config.matches[0].clone();

        let trigger_offset = m.triggers.iter().position(|x| x== "TEST").unwrap();

        let rendered = renderer.render_match(&m, trigger_offset, &config, vec![]);

        verify_render(rendered, "RESULT");
    }
}