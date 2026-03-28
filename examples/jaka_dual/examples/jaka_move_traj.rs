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

    let mut robot: RsBulletRobot<JakaMini2> = physics.robot_builder("robot").load()?;

    robot.move_traj_from_file("E:\\yixing\\code\\Robot-Exp\\drives\\examples\\jaka_dual\\data\\step08_optimized_trajectory_rust.json")?;

    loop {
        physics.step()?;
        sleep(Duration::from_secs_f64(1. / 240.));
    }
}
