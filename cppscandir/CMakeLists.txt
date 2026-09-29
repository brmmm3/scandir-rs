cmake_minimum_required(VERSION 3.16)
project(cppscandir LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Build type
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Find the C library
find_library(CSCANDIR_LIB cscandir
    PATHS "${CMAKE_SOURCE_DIR}/../target/release"
    NO_DEFAULT_PATH
)

if(NOT CSCANDIR_LIB)
    message(WARNING "cscandir library not found at ${CMAKE_SOURCE_DIR}/../target/release")
    find_library(CSCANDIR_LIB cscandir)
endif()

find_path(CSCANDIR_INCLUDE cscandir.h
    PATHS "${CMAKE_SOURCE_DIR}/../cscandir/include"
    NO_DEFAULT_PATH
)

if(NOT CSCANDIR_INCLUDE)
    message(FATAL_ERROR "cscandir.h not found")
endif()

include_directories(${CSCANDIR_INCLUDE} ${CMAKE_SOURCE_DIR}/include)

# Main library target
add_library(cppscandir src/cppscandir.cc)
target_include_directories(cppscandir PUBLIC ${CMAKE_SOURCE_DIR}/include)
target_link_libraries(cppscandir ${CSCANDIR_LIB})

# Examples
add_executable(scandir_collect examples/scandir_collect.cc)
target_link_libraries(scandir_collect cppscandir)

add_executable(scandir_walk examples/scandir_walk.cc)
target_link_libraries(scandir_walk cppscandir)

add_executable(scandir_count examples/scandir_count.cc)
target_link_libraries(scandir_count cppscandir)

add_executable(scandir_filter examples/scandir_filter.cc)
target_link_libraries(scandir_filter cppscandir)

# Tests
if(CMAKE_PROJECT_NAME STREQUAL "cppscandir")
    enable_testing()
    find_package(GTest REQUIRED)
    add_executable(cppscandir_test tests/cppscandir_test.cc)
    target_link_libraries(cppscandir_test cppscandir GTest::gtest GTest::gtest_main)
    add_test(NAME cppscandir_test COMMAND cppscandir_test)
endif()