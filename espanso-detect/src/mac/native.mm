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
#include <Carbon/Carbon.h>

#include <string.h>

const unsigned long long FLAGS = NSEventMaskKeyDown | NSEventMaskKeyUp | NSEventMaskFlagsChanged | NSEventMaskLeftMouseDown | 
                                 NSEventMaskLeftMouseUp | NSEventMaskRightMouseDown | NSEventMaskRightMouseUp | 
                                 NSEventMaskOtherMouseDown | NSEventMaskOtherMouseUp;

void * detect_initialize(EventCallback callback) {
  dispatch_async(dispatch_get_main_queue(), ^(void) {
    [NSEvent addGlobalMonitorForEventsMatchingMask:FLAGS handler:^(NSEvent *event){
        InputEvent inputEvent = {};
        if (event.type == NSEventTypeKeyDown || event.type == NSEventTypeKeyUp ) {
          inputEvent.event_type = INPUT_EVENT_TYPE_KEYBOARD;
          inputEvent.status = (event.type == NSEventTypeKeyDown) ? INPUT_STATUS_PRESSED : INPUT_STATUS_RELEASED;
          inputEvent.key_code = event.keyCode;

          const char *chars = [event.characters UTF8String];
          strncpy(inputEvent.buffer, chars, 23);
          inputEvent.buffer_len = event.characters.length;

          callback(inputEvent);
        }else if (event.type == NSEventTypeLeftMouseDown || event.type == NSEventTypeRightMouseDown || event.type == NSEventTypeOtherMouseDown ||
                  event.type == NSEventTypeLeftMouseUp || event.type == NSEventTypeRightMouseUp || event.type == NSEventTypeOtherMouseUp) {
          inputEvent.event_type = INPUT_EVENT_TYPE_MOUSE;
          inputEvent.status = (event.type == NSEventTypeLeftMouseDown || event.type == NSEventTypeRightMouseDown ||
                               event.type == NSEventTypeOtherMouseDown) ? INPUT_STATUS_PRESSED : INPUT_STATUS_RELEASED;
          if (event.type == NSEventTypeLeftMouseDown || event.type == NSEventTypeLeftMouseUp) {
            inputEvent.key_code = INPUT_MOUSE_LEFT_BUTTON;
          } else if (event.type == NSEventTypeRightMouseDown || event.type == NSEventTypeRightMouseUp) {
            inputEvent.key_code = INPUT_MOUSE_RIGHT_BUTTON;
          } else if (event.type == NSEventTypeOtherMouseDown || event.type == NSEventTypeOtherMouseUp) {
            inputEvent.key_code = INPUT_MOUSE_MIDDLE_BUTTON;
          }

          callback(inputEvent);
        }else{
          // Modifier keys (SHIFT, CTRL, ecc) are handled as a separate case on macOS
          inputEvent.event_type = INPUT_EVENT_TYPE_KEYBOARD;
          inputEvent.key_code = event.keyCode;

          // To determine whether these keys are pressed or released, we have to analyze each case
          if (event.keyCode == kVK_Shift || event.keyCode == kVK_RightShift) {
            inputEvent.status = (([event modifierFlags] & NSEventModifierFlagShift) == 0) ? INPUT_STATUS_RELEASED : INPUT_STATUS_PRESSED;
          } else if (event.keyCode == kVK_Command || event.keyCode == kVK_RightCommand) {
            inputEvent.status = (([event modifierFlags] & NSEventModifierFlagCommand) == 0) ? INPUT_STATUS_RELEASED : INPUT_STATUS_PRESSED;
          } else if (event.keyCode == kVK_Control || event.keyCode == kVK_RightControl) {
            inputEvent.status = (([event modifierFlags] & NSEventModifierFlagControl) == 0) ? INPUT_STATUS_RELEASED : INPUT_STATUS_PRESSED;
          } else if (event.keyCode == kVK_Option || event.keyCode == kVK_RightOption) {
            inputEvent.status = (([event modifierFlags] & NSEventModifierFlagOption) == 0) ? INPUT_STATUS_RELEASED : INPUT_STATUS_PRESSED;
          }
          callback(inputEvent);
        }
    }];
  });
}