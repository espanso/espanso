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

#ifndef MODULO_COMMON
#define MODULO_COMMON

#define _UNICODE

#include <wx/wxprec.h>
#ifndef WX_PRECOMP
#include <wx/wx.h>
#endif

void setFrameIcon(wxString iconPath, wxFrame *frame);

void Activate(wxFrame *frame);

void SetupWindowStyle(wxFrame *frame);

// Dark-mode palette
extern const wxColour DARK_BG;
extern const wxColour DARK_FG;
extern const wxColour DARK_BORDER;
extern const wxColour LIGHT_BORDER;

#ifdef __WXMSW__
// Recursively theme a window and its children.
void applyThemeColors(wxWindow *win, bool dark);

void enableAppDarkMode();
void applyDarkModeToWindow(void *hwnd, bool dark);
void applyWindowTheme(void *hwnd, bool dark);
bool isSystemDark();
bool isColorSchemeChange(WXUINT message, WXLPARAM lParam);
#endif

#endif
