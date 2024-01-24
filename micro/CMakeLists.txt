cmake_minimum_required(VERSION 3.12)

# Pull in SDK (must be before project)
include(pico_sdk_import.cmake)

# include(pico_extras_import_optional.cmake)

project(pico_examples C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS ON)
# set(CXXFLAGS -std=gnu++17)

# Initialize the SDK
pico_sdk_init()

# TODO: CMakeLists in subdirectories
include(FetchContent)
FetchContent_Declare(
  googletest
  URL https://github.com/google/googletest/archive/03597a01ee50ed33e9dfd640b249b4be3799d395.zip
)
# FetchContent_MakeAvailable(googletest)

# enable_testing()

# add_executable(
#   hello_test
#   test/test.cpp
# )
# target_link_libraries(
#   hello_test
#   GTest::gtest_main
# )

# include(GoogleTest)
# gtest_discover_tests(hello_test)


set(APP_SRCS
  src/sensors/encoder.cpp
  src/sensors/imu.cpp
  src/sensors/odom.cpp
  src/control/motor.cpp
  src/control/steer.cpp
  src/comms.cpp
)


# Add blink example
# add_subdirectory(blink)
add_executable(blink src/main.cpp ${APP_SRCS})
target_link_libraries(blink pico_stdlib hardware_pwm hardware_adc)

pico_enable_stdio_usb(blink 1) # 1 for testing?
pico_enable_stdio_uart(blink 1)

pico_add_extra_outputs(blink)

add_compile_options(-Wall
        -Wno-format          # int != int32_t as far as the compiler is concerned because gcc has int32_t as long int
        -Wno-unused-function
        )
if (CMAKE_C_COMPILER_ID STREQUAL "GNU")
    add_compile_options(-Wno-maybe-uninitialized) # why?
endif()
