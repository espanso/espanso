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
use anyhow::Result;
use caps::{CapSet, Capability};
use log::error;

pub fn can_use_capabilities() -> bool {
  match caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_OVERRIDE) {
    Ok(has_cap) => has_cap,
    Err(err) => {
      error!("error while checking if capabilities are enabled: {}", err);
      false
    },
  }
}

pub fn grant_capabilities() -> Result<()> {
  caps::raise(None, CapSet::Effective, Capability::CAP_DAC_OVERRIDE)?;
  Ok(())
}

pub fn clear_capabilities() -> Result<()> {
  caps::clear(None, CapSet::Effective)?;
  caps::clear(None, CapSet::Permitted)?;
  Ok(())
}