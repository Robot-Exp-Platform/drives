//! Execute a preplanned long joint-space Lissajous trajectory on a real Franka.
//!
//! Check the full joint-space workspace before running. The robot returns to
//! `JOINT_DEFAULT` at the end of the path. Override the FCI host with
//! `FRANKA_HOST`; default is `172.16.0.3`. Generate the trajectory first with
//! `generate_joint_lissajous_traj`, then point `FRANKA_LISSAJOUS_TRAJ_FILE` at
//! the JSON file if you do not use the default path.

use anyhow::Result;
use franka_letters::{
    DEFAULT_LONG_JOINT_LISSAJOUS_RANGE_RATIO, long_joint_lissajous_closure_with_range_ratio,
};
use franka_rust::{FrankaEmika, types::robot_types::SetCollisionBehaviorData};
use robot_behavior::{MotionType, behavior::*};
use serde_json::from_reader;
use std::{env, fs::File, io::BufReader};

const DEFAULT_TRAJ_FILE: &str = "data/joint_lissajous_traj.json";

fn main() -> Result<()> {
    let host = env::var("FRANKA_HOST").unwrap_or_else(|_| "172.16.0.3".to_string());
    let mut robot = FrankaEmika::new(&host);

    robot.set_default_behavior()?;

    robot.set_collision_behavior(SetCollisionBehaviorData {
        lower_torque_thresholds_acceleration: [20., 20., 18., 18., 16., 14., 12.],
        upper_torque_thresholds_acceleration: [20., 20., 18., 18., 16., 14., 12.],
        lower_torque_thresholds_nominal: [20., 20., 18., 18., 16., 14., 12.],
        upper_torque_thresholds_nominal: [20., 20., 18., 18., 16., 14., 12.],
        lower_force_thresholds_acceleration: [20., 20., 20., 25., 25., 25.],
        upper_force_thresholds_acceleration: [20., 20., 20., 25., 25., 25.],
        lower_force_thresholds_nominal: [20., 20., 20., 25., 25., 25.],
        upper_force_thresholds_nominal: [20., 20., 20., 25., 25., 25.],
    })?;

    let seed = FrankaEmika::JOINT_DEFAULT;
    robot.move_joint(&seed)?;

    // let range_ratio = env::var("FRANKA_LISSAJOUS_RANGE_RATIO")
    //     .ok()
    //     .and_then(|s| s.parse::<f64>().ok())
    //     .unwrap_or(DEFAULT_LONG_JOINT_LISSAJOUS_RANGE_RATIO);
    // println!(
    //     "[joint-lissajous] planning long joint-space move_path, range_ratio={:.3}",
    //     range_ratio.clamp(0.0, 0.48)
    // );
    // robot.move_path(long_joint_lissajous_closure_with_range_ratio(
    //     seed,
    //     range_ratio,
    // ))?;

    let traj_file =
        env::var("FRANKA_LISSAJOUS_TRAJ_FILE").unwrap_or_else(|_| DEFAULT_TRAJ_FILE.into());
    println!("[joint-lissajous] executing preplanned trajectory on {host}: {traj_file}");

    let file = File::open(traj_file)?;
    let reader = BufReader::new(file);
    let path: Vec<MotionType<7>> = from_reader(reader).unwrap();

    robot.move_to(path[0])?;

    println!("[joint-lissajous] make sure the workspace is clear and the user-stop is reachable!");
    robot.move_traj(path)?;

    Ok(())
}
