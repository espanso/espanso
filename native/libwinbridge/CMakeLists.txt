cmake_minimum_required(VERSION 3.0)
project(libwinbridge)

set (CMAKE_CXX_STANDARD 14)

add_library(winbridge STATIC bridge.cpp bridge.h)

install(TARGETS winbridge DESTINATION .)