cmake_minimum_required(VERSION 3.16) # default of ubuntu:20.04
project(ocipkg_example_cpp_lib)
add_library(ocipkg_static_cpp STATIC lib.cpp)
install(TARGETS ocipkg_static_cpp ARCHIVE)
include(CPack)
