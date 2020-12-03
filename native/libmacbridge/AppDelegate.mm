/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

@implementation AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification
{
    // Setup status icon
    if (show_icon) {
        myStatusItem = [[[NSStatusBar systemStatusBar] statusItemWithLength:NSSquareStatusItemLength] retain];
        [self setIcon: icon_path];
    }

    // Setup key listener
    [NSEvent addGlobalMonitorForEventsMatchingMask:(NSEventMaskKeyDown | NSEventMaskFlagsChanged | NSEventMaskLeftMouseDown | NSEventMaskRightMouseDown)
            handler:^(NSEvent *event){

        if (event.type == NSEventTypeKeyDown
            && event.keyCode != 0x33) { // Send backspace as a modifier

            const char *chars = [event.characters UTF8String];
            int len = event.characters.length;

            keypress_callback(context_instance, chars, len, 0, event.keyCode);
            //NSLog(@"keydown: %@, %d", event.characters, event.keyCode);
        }else if (event.type == NSEventTypeLeftMouseDown || event.type == NSEventTypeRightMouseDown) {
            // Send the mouse button clicks as "other" events, used to improve word matches reliability
            keypress_callback(context_instance, NULL, 0, 2, event.buttonNumber);
        }else{
            // Because this event is triggered for both the press and release of a modifier, trigger the callback
            // only on release
            if (([event modifierFlags] & (NSEventModifierFlagShift | NSEventModifierFlagCommand |
                NSEventModifierFlagControl | NSEventModifierFlagOption)) == 0) {

                keypress_callback(context_instance, NULL, 0, 1, event.keyCode);
            }

            //NSLog(@"keydown: %d", event.keyCode);
        }
    }];
}

- (void) updateIcon: (char *)iconPath {
    if (show_icon) {
        [self setIcon: iconPath];
    }
}

- (void) setIcon: (char *)iconPath {
    if (show_icon) {
        NSString *nsIconPath = [NSString stringWithUTF8String:iconPath];
        NSImage *statusImage = [[NSImage alloc] initWithContentsOfFile:nsIconPath];
        [statusImage setTemplate:YES];

        [myStatusItem.button setImage:statusImage];
        [myStatusItem setHighlightMode:YES];
        [myStatusItem.button setAction:@selector(statusIconClick:)];
        [myStatusItem.button setTarget:self];
    }
}

- (IBAction) statusIconClick: (id) sender {
    icon_click_callback(context_instance);
}

- (IBAction) contextMenuClick: (id) sender {
    NSInteger item_id = [[sender valueForKey:@"tag"] integerValue];

    context_menu_click_callback(context_instance, static_cast<int32_t>(item_id));
}

@end