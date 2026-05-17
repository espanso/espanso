/*
 * This file is part of espanso.
 *
 * Copyright (C) 2026 Federico Terzi and the espanso contributors
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

#import <AppKit/AppKit.h>

// Reports whether the key window's first responder currently has marked
// (uncommitted) text — i.e. an IME composition is in progress.
//
// macOS NSTextField uses a shared field editor (an NSTextView) for editing,
// so when the user is typing into the search bar the firstResponder is
// usually that field editor rather than the NSTextField itself.
//
// We check NSTextView directly first, fall back to NSTextField's
// currentEditor, and finally to any responder conforming to
// NSTextInputClient. Returns false on any unexpected state.
bool IsImeComposingInKeyWindow() {
    NSWindow *keyWindow = [NSApp keyWindow];
    if (!keyWindow) {
        return false;
    }
    NSResponder *responder = [keyWindow firstResponder];
    if (!responder) {
        return false;
    }

    if ([responder isKindOfClass:[NSTextView class]]) {
        return [(NSTextView *)responder hasMarkedText];
    }
    if ([responder isKindOfClass:[NSTextField class]]) {
        NSText *editor = [(NSTextField *)responder currentEditor];
        if ([editor isKindOfClass:[NSTextView class]]) {
            return [(NSTextView *)editor hasMarkedText];
        }
    }
    if ([responder conformsToProtocol:@protocol(NSTextInputClient)]) {
        return [(id<NSTextInputClient>)responder hasMarkedText];
    }
    return false;
}
