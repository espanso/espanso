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

#pragma comment(lib, "Gdi32.lib")
#include <Windows.h>

#include "WinToast/wintoastlib.h"
using namespace WinToastLib;

#define APPWM_ICON_CLICK (WM_APP + 1)
#define APPWM_SHOW_CONTEXT_MENU (WM_APP + 2)
#define APPWM_UPDATE_TRAY_ICON (WM_APP + 3)
#define APPWM_SHOW_NOTIFICATION (WM_APP + 4)

const wchar_t *const ui_winclass = L"EspansoUI";

typedef struct
{
  UIOptions options;
  NOTIFYICONDATA nid;
  HICON g_icons[MAX_ICON_COUNT];
  wchar_t notification_icon_path[MAX_FILE_PATH];

  // Rust interop
  void *rust_instance;
  EventCallback event_callback;
} UIVariables;

// Needed to detect when Explorer crashes
UINT WM_TASKBARCREATED = RegisterWindowMessage(L"TaskbarCreated");

// Notification handler using: https://mohabouje.github.io/WinToast/
class EspansoNotificationHandler : public IWinToastHandler
{
public:
  void toastActivated() const {}
  void toastActivated(int actionIndex) const {}
  void toastDismissed(WinToastDismissalReason state) const {}
  void toastFailed() const {}
};

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
      std::cout << "click menu" << std::flush;
      //context_menu_click_callback(manager_instance, (int32_t)idItem);
    }

    break;
  }
  case APPWM_SHOW_CONTEXT_MENU: // Request to show context menu
  {
    HMENU hPopupMenu = CreatePopupMenu();

    // Create the menu

    /*
            int32_t count = static_cast<int32_t>(lp);
            std::unique_ptr<MenuItem[]> items(reinterpret_cast<MenuItem*>(wp));

            for (int i = 0; i<count; i++) {
                if (items[i].type == 1) {
                    InsertMenu(hPopupMenu, i, MF_BYPOSITION | MF_STRING, items[i].id, items[i].name);
                }else{
                    InsertMenu(hPopupMenu, i, MF_BYPOSITION | MF_SEPARATOR, items[i].id, NULL);
                }
            }

            POINT pt;
            GetCursorPos(&pt);
            SetForegroundWindow(nw);
            TrackPopupMenu(hPopupMenu, TPM_BOTTOMALIGN | TPM_LEFTALIGN, pt.x, pt.y, 0, nw, NULL);
            */
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
  case APPWM_SHOW_NOTIFICATION:
  {
    std::unique_ptr<wchar_t> message(reinterpret_cast<wchar_t *>(lp));

    std::cout << "hello" << variables->notification_icon_path << std::endl;

    WinToastTemplate templ = WinToastTemplate(WinToastTemplate::ImageAndText02);
    templ.setImagePath(variables->notification_icon_path);
    templ.setTextField(L"Espanso", WinToastTemplate::FirstLine);
    templ.setTextField(message.get(), WinToastTemplate::SecondLine);
    WinToast::instance()->showToast(templ, new EspansoNotificationHandler());
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
      wcscpy(variables->notification_icon_path, _options.notification_icon_path);
      SetWindowLongPtrW(window, GWLP_USERDATA, reinterpret_cast<::LONG_PTR>(variables));

      // Load the tray icons
      for (int i = 0; i < variables->options.icon_paths_count; i++)
      {
        variables->g_icons[i] = (HICON)LoadImage(NULL, variables->options.icon_paths[i], IMAGE_ICON, 0, 0, LR_DEFAULTCOLOR | LR_SHARED | LR_DEFAULTSIZE | LR_LOADFROMFILE);
      }

      // Hide the window
      ShowWindow(window, SW_HIDE);

      // Setup the icon in the notification space
      SendMessage(window, WM_SETICON, ICON_BIG, (LPARAM)variables->g_icons[0]);
      SendMessage(window, WM_SETICON, ICON_SMALL, (LPARAM)variables->g_icons[0]);

      // Notification
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

  // Initialize the notification handler
  WinToast::instance()->setAppName(L"Espanso");
  const auto aumi = WinToast::configureAUMI(L"federico.terzi", L"Espanso", L"Espanso", L"1.0.0");
  WinToast::instance()->setAppUserModelId(aumi);
  if (!WinToast::instance()->initialize())
  {
    *error_code = -3;
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

void ui_update_tray_icon(void *window, int32_t index)
{
  if (window)
  {
    PostMessage((HWND)window, APPWM_UPDATE_TRAY_ICON, 0, index);
  }
}

int32_t ui_show_notification(void *window, wchar_t *message)
{
  if (window)
  {
    wchar_t *message_copy = _wcsdup(message);
    PostMessage((HWND)window, APPWM_SHOW_NOTIFICATION, 0, (LPARAM)message_copy);
    return 0;
  }
  return -1;
}