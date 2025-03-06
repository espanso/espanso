/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
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

#include "native.h"
#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#include <string.h>
#import <dispatch/dispatch.h>

int32_t clipboard_get_text(char * buffer, int32_t buffer_size) {
  __block int32_t result = 0;
  dispatch_sync(dispatch_get_main_queue(), ^{
    @autoreleasepool {
      NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
      NSString *string = [pasteboard stringForType:NSPasteboardTypeString];
      if (string != NULL) {
        BOOL success = [string getCString:buffer maxLength:buffer_size encoding:NSUTF8StringEncoding];
        if (success) {
            result = 1;
        }
      }
    }
  });
  return result;
}

int32_t clipboard_set_text(char * text) {
  NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
  NSArray *array = @[NSPasteboardTypeString];
  [pasteboard declareTypes:array owner:nil];

  NSString *nsText = [NSString stringWithUTF8String:text];
  if (![pasteboard setString:nsText forType:NSPasteboardTypeString]) {
    return 0;
  }

  return 1;
}

int32_t clipboard_set_image(char * image_path) {
  NSString *pathString = [NSString stringWithUTF8String:image_path];
  NSImage *image = [[NSImage alloc] initWithContentsOfFile:pathString];
  int result = 0;

  if (image != nil) {
      NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
      [pasteboard clearContents];
      NSArray *copiedObjects = [NSArray arrayWithObject:image];
      [pasteboard writeObjects:copiedObjects];
      result = 1;
  }
  [image release];

  return result;
}

int32_t clipboard_set_html(char * html, char * fallback_text) {
  NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
  NSArray *array = @[NSRTFPboardType, NSPasteboardTypeString];
  [pasteboard declareTypes:array owner:nil];

  NSString *nsHtml = [NSString stringWithUTF8String:html];
  NSDictionary *documentAttributes = [NSDictionary dictionaryWithObjectsAndKeys:NSHTMLTextDocumentType, NSDocumentTypeDocumentAttribute, NSCharacterEncodingDocumentAttribute,[NSNumber numberWithInt:NSUTF8StringEncoding], nil];
  NSAttributedString* atr = [[NSAttributedString alloc] initWithData:[nsHtml dataUsingEncoding:NSUTF8StringEncoding] options:documentAttributes documentAttributes:nil error:nil];

  NSData *rtf = [atr RTFFromRange:NSMakeRange(0, [atr length])
                                  documentAttributes:nil];

  [pasteboard setData:rtf forType:NSRTFPboardType];
  [pasteboard setString:nsHtml forType:NSHTMLPboardType];
  
  if (fallback_text) {
    NSString *nsText = [NSString stringWithUTF8String:fallback_text];
    [pasteboard setString:nsText forType:NSPasteboardTypeString];
  }
  
  return 1;
}

int32_t clipboard_get_length() {
  __block int32_t result = -1;
  dispatch_sync(dispatch_get_main_queue(), ^{
      @autoreleasepool {
        NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
        NSString *string = [pasteboard stringForType:NSPasteboardTypeString];
        if (string != NULL) {
          NSUInteger length = [string lengthOfBytesUsingEncoding:NSUTF8StringEncoding];
            if (length != NSNotFound) {
              result = (int32_t)length;
            } else {
              result = -1;
            }
        } else {
          result = -1;
      }
    }
  });
  return result;
}