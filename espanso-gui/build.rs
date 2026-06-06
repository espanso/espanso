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

// espanso-gui is an embedded library, not a standalone binary. Windows
// resources (manifest / version info / icon) must be owned by the final
// binary crate (`espanso`) instead — embedding them here links a second copy
// of those resources into espanso.exe and breaks the build with duplicate
// VERSION resource errors (CVT1100 / LNK1123). So this build script is a no-op.
fn main() {}
