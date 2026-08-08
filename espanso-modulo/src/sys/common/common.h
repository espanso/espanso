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

// Dark-mode colour palette (shared by all modulo UIs on Windows).
extern const wxColour DARK_BG;
extern const wxColour DARK_FG;

#ifdef __WXMSW__
// Recursively theme a window and its children. wx 3.1.5 doesn't auto-darken
// the client area, so colours are set explicitly (native controls still follow
// OS immersive dark mode on top).
void applyThemeColors(wxWindow *win, bool dark);

// Native Win32 dark-mode helpers. wx 3.1.5 lacks
// EnableAutomaticDarkMode / wxEVT_DARK_MODE_CHANGED, so we drive the native
// APIs directly (also enables live re-theme on OS switches like PowerToys).
void enableAppDarkMode();                 // call once at startup
void applyDarkModeToWindow(void *hwnd, bool dark);
bool isSystemDark();                      // reads the registry

// True for the "ImmersiveColorSet" WM_SETTINGCHANGE broadcast, i.e. a system
// light/dark scheme switch (scheduled switchers included).
bool isColorSchemeChange(WXUINT message, WXLPARAM lParam);
#endif

#endif