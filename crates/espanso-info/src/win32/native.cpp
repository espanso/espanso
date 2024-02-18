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

int32_t info_get_title(wchar_t *buffer, int32_t buffer_size)
{
  HWND hwnd = GetForegroundWindow();
  return GetWindowText(hwnd, buffer, buffer_size);
}

int32_t info_get_exec(wchar_t *buffer, int32_t buffer_size)
{
  HWND hwnd = GetForegroundWindow();

  // Extract the window PID
  DWORD windowPid;
  GetWindowThreadProcessId(hwnd, &windowPid);

  DWORD dsize = (DWORD)buffer_size;

  // Extract the process executable file path
  HANDLE process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, windowPid);
  int res = QueryFullProcessImageNameW(process, 0, buffer, &dsize);
  CloseHandle(process);

  return res;
}