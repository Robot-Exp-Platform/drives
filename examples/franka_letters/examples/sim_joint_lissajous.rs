//! Long joint-space Lissajous `move_path` demo on the PyBullet Franka.
//!
//! Set `FRANKA_LISSAJOUS_DIRECT=1` for headless validation,
//! `FRANKA_LISSAJOUS_STEPS=<n>` to exit after a finite number of sim steps,
//! and `FRANKA_LISSAJOUS_RANGE_RATIO=<0..0.48>` to scale joint amplitudes.

use anyhow::Result;
use franka_letters::{
    DEFAULT_LONG_JOINT_LISSAJOUS_RANGE_RATIO, long_joint_lissajous_closure_with_range_ratio,
};
use franka_rust::FrankaEmika;
use robot_behavior::{behavior::*, roplat_data_dir};
use rsbullet::{Mode, RsBullet, RsBulletRobot};
use std::{env, thread::sleep, time::Duration};

fn main() -> Result<()> {
    let mode = if env::var_os("FRANKA_LISSAJOUS_DIRECT").is_some() {
        Mode::Direct
    } else {
        Mode::Gui
    };
    let mut physics = RsBullet::new(mode)?;
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

    let range_ratio = env::var("FRANKA_LISSAJOUS_RANGE_RATIO")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(DEFAULT_LONG_JOINT_LISSAJOUS_RANGE_RATIO);
    println!(
        "[joint-lissajous] planning long joint-space move_path, range_ratio={:.3}",
        range_ratio.clamp(0.0, 0.48)
    );
    robot.move_path(long_joint_lissajous_closure_with_range_ratio(
        seed,
        range_ratio,
    ))?;

    match env::var("FRANKA_LISSAJOUS_STEPS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
    {
        Some(steps) => {
            for _ in 0..steps {
                physics.step()?;
                sleep(Duration::from_secs_f64(1. / 240.));
            }
        }
        None => loop {
            physics.step()?;
            sleep(Duration::from_secs_f64(1. / 240.));
        },
    }

    Ok(())
}
