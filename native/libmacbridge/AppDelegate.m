#import "AppDelegate.h"

@implementation AppDelegate

// 10.9+ only, see this url for compatibility:
// http://stackoverflow.com/questions/17693408/enable-access-for-assistive-devices-programmatically-on-10-9
BOOL checkAccessibility()
{
    NSDictionary* opts = @{(__bridge id)kAXTrustedCheckOptionPrompt: @YES};
    return AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)opts);
}

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification
{
    if (checkAccessibility()) {
        NSLog(@"Accessibility Enabled");
    }else {
        NSLog(@"Accessibility Disabled");
    }

    NSLog(@"registering keydown mask");
    [NSEvent addGlobalMonitorForEventsMatchingMask:(NSEventMaskKeyDown | NSEventMaskFlagsChanged)
            handler:^(NSEvent *event){
        if (event.type == NSEventTypeKeyDown) {
            NSLog(@"keydown: %@, %d", event.characters, event.keyCode);
        }else{
            NSLog(@"keydown: %d", event.keyCode);
        }
    }];
}

@end