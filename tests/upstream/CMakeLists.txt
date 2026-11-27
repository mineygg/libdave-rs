# Builds Discord's libdave tests, unmodified, from the pinned submodule and links
# them against the static libraries that crates/libdave-sys/build.rs produced.
# Run through `npm run test:upstream`, which supplies the two directories below.
cmake_minimum_required(VERSION 3.20)
project(libdave_upstream_tests C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

foreach(var DAVE_SYS_OUT_DIR OPENSSL_INSTALL_LIB_DIR)
  if(NOT DEFINED ${var})
    message(FATAL_ERROR "${var} is not set; use `npm run test:upstream`")
  endif()
endforeach()

get_filename_component(VENDOR "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/libdave-sys/vendor" ABSOLUTE)
set(DAVE "${VENDOR}/libdave/cpp")
set(MLSPP "${VENDOR}/mlspp")

find_package(GTest REQUIRED)
find_package(Threads REQUIRED)

# The libraries under test. Nothing from libdave or mlspp is recompiled here.
find_library(DAVE_CORE dave_core PATHS "${DAVE_SYS_OUT_DIR}" NO_DEFAULT_PATH REQUIRED)
find_library(DAVE_MLSPP dave_mlspp PATHS "${DAVE_SYS_OUT_DIR}" NO_DEFAULT_PATH REQUIRED)
find_library(CRYPTO crypto PATHS "${OPENSSL_INSTALL_LIB_DIR}" NO_DEFAULT_PATH REQUIRED)

add_library(shipped INTERFACE)
# Mirrors the include order and definitions in build.rs.
target_include_directories(shipped INTERFACE
  "${DAVE}/includes"
  "${DAVE_SYS_OUT_DIR}/include"
  "${MLSPP}/include"
  "${MLSPP}/lib/bytes/include"
  "${MLSPP}/lib/tls_syntax/include"
  "${MLSPP}/lib/hpke/include"
  "${VENDOR}/nlohmann_json/include"
  "${DAVE}/src"
  "${OPENSSL_INSTALL_LIB_DIR}/../include")
target_compile_definitions(shipped INTERFACE WITH_OPENSSL3 DAVE_STATIC)
target_link_libraries(shipped INTERFACE
  "${DAVE_CORE}" "${DAVE_MLSPP}" "${CRYPTO}" Threads::Threads ${CMAKE_DL_LIBS})

enable_testing()

# C++ tests (same source list as upstream's cpp/test/CMakeLists.txt).
file(GLOB CPP_SOURCES CONFIGURE_DEPENDS "${DAVE}/test/*.cpp")
list(FILTER CPP_SOURCES EXCLUDE REGEX ".*/windows_crt_reporting\\.cpp$")
add_executable(libdave_test ${CPP_SOURCES} "${DAVE}/afl-driver/src/main.cpp")
# As upstream: the tests' "mls/..." includes must find libdave's headers first.
target_include_directories(libdave_test PRIVATE
  "${DAVE}/src" "${DAVE}/afl-driver/include")
target_compile_definitions(libdave_test PRIVATE
  DAVE_FUZZ_CORPUS_DIR="${DAVE}/afl-driver/input")
target_link_libraries(libdave_test PRIVATE GTest::gmock GTest::gtest_main shipped)
add_test(NAME libdave_test COMMAND libdave_test)

# C API tests (same sources as upstream's cpp/test/capi/CMakeLists.txt).
add_library(external_sender STATIC
  "${DAVE}/test/external_sender.cpp"
  "${DAVE}/test/capi/external_sender_wrapper.cpp")
target_include_directories(external_sender PRIVATE "${DAVE}/src")
target_link_libraries(external_sender PRIVATE shipped)
file(GLOB C_SOURCES CONFIGURE_DEPENDS "${DAVE}/test/capi/*.c")
add_executable(capi_test ${C_SOURCES})
set_target_properties(capi_test PROPERTIES LINKER_LANGUAGE CXX)
target_link_libraries(capi_test PRIVATE external_sender shipped)
add_test(NAME capi_test COMMAND capi_test)
