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
#include <array>
#include <cstring>
#include <iostream>
#include <memory>
#include <stdio.h>
#include <string>
#include <vector>

#define UNICODE

#ifdef __MINGW32__
#ifndef WINVER
#define WINVER 0x0606
#endif
#define STRSAFE_NO_DEPRECATE
#endif

#include <strsafe.h>
#include <windows.h>
#include <winuser.h>

#include <gdiplus.h>

#include <Windows.h>

int32_t clipboard_get_length() {
    // Open the clipboard
    if (!OpenClipboard(NULL)) {
        return -1; // Failed to open clipboard
    }

    // Check if the clipboard contains text data
    HANDLE hData = GetClipboardData(CF_UNICODETEXT);
    if (hData == NULL) {
        CloseClipboard();
        return -1; // No text data in clipboard
    }

    // Get the size of the clipboard data
    SIZE_T size = GlobalSize(hData);
    if (size == 0) {
        CloseClipboard();
        return -1; // Failed to get size
    }

    // Close the clipboard
    CloseClipboard();

    // Return the size of the clipboard data
    return static_cast<int32_t>(size);
}

int32_t clipboard_get_text(wchar_t *buffer, int32_t buffer_size) {
    int32_t result = 0;

    if (OpenClipboard(NULL)) {
        HANDLE hData;
        if (hData = GetClipboardData(CF_UNICODETEXT)) {
            HGLOBAL hMem;
            if (hMem = GlobalLock(hData)) {
                GlobalUnlock(hMem);
                wcsncpy(buffer, (wchar_t *)hMem, buffer_size);
                if (wcsnlen_s(buffer, buffer_size) > 0) {
                    result = 1;
                }
            }
        }

        CloseClipboard();
    }

    return result;
}

int32_t clipboard_set_text(wchar_t *text) {
    int32_t result = 0;
    const size_t len = wcslen(text) + 1;

    if (OpenClipboard(NULL)) {
        EmptyClipboard();

        HGLOBAL hMem;
        if (hMem = GlobalAlloc(GMEM_MOVEABLE, len * sizeof(wchar_t))) {
            memcpy(GlobalLock(hMem), text, len * sizeof(wchar_t));
            GlobalUnlock(hMem);

            if (SetClipboardData(CF_UNICODETEXT, hMem)) {
                result = 1;
            }
        }

        CloseClipboard();
    }

    return result;
}

int32_t clipboard_set_image(wchar_t *path) {
    int32_t result = 0;

    Gdiplus::GdiplusStartupInput gdiplusStartupInput;
    ULONG_PTR gdiplusToken;
    Gdiplus::GdiplusStartup(&gdiplusToken, &gdiplusStartupInput, NULL);

    Gdiplus::Bitmap *gdibmp = Gdiplus::Bitmap::FromFile(path);
    if (gdibmp) {
        // GetHBITMAP alpha-blends transparent pixels onto the given
        // background color, and CF_BITMAP (a DDB) cannot carry alpha.
        // The background must be white: with the previous value (0 =
        // transparent black) every PNG with an alpha channel came out
        // black wherever it was transparent.
        HBITMAP hbitmap = NULL;
        Gdiplus::Status status =
            gdibmp->GetHBITMAP(Gdiplus::Color(255, 255, 255, 255), &hbitmap);
        if (status == Gdiplus::Ok && OpenClipboard(NULL)) {
            EmptyClipboard();
            DIBSECTION ds;
            if (GetObject(hbitmap, sizeof(DIBSECTION), &ds)) {
                HDC hdc = GetDC(HWND_DESKTOP);
                // create compatible bitmap (get DDB from DIB)
                HBITMAP hbitmap_ddb =
                    CreateDIBitmap(hdc, &ds.dsBmih, CBM_INIT, ds.dsBm.bmBits,
                                   (BITMAPINFO *)&ds.dsBmih, DIB_RGB_COLORS);
                ReleaseDC(HWND_DESKTOP, hdc);
                if (SetClipboardData(CF_BITMAP, hbitmap_ddb)) {
                    result = 1;
                } else {
                    // Ownership only transfers to the clipboard on success.
                    DeleteObject(hbitmap_ddb);
                }
            }

            // Additionally publish the raw bytes under the registered "PNG"
            // clipboard format: alpha-aware targets (browsers, Office,
            // Telegram, ...) prefer it and keep the transparency intact.
            FILE *file = _wfopen(path, L"rb");
            if (file) {
                fseek(file, 0, SEEK_END);
                long file_size = ftell(file);
                fseek(file, 0, SEEK_SET);
                if (file_size > 8) {
                    HGLOBAL hPng =
                        GlobalAlloc(GMEM_MOVEABLE, (SIZE_T)file_size);
                    if (hPng) {
                        bool is_png = false;
                        void *dst = GlobalLock(hPng);
                        if (dst) {
                            size_t read_bytes =
                                fread(dst, 1, (size_t)file_size, file);
                            const unsigned char png_magic[8] = {
                                0x89, 'P', 'N', 'G', '\r', '\n', 0x1A, '\n'};
                            is_png = read_bytes == (size_t)file_size &&
                                     memcmp(dst, png_magic, 8) == 0;
                            GlobalUnlock(hPng);
                        }
                        static UINT png_format = 0;
                        if (!png_format) {
                            png_format = RegisterClipboardFormat(L"PNG");
                        }
                        if (is_png && png_format &&
                            SetClipboardData(png_format, hPng)) {
                            result = 1;
                        } else {
                            GlobalFree(hPng);
                        }
                    }
                }
                fclose(file);
            }

            CloseClipboard();
        }

        if (hbitmap) {
            DeleteObject(hbitmap);
        }
        delete gdibmp;
    }

    Gdiplus::GdiplusShutdown(gdiplusToken);

    return result;
}

// Inspired by
// https://docs.microsoft.com/en-za/troubleshoot/cpp/add-html-code-clipboard
int32_t clipboard_set_html(char *html_descriptor, wchar_t *fallback_text) {
    // Get clipboard id for HTML format
    static int cfid = 0;
    if (!cfid) {
        cfid = RegisterClipboardFormat(L"HTML Format");
    }

    int32_t result = 0;

    const size_t html_len = strlen(html_descriptor) + 1;
    const size_t fallback_len =
        (fallback_text != nullptr) ? wcslen(fallback_text) + 1 : 0;

    if (OpenClipboard(NULL)) {
        EmptyClipboard();

        // First copy the HTML
        HGLOBAL hMem;
        if (hMem = GlobalAlloc(GMEM_MOVEABLE, html_len * sizeof(char))) {
            memcpy(GlobalLock(hMem), html_descriptor, html_len * sizeof(char));
            GlobalUnlock(hMem);

            if (SetClipboardData(cfid, hMem)) {
                result = 1;
            }
        }

        // Then try to set the fallback text, if present.
        if (fallback_len > 0) {
            if (hMem = GlobalAlloc(GMEM_MOVEABLE,
                                   fallback_len * sizeof(wchar_t))) {
                memcpy(GlobalLock(hMem), fallback_text,
                       fallback_len * sizeof(wchar_t));
                GlobalUnlock(hMem);

                SetClipboardData(CF_UNICODETEXT, hMem);
            }
        }

        CloseClipboard();
    }

    return result;
}