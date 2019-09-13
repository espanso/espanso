#import "AppDelegate.h"

@implementation AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification
{
    // Setup status icon
    myStatusItem = [[[NSStatusBar systemStatusBar] statusItemWithLength:NSSquareStatusItemLength] retain];

    NSString *nsIconPath = [NSString stringWithUTF8String:icon_path];
    NSImage *statusImage = [[NSImage alloc] initWithContentsOfFile:nsIconPath];
    [statusImage setTemplate:YES];

    [myStatusItem.button setImage:statusImage];
    [myStatusItem setHighlightMode:YES];
    [myStatusItem.button setAction:@selector(statusIconClick:)];
    [myStatusItem.button setTarget:self];

    // Setup key listener
    [NSEvent addGlobalMonitorForEventsMatchingMask:(NSEventMaskKeyDown | NSEventMaskFlagsChanged)
            handler:^(NSEvent *event){
        if (event.type == NSEventTypeKeyDown
            && event.keyCode != 0x33) { // Send backspace as a modifier

            const char * chars = [event.characters UTF8String];
            int len = event.characters.length;

            keypress_callback(context_instance, chars, len, 0, event.keyCode);
            //NSLog(@"keydown: %@, %d", event.characters, event.keyCode);
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

- (IBAction) statusIconClick: (id) sender {
    icon_click_callback(context_instance);
}

- (IBAction) contextMenuClick: (id) sender {
    NSInteger item_id = [[sender valueForKey:@"tag"] integerValue];

    context_menu_click_callback(context_instance, static_cast<int32_t>(item_id));
}

@end