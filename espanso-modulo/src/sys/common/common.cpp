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
const wxColour DARK_BORDER(60, 60, 60);
const wxColour LIGHT_BORDER(218, 220, 224);

#ifdef __WXMSW__
#include <dwmapi.h>
#include <wchar.h>
#include <windows.h>
#pragma comment(lib, "dwmapi.lib")

#ifndef DWMWA_USE_IMMERSIVE_DARK_MODE
#define DWMWA_USE_IMMERSIVE_DARK_MODE 20
#endif
#ifndef DWMWA_BORDER_COLOR
#define DWMWA_BORDER_COLOR 34
#endif
#ifndef DWMWA_CAPTION_COLOR
#define DWMWA_CAPTION_COLOR 35
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
// SetPreferredAppMode is an undocumented uxtheme export; ordinal 135 is only
// used after checking the Windows version because its signature changed.
typedef BOOL(WINAPI *SetPreferredAppModeFn)(DWORD);

void enableAppDarkMode() {
    // Ordinal 135 had a different signature before Windows 10 1903.
    // Never call it based on its presence alone.
    typedef LONG(WINAPI *RtlGetVersionFn)(OSVERSIONINFOW *);
    HMODULE ntdll = GetModuleHandleW(L"ntdll.dll");
    RtlGetVersionFn getVersion = ntdll
        ? (RtlGetVersionFn)GetProcAddress(ntdll, "RtlGetVersion")
        : nullptr;
    OSVERSIONINFOW version = {};
    version.dwOSVersionInfoSize = sizeof(version);
    if (!getVersion || getVersion(&version) != 0 ||
        version.dwMajorVersion < 10 || version.dwBuildNumber < 18362) {
        return;
    }
    HMODULE hUxTheme = LoadLibraryW(L"uxtheme.dll");
    if (hUxTheme) {
        SetPreferredAppModeFn fn =
            (SetPreferredAppModeFn)GetProcAddress(hUxTheme, (LPCSTR)135);
        if (fn) {
            fn(1); // 1 = AllowDark
        }
        FreeLibrary(hUxTheme);
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

    // Windows 11 allows explicitly customizing the window frame border color.
    // In dark mode, set it to dark gray (#3C3C3C) so it doesn't show bright
    // white edges.
    COLORREF borderColor =
        dark ? RGB(60, 60, 60) : 0xFFFFFFFF; // 0xFFFFFFFF = DWMWA_COLOR_DEFAULT
    DwmSetWindowAttribute(h, DWMWA_BORDER_COLOR, &borderColor,
                          sizeof(borderColor));

    COLORREF captionColor = dark ? RGB(32, 32, 32) : 0xFFFFFFFF;
    DwmSetWindowAttribute(h, DWMWA_CAPTION_COLOR, &captionColor,
                          sizeof(captionColor));
}

typedef HRESULT(WINAPI *SetWindowThemeFn)(HWND, LPCWSTR, LPCWSTR);

void applyWindowTheme(void *hwnd, bool dark) {
    HWND h = (HWND)hwnd;
    if (!h) {
        return;
    }
    HMODULE hUxTheme = LoadLibraryW(L"uxtheme.dll");
    if (hUxTheme) {
        SetWindowThemeFn fn =
            (SetWindowThemeFn)GetProcAddress(hUxTheme, "SetWindowTheme");
        if (fn) {
            fn(h, dark ? L"DarkMode_Explorer" : nullptr, nullptr);
        }
        FreeLibrary(hUxTheme);
    }
}

bool isSystemDark() {
    HKEY hKey = NULL;
    DWORD value = 1; // default: light (AppsUseLightTheme = 1)
    if (RegOpenKeyExW(HKEY_CURRENT_USER,
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

    if (win->GetHandle()) {
        applyWindowTheme(win->GetHandle(), dark);
    }

    const wxWindowList &children = win->GetChildren();
    for (wxWindowList::const_iterator it = children.begin();
         it != children.end(); ++it) {
        applyThemeColors(*it, dark);
    }
    // Updating colours alone does not invalidate every native child control.
    win->Refresh();
}

bool isColorSchemeChange(WXUINT message, WXLPARAM lParam) {
    return message == WM_SETTINGCHANGE && lParam != 0 &&
           wcscmp((LPCWSTR)lParam, L"ImmersiveColorSet") == 0;
}
#endif
