//! Long joint-space Lissajous `move_path` demo on a real Franka Emika arm.
//!
//! Check the full joint-space workspace before running. The robot returns to
//! `JOINT_DEFAULT` at the end of the path. Override the FCI host with
//! `FRANKA_HOST`; default is `172.16.0.3`. Set
//! `FRANKA_LISSAJOUS_RANGE_RATIO=<0..0.48>` to scale joint amplitudes.

use anyhow::Result;
use franka_letters::{
    DEFAULT_LONG_JOINT_LISSAJOUS_RANGE_RATIO, long_joint_lissajous_closure_with_range_ratio,
};
use franka_rust::FrankaEmika;
use robot_behavior::behavior::*;
use std::env;

fn main() -> Result<()> {
    let host = env::var("FRANKA_HOST").unwrap_or_else(|_| "172.16.0.3".to_string());
    let mut robot = FrankaEmika::new(&host);

    let seed = FrankaEmika::JOINT_DEFAULT;
    robot.move_joint(&seed)?;

    let range_ratio = env::var("FRANKA_LISSAJOUS_RANGE_RATIO")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(DEFAULT_LONG_JOINT_LISSAJOUS_RANGE_RATIO);
    println!(
        "[joint-lissajous] planning long joint-space move_path on {host}, range_ratio={:.3}",
        range_ratio.clamp(0.0, 0.48)
    );
    robot.move_path(long_joint_lissajous_closure_with_range_ratio(
        seed,
        range_ratio,
    ))?;

    Ok(())
}
