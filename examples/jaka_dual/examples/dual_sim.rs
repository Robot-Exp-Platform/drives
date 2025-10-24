use anyhow::Result;
use libjaka::JakaMini2;
use robot_behavior::behavior::*;
use rsbullet::{Mode, RsBullet};
use std::{f64::consts::FRAC_PI_4, thread::sleep, time::Duration};

fn main() -> Result<()> {
    let mut physics = RsBullet::new(Mode::Gui)?;
    physics
        .set_additional_search_path("E:\\yixing\\code\\Robot-Exp\\drives\\asserts")?
        .set_gravity([0., 0., -10.])?
        .set_step_time(Duration::from_secs_f64(1. / 240.))?;

    let mut robot_1 = physics
        .robot_builder::<JakaMini2>("robot_1")
        .base([0.0, 0.2, 0.0])
        .base_fixed(true)
        .load()?;
    let mut robot_2 = physics
        .robot_builder::<JakaMini2>("robot_2")
        .base([0.0, -0.2, 0.0])
        .base_fixed(true)
        .load()?;

    for _ in 0..100_000 {
        physics.step()?;
    }
    robot_1
        .with_velocity(&[5.; 6])
        .with_acceleration(&[2.; 6])
        .move_joint(&[0.; 6])?;
    robot_2
        .with_velocity(&[5.; 6])
        .move_joint(&[FRAC_PI_4; 6])?;

    loop {
        physics.step()?;
        sleep(Duration::from_secs_f64(0.01));
    }
}
