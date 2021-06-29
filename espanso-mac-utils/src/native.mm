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
#include <libproc.h>
#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#import <Carbon/Carbon.h>

// Taken (with a few modifications) from the MagicKeys project: https://github.com/zsszatmari/MagicKeys
int32_t mac_utils_get_secure_input_process(int64_t *pid) {
  NSArray *consoleUsersArray;
  io_service_t rootService;
  int32_t result = 0;

  if ((rootService = IORegistryGetRootEntry(kIOMasterPortDefault)) != 0)
  {
    if ((consoleUsersArray = (NSArray *)IORegistryEntryCreateCFProperty((io_registry_entry_t)rootService, CFSTR("IOConsoleUsers"), kCFAllocatorDefault, 0)) != nil)
    {
      if ([consoleUsersArray isKindOfClass:[NSArray class]])  // Be careful - ensure this really is an array
      {
        for (NSDictionary *consoleUserDict in consoleUsersArray) {
          NSNumber *secureInputPID;

          if ((secureInputPID = [consoleUserDict objectForKey:@"kCGSSessionSecureInputPID"]) != nil)
          {
            if ([secureInputPID isKindOfClass:[NSNumber class]])
            {
              *pid = ((UInt64) [secureInputPID intValue]);
              result = 1;
              break;
            }
          }
        }
      }

      CFRelease((CFTypeRef)consoleUsersArray);
    }

    IOObjectRelease((io_object_t) rootService);
  }

  return result;
}

int32_t mac_utils_get_path_from_pid(int64_t pid, char *buff, int buff_size) {
  int res = proc_pidpath((pid_t) pid, buff, buff_size);
  if ( res <= 0 ) {
    return 0;
  } else {
    return 1;
  }
}

int32_t mac_utils_check_accessibility() {
  NSDictionary* opts = @{(__bridge id)kAXTrustedCheckOptionPrompt: @NO};
  return AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)opts);
}

int32_t mac_utils_prompt_accessibility() {
  NSDictionary* opts = @{(__bridge id)kAXTrustedCheckOptionPrompt: @YES};
  return AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)opts);
}

void mac_utils_transition_to_foreground_app() {
  ProcessSerialNumber psn = { 0, kCurrentProcess };
  TransformProcessType(&psn, kProcessTransformToForegroundApplication);
}

void mac_utils_transition_to_background_app() {
  ProcessSerialNumber psn = { 0, kCurrentProcess };
  TransformProcessType(&psn, kProcessTransformToUIElementApplication);
}