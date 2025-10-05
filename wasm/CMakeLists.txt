cmake_minimum_required(VERSION 3.20)
project(AchronymeWASM VERSION 1.0.0 LANGUAGES CXX)

# C++ Standard
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Emscripten-specific settings
if(EMSCRIPTEN)
    message(STATUS "Building for Emscripten/WASM")

    # Optimization flags
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -O3")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -ffast-math")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -flto")

    # WASM-specific flags
    set(EMSCRIPTEN_FLAGS
        "-s WASM=1"
        "-s ALLOW_MEMORY_GROWTH=1"
        "-s INITIAL_MEMORY=33554432"  # 32MB initial
        "-s MAXIMUM_MEMORY=268435456" # 256MB max
        "-s MODULARIZE=1"
        "-s EXPORT_ES6=1"
        "-s EXPORT_NAME='createAchronymeModule'"
        "-s EXPORTED_RUNTIME_METHODS=['ccall','cwrap']"
        "-s NO_EXIT_RUNTIME=1"
        "-s NO_FILESYSTEM=1"
        "-s DISABLE_EXCEPTION_CATCHING=0"
        "--bind"
    )

    # Enable SIMD if supported
    option(ENABLE_SIMD "Enable WebAssembly SIMD" ON)
    if(ENABLE_SIMD)
        list(APPEND EMSCRIPTEN_FLAGS "-msimd128")
    endif()

    # Debug vs Release
    if(CMAKE_BUILD_TYPE STREQUAL "Debug")
        list(APPEND EMSCRIPTEN_FLAGS
            "-s ASSERTIONS=1"
            "-s SAFE_HEAP=1"
            "-g"
        )
    else()
        list(APPEND EMSCRIPTEN_FLAGS
            "-s ASSERTIONS=0"
            "--closure 1"
            "--closure-args=--language_out=ECMASCRIPT_2020"
        )
    endif()

    string(REPLACE ";" " " EMSCRIPTEN_FLAGS_STR "${EMSCRIPTEN_FLAGS}")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${EMSCRIPTEN_FLAGS_STR}")
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} ${EMSCRIPTEN_FLAGS_STR}")
endif()

# Include directories
include_directories(${CMAKE_SOURCE_DIR}/include)

# Core library (static)
add_library(achronyme_core STATIC
    src/core/types.cpp
    src/core/constants.cpp
    src/core/utils.cpp
)

# Parser library
add_library(achronyme_parser STATIC
    src/parser/lexer.cpp
    src/parser/parser.cpp
    src/parser/evaluator.cpp
    src/parser/functions.cpp
)
target_link_libraries(achronyme_parser achronyme_core)

# DSP library
add_library(achronyme_dsp STATIC
    src/dsp/fourier.cpp
    src/dsp/fft.cpp
    src/dsp/convolution.cpp
    src/dsp/filters.cpp
    src/dsp/windows.cpp
)
target_link_libraries(achronyme_dsp achronyme_core)

# Linear Algebra library
add_library(achronyme_linalg STATIC
    src/linalg/matrix.cpp
    src/linalg/vector.cpp
    src/linalg/decomposition.cpp
    src/linalg/eigenvalues.cpp
    src/linalg/solver.cpp
)
target_link_libraries(achronyme_linalg achronyme_core)

# Numerical methods library
add_library(achronyme_numerical STATIC
    src/numerical/integration.cpp
    src/numerical/differentiation.cpp
    src/numerical/rootfinding.cpp
    src/numerical/interpolation.cpp
    src/numerical/ode.cpp
)
target_link_libraries(achronyme_numerical achronyme_core)

# Optimization library
add_library(achronyme_optimization STATIC
    src/optimization/simplex.cpp
    src/optimization/gradient.cpp
)
target_link_libraries(achronyme_optimization achronyme_core achronyme_linalg)

# Main WASM module (all-in-one for now)
if(EMSCRIPTEN)
    add_executable(achronyme-core
        src/bindings/main.cpp
        src/bindings/parser_bindings.cpp
        src/bindings/dsp_bindings.cpp
        src/bindings/linalg_bindings.cpp
    )

    target_link_libraries(achronyme-core
        achronyme_parser
        achronyme_dsp
        achronyme_linalg
        achronyme_numerical
        achronyme_optimization
    )

    # Output to dist/
    set_target_properties(achronyme-core PROPERTIES
        RUNTIME_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/dist"
        SUFFIX ".js"
    )

    # Ensure public/wasm/ directory exists
    add_custom_command(TARGET achronyme-core POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E make_directory "${CMAKE_SOURCE_DIR}/../public/wasm"
        COMMENT "Creating public/wasm/ directory if it doesn't exist"
    )

    # Copy WASM and JS files to public/ (only if they exist)
    add_custom_command(TARGET achronyme-core POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "$<TARGET_FILE_DIR:achronyme-core>/achronyme-core.wasm"
            "${CMAKE_SOURCE_DIR}/../public/wasm/"
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "$<TARGET_FILE_DIR:achronyme-core>/achronyme-core.js"
            "${CMAKE_SOURCE_DIR}/../public/wasm/"
        COMMENT "Copying WASM and JS files to public/wasm/"
        BYPRODUCTS
            "${CMAKE_SOURCE_DIR}/../public/wasm/achronyme-core.wasm"
            "${CMAKE_SOURCE_DIR}/../public/wasm/achronyme-core.js"
    )
endif()

# Tests (only when not building for WASM)
if(NOT EMSCRIPTEN)
    option(BUILD_TESTS "Build tests" ON)
    if(BUILD_TESTS)
        enable_testing()

        # Download GoogleTest
        include(FetchContent)
        FetchContent_Declare(
            googletest
            GIT_REPOSITORY https://github.com/google/googletest.git
            GIT_TAG release-1.12.1
        )
        FetchContent_MakeAvailable(googletest)

        # Test executables
        add_executable(test_parser tests/test_parser.cpp)
        target_link_libraries(test_parser achronyme_parser gtest_main)

        add_executable(test_fourier tests/test_fourier.cpp)
        target_link_libraries(test_fourier achronyme_dsp gtest_main)

        add_executable(test_matrix tests/test_matrix.cpp)
        target_link_libraries(test_matrix achronyme_linalg gtest_main)

        # Register tests
        include(GoogleTest)
        gtest_discover_tests(test_parser)
        gtest_discover_tests(test_fourier)
        gtest_discover_tests(test_matrix)
    endif()

    # Benchmarks
    option(BUILD_BENCHMARKS "Build benchmarks" OFF)
    if(BUILD_BENCHMARKS)
        FetchContent_Declare(
            benchmark
            GIT_REPOSITORY https://github.com/google/benchmark.git
            GIT_TAG v1.8.3
        )
        FetchContent_MakeAvailable(benchmark)

        add_executable(benchmark_all tests/benchmark.cpp)
        target_link_libraries(benchmark_all
            achronyme_dsp
            achronyme_linalg
            benchmark::benchmark
        )
    endif()
endif()

# Install rules
if(EMSCRIPTEN)
    install(FILES
        ${CMAKE_SOURCE_DIR}/dist/achronyme-core.js
        ${CMAKE_SOURCE_DIR}/dist/achronyme-core.wasm
        DESTINATION ${CMAKE_SOURCE_DIR}/../public/wasm/
    )
endif()
