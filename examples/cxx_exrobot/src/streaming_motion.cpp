#include <iostream>
#include "to_cxx.rs.h"
#include "cxx.h"

int main()
{
  try
  {

    rust::Box<ExRobot> robot = ExRobot::attach();
    // 使用其他机器人时（以franka 为例）
    // rust::Box<FrankaRobot> robot = FrankaRobot::attach("192.16.0.3“）;

    auto stream = robot->start_streaming();

    for (int i = 0; i < 100; i++)
    {
      stream->move_to(CxxMotionType{
          .mode = CxxMotionTypeMode::Joint,
          .values = rust::Vec<double>({0.0, -1.57, 1.57, 0.0, 1.57, 0.0}),
      });
    }

    return 0;
  }
  catch (const std::exception &e)
  {
    std::cerr << "Exception: " << e.what() << "\n";
    return 1;
  }
}
