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

void inject_string(wchar_t *string)
{
  std::wstring msg(string);
  std::cout << msg.length() << std::endl;

  std::vector<INPUT> vec;
  for (auto ch : msg)
  {
    INPUT input = {0};
    input.type = INPUT_KEYBOARD;
    input.ki.dwFlags = KEYEVENTF_UNICODE;
    input.ki.wScan = ch;
    vec.push_back(input);

    input.ki.dwFlags |= KEYEVENTF_KEYUP;
    vec.push_back(input);
  }

  SendInput(vec.size(), vec.data(), sizeof(INPUT));
}

void inject_separate_vkeys(int32_t *vkey_array, int32_t vkey_count)
{
  std::vector<INPUT> vec;

  for (int i = 0; i < vkey_count; i++)
  {
    INPUT input = {0};

    input.type = INPUT_KEYBOARD;
    input.ki.wScan = 0;
    input.ki.time = 0;
    input.ki.dwExtraInfo = 0;
    input.ki.wVk = vkey_array[i];
    input.ki.dwFlags = 0; // 0 for key press
    vec.push_back(input);

    input.ki.dwFlags = KEYEVENTF_KEYUP;
    vec.push_back(input);
  }

  SendInput(vec.size(), vec.data(), sizeof(INPUT));
}

void inject_vkeys_combination(int32_t *vkey_array, int32_t vkey_count)
{
  std::vector<INPUT> vec;

  // First send the presses
  for (int i = 0; i < vkey_count; i++)
  {
    INPUT input = {0};
    input.type = INPUT_KEYBOARD;
    input.ki.wScan = 0;
    input.ki.time = 0;
    input.ki.dwExtraInfo = 0;
    input.ki.wVk = vkey_array[i];
    input.ki.dwFlags = 0;
    vec.push_back(input);
  }

  // Then the releases
  for (int i = (vkey_count - 1); i >= 0; i--)
  {
    INPUT input = {0};
    input.type = INPUT_KEYBOARD;
    input.ki.wScan = 0;
    input.ki.time = 0;
    input.ki.dwExtraInfo = 0;
    input.ki.wVk = vkey_array[i];
    input.ki.dwFlags = KEYEVENTF_KEYUP;
    vec.push_back(input);
  }

  SendInput(vec.size(), vec.data(), sizeof(INPUT));
}

void inject_separate_vkeys_with_delay(int32_t *vkey_array, int32_t vkey_count, int32_t delay)
{
  for (int i = 0; i < vkey_count; i++)
  {
    INPUT input = {0};

    input.type = INPUT_KEYBOARD;
    input.ki.wScan = 0;
    input.ki.time = 0;
    input.ki.dwExtraInfo = 0;
    input.ki.wVk = vkey_array[i];
    input.ki.dwFlags = 0; // 0 for key press
    SendInput(1, &input, sizeof(INPUT));

    Sleep(delay);

    input.ki.dwFlags = KEYEVENTF_KEYUP; // KEYEVENTF_KEYUP for key release
    SendInput(1, &input, sizeof(INPUT));

    Sleep(delay);
  }
}

void inject_vkeys_combination_with_delay(int32_t *vkey_array, int32_t vkey_count, int32_t delay)
{
  // First send the presses
  for (int i = 0; i < vkey_count; i++)
  {
    INPUT input = {0};
    input.type = INPUT_KEYBOARD;
    input.ki.wScan = 0;
    input.ki.time = 0;
    input.ki.dwExtraInfo = 0;
    input.ki.wVk = vkey_array[i];
    input.ki.dwFlags = 0;
    
    SendInput(1, &input, sizeof(INPUT));
    Sleep(delay);
  }

  // Then the releases
  for (int i = (vkey_count - 1); i >= 0; i--)
  {
    INPUT input = {0};
    input.type = INPUT_KEYBOARD;
    input.ki.wScan = 0;
    input.ki.time = 0;
    input.ki.dwExtraInfo = 0;
    input.ki.wVk = vkey_array[i];
    input.ki.dwFlags = KEYEVENTF_KEYUP;

    SendInput(1, &input, sizeof(INPUT));
    Sleep(delay);
  }
}