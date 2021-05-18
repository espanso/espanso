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
#include "AppDelegate.h"
#import <Foundation/Foundation.h>
#include <IOKit/IOKitLib.h>
#include <stdio.h>
#include <string.h>
#include <libproc.h>

void ui_initialize(void *_self, UIOptions _options)
{
  AppDelegate *delegate = [[AppDelegate alloc] init];
  delegate->options = _options;
  delegate->rust_instance = _self;

  NSApplication * application = [NSApplication sharedApplication];
  [application setDelegate:delegate];
}

int32_t ui_eventloop(EventCallback _callback)
{
  AppDelegate *delegate = (AppDelegate*)[[NSApplication sharedApplication] delegate];
  delegate->event_callback = _callback;

  [NSApp run];

  return 1;
}

void ui_exit() {
  [NSApp stop:nil];
  [NSApp abortModal];
}

void ui_update_tray_icon(int32_t index)
{
  dispatch_async(dispatch_get_main_queue(), ^(void) {
    AppDelegate *delegate = (AppDelegate*)[[NSApplication sharedApplication] delegate];
    [delegate setIcon: index];
  });
}

void ui_show_notification(char *message, double delay)
{
  NSString *nsMessage = [NSString stringWithUTF8String:message];
  dispatch_async(dispatch_get_main_queue(), ^(void) {
    AppDelegate *delegate = (AppDelegate*)[[NSApplication sharedApplication] delegate];
    [delegate showNotification: nsMessage withDelay: delay];
  });
}

void ui_show_context_menu(char *payload)
{
  NSString *nsPayload = [NSString stringWithUTF8String:payload];
  dispatch_async(dispatch_get_main_queue(), ^(void) {
    AppDelegate *delegate = (AppDelegate*)[[NSApplication sharedApplication] delegate];
    [delegate popupMenu: nsPayload];
  });
}