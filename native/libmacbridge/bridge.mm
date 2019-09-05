#include "bridge.h"

#import <Foundation/Foundation.h>

extern "C" {
    #include "AppDelegate.h"
}

int32_t initialize() {
    AppDelegate *delegate = [[AppDelegate alloc] init];
    NSApplication * application = [NSApplication sharedApplication];
    [application setDelegate:delegate];
}

int32_t eventloop() {
    [NSApp run];
}