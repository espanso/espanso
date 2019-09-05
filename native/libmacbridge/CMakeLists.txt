cmake_minimum_required(VERSION 3.0)
project(libmacbridge)

set (CMAKE_CXX_STANDARD 11)
set(CMAKE_C_FLAGS "-x objective-c")

add_library(macbridge STATIC bridge.mm bridge.h AppDelegate.h AppDelegate.mm)

install(TARGETS macbridge DESTINATION .)