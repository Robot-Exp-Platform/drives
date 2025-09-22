use libjaka::JakaRobot;
use robot_behavior::behavior::*;
fn main() {
    let (mut robot_1, mut robot_2) = (JakaRobot::new(""), JakaRobot::new(""));
    robot_1.enable().unwrap();
    robot_2.enable().unwrap();
}
