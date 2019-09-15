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

@interface AppDelegate ()

@property (weak) IBOutlet NSWindow *window;
@end

@implementation AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification {
    [[NSUserNotificationCenter defaultUserNotificationCenter] setDelegate:self];
    
    NSArray *args = [[NSProcessInfo processInfo] arguments];
    
    NSString *title = @"Title";
    NSString *desc = @"Description";
    double delay = 1.5;
    
    if ([args count] > 3) {
        title = args[1];
        desc = args[2];
        delay = [args[3] doubleValue];
    }
    
    NSUserNotification *notification = [[NSUserNotification alloc] init];
    notification.title = title;
    notification.informativeText = desc;
    notification.soundName = nil;
    
    [[NSUserNotificationCenter defaultUserNotificationCenter] deliverNotification:notification];
    
    [[NSUserNotificationCenter defaultUserNotificationCenter] performSelector:@selector(removeDeliveredNotification:) withObject:notification afterDelay:delay];
    
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 3 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
        NSRunningApplication *app = [NSRunningApplication currentApplication];
        [app terminate];
    });
}


- (void)applicationWillTerminate:(NSNotification *)aNotification {
    // Insert code here to tear down your application
}

- (BOOL)userNotificationCenter:(NSUserNotificationCenter *)center shouldPresentNotification:(NSUserNotification *)notification{
    return YES;
}


@end
