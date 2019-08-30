cmake_minimum_required(VERSION 3.0)
project(libwinbridge)

add_library(winbridge STATIC bridge.cpp bridge.h)

install(TARGETS winbridge DESTINATION .)