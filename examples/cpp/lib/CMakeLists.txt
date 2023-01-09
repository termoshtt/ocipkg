cmake_minimum_required(VERSION 3.16) # default of ubuntu:20.04
project(ocipkg_static_cpp)

# Build C++ source as a static library
add_library(ocipkg_static_cpp STATIC lib.cpp)

# Get git short hash to use in image tag
execute_process(
  COMMAND git rev-parse --short HEAD
  OUTPUT_VARIABLE IMAGE_TAG
  OUTPUT_STRIP_TRAILING_WHITESPACE
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)
if(NOT IMAGE_TAG)
  set(IMAGE_TAG "latest")
endif()

set(REGISTRY "ghcr.io/termoshtt/ocipkg/static/cpp"
  CACHE STRING
  "OCI registry where the container will be pushed"
)
set(IMAGE_NAME "${REGISTRY}:${IMAGE_TAG}")

# Call ocipkg-compose to create OCI archive
add_custom_target(ocipkg ALL
  COMMAND ocipkg compose
    -o ${CMAKE_PROJECT_NAME}.tar
    -t ${IMAGE_NAME}
    $<TARGET_FILE:ocipkg_static_cpp>
  DEPENDS ocipkg_static_cpp
  COMMENT "Creating OCI archive: ${IMAGE_NAME}"
)
