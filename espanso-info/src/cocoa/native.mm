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
#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>

int32_t info_get_title(char *buffer, int32_t buffer_size)
{
  @autoreleasepool {
    CFArrayRef windows = CGWindowListCopyWindowInfo(kCGWindowListExcludeDesktopElements | kCGWindowListOptionOnScreenOnly, kCGNullWindowID);
    int32_t result = 0;

    if (windows) {
      for (NSDictionary *window in (NSArray *)windows) {
        NSNumber *ownerPid = window[(id) kCGWindowOwnerPID];

        NSRunningApplication *currentApp = [NSRunningApplication runningApplicationWithProcessIdentifier: [ownerPid intValue]];

        if ([currentApp isActive]) {
          NSString *name = window[(id) kCGWindowName];
          if (name.length > 0) {
            const char * title = [name UTF8String];
            snprintf(buffer, buffer_size, "%s", title);
            result = 1;
          }
          break;
        }
      }

      CFRelease(windows);
    }
  }

  return 0;
}

// Partially taken from: https://stackoverflow.com/questions/480866/get-the-title-of-the-current-active-window-document-in-mac-os-x/23451568#23451568
int32_t info_get_title_fallback(char *buffer, int32_t buffer_size)
{
  @autoreleasepool {
    // Get the process ID of the frontmost application.
    NSRunningApplication* app = [[NSWorkspace sharedWorkspace] frontmostApplication];
    pid_t pid = [app processIdentifier];

    AXUIElementRef appElem = AXUIElementCreateApplication(pid);
    if (!appElem) {
      return -1;
    }

    // Get the accessibility element corresponding to the frontmost window
    // of the frontmost application.
    AXUIElementRef window = NULL;
    if (AXUIElementCopyAttributeValue(appElem, 
          kAXFocusedWindowAttribute, (CFTypeRef*)&window) != kAXErrorSuccess) {
      CFRelease(appElem);
      return -2;
    }

    // Finally, get the title of the frontmost window.
    CFStringRef title = NULL;
    AXError result = AXUIElementCopyAttributeValue(window, kAXTitleAttribute,
                      (CFTypeRef*)&title);

    // At this point, we don't need window and appElem anymore.
    CFRelease(window);
    CFRelease(appElem);

    if (result != kAXErrorSuccess) {
      // Failed to get the window title.
      return -3;
    }

    if (CFStringGetCString(title, buffer, buffer_size, kCFStringEncodingUTF8)) {
      CFRelease(title);
      return 1;
    } else {
      return -4;
    }
  }
}

int32_t info_get_exec(char *buffer, int32_t buffer_size)
{
  @autoreleasepool {
    NSRunningApplication *frontApp = [[NSWorkspace sharedWorkspace] frontmostApplication];
    NSString *bundlePath = [frontApp bundleURL].path;
    const char * path = [bundlePath UTF8String];

    snprintf(buffer, buffer_size, "%s", path);
  }

  return 1;
}

int32_t info_get_class(char *buffer, int32_t buffer_size)
{
  @autoreleasepool {
    NSRunningApplication *frontApp = [[NSWorkspace sharedWorkspace] frontmostApplication];
    NSString *bundleId = frontApp.bundleIdentifier;
    const char * bundle = [bundleId UTF8String];

    snprintf(buffer, buffer_size, "%s", bundle);
  }

  return 1;
}