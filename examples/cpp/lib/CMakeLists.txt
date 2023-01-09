cmake_minimum_required(VERSION 3.16) # default of ubuntu:20.04
project(ocipkg_example_cpp_lib)

# Build C++ source as a static library
add_library(ocipkg_static_cpp STATIC lib.cpp)

# Get git short hash to use in image tag
execute_process(
  COMMAND git rev-parse --short HEAD
  OUTPUT_VARIABLE IMAGE_TAG
  OUTPUT_STRIP_TRAILING_WHITESPACE
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)
if(IMAGE_TAG)
  message(STATUS "Git commit = ${IMAGE_TAG}")
else()
  set(IMAGE_TAG "latest")
endif()

# Call ocipkg-compose to create OCI archive
add_custom_target(ocipkg ALL
  COMMAND ocipkg compose
    -o ${CMAKE_PROJECT_NAME}.tar
    -t ${CMAKE_PROJECT_NAME}:${IMAGE_TAG}
    $<TARGET_FILE:ocipkg_static_cpp>
  DEPENDS ocipkg_static_cpp
)
