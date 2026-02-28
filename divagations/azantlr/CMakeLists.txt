cmake_minimum_required(VERSION 3.10)
project(azantlr CXX)

set(CMAKE_CXX_STANDARD 17)

# Find Java (needed for ANTLR tool execution)
find_package(Java COMPONENTS Runtime REQUIRED)


# Find ANTLR4 Runtime
# The system package provides antlr4-runtime-config.cmake
find_package(antlr4-runtime REQUIRED)
message(STATUS "ANTLR4_RUNTIME_INCLUDE_DIRS: ${ANTLR4_RUNTIME_INCLUDE_DIRS}")
message(STATUS "antlr4_runtime_INCLUDE_DIRS: ${antlr4_runtime_INCLUDE_DIRS}")
message(STATUS "antlr4_runtime_LIBRARIES: ${antlr4_runtime_LIBRARIES}")


# ANTLR4 Generator
# We assume 'antlr4' is in the PATH.
# If not, we might need to find the jar.
find_program(ANTLR4_EXECUTABLE antlr4)
if(NOT ANTLR4_EXECUTABLE)
    message(FATAL_ERROR "antlr4 executable not found. Please install the ANTLR4 tool.")
endif()

set(ANTLR4_GENERATED_SRC 
    ${CMAKE_CURRENT_BINARY_DIR}/AzadiLexer.cpp
    ${CMAKE_CURRENT_BINARY_DIR}/AzadiLexer.h
)

add_custom_command(
    OUTPUT ${ANTLR4_GENERATED_SRC}
    COMMAND ${ANTLR4_EXECUTABLE} -Dlanguage=Cpp -o ${CMAKE_CURRENT_BINARY_DIR} -visitor -no-listener ${CMAKE_CURRENT_SOURCE_DIR}/AzadiLexer.g4
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/AzadiLexer.g4
    COMMENT "Generating ANTLR4 C++ lexer"
)

add_executable(azantlr main.cpp ${ANTLR4_GENERATED_SRC})

# Include directories
# The antlr4-runtime package should export the include directories in the target,
# but we add the binary dir for generated files.
# We also explicitly add the system include path found by the user.
target_include_directories(azantlr PRIVATE ${CMAKE_CURRENT_BINARY_DIR} /usr/include/antlr4-runtime)

# Link against ANTLR4 runtime
# The target name exported by Arch's antlr4-runtime is typically 'antlr4_runtime' or 'antlr4-runtime::antlr4-runtime'
if(TARGET antlr4_runtime)
    target_link_libraries(azantlr antlr4_runtime)
elseif(TARGET antlr4-runtime::antlr4-runtime)
    target_link_libraries(azantlr antlr4-runtime::antlr4-runtime)
else()
    # Fallback: try to link using the variable if target not found (though REQUIRED should have found it)
    target_link_libraries(azantlr antlr4-runtime)
endif()
