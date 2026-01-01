cmake_minimum_required(VERSION 3.14)
project(BFVBenchmark CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# Find OpenFHE
find_package(OpenFHE REQUIRED)

# Handle potential OpenMP flag mismatch (e.g. if OpenFHE was built with Clang but we use GCC)
if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
    string(REPLACE "-fopenmp=libomp" "-fopenmp" OpenFHE_LIBRARIES "${OpenFHE_LIBRARIES}")
    foreach(target OPENFHEcore OPENFHEpke OPENFHEbinfhe)
        if(TARGET ${target})
            get_target_property(ill ${target} INTERFACE_LINK_LIBRARIES)
            if(ill)
                string(REPLACE "-fopenmp=libomp" "-fopenmp" ill_clean "${ill}")
                set_target_properties(${target} PROPERTIES INTERFACE_LINK_LIBRARIES "${ill_clean}")
            endif()
        endif()
    endforeach()
endif()

include_directories(SYSTEM 
    ${OpenFHE_INCLUDE}
    ${OpenFHE_INCLUDE}/core
    ${OpenFHE_INCLUDE}/pke
    ${OpenFHE_INCLUDE}/binfhe
)
link_directories(${OpenFHE_LIBDIR})

find_package(OpenMP REQUIRED)

# Fetch Google Benchmark
include(FetchContent)
set(BENCHMARK_ENABLE_TESTING OFF CACHE BOOL "Enable testing of the benchmark library." FORCE)
FetchContent_Declare(
  googlebenchmark
  GIT_REPOSITORY https://github.com/google/benchmark.git
  GIT_TAG v1.8.3
)
FetchContent_MakeAvailable(googlebenchmark)

# Create executable
add_executable(bfv_benchmark src/bfv_benchmark.cpp)

# Link libraries
target_link_libraries(bfv_benchmark PRIVATE
    ${OpenFHE_LIBRARIES}
    OpenMP::OpenMP_CXX
    benchmark::benchmark
    benchmark::benchmark_main
    pthread
)
