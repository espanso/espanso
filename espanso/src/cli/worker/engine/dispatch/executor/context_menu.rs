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

use espanso_ui::UIRemote;

use espanso_engine::dispatch::ContextMenuHandler;

pub struct ContextMenuHandlerAdapter<'a> {
  remote: &'a dyn UIRemote,
}

impl<'a> ContextMenuHandlerAdapter<'a> {
  pub fn new(remote: &'a dyn UIRemote) -> Self {
    Self { remote }
  }
}

impl<'a> ContextMenuHandler for ContextMenuHandlerAdapter<'a> {
  fn show_context_menu(&self, items: &[espanso_engine::event::ui::MenuItem]) -> anyhow::Result<()> {
    let ui_menu_items: Vec<espanso_ui::menu::MenuItem> =
      items.iter().map(convert_to_ui_menu_item).collect();
    let ui_menu = espanso_ui::menu::Menu {
      items: ui_menu_items,
    };

    self.remote.show_context_menu(&ui_menu);

    Ok(())
  }
}

fn convert_to_ui_menu_item(
  item: &espanso_engine::event::ui::MenuItem,
) -> espanso_ui::menu::MenuItem {
  match item {
    espanso_engine::event::ui::MenuItem::Simple(simple) => {
      espanso_ui::menu::MenuItem::Simple(espanso_ui::menu::SimpleMenuItem {
        id: simple.id,
        label: simple.label.clone(),
      })
    }
    espanso_engine::event::ui::MenuItem::Sub(sub) => {
      espanso_ui::menu::MenuItem::Sub(espanso_ui::menu::SubMenuItem {
        label: sub.label.clone(),
        items: sub.items.iter().map(convert_to_ui_menu_item).collect(),
      })
    }
    espanso_engine::event::ui::MenuItem::Separator => espanso_ui::menu::MenuItem::Separator,
  }
}
