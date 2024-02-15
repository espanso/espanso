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
#include <string.h>
#import <Foundation/Foundation.h>
#import <CoreGraphics/CoreGraphics.h>
#include <vector>

// Events dispatched by espanso are "marked" with a custom location
// so that we can later skip them in the detect module.
CGPoint ESPANSO_POINT_MARKER = CGPointMake(-27469, 0);

void inject_string(char *string, int32_t delay)
{
  long udelay = delay * 1000;

  char * stringCopy = strdup(string);
  dispatch_async(dispatch_get_main_queue(), ^(void) {
    // Convert the c string to a UniChar array as required by the CGEventKeyboardSetUnicodeString method
    NSString *nsString = [NSString stringWithUTF8String:stringCopy];
    CFStringRef cfString = (__bridge CFStringRef) nsString;
    std::vector <UniChar> buffer(nsString.length);
    CFStringGetCharacters(cfString, CFRangeMake(0, nsString.length), buffer.data());

    free(stringCopy);

    // Send the event

    // Check if the shift key is down, and if so, release it
    // To see why: https://github.com/espanso/espanso/issues/279
    if (CGEventSourceKeyState(kCGEventSourceStateHIDSystemState, 0x38)) {
      CGEventRef e2 = CGEventCreateKeyboardEvent(NULL, 0x38, false);
      CGEventSetLocation(e2, ESPANSO_POINT_MARKER);
      CGEventPost(kCGHIDEventTap, e2);
      CFRelease(e2);

      usleep(udelay);
    }

    // Because of a bug ( or undocumented limit ) of the CGEventKeyboardSetUnicodeString method
    // the string gets truncated after 20 characters, so we need to send multiple events.

    int i = 0;
    while (i < buffer.size()) {
      int chunk_size = 20;
      if ((i+chunk_size) >  buffer.size()) {
        chunk_size = buffer.size() - i;
      }

      UniChar * offset_buffer = buffer.data() + i;
      CGEventRef e = CGEventCreateKeyboardEvent(NULL, 0x31, true);
      CGEventSetLocation(e, ESPANSO_POINT_MARKER);
      CGEventKeyboardSetUnicodeString(e, chunk_size, offset_buffer);
      CGEventPost(kCGHIDEventTap, e);
      CFRelease(e);

      usleep(udelay);

      // Some applications require an explicit release of the space key
      // For more information: https://github.com/espanso/espanso/issues/159
      CGEventRef e2 = CGEventCreateKeyboardEvent(NULL, 0x31, false);
      CGEventSetLocation(e2, ESPANSO_POINT_MARKER);
      CGEventPost(kCGHIDEventTap, e2);
      CFRelease(e2);

      usleep(udelay);

      i += chunk_size;
    }
  });
}

void inject_separate_vkeys(int32_t *_vkey_array, int32_t vkey_count, int32_t delay)
{
  long udelay = delay * 1000;
  
  // Create an heap allocated copy of the array, so that it doesn't get freed within the block
  int32_t *vkey_array = (int32_t*)malloc(sizeof(int32_t)*vkey_count);
  memcpy(vkey_array, _vkey_array, sizeof(int32_t)*vkey_count);

  dispatch_async(dispatch_get_main_queue(), ^(void) {
    for (int i = 0; i<vkey_count; i++) {
      CGEventRef keydown;
      keydown = CGEventCreateKeyboardEvent(NULL, vkey_array[i], true);
      CGEventSetLocation(keydown, ESPANSO_POINT_MARKER);
      CGEventPost(kCGHIDEventTap, keydown);
      CFRelease(keydown);

      usleep(udelay);

      CGEventRef keyup;
      keyup = CGEventCreateKeyboardEvent(NULL, vkey_array[i], false);
      CGEventSetLocation(keyup, ESPANSO_POINT_MARKER);
      CGEventPost(kCGHIDEventTap, keyup);
      CFRelease(keyup);

      usleep(udelay);
    }

    free(vkey_array);
  });
}

void inject_vkeys_combination(int32_t *_vkey_array, int32_t vkey_count, int32_t delay)
{
  long udelay = delay * 1000;
  
  // Create an heap allocated copy of the array, so that it doesn't get freed within the block
  int32_t *vkey_array = (int32_t*)malloc(sizeof(int32_t)*vkey_count);
  memcpy(vkey_array, _vkey_array, sizeof(int32_t)*vkey_count);

  dispatch_async(dispatch_get_main_queue(), ^(void) {
    // First send the presses
    for (int i = 0; i < vkey_count; i++)
    {
      CGEventRef keydown;
      keydown = CGEventCreateKeyboardEvent(NULL, vkey_array[i], true);
      CGEventSetLocation(keydown, ESPANSO_POINT_MARKER);
      CGEventPost(kCGHIDEventTap, keydown);
      CFRelease(keydown);

      usleep(udelay);
    }

    // Then the releases
    for (int i = (vkey_count - 1); i >= 0; i--)
    {
      CGEventRef keyup;
      keyup = CGEventCreateKeyboardEvent(NULL, vkey_array[i], false);
      CGEventSetLocation(keyup, ESPANSO_POINT_MARKER);
      CGEventPost(kCGHIDEventTap, keyup);
      CFRelease(keyup);

      usleep(udelay);
    }
    
    free(vkey_array);
  });
}
