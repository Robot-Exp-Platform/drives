cmake_minimum_required(VERSION 3.16)
project(exrobot_cpp_demo LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# === 手动指定 Rust 构建产物位置（按需修改这两个变量） ===
# RUST_TARGET_DIR 指向你的 Rust 工程的 target 目录
# RUST_CRATE_NAME 是 Cargo.toml 中 [lib] 的 name
set(RUST_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target") 
set(RUST_CRATE_NAME "roplat_exrobot") # ← 如有不同请改

# 声明一个 IMPORTED 的静态库目标，供 C++ 链接
add_library(${RUST_CRATE_NAME} STATIC IMPORTED GLOBAL)
set_target_properties(${RUST_CRATE_NAME} PROPERTIES
  IMPORTED_LOCATION "${RUST_TARGET_DIR}/release/lib${RUST_CRATE_NAME}.a"
)

# 可执行程序
add_executable(main src/main.cpp)
add_executable(streaming_motion src/streaming_motion.cpp)

# 包含 cxx 生成的头，以及 rust/cxx.h 所在目录
target_include_directories(main PRIVATE
  "${RUST_TARGET_DIR}/cxxbridge/rust" # …/target/cxxbridge/rust/ 里有 rust/cxx.h
  "${RUST_TARGET_DIR}/cxxbridge/${RUST_CRATE_NAME}/src"
)

target_include_directories(streaming_motion PRIVATE
  "${RUST_TARGET_DIR}/cxxbridge/rust" # …/target/cxxbridge/rust/ 里有 rust/cxx.h
  "${RUST_TARGET_DIR}/cxxbridge/${RUST_CRATE_NAME}/src" 
)

# 线程与系统库（Rust 静态库常用依赖）
find_package(Threads REQUIRED)
target_link_libraries(main PRIVATE
  ${RUST_CRATE_NAME}
  # Threads::Threads
  # dl
  # m
)

target_link_libraries(streaming_motion PRIVATE
  ${RUST_CRATE_NAME}
  # Threads::Threads
  # dl
  # m
)

# 如果你是用 clang 并需要 libc++，可按需启用（通常不必）
# target_compile_options(demo PRIVATE -stdlib=libc++)
# target_link_options(demo PRIVATE -stdlib=libc++)
