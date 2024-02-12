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

#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>

#include "native.h"

@interface AppDelegate : NSObject <NSApplicationDelegate, NSUserNotificationCenterDelegate> {
  @public NSStatusItem *statusItem;
  @public UIOptions options;
  @public void *rust_instance;
  @public EventCallback event_callback;
}

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification;
- (void) setIcon: (int32_t) iconIndex;
- (void) popupMenu: (NSString *) payload;
- (void) showNotification: (NSString *) message withDelay:(double) delay;
- (IBAction) statusIconClick: (id) sender;
- (IBAction) contextMenuClick: (id) sender;
- (void) heartbeatHandler: (NSTimer *)timer;

@end