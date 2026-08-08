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

#include "common.h"

// Dark-mode palette (always defined so client code can reference it anywhere).
const wxColour DARK_BG(32, 32, 32);
const wxColour DARK_FG(235, 235, 235);

#ifdef __WXMSW__
#include <dwmapi.h>
#include <wchar.h>
#include <windows.h>
#pragma comment(lib, "dwmapi.lib")

#ifndef DWMWA_USE_IMMERSIVE_DARK_MODE
#define DWMWA_USE_IMMERSIVE_DARK_MODE 20
#endif
#endif
#ifdef __WXOSX__
#include "mac.h"
#endif

void setFrameIcon(wxString iconPath, wxFrame *frame) {
    if (!iconPath.IsEmpty()) {
        wxBitmapType imgType = wxICON_DEFAULT_TYPE;

#ifdef __WXMSW__
        imgType = wxBITMAP_TYPE_ICO;
#endif

        wxIcon icon;
        icon.LoadFile(iconPath, imgType);
        if (icon.IsOk()) {
            frame->SetIcon(icon);
        }
    }
}

void Activate(wxFrame *frame) {
#ifdef __WXMSW__

    HWND handle = frame->GetHandle();
    if (handle == GetForegroundWindow()) {
        return;
    }

    if (IsIconic(handle)) {
        ShowWindow(handle, 9);
    }

    INPUT ip;
    ip.type = INPUT_KEYBOARD;
    ip.ki.wScan = 0;
    ip.ki.time = 0;
    ip.ki.dwExtraInfo = 0;
    ip.ki.wVk = VK_MENU;
    ip.ki.dwFlags = 0;

    SendInput(1, &ip, sizeof(INPUT));
    ip.ki.dwFlags = KEYEVENTF_KEYUP;

    SendInput(1, &ip, sizeof(INPUT));

    SetForegroundWindow(handle);

#endif
#ifdef __WXOSX__
    ActivateApp();
#endif
}

void SetupWindowStyle(wxFrame *frame) {
#ifdef __WXOSX__
    SetWindowStyles((NSWindow *)frame->MacGetTopLevelWindowRef());
#endif
}

#ifdef __WXMSW__
// SetPreferredAppMode is exported by ordinal 135 from uxtheme.dll.
typedef BOOL(WINAPI *SetPreferredAppModeFn)(DWORD);

void enableAppDarkMode() {
    HMODULE hUxTheme = LoadLibraryW(L"uxtheme.dll");
    if (hUxTheme) {
        SetPreferredAppModeFn fn =
            (SetPreferredAppModeFn)GetProcAddress(hUxTheme, (LPCSTR)135);
        if (fn) {
            fn(1); // 1 = AllowDark
        }
    }
}

void applyDarkModeToWindow(void *hwnd, bool dark) {
    HWND h = (HWND)hwnd;
    if (!h) {
        return;
    }
    BOOL value = dark ? TRUE : FALSE;
    DwmSetWindowAttribute(h, DWMWA_USE_IMMERSIVE_DARK_MODE, &value,
                          sizeof(value));
}

bool isSystemDark() {
    HKEY hKey = NULL;
    DWORD value = 1; // default: light (AppsUseLightTheme = 1)
    if (RegOpenKeyExW(
            HKEY_CURRENT_USER,
            L"Software\\Microsoft\\Windows\\CurrentVersion\\"
            L"Themes\\Personalize",
            0, KEY_READ, &hKey) == ERROR_SUCCESS) {
        DWORD size = sizeof(DWORD);
        RegQueryValueExW(hKey, L"AppsUseLightTheme", nullptr, nullptr,
                         (LPBYTE)&value, &size);
        RegCloseKey(hKey);
    }
    // AppsUseLightTheme: 1 = light, 0 = dark
    return value == 0;
}

void applyThemeColors(wxWindow *win, bool dark) {
    if (win == nullptr) {
        return;
    }
    wxColour bg =
        dark ? DARK_BG : wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOW);
    wxColour fg =
        dark ? DARK_FG : wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOWTEXT);
    win->SetBackgroundColour(bg);
    win->SetForegroundColour(fg);

    const wxWindowList &children = win->GetChildren();
    for (wxWindowList::const_iterator it = children.begin();
         it != children.end(); ++it) {
        applyThemeColors(*it, dark);
    }
}

bool isColorSchemeChange(WXUINT message, WXLPARAM lParam) {
    return message == WM_SETTINGCHANGE && lParam != 0 &&
           wcscmp((LPCWSTR)lParam, L"ImmersiveColorSet") == 0;
}
#endif
