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

#ifdef __WXMSW__
#include <windows.h>
#endif
#ifdef __WXOSX__
#include "mac.h"
#endif

void setFrameIcon(const char * iconPath, wxFrame * frame) {
    if (iconPath) {
        wxString iconPath(iconPath);
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

void Activate(wxFrame * frame) {
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

void SetupWindowStyle(wxFrame * frame) {
    #ifdef __WXOSX__
        SetWindowStyles((NSWindow*) frame->MacGetTopLevelWindowRef());
    #endif
}