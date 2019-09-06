//
//  main.m
//  NotificationHelper
//
//  Created by Federico on 06/09/2019.
//  Copyright © 2019 Federico Terzi. All rights reserved.
//

#import <Cocoa/Cocoa.h>
#import "AppDelegate.h"

int main(int argc, const char * argv[]) {
    AppDelegate *delegate = [[AppDelegate alloc] init];
    NSApplication * application = [NSApplication sharedApplication];
    [application setDelegate:delegate];
    [NSApp run];
    
    return 0;
}
