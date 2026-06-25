use anyhow::Result;
use libjaka::JakaMini2;
use robot_behavior::behavior::*;
use rsbullet::{Mode, RsBullet, RsBulletRobot};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    let mut physics = RsBullet::new(Mode::Gui)?;
    physics
        .add_search_path("./asserts")?
        .set_gravity([0., 0., -10.])?
        .set_step_time(Duration::from_secs_f64(1. / 240.))?;

    let mut robot_1: RsBulletRobot<JakaMini2> = physics
        .robot_builder::<JakaMini2>("robot_1")
        .base_fixed(true)
        .load()?;

    robot_1.move_to::<JointSpace<6>>(JakaMini2::JOINT_DEFAULT)?;

    loop {
        physics.step()?;
        sleep(Duration::from_secs_f64(0.01));
    }
}
