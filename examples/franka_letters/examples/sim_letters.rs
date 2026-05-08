//! Draw block-style "HELLO" with `move_waypoints`, then a cursive "S" with
//! `move_path`, on the PyBullet-simulated Franka. All Cartesian targets are
//! resolved to joints via `ArmInverseKinematics::ik_solve` (DLS).

use anyhow::Result;
use franka_letters::{block_text_waypoints, cursive_s_closure};
use franka_rust::FrankaEmika;
use robot_behavior::{behavior::*, roplat_data_dir};
use rsbullet::{Mode, RsBullet, RsBulletRobot};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    let mut physics = RsBullet::new(Mode::Gui)?;
    physics
        .add_search_path(roplat_data_dir().unwrap())?
        .set_gravity([0., 0., -10.])?
        .set_step_time(Duration::from_secs_f64(1. / 240.))?;

    let mut robot: RsBulletRobot<FrankaEmika> = physics
        .robot_builder::<FrankaEmika>("franka_writer")
        .base_fixed(true)
        .load()?;

    let seed = FrankaEmika::JOINT_DEFAULT;
    robot.move_joint(&seed)?;

    let waypoints = block_text_waypoints("HELLO", &seed);
    println!("[block] {} waypoints", waypoints.len());
    robot.move_waypoints(waypoints)?;

    println!("[cursive] tracing parametric S");
    robot.move_path(cursive_s_closure(seed))?;

    loop {
        physics.step()?;
        sleep(Duration::from_secs_f64(1. / 240.));
    }
}
