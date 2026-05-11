//! Execute a preplanned long joint-space Lissajous trajectory on a real Franka.
//!
//! Check the full joint-space workspace before running. The robot returns to
//! `JOINT_DEFAULT` at the end of the path. Override the FCI host with
//! `FRANKA_HOST`; default is `172.16.0.3`. Generate the trajectory first with
//! `generate_joint_lissajous_traj`, then point `FRANKA_LISSAJOUS_TRAJ_FILE` at
//! the JSON file if you do not use the default path.

use anyhow::Result;
use franka_rust::FrankaEmika;
use robot_behavior::behavior::*;
use std::env;

const DEFAULT_TRAJ_FILE: &str = "examples/franka_letters/data/joint_lissajous_traj.json";

fn main() -> Result<()> {
    let host = env::var("FRANKA_HOST").unwrap_or_else(|_| "172.16.0.3".to_string());
    let mut robot = FrankaEmika::new(&host);

    let seed = FrankaEmika::JOINT_DEFAULT;
    robot.move_joint(&seed)?;

    let traj_file =
        env::var("FRANKA_LISSAJOUS_TRAJ_FILE").unwrap_or_else(|_| DEFAULT_TRAJ_FILE.into());
    println!("[joint-lissajous] executing preplanned trajectory on {host}: {traj_file}");
    robot.move_traj_from_file(&traj_file)?;

    Ok(())
}
