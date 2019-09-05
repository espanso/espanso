cmake_minimum_required(VERSION 3.0)
project(libmacbridge)

set (CMAKE_CXX_STANDARD 14)
set(CMAKE_C_FLAGS "-x objective-c")

add_library(macbridge STATIC bridge.mm bridge.h AppDelegate.h AppDelegate.m)

install(TARGETS macbridge DESTINATION .)