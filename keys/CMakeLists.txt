cmake_minimum_required(VERSION 3.14)
project(OpenFHEExperiments CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# Find OpenFHE
find_package(OpenFHE REQUIRED)

# Handle potential OpenMP flag mismatch
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

# Project programs
add_executable(keygen src/keygen.cpp)
target_link_libraries(keygen PRIVATE ${OpenFHE_LIBRARIES} OpenMP::OpenMP_CXX pthread)

add_executable(hospital1 src/hospital1.cpp)
target_link_libraries(hospital1 PRIVATE ${OpenFHE_LIBRARIES} OpenMP::OpenMP_CXX pthread)

add_executable(hospital2 src/hospital2.cpp)
target_link_libraries(hospital2 PRIVATE ${OpenFHE_LIBRARIES} OpenMP::OpenMP_CXX pthread)

add_executable(server1 src/server1.cpp)
target_link_libraries(server1 PRIVATE ${OpenFHE_LIBRARIES} OpenMP::OpenMP_CXX pthread)

add_executable(server2 src/server2.cpp)
target_link_libraries(server2 PRIVATE ${OpenFHE_LIBRARIES} OpenMP::OpenMP_CXX pthread)

add_executable(decrypt src/decrypt.cpp)
target_link_libraries(decrypt PRIVATE ${OpenFHE_LIBRARIES} OpenMP::OpenMP_CXX pthread)

add_executable(simulation src/simulation.cpp)
target_link_libraries(simulation PRIVATE ${OpenFHE_LIBRARIES} OpenMP::OpenMP_CXX pthread)

add_executable(main src/main.cpp)
target_link_libraries(main PRIVATE ${OpenFHE_LIBRARIES} OpenMP::OpenMP_CXX pthread)