// Standalone AppKit regression test. Run scripts/test_macos_shutdown.sh on
// macOS.
#include "AppDelegate.h"
#include "native.h"
#import <AppKit/AppKit.h>
#include <atomic>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <thread>

extern "C" void mac_utils_start_headless_eventloop();
extern "C" void mac_utils_exit_headless_eventloop();
extern void addSubMenu(NSMenu *parent, NSArray *items);

static std::atomic<bool> stopped_on_main(false);
static std::atomic<bool> used_abort_modal(false);
static std::atomic<bool> threw(false);
static std::atomic<bool> engine_alive(true);
static std::atomic<bool> late_callback(false);
static std::atomic<bool> reload_requested(false);
static std::atomic<bool> receiver_released_early(false);
static bool headless_mode = false;

@interface ShutdownTestApplication : NSApplication
@end
@implementation ShutdownTestApplication
- (void)stop:(id)sender {
    stopped_on_main = [NSThread isMainThread];
    if (!headless_mode && !engine_alive)
        receiver_released_early = true;
    [super stop:sender];
}
- (void)abortModal {
    used_abort_modal = true;
    [super abortModal];
}
@end

static void callback(void *, UIEvent event) {
    if (!engine_alive)
        late_callback = true;
    if (event.event_type == UI_EVENT_TYPE_CONTEXT_MENU_CLICK &&
        event.context_menu_id == 1)
        reload_requested = true;
}

int main(int argc, char **argv) {
    const bool headless = argc > 1 && std::strcmp(argv[1], "headless") == 0;
    const bool menu = argc > 1 && std::strcmp(argv[1], "menu") == 0;
    const bool main_thread =
        argc > 1 && std::strcmp(argv[1], "main-thread") == 0;
    const bool worker = argc > 1 && std::strcmp(argv[1], "worker") == 0;
    if (argc != 2 || (!headless && !menu && !main_thread && !worker)) {
        std::fprintf(stderr,
                     "Usage: shutdown worker|menu|main-thread|headless\n");
        return 2;
    }
    headless_mode = headless;
    @autoreleasepool {
        [ShutdownTestApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyProhibited];
        UIOptions options = {};
        int token = 0;
        if (!headless)
            ui_initialize(&token, options);
        if (menu) {
            dispatch_after(
                dispatch_time(DISPATCH_TIME_NOW, 300 * NSEC_PER_MSEC),
                dispatch_get_main_queue(), ^{
                  NSMenu *context_menu =
                      [[NSMenu alloc] initWithTitle:@"Espanso"];
                  addSubMenu(context_menu, @[ @{
                                 @"type" : @"simple",
                                 @"id" : @1,
                                 @"label" : @"Reload config"
                             } ]);
                  [[context_menu itemAtIndex:0] setTarget:[NSApp delegate]];
                  [context_menu performActionForItemAtIndex:0];
                  [context_menu release];
                });
        }

        // A failed wake-up must fail the test instead of hanging indefinitely.
        dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 15 * NSEC_PER_SEC),
                       dispatch_get_global_queue(QOS_CLASS_DEFAULT, 0), ^{
                         std::fprintf(stderr,
                                      "FAIL: application loop did not stop\n");
                         std::_Exit(124);
                       });
        std::thread engine;
        if (main_thread) {
            dispatch_after(
                dispatch_time(DISPATCH_TIME_NOW, 300 * NSEC_PER_MSEC),
                dispatch_get_main_queue(), ^{
                  ui_exit();
                  engine_alive = false;
                });
        } else {
            engine = std::thread([headless, menu] {
                if (menu) {
                    while (!reload_requested)
                        std::this_thread::sleep_for(
                            std::chrono::milliseconds(1));
                } else {
                    std::this_thread::sleep_for(std::chrono::milliseconds(300));
                }
                @try {
                    if (headless)
                        mac_utils_exit_headless_eventloop();
                    else
                        ui_exit();
                } @catch (NSException *exception) {
                    std::fprintf(stderr, "shutdown exception: %s\n",
                                 [[exception name] UTF8String]);
                    threw = true;
                }
                engine_alive = false;
            });
        }
        @try {
            if (headless)
                mac_utils_start_headless_eventloop();
            else
                ui_eventloop(callback);
        } @catch (NSException *exception) {
            std::fprintf(stderr, "event loop exception: %s\n",
                         [[exception name] UTF8String]);
            threw = true;
        }
        if (engine.joinable())
            engine.join();
        if (!headless) {
            // A queued timer or menu event must not reach a disconnected
            // engine.
            AppDelegate *delegate = (AppDelegate *)[NSApp delegate];
            [delegate heartbeatHandler:nil];
            [delegate statusIconClick:nil];
            [delegate contextMenuClick:nil];
        }
        const bool passed = stopped_on_main && !used_abort_modal && !threw &&
                            !late_callback && !receiver_released_early;
        std::fprintf(stderr,
                     "%s %s: main_thread=%d abortModal=%d exception=%d "
                     "late_callback=%d receiver_released_early=%d\n",
                     passed ? "PASS" : "FAIL", argv[1],
                     (int)stopped_on_main.load(), (int)used_abort_modal.load(),
                     (int)threw.load(), (int)late_callback.load(),
                     (int)receiver_released_early.load());
        return passed ? 0 : 1;
    }
}
