#include "bridge.h"

#import <Foundation/Foundation.h>
#include "AppDelegate.h"
#include <string.h>
extern "C" {

}

#include <vector>


void register_keypress_callback(void * self, KeypressCallback callback) {
    keypress_callback = callback;
    interceptor_instance = self;
}

int32_t initialize() {
    AppDelegate *delegate = [[AppDelegate alloc] init];
    NSApplication * application = [NSApplication sharedApplication];
    [application setDelegate:delegate];
}

int32_t eventloop() {
    [NSApp run];
}

void send_string(const char * string) {
    char * stringCopy = strdup(string);
    dispatch_async(dispatch_get_main_queue(), ^(void) {
        // Convert the c string to a UniChar array as required by the CGEventKeyboardSetUnicodeString method
        NSString *nsString = [NSString stringWithUTF8String:stringCopy];
        CFStringRef cfString = (__bridge CFStringRef) nsString;
        std::vector <UniChar> buffer(nsString.length);
        CFStringGetCharacters(cfString, CFRangeMake(0, nsString.length), buffer.data());

        free(stringCopy);

        // Send the event
        CGEventRef e = CGEventCreateKeyboardEvent(NULL, 0x31, true);
        CGEventKeyboardSetUnicodeString(e, buffer.size(), buffer.data());
        CGEventPost(kCGHIDEventTap, e);
        CFRelease(e);
    });
}

void delete_string(int32_t count) {
    dispatch_async(dispatch_get_main_queue(), ^(void) {
        for (int i = 0; i < count; i++) {
            CGEventRef keydown;
            keydown = CGEventCreateKeyboardEvent(NULL, 0x33, true);
            CGEventPost(kCGHIDEventTap, keydown);
            CFRelease(keydown);

            usleep(2000);

            CGEventRef keyup;
            keyup = CGEventCreateKeyboardEvent(NULL, 0x33, false);
            CGEventPost(kCGHIDEventTap, keyup);
            CFRelease(keyup);

            usleep(2000);
        }
    });
}