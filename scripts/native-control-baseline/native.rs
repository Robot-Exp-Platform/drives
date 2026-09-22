//! Test-only compatibility surface for the current loopback measurement tool.
use super::*;
use crate::{FrankaEmika, FrankaRobot};
use robot_behavior::{ControlWith, JointPositionControl};

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap()
}
fn public_robot(robot: FrankaRobotImpl) -> FrankaEmika {
    FrankaRobot::from_test_impl(robot)
}

mod performance;
