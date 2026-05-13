cmake_minimum_required(VERSION 3.24)

project(torel_llvm_backend LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(LLVM REQUIRED CONFIG)

add_library(torel_llvm_backend STATIC
  src/backend_api.cpp
  src/control_flow_emitter.cpp
  src/error.cpp
  src/expression_emitter.cpp
  src/function_emitter.cpp
  src/llvm_verifier.cpp
  src/mir_decoder.cpp
  src/module_emitter.cpp
  src/runtime_calls.cpp
  src/type_mapper.cpp
)

target_include_directories(torel_llvm_backend
  PUBLIC
    include
    ${LLVM_INCLUDE_DIRS}
)

target_compile_definitions(torel_llvm_backend PUBLIC ${LLVM_DEFINITIONS})

llvm_map_components_to_libnames(LLVM_LIBS
  core
  support
  target
  native
)

target_link_libraries(torel_llvm_backend PRIVATE ${LLVM_LIBS})
