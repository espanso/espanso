/*
 * This file is part of modulo.
 *
 * Copyright (C) 2020-2021 Federico Terzi
 *
 * modulo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * modulo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with modulo.  If not, see <https://www.gnu.org/licenses/>.
 */

use super::config::{FieldConfig, FieldTypeConfig, FormConfig};
use super::parser::layout::Token;
use crate::sys::form::types::*;
use std::collections::HashMap;

pub fn generate(config: FormConfig) -> Form {
  let structure = super::parser::layout::parse_layout(&config.layout);
  build_form(config, structure)
}

fn create_field(token: &Token, field_map: &HashMap<String, FieldConfig>) -> Field {
  match token {
    Token::Text(text) => Field {
      field_type: FieldType::Label(LabelMetadata { text: text.clone() }),
      ..Default::default()
    },
    Token::Field(name) => {
      let config = if let Some(config) = field_map.get(name) {
        config.clone()
      } else {
        Default::default()
      };

      let field_type = match &config.field_type {
        FieldTypeConfig::Text(config) => FieldType::Text(TextMetadata {
          default_text: config.default.clone(),
          multiline: config.multiline,
        }),
        FieldTypeConfig::Choice(config) => FieldType::Choice(ChoiceMetadata {
          values: config.values.clone(),
          choice_type: ChoiceType::Dropdown,
          default_value: config.default.clone(),
        }),
        FieldTypeConfig::List(config) => FieldType::Choice(ChoiceMetadata {
          values: config.values.clone(),
          choice_type: ChoiceType::List,
          default_value: config.default.clone(),
        }),
      };

      Field {
        id: Some(name.clone()),
        field_type,
      }
    }
  }
}

fn build_form(form: FormConfig, structure: Vec<Vec<Token>>) -> Form {
  let field_map = form.fields;
  let mut fields = Vec::new();

  for row in structure.iter() {
    let current_field = if row.len() == 1 {
      // Single field
      create_field(&row[0], &field_map)
    } else {
      // Row field
      let inner_fields = row
        .iter()
        .map(|token| create_field(token, &field_map))
        .collect();

      Field {
        field_type: FieldType::Row(RowMetadata {
          fields: inner_fields,
        }),
        ..Default::default()
      }
    };

    fields.push(current_field)
  }

  Form {
    title: form.title,
    icon: form.icon,
    fields,
  }
}
