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

#include <gdiplus.h>

#include <Windows.h>

int32_t clipboard_get_text(wchar_t *buffer, int32_t buffer_size)
{
  int32_t result = 0;

  if (OpenClipboard(NULL))
  {
    HANDLE hData;
    if (hData = GetClipboardData(CF_UNICODETEXT))
    {
      HGLOBAL hMem;
      if (hMem = GlobalLock(hData))
      {
        GlobalUnlock(hMem);
        wcsncpy(buffer, (wchar_t *)hMem, buffer_size);
        if (wcsnlen_s(buffer, buffer_size) > 0)
        {
          result = 1;
        }
      }
    }

    CloseClipboard();
  }

  return result;
}

int32_t clipboard_set_text(wchar_t *text)
{
  int32_t result = 0;
  const size_t len = wcslen(text) + 1;

  if (OpenClipboard(NULL))
  {
    EmptyClipboard();

    HGLOBAL hMem;
    if (hMem = GlobalAlloc(GMEM_MOVEABLE, len * sizeof(wchar_t)))
    {
      memcpy(GlobalLock(hMem), text, len * sizeof(wchar_t));
      GlobalUnlock(hMem);

      if (SetClipboardData(CF_UNICODETEXT, hMem))
      {
        result = 1;
      }
    }

    CloseClipboard();
  }

  return result;
}

int32_t clipboard_set_image(wchar_t *path)
{
  int32_t result = 0;

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
                                             ds.dsBm.bmBits, (BITMAPINFO *)&ds.dsBmih, DIB_RGB_COLORS);
        ReleaseDC(HWND_DESKTOP, hdc);
        SetClipboardData(CF_BITMAP, hbitmap_ddb);
        DeleteObject(hbitmap_ddb);
        result = 1;
      }
      CloseClipboard();
    }

    DeleteObject(hbitmap);
    delete gdibmp;
  }

  Gdiplus::GdiplusShutdown(gdiplusToken);

  return result;
}

// Inspired by https://docs.microsoft.com/en-za/troubleshoot/cpp/add-html-code-clipboard
int32_t clipboard_set_html(char * html_descriptor, wchar_t * fallback_text) {
  // Get clipboard id for HTML format
  static int cfid = 0;
  if(!cfid) {
      cfid = RegisterClipboardFormat(L"HTML Format");
  }

  int32_t result = 0;
  
  const size_t html_len = strlen(html_descriptor) + 1;  
  const size_t fallback_len = (fallback_text != nullptr) ? wcslen(fallback_text) + 1 : 0;

  if (OpenClipboard(NULL))
  {
    EmptyClipboard();

    // First copy the HTML
    HGLOBAL hMem;
    if (hMem = GlobalAlloc(GMEM_MOVEABLE, html_len * sizeof(char)))
    {
      memcpy(GlobalLock(hMem), html_descriptor, html_len * sizeof(char));
      GlobalUnlock(hMem);

      if (SetClipboardData(cfid, hMem))
      {
        result = 1;
      }
    }

    // Then try to set the fallback text, if present.
    if (fallback_len > 0) {
      if (hMem = GlobalAlloc(GMEM_MOVEABLE, fallback_len * sizeof(wchar_t)))
      {
        memcpy(GlobalLock(hMem), fallback_text, fallback_len * sizeof(wchar_t));
        GlobalUnlock(hMem);

        SetClipboardData(CF_UNICODETEXT, hMem);
      }
    }

    CloseClipboard();
  }

  return result;
}