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