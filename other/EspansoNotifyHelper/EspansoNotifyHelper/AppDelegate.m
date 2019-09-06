//
//  AppDelegate.m
//  NotificationHelper
//
//  Created by Federico on 06/09/2019.
//  Copyright © 2019 Federico Terzi. All rights reserved.
//

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
    
    if ([args count] > 2) {
        title = args[1];
        desc = args[2];
    }
    
    NSUserNotification *notification = [[NSUserNotification alloc] init];
    notification.title = title;
    notification.informativeText = desc;
    notification.soundName = nil;
    
    [[NSUserNotificationCenter defaultUserNotificationCenter] deliverNotification:notification];
    
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
