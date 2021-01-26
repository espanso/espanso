/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

#include "bridge.h"
#include <stdio.h>
#include <iostream>
#include <string>
#include <vector>
#include <memory>
#include <array>

#define UNICODE

#ifdef __MINGW32__
# ifndef WINVER
#  define WINVER 0x0606
# endif
# define STRSAFE_NO_DEPRECATE
#endif

#include <windows.h>
#include <winuser.h>
#include <strsafe.h>
#include <shellapi.h>

#pragma comment( lib, "gdiplus.lib" )
#include <gdiplus.h>
#include <Windows.h>

// How many milliseconds must pass between keystrokes to refresh the keyboard layout
const long refreshKeyboardLayoutInterval = 2000;

void * manager_instance;
int32_t show_icon;

// Keyboard listening

DWORD lastKeyboardPressTick = 0;
HKL currentKeyboardLayout;
HWND window;
const wchar_t* const winclass = L"Espanso";



// UI

#define APPWM_ICON_CLICK (WM_APP + 1)
#define APPWM_NOTIFICATION_POPUP (WM_APP + 2)
#define APPWM_NOTIFICATION_CLOSE (WM_APP + 3)
#define APPWM_SHOW_CONTEXT_MENU (WM_APP + 4)

const wchar_t* const notification_winclass = L"EspansoNotification";
HWND nw = NULL;
HWND hwnd_st_u = NULL;
HBITMAP g_espanso_bmp = NULL;
HICON g_espanso_ico = NULL;
HICON g_espanso_red_ico = NULL;
NOTIFYICONDATA nid = {};

UINT WM_TASKBARCREATED = RegisterWindowMessage(L"TaskbarCreated");

// Callbacks

KeypressCallback keypress_callback = NULL;
IconClickCallback icon_click_callback = NULL;
ContextMenuClickCallback context_menu_click_callback = NULL;

void register_keypress_callback(KeypressCallback callback) {
    keypress_callback = callback;
}

void register_icon_click_callback(IconClickCallback callback) {
    icon_click_callback = callback;
}

void register_context_menu_click_callback(ContextMenuClickCallback callback) {
    context_menu_click_callback = callback;
}

/*
 * Message handler procedure for the windows
 */
LRESULT CALLBACK window_procedure(HWND window, unsigned int msg, WPARAM wp, LPARAM lp)
{
    HDC hdcStatic = NULL;

    switch (msg)
    {
        case WM_DESTROY:
            std::cout << "\ndestroying window\n";
            PostQuitMessage(0);
            DeleteObject(g_espanso_bmp);
            DeleteObject(g_espanso_ico);
            return 0L;
        case WM_COMMAND:  // Click on the tray icon context menu
        {
            UINT  idItem = (UINT)LOWORD(wp);
            UINT  flags = (UINT)HIWORD(wp);

            if (flags == 0) {
                context_menu_click_callback(manager_instance, (int32_t)idItem);
            }

            break;
        }
        case APPWM_NOTIFICATION_POPUP:  // Request to show a notification
        {
            std::unique_ptr<wchar_t[]> ptr(reinterpret_cast<wchar_t*>(wp));

            SetWindowText(hwnd_st_u, L"                                                 ");  // Clear the previous text
            SetWindowText(hwnd_st_u, ptr.get());

            // Show the window
            ShowWindow(nw, SW_SHOWNOACTIVATE);
            break;
        }
        case APPWM_NOTIFICATION_CLOSE:  // Request to close a notification
        {
            // Hide the window
            ShowWindow(nw, SW_HIDE);
            break;
        }
        case APPWM_SHOW_CONTEXT_MENU:  // Request to show context menu
        {
            HMENU hPopupMenu = CreatePopupMenu();

            // Create the menu

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
            break;
        }
        case APPWM_ICON_CLICK:  // Click on the tray icon
        {
            switch (lp)
            {
            case WM_LBUTTONUP:
            case WM_RBUTTONUP:
                icon_click_callback(manager_instance);
                break;
            }
        }
        case WM_PAINT:
        {
            BITMAP bm;
            PAINTSTRUCT ps;

            HDC hdc = BeginPaint(window, &ps);

            HDC hdcMem = CreateCompatibleDC(hdc);
            HBITMAP hbmOld = (HBITMAP) SelectObject(hdcMem, g_espanso_bmp);

            GetObject(g_espanso_bmp, sizeof(bm), &bm);

            BitBlt(hdc, 10, 10, 80, 80, hdcMem, 0, 0, SRCCOPY);

            SelectObject(hdcMem, hbmOld);
            DeleteDC(hdcMem);

            EndPaint(window, &ps);
            break;
        }
        case WM_CTLCOLORSTATIC:
            hdcStatic = (HDC)wp;
            SetTextColor(hdcStatic, RGB(0, 0, 0));
            SetBkColor(hdcStatic, RGB(255, 255, 255));
            //SetBkMode(hdcStatic, OPAQUE);

            return (LRESULT)GetStockObject(NULL_BRUSH);
        case WM_INPUT:  // Message relative to the RAW INPUT events
        {
            // Get the input size
            UINT dwSize;
            GetRawInputData(
                    (HRAWINPUT)lp,
                    RID_INPUT,
                    NULL,
                    &dwSize,
                    sizeof(RAWINPUTHEADER)
            );

            // Create a proper sized structure to hold the data
            std::vector<BYTE> lpb(dwSize);

            // Request the Raw input data
            if (GetRawInputData((HRAWINPUT)lp, RID_INPUT, lpb.data(), &dwSize,
                                sizeof(RAWINPUTHEADER)) != dwSize) {
                return 0;
            }

            // Convert the input data
            RAWINPUT* raw = reinterpret_cast<RAWINPUT*>(lpb.data());
            // Make sure it's a keyboard type event, relative to a key press.
            if (raw->header.dwType == RIM_TYPEKEYBOARD)
            {
                // We only want KEY UP AND KEY DOWN events
                if (raw->data.keyboard.Message != WM_KEYDOWN && raw->data.keyboard.Message != WM_KEYUP &&
                    raw->data.keyboard.Message != WM_SYSKEYDOWN) {
                    return 0;
                }

                // The alt key sends a SYSKEYDOWN instead of KEYDOWN event
                int is_key_down = raw->data.keyboard.Message == WM_KEYDOWN ||
                                  raw->data.keyboard.Message == WM_SYSKEYDOWN;

                DWORD currentTick = GetTickCount();

                // If enough time has passed between the last keypress and now, refresh the keyboard layout
                if ((currentTick - lastKeyboardPressTick) > refreshKeyboardLayoutInterval) {

                    // Because keyboard layouts on windows are Window-specific, to get the current
                    // layout we need to get the foreground window and get its layout.

                    HWND hwnd = GetForegroundWindow();
                    if (hwnd) {
                        DWORD threadID = GetWindowThreadProcessId(hwnd, NULL);
                        HKL newKeyboardLayout = GetKeyboardLayout(threadID);

                        // It's not always valid, so update the current value only if available.
                        if (newKeyboardLayout != 0) {
                            currentKeyboardLayout = newKeyboardLayout;
                        }
                    }

                    lastKeyboardPressTick = currentTick;
                }

                // Get keyboard state ( necessary to decode the associated Unicode char )
                std::vector<BYTE> lpKeyState(256);
                if (GetKeyboardState(lpKeyState.data())) {
                    // Convert the virtual key to an unicode char
                    std::array<WCHAR, 4> buffer;

                    // This flag is needed to avoid changing the keyboard state for some layouts.
                    // Refer to issue: https://github.com/federico-terzi/espanso/issues/86
                    UINT flags = 1 << 2;

                    int result = ToUnicodeEx(raw->data.keyboard.VKey, raw->data.keyboard.MakeCode, lpKeyState.data(), buffer.data(), buffer.size(), flags, currentKeyboardLayout);

                    //std::cout << result << " " << buffer[0] << " " << raw->data.keyboard.VKey << std::endl;

                    // We need to call the callback in two different ways based on the type of key
                    // The only modifier we use that has a result > 0 is the BACKSPACE, so we have to consider it.
                    if (result >= 1 && raw->data.keyboard.VKey != VK_BACK) {
                        keypress_callback(manager_instance, reinterpret_cast<uint16_t*>(buffer.data()), buffer.size(), 0, raw->data.keyboard.VKey, 0, is_key_down);
                    }else{
                        //std::cout << raw->data.keyboard.MakeCode << " " << raw->data.keyboard.Flags << std::endl;
                        int variant = 0;
                        if (raw->data.keyboard.VKey == VK_SHIFT) {
                            // To discriminate between the left and right shift, we need to employ a workaround.
                            // See: https://stackoverflow.com/questions/5920301/distinguish-between-left-and-right-shift-keys-using-rawinput
                            if (raw->data.keyboard.MakeCode == 42) { // Left shift
                                variant = LEFT_VARIANT;
                            }if (raw->data.keyboard.MakeCode == 54) { // Right shift
                                variant = RIGHT_VARIANT;
                            }
                        }else{
                            // Also the ALT and CTRL key are special cases
                            // Check out the previous Stackoverflow question for more information
                            if (raw->data.keyboard.VKey == VK_CONTROL || raw->data.keyboard.VKey == VK_MENU) {
                                if ((raw->data.keyboard.Flags & RI_KEY_E0) != 0) {
                                    variant = RIGHT_VARIANT;
                                }else{
                                    variant = LEFT_VARIANT;
                                }
                            }
                        }

                        keypress_callback(manager_instance, nullptr, 0, 1, raw->data.keyboard.VKey, variant, is_key_down);
                    }
                }
            }else if (raw->header.dwType == RIM_TYPEMOUSE) // Mouse input, registered as "other" events. Needed to improve the reliability of word matches
            {
                if ((raw->data.mouse.usButtonFlags & (RI_MOUSE_LEFT_BUTTON_DOWN | RI_MOUSE_RIGHT_BUTTON_DOWN | RI_MOUSE_MIDDLE_BUTTON_DOWN)) != 0) {
                    //std::cout << "mouse down" << std::endl;
                    keypress_callback(manager_instance, nullptr, 0, 2, raw->data.mouse.usButtonFlags, 0, 0);
                }
            }

            return 0;
        }
        default:
            if (msg == WM_TASKBARCREATED) {  // Explorer crashed, recreate the icon
                if (show_icon) {
                    Shell_NotifyIcon(NIM_ADD, &nid);
                }
            }
            return DefWindowProc(window, msg, wp, lp);
    }
}

int32_t initialize(void * self, wchar_t * ico_path, wchar_t * red_ico_path, wchar_t * bmp_path, int32_t _show_icon) {
    manager_instance = self;
    show_icon = _show_icon;

    // Load the images
    g_espanso_bmp = (HBITMAP)LoadImage(NULL, bmp_path, IMAGE_BITMAP, 0, 0, LR_LOADFROMFILE);
    g_espanso_ico = (HICON)LoadImage(NULL, ico_path, IMAGE_ICON, 0, 0, LR_DEFAULTCOLOR | LR_SHARED | LR_DEFAULTSIZE | LR_LOADFROMFILE);
    g_espanso_red_ico = (HICON)LoadImage(NULL, red_ico_path, IMAGE_ICON, 0, 0, LR_DEFAULTCOLOR | LR_SHARED | LR_DEFAULTSIZE | LR_LOADFROMFILE);

    // Make the notification capable of handling different screen definitions
    SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE);

    // Initialize the default keyboard layout
    currentKeyboardLayout = GetKeyboardLayout(0);

    // Initialize the Worker window

    // Docs: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexa
    WNDCLASSEX wndclass = {
            sizeof(WNDCLASSEX),				// cbSize: Size of this structure
            0,								// style: Class styles
            window_procedure,		        // lpfnWndProc: Pointer to the window procedure
            0,								// cbClsExtra: Number of extra bytes to allocate following the window-class structure
            0,								// cbWndExtra: The number of extra bytes to allocate following the window instance.
            GetModuleHandle(0),				// hInstance: A handle to the instance that contains the window procedure for the class.
            NULL,							// hIcon: A handle to the class icon.
            LoadCursor(0,IDC_ARROW),	    // hCursor: A handle to the class cursor.
            NULL,							// hbrBackground: A handle to the class background brush.
            NULL,							// lpszMenuName: Pointer to a null-terminated character string that specifies the resource name of the class menu
            winclass,						// lpszClassName: A pointer to a null-terminated string or is an atom.
            NULL							// hIconSm: A handle to a small icon that is associated with the window class.
    };

    // Notification Window

    // Docs: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexa
    WNDCLASSEX notificationwndclass = {
            sizeof(WNDCLASSEX),				// cbSize: Size of this structure
            0,								// style: Class styles
            window_procedure,	            // lpfnWndProc: Pointer to the window procedure
            0,								// cbClsExtra: Number of extra bytes to allocate following the window-class structure
            0,								// cbWndExtra: The number of extra bytes to allocate following the window instance.
            GetModuleHandle(0),				// hInstance: A handle to the instance that contains the window procedure for the class.
            NULL,							// hIcon: A handle to the class icon.
            LoadCursor(0,IDC_ARROW),	    // hCursor: A handle to the class cursor.
            NULL,							// hbrBackground: A handle to the class background brush.
            NULL,							// lpszMenuName: Pointer to a null-terminated character string that specifies the resource name of the class menu
            notification_winclass,			// lpszClassName: A pointer to a null-terminated string or is an atom.
            NULL							// hIconSm: A handle to a small icon that is associated with the window class.
    };

    if (RegisterClassEx(&wndclass) && RegisterClassEx(&notificationwndclass))
    {
        // Docs: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw
        window = CreateWindowEx(
                0,										// dwExStyle: The extended window style of the window being created.
                winclass,								// lpClassName: A null-terminated string or a class atom created by a previous call to the RegisterClass
                L"Espanso Worker Window",				// lpWindowName: The window name.
                WS_OVERLAPPEDWINDOW,					// dwStyle: The style of the window being created.
                CW_USEDEFAULT,							// X: The initial horizontal position of the window.
                CW_USEDEFAULT,							// Y: The initial vertical position of the window.
                100,									// nWidth: The width, in device units, of the window.
                100,									// nHeight: The height, in device units, of the window.
                NULL,									// hWndParent:  handle to the parent or owner window of the window being created.
                NULL,									// hMenu: A handle to a menu, or specifies a child-window identifier, depending on the window style.
                GetModuleHandle(0),						// hInstance: A handle to the instance of the module to be associated with the window.
                NULL									// lpParam: Pointer to a value to be passed to the window
        );

        // Register raw inputs
        RAWINPUTDEVICE Rid[2];

        Rid[0].usUsagePage = 0x01;
        Rid[0].usUsage = 0x06;
        Rid[0].dwFlags = RIDEV_NOLEGACY | RIDEV_INPUTSINK;   // adds HID keyboard and also ignores legacy keyboard messages
        Rid[0].hwndTarget = window;

        Rid[1].usUsagePage = 0x01;
        Rid[1].usUsage = 0x02;
        Rid[1].dwFlags = RIDEV_INPUTSINK;   // adds HID mouse and also ignores legacy mouse messages
        Rid[1].hwndTarget = window;

        if (RegisterRawInputDevices(Rid, 2, sizeof(Rid[0])) == FALSE) {  // Something went wrong, error.
            return -1;
        }

        // Initialize the notification window
        nw = CreateWindowEx(
                WS_EX_TOOLWINDOW | WS_EX_TOPMOST,		// dwExStyle: The extended window style of the window being created.
                notification_winclass, 					// lpClassName: A null-terminated string or a class atom created by a previous call to the RegisterClass
                L"Espanso Notification",				// lpWindowName: The window name.
                WS_POPUPWINDOW,							// dwStyle: The style of the window being created.
                CW_USEDEFAULT,							// X: The initial horizontal position of the window.
                CW_USEDEFAULT,							// Y: The initial vertical position of the window.
                300,									// nWidth: The width, in device units, of the window.
                100,									// nHeight: The height, in device units, of the window.
                NULL,									// hWndParent:  handle to the parent or owner window of the window being created.
                NULL,									// hMenu: A handle to a menu, or specifies a child-window identifier, depending on the window style.
                GetModuleHandle(0),						// hInstance: A handle to the instance of the module to be associated with the window.
                NULL									// lpParam: Pointer to a value to be passed to the window
        );

        if (nw)
        {
            int x, w, y, h;
            y = 40; h = 30;
            x = 100; w = 180;
            hwnd_st_u = CreateWindowEx(0, L"static", L"ST_U",
                                       WS_CHILD | WS_VISIBLE | WS_TABSTOP | SS_CENTER,
                                       x, y, w, h,
                                       nw, (HMENU)(501),
                                       (HINSTANCE)GetWindowLong(nw, GWLP_HINSTANCE), NULL);

            SetWindowText(hwnd_st_u, L"Loading...");

            int posX = GetSystemMetrics(SM_CXSCREEN) - 350;
            int posY = GetSystemMetrics(SM_CYSCREEN) - 200;

            SetWindowPos(nw, HWND_TOP, posX, posY, 0, 0, SWP_NOSIZE);

            // Hide the window
            ShowWindow(nw, SW_HIDE);

            // Setup the icon in the notification space

            SendMessage(nw, WM_SETICON, ICON_BIG, (LPARAM)g_espanso_ico);
            SendMessage(nw, WM_SETICON, ICON_SMALL, (LPARAM)g_espanso_ico);

            //Notification
            nid.cbSize = sizeof(nid);
            nid.hWnd = nw;
            nid.uID = 1;
            nid.uFlags = NIF_ICON | NIF_TIP | NIF_MESSAGE;
            nid.uCallbackMessage = APPWM_ICON_CLICK;
            nid.hIcon = g_espanso_ico;
            StringCchCopy(nid.szTip, ARRAYSIZE(nid.szTip), L"espanso");

            // Show the notification.
            if (show_icon) {
                Shell_NotifyIcon(NIM_ADD, &nid);
            }
        }
    }else{
        // Something went wrong, error.
        return -1;
    }

    return 1;
}

void eventloop() {
    if (window)
    {
        // Hide the window
        ShowWindow(window, SW_HIDE);

        // Enter the Event loop
        MSG msg;
        while (GetMessage(&msg, 0, 0, 0))  DispatchMessage(&msg);
    }

    // Something went wrong, this should have been an infinite loop.
}

void update_tray_icon(int32_t enabled) {
    if (enabled) {
        nid.hIcon = g_espanso_ico;
    }else{
        nid.hIcon = g_espanso_red_ico;
    }

    // Update the icon
    if (show_icon) {
        Shell_NotifyIcon(NIM_MODIFY, &nid);
    }
}

/*
 * Type the given string simulating keyboard presses.
 */
void send_string(const wchar_t * string) {
    std::wstring msg = string;

    std::vector<INPUT> vec;
    for (auto ch : msg)
    {
    	INPUT input = { 0 };
    	input.type = INPUT_KEYBOARD;
    	input.ki.dwFlags = KEYEVENTF_UNICODE;
    	input.ki.wScan = ch;
    	vec.push_back(input);

    	input.ki.dwFlags |= KEYEVENTF_KEYUP;
    	vec.push_back(input);
    }

    SendInput(vec.size(), vec.data(), sizeof(INPUT));
}

/*
 * Send the backspace keypress, *count* times.
 */
void delete_string(int32_t count, int32_t delay) {
    if (delay != 0) {
        send_multi_vkey_with_delay(VK_BACK, count, delay);
    }else{
        send_multi_vkey(VK_BACK, count);
    }
}

void send_vkey(int32_t vk) {
    std::vector<INPUT> vec;

    INPUT input = { 0 };

    input.type = INPUT_KEYBOARD;
    input.ki.wScan = 0;
    input.ki.time = 0;
    input.ki.dwExtraInfo = 0;
    input.ki.wVk = vk;
    input.ki.dwFlags = 0; // 0 for key press
    vec.push_back(input);

    input.ki.dwFlags = KEYEVENTF_KEYUP; // KEYEVENTF_KEYUP for key release
    vec.push_back(input);

    SendInput(vec.size(), vec.data(), sizeof(INPUT));
}

void send_multi_vkey(int32_t vk, int32_t count) {
    std::vector<INPUT> vec;

    for (int i = 0; i < count; i++) {
        INPUT input = { 0 };

        input.type = INPUT_KEYBOARD;
        input.ki.wScan = 0;
        input.ki.time = 0;
        input.ki.dwExtraInfo = 0;
        input.ki.wVk = vk;
        input.ki.dwFlags = 0; // 0 for key press
        vec.push_back(input);

        input.ki.dwFlags = KEYEVENTF_KEYUP; // KEYEVENTF_KEYUP for key release
        vec.push_back(input);
    }

    SendInput(vec.size(), vec.data(), sizeof(INPUT));
}

void send_multi_vkey_with_delay(int32_t vk, int32_t count, int32_t delay) {
    for (int i = 0; i < count; i++) {
        INPUT input = { 0 };

        input.type = INPUT_KEYBOARD;
        input.ki.wScan = 0;
        input.ki.time = 0;
        input.ki.dwExtraInfo = 0;
        input.ki.wVk = vk;
        input.ki.dwFlags = 0; // 0 for key press
        SendInput(1, &input, sizeof(INPUT));

        Sleep(delay);

        input.ki.dwFlags = KEYEVENTF_KEYUP; // KEYEVENTF_KEYUP for key release
        SendInput(1, &input, sizeof(INPUT));

        Sleep(delay);
    }
}


void trigger_shift_paste() {
    std::vector<INPUT> vec;

    INPUT input = { 0 };

    input.type = INPUT_KEYBOARD;
    input.ki.wScan = 0;
    input.ki.time = 0;
    input.ki.dwExtraInfo = 0;
    input.ki.wVk = VK_CONTROL;
    input.ki.dwFlags = 0; // 0 for key press
    vec.push_back(input);

    input.ki.wVk = VK_SHIFT;  // SHIFT KEY
    vec.push_back(input);

    input.ki.wVk = 0x56;  // V KEY
    vec.push_back(input);

    input.ki.dwFlags = KEYEVENTF_KEYUP; // KEYEVENTF_KEYUP for key release
    vec.push_back(input);

    input.ki.wVk = VK_SHIFT;  // SHIFT KEY
    vec.push_back(input);

    input.ki.wVk = VK_CONTROL;
    vec.push_back(input);

    SendInput(vec.size(), vec.data(), sizeof(INPUT));
}

void trigger_paste() {
    std::vector<INPUT> vec;

    INPUT input = { 0 };

    input.type = INPUT_KEYBOARD;
    input.ki.wScan = 0;
    input.ki.time = 0;
    input.ki.dwExtraInfo = 0;
    input.ki.wVk = VK_CONTROL;
    input.ki.dwFlags = 0; // 0 for key press
    vec.push_back(input);

    input.ki.wVk = 0x56;  // V KEY
    vec.push_back(input);

    input.ki.dwFlags = KEYEVENTF_KEYUP; // KEYEVENTF_KEYUP for key release
    vec.push_back(input);

    input.ki.wVk = VK_CONTROL;
    vec.push_back(input);

    SendInput(vec.size(), vec.data(), sizeof(INPUT));
}

void trigger_copy() {
    std::vector<INPUT> vec;

    INPUT input = { 0 };

    input.type = INPUT_KEYBOARD;
    input.ki.wScan = 0;
    input.ki.time = 0;
    input.ki.dwExtraInfo = 0;
    input.ki.wVk = VK_CONTROL;
    input.ki.dwFlags = 0; // 0 for key press
    vec.push_back(input);

    input.ki.wVk = 0x43;  // C KEY
    vec.push_back(input);

    input.ki.dwFlags = KEYEVENTF_KEYUP; // KEYEVENTF_KEYUP for key release
    vec.push_back(input);

    input.ki.wVk = VK_CONTROL;
    vec.push_back(input);

    SendInput(vec.size(), vec.data(), sizeof(INPUT));
}

int32_t are_modifiers_pressed() {
    short ctrl_pressed = GetAsyncKeyState(VK_CONTROL);
    short enter_pressed = GetAsyncKeyState(VK_RETURN);
    short alt_pressed = GetAsyncKeyState(VK_MENU);
    short shift_pressed = GetAsyncKeyState(VK_SHIFT);
    short meta_pressed = GetAsyncKeyState(VK_LWIN);
    short rmeta_pressed = GetAsyncKeyState(VK_RWIN);
    if (((ctrl_pressed & 0x8000) + 
        (enter_pressed & 0x8000) + 
        (alt_pressed & 0x8000) +
        (shift_pressed & 0x8000) + 
        (meta_pressed & 0x8000) + 
        (rmeta_pressed & 0x8000)) != 0) {
            return 1;
        }
    
    return 0;
}


// SYSTEM

int32_t get_active_window_name(wchar_t * buffer, int32_t size) {
    HWND hwnd = GetForegroundWindow();

    return GetWindowText(hwnd, buffer, size);
}

int32_t get_active_window_executable(wchar_t * buffer, int32_t size) {
    HWND hwnd = GetForegroundWindow();

    // Extract the window PID
    DWORD windowPid;
    GetWindowThreadProcessId(hwnd, &windowPid);

    DWORD dsize = (DWORD) size;

    // Extract the process executable file path
    HANDLE process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, windowPid);
    int res = QueryFullProcessImageNameW(process, 0, buffer, &dsize);
    CloseHandle(process);

    return res;
}

// Notifications

int32_t show_notification(wchar_t * message) {
    if (nw != NULL) {
        wchar_t * buffer = new wchar_t[100];
        swprintf(buffer, 100, L"%ls", message);

        PostMessage(nw, APPWM_NOTIFICATION_POPUP, reinterpret_cast<WPARAM>(buffer), 0);
        return 1;
    }

    return -1;
}

void close_notification() {
    if (nw != NULL) {
        PostMessage(nw, APPWM_NOTIFICATION_CLOSE, 0, 0);
    }
}

int32_t show_context_menu(MenuItem * items, int32_t count) {
    if (nw != NULL) {
        MenuItem * items_buffer = new MenuItem[count];
        memcpy(items_buffer, items, sizeof(MenuItem)*count);

        PostMessage(nw, APPWM_SHOW_CONTEXT_MENU, reinterpret_cast<WPARAM>(items_buffer), static_cast<LPARAM>(count));
        return 1;
    }

    return -1;
}

void cleanup_ui() {
    Shell_NotifyIcon(NIM_DELETE, &nid);
}

// SYSTEM

int32_t start_daemon_process() {
    wchar_t cmd[MAX_PATH];
    swprintf(cmd, MAX_PATH, L"espanso.exe daemon");

    // Get current espanso directory
    TCHAR espansoFilePath[MAX_PATH];
    GetModuleFileName(NULL, espansoFilePath, MAX_PATH);

    STARTUPINFO si = { sizeof(si) };
    PROCESS_INFORMATION pi;

    // Documentation: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw
    BOOL res = CreateProcess(
            espansoFilePath,
            cmd,
            NULL,
            NULL,
            FALSE,
            DETACHED_PROCESS | CREATE_NO_WINDOW,
            NULL,
            NULL,
            &si,
            &pi
    );

    if (!res) {
        return -1;
    }

    return 1;
}


int32_t start_process(wchar_t * _cmd) {
    wchar_t cmd[MAX_PATH];
    swprintf(cmd, MAX_PATH, _cmd);

    STARTUPINFO si = { sizeof(si) };
    PROCESS_INFORMATION pi;

    // Documentation: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw
    BOOL res = CreateProcess(
            NULL,
            cmd,
            NULL,
            NULL,
            FALSE,
            DETACHED_PROCESS,
            NULL,
            NULL,
            &si,
            &pi
    );

    if (!res) {
        return -1;
    }

    return 1;
}

// CLIPBOARD

int32_t set_clipboard(wchar_t *text) {
    int32_t result = 0;
    const size_t len = wcslen(text) + 1;
    HGLOBAL hMem =  GlobalAlloc(GMEM_MOVEABLE, len * sizeof(wchar_t));
    memcpy(GlobalLock(hMem), text, len * sizeof(wchar_t));
    GlobalUnlock(hMem);
    if (!OpenClipboard(NULL)) {
        return -1;
    }
    EmptyClipboard();
    if (!SetClipboardData(CF_UNICODETEXT, hMem)) {
        result = -2;
    }
    CloseClipboard();
    return result;
}

int32_t get_clipboard(wchar_t *buffer, int32_t size) {
    int32_t result = 1;
    if (!OpenClipboard(NULL)) {
        return -1;
    }

    // Get handle of clipboard object for ANSI text
    HANDLE hData = GetClipboardData(CF_UNICODETEXT);
    if (!hData) {
        result = -2;
    }else{
        HGLOBAL hMem = GlobalLock(hData);
        if (!hMem) {
            result = -3;
        }else{
            GlobalUnlock(hMem);
            swprintf(buffer, size, L"%s", hMem);
        }
    }

    CloseClipboard();
    return result;
}

int32_t set_clipboard_image(wchar_t *path) {
    bool result = false;

    Gdiplus::GdiplusStartupInput gdiplusStartupInput;
    ULONG_PTR gdiplusToken;
    Gdiplus::GdiplusStartup(&gdiplusToken, &gdiplusStartupInput, NULL);

    Gdiplus::Bitmap *gdibmp = Gdiplus::Bitmap::FromFile(path);
    if (gdibmp)
    {
        HBITMAP hbitmap;
        gdibmp->GetHBITMAP(0, &hbitmap);
        if (OpenClipboard(NULL))
        {
            EmptyClipboard();
            DIBSECTION ds;
            if (GetObject(hbitmap, sizeof(DIBSECTION), &ds))
            {
                HDC hdc = GetDC(HWND_DESKTOP);
                //create compatible bitmap (get DDB from DIB)
                HBITMAP hbitmap_ddb = CreateDIBitmap(hdc, &ds.dsBmih, CBM_INIT,
                                                     ds.dsBm.bmBits, (BITMAPINFO*)&ds.dsBmih, DIB_RGB_COLORS);
                ReleaseDC(HWND_DESKTOP, hdc);
                SetClipboardData(CF_BITMAP, hbitmap_ddb);
                DeleteObject(hbitmap_ddb);
                result = true;
            }
            CloseClipboard();
        }

        //cleanup:
        DeleteObject(hbitmap);
        delete gdibmp;
    }

    Gdiplus::GdiplusShutdown(gdiplusToken);

    return result;
}

// Inspired by https://docs.microsoft.com/en-za/troubleshoot/cpp/add-html-code-clipboard
int32_t set_clipboard_html(char * html, wchar_t * text_fallback) {
    // Get clipboard id for HTML format
    static int cfid = 0;
    if(!cfid) {
        cfid = RegisterClipboardFormat(L"HTML Format");
    }

    int32_t result = 0;
    const size_t html_len = strlen(html) + 1;
    HGLOBAL hMem =  GlobalAlloc(GMEM_MOVEABLE, html_len * sizeof(char));
    memcpy(GlobalLock(hMem), html, html_len * sizeof(char));
    GlobalUnlock(hMem);

    const size_t fallback_len = wcslen(text_fallback) + 1;
    HGLOBAL hMemFallback =  GlobalAlloc(GMEM_MOVEABLE, fallback_len * sizeof(wchar_t));
    memcpy(GlobalLock(hMemFallback), text_fallback, fallback_len * sizeof(wchar_t));
    GlobalUnlock(hMemFallback);

    if (!OpenClipboard(NULL)) {
        return -1;
    }
    EmptyClipboard();
    if (!SetClipboardData(cfid, hMem)) {
        result = -2;
    }
    
    if (!SetClipboardData(CF_UNICODETEXT, hMemFallback)) {
        result = -3;
    }
    CloseClipboard();
    GlobalFree(hMem);
    return result;
}
