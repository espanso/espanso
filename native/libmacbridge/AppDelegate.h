#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>

#include "bridge.h"

@interface AppDelegate : NSObject <NSApplicationDelegate> {
    @public NSStatusItem *myStatusItem;
}

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification;
- (IBAction) statusIconClick: (id) sender;
- (IBAction) contextMenuClick: (id) sender;

@end