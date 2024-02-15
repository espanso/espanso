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
#include <iostream>
#include <stdio.h>
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
#include <strsafe.h>
#include <Windows.h>

const wchar_t *const DETECT_WINCLASS = L"EspansoDetect";
const USHORT MOUSE_DOWN_FLAGS = RI_MOUSE_LEFT_BUTTON_DOWN | RI_MOUSE_RIGHT_BUTTON_DOWN | RI_MOUSE_MIDDLE_BUTTON_DOWN |
                        RI_MOUSE_BUTTON_1_DOWN | RI_MOUSE_BUTTON_2_DOWN | RI_MOUSE_BUTTON_3_DOWN |
                        RI_MOUSE_BUTTON_4_DOWN | RI_MOUSE_BUTTON_5_DOWN;
const USHORT MOUSE_UP_FLAGS = RI_MOUSE_LEFT_BUTTON_UP | RI_MOUSE_RIGHT_BUTTON_UP | RI_MOUSE_MIDDLE_BUTTON_UP |
                      RI_MOUSE_BUTTON_1_UP | RI_MOUSE_BUTTON_2_UP | RI_MOUSE_BUTTON_3_UP |
                      RI_MOUSE_BUTTON_4_UP | RI_MOUSE_BUTTON_5_UP;

typedef struct {
  HKL current_keyboard_layout;
  DWORD last_key_press_tick;
  // How many milliseconds must pass between events before refreshing the keyboard layout
  long keyboard_layout_cache_interval;

  // Rust interop
  void * rust_instance;
  EventCallback event_callback;
} DetectVariables;

/*
 * Message handler procedure for the window
 */
LRESULT CALLBACK detect_window_procedure(HWND window, unsigned int msg, WPARAM wp, LPARAM lp)
{
  DetectVariables * variables = reinterpret_cast<DetectVariables*>(GetWindowLongPtrW(window, GWLP_USERDATA));

  switch (msg)
  {
  case WM_DESTROY:
    PostQuitMessage(0);

    // Free the window variables
    delete variables;
    SetWindowLongPtrW(window, GWLP_USERDATA, NULL);

    return 0L;
  case WM_HOTKEY: // Hotkeys
  {
    InputEvent event = {};
    event.event_type = INPUT_EVENT_TYPE_HOTKEY;
    event.key_code = (int32_t) wp;
    if (variables->rust_instance != NULL && variables->event_callback != NULL)
    {
      variables->event_callback(variables->rust_instance, event);
    }
    break;
  }
  case WM_INPUT: // Message relative to the RAW INPUT events
  {
    InputEvent event = {};

    // Get the input size
    UINT dwSize;
    GetRawInputData(
        (HRAWINPUT)lp,
        RID_INPUT,
        NULL,
        &dwSize,
        sizeof(RAWINPUTHEADER));

    // Create a proper sized structure to hold the data
    std::vector<BYTE> lpb(dwSize);

    // Request the Raw input data
    if (GetRawInputData((HRAWINPUT)lp, RID_INPUT, lpb.data(), &dwSize,
                        sizeof(RAWINPUTHEADER)) != dwSize)
    {
      return 0;
    }

    // Convert the input data
    RAWINPUT *raw = reinterpret_cast<RAWINPUT *>(lpb.data());
    
    if (raw->header.dwType == RIM_TYPEKEYBOARD)  // Keyboard events
    {
      // We only want KEY UP AND KEY DOWN events
      if (raw->data.keyboard.Message != WM_KEYDOWN && raw->data.keyboard.Message != WM_KEYUP &&
          raw->data.keyboard.Message != WM_SYSKEYDOWN && raw->data.keyboard.Message != WM_SYSKEYUP)
      {
        return 0;
      }

      event.has_known_source = (raw->header.hDevice == 0) ? 0 : 1;

      // The alt key sends a SYSKEYDOWN instead of KEYDOWN event
      int is_key_down = raw->data.keyboard.Message == WM_KEYDOWN ||
                        raw->data.keyboard.Message == WM_SYSKEYDOWN;

      DWORD currentTick = GetTickCount();

      // If enough time has passed between the last keypress and now, refresh the keyboard layout
      if ((currentTick - variables->last_key_press_tick) > variables->keyboard_layout_cache_interval)
      {

        // Because keyboard layouts on windows are Window-specific, to get the current
        // layout we need to get the foreground window and get its layout.

        HWND hwnd = GetForegroundWindow();
        if (hwnd)
        {
          DWORD threadID = GetWindowThreadProcessId(hwnd, NULL);
          HKL newKeyboardLayout = GetKeyboardLayout(threadID);

          // It's not always valid, so update the current value only if available.
          if (newKeyboardLayout != 0)
          {
            variables->current_keyboard_layout = newKeyboardLayout;
          }
        }

        variables->last_key_press_tick = currentTick;
      }

      // Get keyboard state ( necessary to decode the associated Unicode char )
      std::vector<BYTE> lpKeyState(256);
      if (GetKeyboardState(lpKeyState.data()))
      {
        // This flag is needed to avoid changing the keyboard state for some layouts.
        // The 1 << 2 (setting bit 2) part is needed due to this issue: https://github.com/espanso/espanso/issues/86
        // while the 1 (setting bit 0) part is needed due to this issue: https://github.com/espanso/espanso/issues/552
        UINT flags = 1 << 2 | 1;

        int result = ToUnicodeEx(raw->data.keyboard.VKey, raw->data.keyboard.MakeCode, lpKeyState.data(), reinterpret_cast<LPWSTR>(event.buffer), (sizeof(event.buffer)/sizeof(event.buffer[0])) - 1, flags, variables->current_keyboard_layout);

        // Handle the corresponding string if present
        if (result >= 1)
        {
          event.buffer_len = result;

          // Filter out the value if the key was pressed while the ALT key was down
          // but not if AltGr is down (which is a shortcut to ALT+CTRL on some keyboards, such 
          // as the italian one).
          // This is needed in conjunction with the fix for: https://github.com/espanso/espanso/issues/725
          if ((lpKeyState[VK_MENU] & 0x80) != 0 && (lpKeyState[VK_CONTROL] & 0x80) == 0) {
            memset(event.buffer, 0, sizeof(event.buffer));
            event.buffer_len = 0;
          }
        }
        else
        {
          // If the given key does not have a correspondent string, reset the buffer
          memset(event.buffer, 0, sizeof(event.buffer));
          event.buffer_len = 0;
        }

        event.event_type = INPUT_EVENT_TYPE_KEYBOARD;
        event.key_code = raw->data.keyboard.VKey;
        event.status = is_key_down ? INPUT_STATUS_PRESSED : INPUT_STATUS_RELEASED;

        // Load the key variants when appropriate
        if (raw->data.keyboard.VKey == VK_SHIFT)
        {
          // To discriminate between the left and right shift, we need to employ a workaround.
          // See: https://stackoverflow.com/questions/5920301/distinguish-between-left-and-right-shift-keys-using-rawinput
          if (raw->data.keyboard.MakeCode == 42)
          { // Left shift
            event.variant = INPUT_LEFT_VARIANT;
          }
          if (raw->data.keyboard.MakeCode == 54)
          { // Right shift
            event.variant = INPUT_RIGHT_VARIANT;
          }
        }
        else
        {
          // Also the ALT and CTRL key are special cases
          // Check out the previous Stackoverflow question for more information
          if (raw->data.keyboard.VKey == VK_CONTROL || raw->data.keyboard.VKey == VK_MENU)
          {
            if ((raw->data.keyboard.Flags & RI_KEY_E0) != 0)
            {
              event.variant = INPUT_RIGHT_VARIANT;
            }
            else
            {
              event.variant = INPUT_LEFT_VARIANT;
            }
          }
        }
      }
    }
    else if (raw->header.dwType == RIM_TYPEMOUSE)  // Mouse events
    {
      // Make sure the mouse event belongs to the supported ones
      if ((raw->data.mouse.usButtonFlags & (MOUSE_DOWN_FLAGS | MOUSE_UP_FLAGS)) == 0) {
        return 0;        
      }

      event.event_type = INPUT_EVENT_TYPE_MOUSE;

      if ((raw->data.mouse.usButtonFlags & MOUSE_DOWN_FLAGS) != 0)
      {
        event.status = INPUT_STATUS_PRESSED;
      } else if ((raw->data.mouse.usButtonFlags & MOUSE_UP_FLAGS) != 0) {
        event.status = INPUT_STATUS_RELEASED;
      }

      // Convert the mouse flags into custom button mappings
      if ((raw->data.mouse.usButtonFlags & (RI_MOUSE_LEFT_BUTTON_DOWN | RI_MOUSE_LEFT_BUTTON_UP)) != 0) {
        event.key_code = INPUT_MOUSE_LEFT_BUTTON;
      } else if ((raw->data.mouse.usButtonFlags & (RI_MOUSE_RIGHT_BUTTON_DOWN | RI_MOUSE_RIGHT_BUTTON_UP)) != 0) {
        event.key_code = INPUT_MOUSE_RIGHT_BUTTON;
      } else if ((raw->data.mouse.usButtonFlags & (RI_MOUSE_MIDDLE_BUTTON_DOWN | RI_MOUSE_MIDDLE_BUTTON_UP)) != 0) {
        event.key_code = INPUT_MOUSE_MIDDLE_BUTTON;
      } else if ((raw->data.mouse.usButtonFlags & (RI_MOUSE_BUTTON_1_DOWN | RI_MOUSE_BUTTON_1_UP)) != 0) {
        event.key_code = INPUT_MOUSE_BUTTON_1;
      } else if ((raw->data.mouse.usButtonFlags & (RI_MOUSE_BUTTON_2_DOWN | RI_MOUSE_BUTTON_2_UP)) != 0) {
        event.key_code = INPUT_MOUSE_BUTTON_2;
      } else if ((raw->data.mouse.usButtonFlags & (RI_MOUSE_BUTTON_3_DOWN | RI_MOUSE_BUTTON_3_UP)) != 0) {
        event.key_code = INPUT_MOUSE_BUTTON_3;
      } else if ((raw->data.mouse.usButtonFlags & (RI_MOUSE_BUTTON_4_DOWN | RI_MOUSE_BUTTON_4_UP)) != 0) {
        event.key_code = INPUT_MOUSE_BUTTON_4;
      } else if ((raw->data.mouse.usButtonFlags & (RI_MOUSE_BUTTON_5_DOWN | RI_MOUSE_BUTTON_5_UP)) != 0) {
        event.key_code = INPUT_MOUSE_BUTTON_5;
      }
    }

    // If valid, send the event to the Rust layer
    if (event.event_type != 0 &&  variables->rust_instance != NULL && variables->event_callback != NULL)
    {
      variables->event_callback(variables->rust_instance, event);
    }

    return 0;
  }
  default:
    return DefWindowProc(window, msg, wp, lp);
  }
}

void * detect_initialize(void *_self, InitOptions * options, int32_t *error_code)
{
  HWND window = NULL;

  // Initialize the Worker window
  // Docs: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexa
  WNDCLASSEX wndclass = {
      sizeof(WNDCLASSEX),       // cbSize: Size of this structure
      0,                        // style: Class styles
      detect_window_procedure,         // lpfnWndProc: Pointer to the window procedure
      0,                        // cbClsExtra: Number of extra bytes to allocate following the window-class structure
      0,                        // cbWndExtra: The number of extra bytes to allocate following the window instance.
      GetModuleHandle(0),       // hInstance: A handle to the instance that contains the window procedure for the class.
      NULL,                     // hIcon: A handle to the class icon.
      LoadCursor(0, IDC_ARROW), // hCursor: A handle to the class cursor.
      NULL,                     // hbrBackground: A handle to the class background brush.
      NULL,                     // lpszMenuName: Pointer to a null-terminated character string that specifies the resource name of the class menu
      DETECT_WINCLASS,                 // lpszClassName: A pointer to a null-terminated string or is an atom.
      NULL                      // hIconSm: A handle to a small icon that is associated with the window class.
  };

  if (RegisterClassEx(&wndclass))
  {
    DetectVariables * variables = new DetectVariables();
    variables->rust_instance = _self;

    // Initialize the default keyboard layout
    variables->current_keyboard_layout = GetKeyboardLayout(0);
    variables->keyboard_layout_cache_interval = options->keyboard_layout_cache_interval;

    // Docs: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw
    window = CreateWindowEx(
        0,                        // dwExStyle: The extended window style of the window being created.
        DETECT_WINCLASS,                 // lpClassName: A null-terminated string or a class atom created by a previous call to the RegisterClass
        L"Espanso Worker Window", // lpWindowName: The window name.
        WS_OVERLAPPEDWINDOW,      // dwStyle: The style of the window being created.
        CW_USEDEFAULT,            // X: The initial horizontal position of the window.
        CW_USEDEFAULT,            // Y: The initial vertical position of the window.
        100,                      // nWidth: The width, in device units, of the window.
        100,                      // nHeight: The height, in device units, of the window.
        NULL,                     // hWndParent:  handle to the parent or owner window of the window being created.
        NULL,                     // hMenu: A handle to a menu, or specifies a child-window identifier, depending on the window style.
        GetModuleHandle(0),       // hInstance: A handle to the instance of the module to be associated with the window.
        NULL                      // lpParam: Pointer to a value to be passed to the window
    );

    SetWindowLongPtrW(window, GWLP_USERDATA, reinterpret_cast<::LONG_PTR>(variables));

    // Register raw inputs
    RAWINPUTDEVICE Rid[2];

    Rid[0].usUsagePage = 0x01;
    Rid[0].usUsage = 0x06;
    Rid[0].dwFlags = RIDEV_NOLEGACY | RIDEV_INPUTSINK;
    Rid[0].hwndTarget = window;

    Rid[1].usUsagePage = 0x01;
    Rid[1].usUsage = 0x02;
    Rid[1].dwFlags = RIDEV_INPUTSINK;
    Rid[1].hwndTarget = window;

    if (RegisterRawInputDevices(Rid, 2, sizeof(Rid[0])) == FALSE)
    { // Something went wrong, error.
      *error_code = -2;
      return nullptr;
    }
  }
  else
  {
    // Something went wrong, error.
    *error_code = -1;
    return nullptr;
  }

  return window;
}

int32_t detect_register_hotkey(void * window, HotKey hotkey) {
  return RegisterHotKey((HWND)window, hotkey.hk_id, hotkey.flags, hotkey.key_code);
}

int32_t detect_eventloop(void * window, EventCallback _callback)
{
  if (window)
  {
    DetectVariables * variables = reinterpret_cast<DetectVariables*>(GetWindowLongPtrW((HWND) window, GWLP_USERDATA));
    variables->event_callback = _callback;

    // Hide the window
    ShowWindow((HWND) window, SW_HIDE);

    // Enter the Event loop
    MSG msg;
    while (GetMessage(&msg, 0, 0, 0))
      DispatchMessage(&msg);
  }

  return 1;
}

int32_t detect_destroy(void * window) {
  if (window) {
    return DestroyWindow((HWND) window);
  }
}