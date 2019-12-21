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

use serde_yaml::{Mapping, Value};
use std::path::PathBuf;
use std::collections::HashMap;
use regex::{Regex, Captures};
use log::{warn, error};
use super::*;
use crate::matcher::{Match, MatchContentType};
use crate::config::Configs;
use crate::extension::Extension;

lazy_static! {
    static ref VAR_REGEX: Regex = Regex::new("\\{\\{\\s*(?P<name>\\w+)\\s*\\}\\}").unwrap();
}

pub struct DefaultRenderer {
    extension_map: HashMap<String, Box<dyn Extension>>,
}

impl DefaultRenderer {
    pub fn new(extensions: Vec<Box<dyn Extension>>) -> DefaultRenderer {
        // Register all the extensions
        let mut extension_map = HashMap::new();
        for extension in extensions.into_iter() {
            extension_map.insert(extension.name(), extension);
        }

        DefaultRenderer{
            extension_map
        }
    }

    fn find_match(config: &Configs, trigger: &str) -> Option<Match> {
        let mut result = None;

        // TODO: if performances become a problem, implement a more efficient lookup
        for m in config.matches.iter() {
            if m.trigger == trigger {
                result = Some(m.clone());
                break;
            }
        }

        result
    }
}

impl super::Renderer for DefaultRenderer {
    fn render_match(&self, m: &Match, config: &Configs, args: Vec<String>) -> RenderResult {
        // Manage the different types of matches
        match &m.content {
            // Text Match
            MatchContentType::Text(content) => {
                let mut target_string = if content._has_vars {
                    let mut output_map = HashMap::new();

                    for variable in content.vars.iter() {
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

                            let inner_match = inner_match.unwrap();

                            // Render the inner match
                            // TODO: inner arguments
                            let result = self.render_match(&inner_match, config, vec![]);

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
                                let ext_out = extension.calculate(&variable.params);
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
                        output.unwrap()
                    });

                    result.to_string()
                }else{  // No variables, simple text substitution
                    content.replace.clone()
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
}

// TODO: tests