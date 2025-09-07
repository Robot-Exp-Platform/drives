#include <iostream>

// 生成的桥接头：文件名通常就是 <rust_file>.rs.h
// 例如你的模块在 src/to_cxx.rs，那么生成的常见路径/名字是 to_cxx.rs.h
// 该头里会包含 "rust/cxx.h"，因此还需要把 rust/cxx.h 的目录加入 include 路径
#include "to_cxx.rs.h"
#include "cxx.h"

int main() {
  try {
    // 1) 创建机器人对象（Rust 端的关联函数）
    rust::Box<ExRobot> robot = ExRobot::attach();

    // 2) 读取版本
    std::cout << "Robot version: " << ExRobot::version() << std::endl;

    // 3) 构造一个关节位姿目标（Joint 模式，6 维）
    CxxMotionType target;
    target.mode = CxxMotionTypeMode::Joint;

    // cxx 中 Vec<f64> ↔ rust::Vec<double>
    target.values = rust::Vec<double>({0.0, -1.57, 1.57, 0.0, 1.57, 0.0});

    // 4) 调用 move_to（返回 ::rust::Result<void>）
    robot->move_to(target);

    std::cout << "move_to succeeded.\n";
    return 0;
  } catch (const std::exception& e) {
    std::cerr << "Exception: " << e.what() << "\n";
    return 1;
  }
}
