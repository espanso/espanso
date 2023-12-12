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

use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
};

use anyhow::{anyhow, Result};
use log::error;

use crate::Variable;

use super::RendererError;

struct Node<'a> {
  name: &'a str,
  variable: Option<&'a Variable>,
  dependencies: Option<HashSet<&'a str>>,
}

pub(crate) fn resolve_evaluation_order<'a>(
  body: &'a str,
  local_vars: &'a [&'a Variable],
  global_vars: &'a [&'a Variable],
) -> Result<Vec<&'a Variable>> {
  let node_map = generate_nodes(body, local_vars, global_vars);

  let body_node = node_map
    .get("__match_body")
    .ok_or_else(|| anyhow!("missing body node"))?;

  let eval_order = RefCell::new(Vec::new());
  let resolved = RefCell::new(HashSet::new());
  let seen = RefCell::new(HashSet::new());
  {
    resolve_dependencies(body_node, &node_map, &eval_order, &resolved, &seen)?;
  }

  let eval_order_ref = eval_order.borrow();

  let mut ordered_variables = Vec::new();
  for var_name in (*eval_order_ref).iter() {
    let node = node_map
      .get(var_name)
      .ok_or_else(|| anyhow!("could not find dependency node for variable: {}", var_name))?;
    if let Some(var) = node.variable {
      ordered_variables.push(var);
    }
  }

  Ok(ordered_variables)
}

fn generate_nodes<'a>(
  body: &'a str,
  local_vars: &'a [&'a Variable],
  global_vars: &'a [&'a Variable],
) -> HashMap<&'a str, Node<'a>> {
  let mut local_vars_nodes = Vec::new();
  for (index, var) in local_vars.iter().enumerate() {
    let mut dependencies = HashSet::new();
    if var.inject_vars {
      dependencies.extend(super::util::get_params_variable_names(&var.params));
    }
    dependencies.extend(var.depends_on.iter().map(String::as_str));

    // Every local variable depends on the one before it.
    // Needed to guarantee execution order within local vars.
    if index > 0 {
      let previous_var = local_vars.get(index - 1);
      if let Some(previous_var) = previous_var {
        dependencies.insert(&previous_var.name);
      }
    }

    local_vars_nodes.push(Node {
      name: &var.name,
      variable: Some(var),
      dependencies: Some(dependencies),
    });
  }

  let global_vars_nodes = global_vars.iter().map(|var| create_node_from_var(var));

  // The body depends on all local variables + the variables read inside it (which might be global)
  let mut body_dependencies: HashSet<&str> =
    local_vars_nodes.iter().map(|node| node.name).collect();
  body_dependencies.extend(super::util::get_body_variable_names(body));

  let body_node = Node {
    name: "__match_body",
    variable: None,
    dependencies: Some(body_dependencies),
  };

  let mut node_map = HashMap::new();

  node_map.insert(body_node.name, body_node);
  global_vars_nodes.into_iter().for_each(|node| {
    node_map.insert(node.name, node);
  });
  for node in local_vars_nodes.into_iter() {
    node_map.insert(node.name, node);
  }

  node_map
}

fn create_node_from_var(var: &Variable) -> Node {
  let dependencies = if var.inject_vars || !var.depends_on.is_empty() {
    let mut vars = HashSet::new();

    if var.inject_vars {
      vars.extend(super::util::get_params_variable_names(&var.params));
    }

    vars.extend(var.depends_on.iter().map(String::as_str));

    Some(vars)
  } else {
    None
  };

  Node {
    name: &var.name,
    variable: Some(var),
    dependencies,
  }
}

fn resolve_dependencies<'a>(
  node: &'a Node,
  node_map: &'a HashMap<&'a str, Node<'a>>,
  eval_order: &'a RefCell<Vec<&'a str>>,
  resolved: &'a RefCell<HashSet<&'a str>>,
  seen: &'a RefCell<HashSet<&'a str>>,
) -> Result<()> {
  {
    let mut seen_ref = seen.borrow_mut();
    seen_ref.insert(node.name);
  }

  if let Some(dependencies) = &node.dependencies {
    for dependency in dependencies.iter() {
      let has_been_resolved = {
        let resolved_ref = resolved.borrow();
        resolved_ref.contains(dependency)
      };
      let has_been_seen = {
        let seen_ref = seen.borrow();
        seen_ref.contains(dependency)
      };

      if !has_been_resolved {
        if has_been_seen {
          return Err(
            RendererError::CircularDependency(node.name.to_string(), dependency.to_string()).into(),
          );
        }

        match node_map.get(dependency) {
          Some(dependency_node) => {
            resolve_dependencies(dependency_node, node_map, eval_order, resolved, seen)?;
          }
          None => {
            error!("could not resolve variable {:?}", dependency);
            if let Some(variable) = &node.variable {
              if variable.var_type == "form" {
                super::log_new_form_syntax_tip();
              }
            }
            return Err(RendererError::MissingVariable(dependency.to_string()).into());
          }
        }
      }
    }
  }

  {
    let mut eval_order_ref = eval_order.borrow_mut();
    eval_order_ref.push(node.name);
    let mut resolved_ref = resolved.borrow_mut();
    resolved_ref.insert(node.name);
  }

  Ok(())
}
