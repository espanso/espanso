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

#include "native.h"
#include <stdio.h>
#include <iostream>
#include <string>
#include <vector>
#include <memory>
#include <array>

#define UNICODE

#ifdef __MINGW32__
#ifndef WINVER
#define WINVER 0x0606
#endif
#define STRSAFE_NO_DEPRECATE
#endif

#include <windows.h>
#include <winuser.h>
#include <string.h>
#include <strsafe.h>
#pragma comment(lib, "Shell32.lib")
#include <shellapi.h>

#include <Windows.h>

#include "json/json.hpp"
using json = nlohmann::json;

#define APPWM_ICON_CLICK (WM_APP + 1)
#define APPWM_SHOW_CONTEXT_MENU (WM_APP + 2)
#define APPWM_UPDATE_TRAY_ICON (WM_APP + 3)

#define HEARTBEAT_TIMER_ID 10001

const wchar_t *const ui_winclass = L"EspansoUI";

typedef struct
{
  UIOptions options;
  NOTIFYICONDATA nid;
  HICON g_icons[MAX_ICON_COUNT];

  // Rust interop
  void *rust_instance;
  EventCallback event_callback;
} UIVariables;

// Needed to detect when Explorer crashes
UINT WM_TASKBARCREATED = RegisterWindowMessage(L"TaskbarCreated");

/*
 * Message handler procedure for the window
 */
LRESULT CALLBACK ui_window_procedure(HWND window, unsigned int msg, WPARAM wp, LPARAM lp)
{
  UIEvent event = {};
  UIVariables *variables = reinterpret_cast<UIVariables *>(GetWindowLongPtrW(window, GWLP_USERDATA));

  switch (msg)
  {
  case WM_DESTROY:
    PostQuitMessage(0);

    // Remove tray icon
    if (variables->options.show_icon)
    {
      Shell_NotifyIcon(NIM_DELETE, &variables->nid);
    }

    // Free the tray icons
    for (int i = 0; i < variables->options.icon_paths_count; i++)
    {
      DeleteObject(variables->g_icons[i]);
    }

    // Free the window variables
    delete variables;
    SetWindowLongPtrW(window, GWLP_USERDATA, NULL);

    return 0L;
  case WM_COMMAND: // Click on the tray icon context menu
  {
    UINT idItem = (UINT)LOWORD(wp);
    UINT flags = (UINT)HIWORD(wp);

    if (flags == 0)
    {
      event.event_type = UI_EVENT_TYPE_CONTEXT_MENU_CLICK;
      event.context_menu_id = (uint32_t)idItem;
      if (variables->event_callback && variables->rust_instance)
      {
        variables->event_callback(variables->rust_instance, event);
      }
    }

    break;
  }
  case APPWM_SHOW_CONTEXT_MENU: // Request to show context menu
  {
    HMENU menu = (HMENU)lp;
    POINT pt;
    GetCursorPos(&pt);
    SetForegroundWindow(window);
    TrackPopupMenu(menu, TPM_BOTTOMALIGN | TPM_LEFTALIGN, pt.x, pt.y, 0, window, NULL);

    break;
  }
  case APPWM_UPDATE_TRAY_ICON: // Request to update the tray icon
  {
    int32_t index = (int32_t)lp;
    if (index >= variables->options.icon_paths_count)
    {
      break;
    }

    variables->nid.hIcon = variables->g_icons[index];
    if (variables->options.show_icon)
    {
      Shell_NotifyIcon(NIM_MODIFY, &variables->nid);
    }

    break;
  }
  case APPWM_ICON_CLICK: // Click on the tray icon
  {
    switch (lp)
    {
    case WM_LBUTTONUP:
    case WM_RBUTTONUP:
      event.event_type = UI_EVENT_TYPE_ICON_CLICK;
      if (variables->event_callback && variables->rust_instance)
      {
        variables->event_callback(variables->rust_instance, event);
      }
      break;
    }
  }
  case WM_TIMER: // Regular timer check event
  {
    if (wp == HEARTBEAT_TIMER_ID)
    {
      event.event_type = UI_EVENT_TYPE_HEARTBEAT;
      if (variables->event_callback && variables->rust_instance)
      {
        variables->event_callback(variables->rust_instance, event);
      }
      break;
    }
  }
  default:
    if (msg == WM_TASKBARCREATED)
    { // Explorer crashed, recreate the icon
      if (variables->options.show_icon)
      {
        Shell_NotifyIcon(NIM_ADD, &variables->nid);
      }
    }
    return DefWindowProc(window, msg, wp, lp);
  }
}

void *ui_initialize(void *_self, UIOptions _options, int32_t *error_code)
{
  HWND window = NULL;

  SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE);

  // Service Window

  // Docs: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexa
  WNDCLASSEX uiwndclass = {
      sizeof(WNDCLASSEX),       // cbSize: Size of this structure
      0,                        // style: Class styles
      ui_window_procedure,      // lpfnWndProc: Pointer to the window procedure
      0,                        // cbClsExtra: Number of extra bytes to allocate following the window-class structure
      0,                        // cbWndExtra: The number of extra bytes to allocate following the window instance.
      GetModuleHandle(0),       // hInstance: A handle to the instance that contains the window procedure for the class.
      NULL,                     // hIcon: A handle to the class icon.
      LoadCursor(0, IDC_ARROW), // hCursor: A handle to the class cursor.
      NULL,                     // hbrBackground: A handle to the class background brush.
      NULL,                     // lpszMenuName: Pointer to a null-terminated character string that specifies the resource name of the class menu
      ui_winclass,              // lpszClassName: A pointer to a null-terminated string or is an atom.
      NULL                      // hIconSm: A handle to a small icon that is associated with the window class.
  };

  if (RegisterClassEx(&uiwndclass))
  {
    // Initialize the service window
    window = CreateWindowEx(
        0,                    // dwExStyle: The extended window style of the window being created.
        ui_winclass,          // lpClassName: A null-terminated string or a class atom created by a previous call to the RegisterClass
        L"Espanso UI Window", // lpWindowName: The window name.
        WS_OVERLAPPEDWINDOW,  // dwStyle: The style of the window being created.
        CW_USEDEFAULT,        // X: The initial horizontal position of the window.
        CW_USEDEFAULT,        // Y: The initial vertical position of the window.
        100,                  // nWidth: The width, in device units, of the window.
        100,                  // nHeight: The height, in device units, of the window.
        NULL,                 // hWndParent:  handle to the parent or owner window of the window being created.
        NULL,                 // hMenu: A handle to a menu, or specifies a child-window identifier, depending on the window style.
        GetModuleHandle(0),   // hInstance: A handle to the instance of the module to be associated with the window.
        NULL                  // lpParam: Pointer to a value to be passed to the window
    );

    if (window)
    {
      UIVariables *variables = new UIVariables();
      variables->options = _options;
      variables->rust_instance = _self;
      SetWindowLongPtrW(window, GWLP_USERDATA, reinterpret_cast<::LONG_PTR>(variables));

      // Load the tray icons
      for (int i = 0; i < variables->options.icon_paths_count; i++)
      {
        variables->g_icons[i] = (HICON)LoadImage(NULL, variables->options.icon_paths[i], IMAGE_ICON, 0, 0, LR_DEFAULTCOLOR | LR_SHARED | LR_DEFAULTSIZE | LR_LOADFROMFILE);
      }

      // Hide the window
      ShowWindow(window, SW_HIDE);

      // Setup the icon in the tray space
      SendMessage(window, WM_SETICON, ICON_BIG, (LPARAM)variables->g_icons[0]);
      SendMessage(window, WM_SETICON, ICON_SMALL, (LPARAM)variables->g_icons[0]);

      // Tray icon
      variables->nid.cbSize = sizeof(variables->nid);
      variables->nid.hWnd = window;
      variables->nid.uID = 1;
      variables->nid.uFlags = NIF_ICON | NIF_TIP | NIF_MESSAGE;
      variables->nid.uCallbackMessage = APPWM_ICON_CLICK;
      variables->nid.hIcon = variables->g_icons[0];
      StringCchCopyW(variables->nid.szTip, ARRAYSIZE(variables->nid.szTip), L"espanso");

      // Show the tray icon
      if (variables->options.show_icon)
      {
        Shell_NotifyIcon(NIM_ADD, &variables->nid);
      }

      // Setup heartbeat timer
      SetTimer(window, HEARTBEAT_TIMER_ID, 1000, (TIMERPROC)NULL);
    }
    else
    {
      *error_code = -2;
      return nullptr;
    }
  }
  else
  {
    *error_code = -1;
    return nullptr;
  }

  return window;
}

int32_t ui_eventloop(void *window, EventCallback _callback)
{
  if (window)
  {
    UIVariables *variables = reinterpret_cast<UIVariables *>(GetWindowLongPtrW((HWND)window, GWLP_USERDATA));
    variables->event_callback = _callback;

    // Enter the Event loop
    MSG msg;
    while (GetMessage(&msg, 0, 0, 0))
      DispatchMessage(&msg);
  }

  return 1;
}

int32_t ui_destroy(void *window)
{
  if (window)
  {
    return DestroyWindow((HWND)window);
  }
  return -1;
}

void ui_exit(void *window)
{
  if (window)
  {
    PostMessage((HWND)window, WM_CLOSE, 0, 0);
  }
}

void ui_update_tray_icon(void *window, int32_t index)
{
  if (window)
  {
    PostMessage((HWND)window, APPWM_UPDATE_TRAY_ICON, 0, index);
  }
}

// Menu related methods

void _insert_separator_menu(HMENU parent)
{
  InsertMenu(parent, -1, MF_BYPOSITION | MF_SEPARATOR, 0, NULL);
}

void _insert_single_menu(HMENU parent, json item)
{
  if (!item["label"].is_string() || !item["id"].is_number())
  {
    return;
  }
  std::string label = item["label"];
  uint32_t raw_id = item["id"];

  // Convert to wide chars
  std::wstring wide_label(label.length(), L'#');
  mbstowcs(&wide_label[0], label.c_str(), label.length());

  InsertMenu(parent, -1, MF_BYPOSITION | MF_STRING, raw_id, wide_label.c_str());
}

void _insert_sub_menu(HMENU parent, json items)
{
  for (auto &item : items)
  {
    if (item["type"] == "simple")
    {
      _insert_single_menu(parent, item);
    }
    else if (item["type"] == "separator")
    {
      _insert_separator_menu(parent);
    }
    else if (item["type"] == "sub")
    {
      HMENU subMenu = CreatePopupMenu();
      std::string label = item["label"];

      // Convert to wide chars
      std::wstring wide_label(label.length(), L'#');
      mbstowcs(&wide_label[0], label.c_str(), label.length());

      InsertMenu(parent, -1, MF_BYPOSITION | MF_POPUP, (UINT_PTR)subMenu, wide_label.c_str());
      _insert_sub_menu(subMenu, item["items"]);
    }
  }
}

int32_t ui_show_context_menu(void *window, char *payload)
{
  if (window)
  {
    auto j_menu = json::parse(payload);
    // Generate the menu from the JSON payload
    HMENU parentMenu = CreatePopupMenu();
    _insert_sub_menu(parentMenu, j_menu);

    PostMessage((HWND)window, APPWM_SHOW_CONTEXT_MENU, 0, (LPARAM)parentMenu);
    return 0;
  }
  return -1;
}