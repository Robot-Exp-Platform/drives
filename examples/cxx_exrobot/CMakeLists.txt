cmake_minimum_required(VERSION 3.16)
project(exrobot_cpp_demo LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# === 手动指定 Rust 构建产物位置（按需修改这两个变量） ===
# RUST_TARGET_DIR 指向你的 Rust 工程的 target 目录
# RUST_CRATE_NAME 是 Cargo.toml 中 [lib] 的 name
set(RUST_TARGET_DIR "/home/yixing/ws_robot/drives/target")   # ← 改成绝对路径
set(RUST_CRATE_NAME "roplat_exrobot")                                      # ← 如有不同请改

# 头文件位置：
# 1) cxx 生成的 bridge 头：target/cxxbridge/<crate_name>/
# 2) cxx 支持头 "rust/cxx.h"：target/cxxbridge/rust/   （如缺失，改为你自带的路径）
set(CXXBRIDGE_INCLUDE_DIR "${RUST_TARGET_DIR}/cxxbridge")
set(CRATE_INCLUDE_DIR     "${CXXBRIDGE_INCLUDE_DIR}/${RUST_CRATE_NAME}/src")

# 库文件（静态库）
set(RUST_LIB "${RUST_TARGET_DIR}/release/lib${RUST_CRATE_NAME}.a")

# 校验文件是否存在（可选，但推荐）
if(NOT EXISTS "${RUST_LIB}")
  message(FATAL_ERROR "Rust static library not found at: ${RUST_LIB}. Did you run cargo build --release?")
endif()
if(NOT EXISTS "${CRATE_INCLUDE_DIR}")
  message(FATAL_ERROR "Generated headers not found at: ${CRATE_INCLUDE_DIR}. Did you run cargo build --release?")
endif()

# 声明一个 IMPORTED 的静态库目标，供 C++ 链接
add_library(${RUST_CRATE_NAME} STATIC IMPORTED GLOBAL)
set_target_properties(${RUST_CRATE_NAME} PROPERTIES
  IMPORTED_LOCATION "${RUST_LIB}"
)

# 可执行程序
add_executable(demo src/main.cpp)

# 包含 cxx 生成的头，以及 rust/cxx.h 所在目录
target_include_directories(demo PRIVATE
  "${CRATE_INCLUDE_DIR}"            # …/target/cxxbridge/<crate_name> 里有 to_cxx.rs.h
  "${CXXBRIDGE_INCLUDE_DIR}/rust"        # …/target/cxxbridge/rust/ 里有 rust/cxx.h
)

# 线程与系统库（Rust 静态库常用依赖）
find_package(Threads REQUIRED)
target_link_libraries(demo PRIVATE
  ${RUST_CRATE_NAME}
  Threads::Threads
  dl
  m
)

# 如果你是用 clang 并需要 libc++，可按需启用（通常不必）
# target_compile_options(demo PRIVATE -stdlib=libc++)
# target_link_options(demo PRIVATE -stdlib=libc++)
