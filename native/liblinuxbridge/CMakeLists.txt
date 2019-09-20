cmake_minimum_required(VERSION 3.0)
project(liblinuxbridge)

set (CMAKE_CXX_STANDARD 14)
set (CMAKE_SHARED_LINKER_FLAGS "-lX11 -lXtst")

add_library(linuxbridge STATIC bridge.cpp bridge.h)

install(TARGETS linuxbridge DESTINATION .)