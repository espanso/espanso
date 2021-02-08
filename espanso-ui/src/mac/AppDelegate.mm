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

#import "AppDelegate.h"

void addSeparatorMenu(NSMenu * parent);
void addSingleMenu(NSMenu * parent, id item);
void addSubMenu(NSMenu * parent, NSArray * items);

@implementation AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification
{
  if (options.show_icon) {
    statusItem = [[[NSStatusBar systemStatusBar] statusItemWithLength:NSSquareStatusItemLength] retain];
    [self setIcon: 0];
  }

  [[NSUserNotificationCenter defaultUserNotificationCenter] setDelegate:self];
}

- (void) setIcon: (int32_t)iconIndex {
  if (options.show_icon) {
    char * iconPath = options.icon_paths[iconIndex];
    NSString *nsIconPath = [NSString stringWithUTF8String:iconPath];

    NSImage *statusImage = [[NSImage alloc] initWithContentsOfFile:nsIconPath];
    [statusImage setTemplate:YES];

    [statusItem.button setImage:statusImage];
    [statusItem setHighlightMode:YES];
    [statusItem.button setAction:@selector(statusIconClick:)];
    [statusItem.button setTarget:self];
  }
}

- (IBAction) statusIconClick: (id) sender {
  UIEvent event = {};
  event.event_type = UI_EVENT_TYPE_ICON_CLICK;
  if (event_callback && rust_instance) {
    event_callback(rust_instance, event);
  }
}

- (void) popupMenu: (NSString *) payload {
  NSError *jsonError;
  NSData *data = [payload dataUsingEncoding:NSUTF8StringEncoding];
  NSArray *jsonMenuItems = [NSJSONSerialization JSONObjectWithData:data options:kNilOptions error:&jsonError];
  NSMenu *menu = [[NSMenu alloc] initWithTitle:@"Espanso"];
  addSubMenu(menu, jsonMenuItems);
  [statusItem popUpStatusItemMenu: menu];
}

- (IBAction) contextMenuClick: (id) sender {
  NSInteger itemId = [[sender valueForKey:@"tag"] integerValue];

  UIEvent event = {};
  event.event_type = UI_EVENT_TYPE_CONTEXT_MENU_CLICK;
  event.context_menu_id = (uint32_t) [itemId intValue];
  if (event_callback && rust_instance) {
    event_callback(rust_instance, event);
  }
}

- (void) showNotification: (NSString *) message withDelay: (double) delay {
  NSUserNotification *notification = [[NSUserNotification alloc] init];
  notification.title = @"Espanso";
  notification.informativeText = message;
  notification.soundName = nil;
  
  [[NSUserNotificationCenter defaultUserNotificationCenter] deliverNotification:notification];
  [[NSUserNotificationCenter defaultUserNotificationCenter] performSelector:@selector(removeDeliveredNotification:) withObject:notification afterDelay:delay];
}

@end

// Menu utility methods

void addSeparatorMenu(NSMenu * parent)
{
  [parent addItem: [NSMenuItem separatorItem]];
}

void addSingleMenu(NSMenu * parent, id item)
{
  id label = [item objectForKey:@"label"];
  id raw_id = [item objectForKey:@"raw_id"];
  if (label == nil || raw_id == nil)
  {
    return;
  }
  NSMenuItem *newMenu = [[NSMenuItem alloc] initWithTitle:label action:@selector(contextMenuClick:) keyEquivalent:@""];
  [newMenu setTag:(NSInteger)raw_id];
  [parent addItem: newMenu]; 
}

void addSubMenu(NSMenu * parent, NSArray * items)
{
  for (id item in items) {
    id type = [item objectForKey:@"type"];
    if ([type isEqualToString:@"simple"])
    {
      addSingleMenu(parent, item);
    }
    else if ([type isEqualToString:@"separator"])
    {
      addSeparatorMenu(parent);
    }
    else if ([type isEqualToString:@"sub"])
    {
      NSMenuItem *menuItem = [[NSMenuItem alloc] initWithTitle:[item objectForKey:@"label"] action:nil keyEquivalent:@""];
      NSMenu *subMenu = [[NSMenu alloc] init];
      [parent addItem: menuItem];
      addSubMenu(subMenu, [item objectForKey:@"items"]);
      [menuItem setSubmenu: subMenu];
    }
  }
}