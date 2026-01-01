cmake_minimum_required(VERSION 3.14)
project(LinearRegression CXX)

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
find_package(Eigen3 REQUIRED)

# Create executable
add_executable(linear_regression src/linear_regression.cpp)

# Link libraries
target_link_libraries(linear_regression PRIVATE
    ${OpenFHE_LIBRARIES}
    OpenMP::OpenMP_CXX
    pthread
)

# Create baseline executable
add_executable(baseline_linear_regression src/baseline_linear_regression.cpp)

# Link baseline libraries
target_link_libraries(baseline_linear_regression PRIVATE
    ${OpenFHE_LIBRARIES}
    OpenMP::OpenMP_CXX
    pthread
    Eigen3::Eigen
)

# Copy data directory to build folder to allow running from build dir
file(COPY "${CMAKE_CURRENT_SOURCE_DIR}/data" DESTINATION "${CMAKE_CURRENT_BINARY_DIR}")
