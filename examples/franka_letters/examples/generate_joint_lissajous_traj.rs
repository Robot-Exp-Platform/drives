//! Offline generator for the long joint-space Lissajous trajectory.
//!
//! This runs the same default `move_path`/COPP planning pipeline as the online
//! demos, but captures the planned `move_traj` output and writes it to JSON.
//! The real-arm example can then execute the file without spending time on
//! planning at the robot.

#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use anyhow::{Result, anyhow};
use franka_letters::{
    DEFAULT_LONG_JOINT_LISSAJOUS_RANGE_RATIO, long_joint_lissajous_closure_with_range_ratio,
};
use franka_rust::FrankaEmika;
use robot_behavior::{MotionType, RobotResult, behavior::*};
use std::{env, fs::File, path::Path};

const DOF: usize = 7;
const DEFAULT_OUTPUT: &str = "examples/franka_letters/data/joint_lissajous_traj.json";

#[derive(Default)]
struct OfflineFrankaPlanner {
    traj: Option<Vec<MotionType<DOF>>>,
}

impl ArmParam<DOF> for OfflineFrankaPlanner {
    const JOINT_DEFAULT: [f64; DOF] = FrankaEmika::JOINT_DEFAULT;
    const JOINT_MIN: [f64; DOF] = FrankaEmika::JOINT_MIN;
    const JOINT_MAX: [f64; DOF] = FrankaEmika::JOINT_MAX;
    const JOINT_VEL_BOUND: [f64; DOF] = FrankaEmika::JOINT_VEL_BOUND;
    const JOINT_ACC_BOUND: [f64; DOF] = FrankaEmika::JOINT_ACC_BOUND;
    const JOINT_JERK_BOUND: [f64; DOF] = FrankaEmika::JOINT_JERK_BOUND;
    const CONTROL_PERIOD: f64 = FrankaEmika::CONTROL_PERIOD;
}

impl ArmPreplannedPath<DOF> for OfflineFrankaPlanner {
    fn move_traj(&mut self, path: Vec<MotionType<DOF>>) -> RobotResult<()> {
        self.traj = Some(path);
        Ok(())
    }

    fn move_traj_async(&mut self, path: Vec<MotionType<DOF>>) -> RobotResult<()> {
        self.move_traj(path)
    }
}

fn main() -> Result<()> {
    let range_ratio = env::var("FRANKA_LISSAJOUS_RANGE_RATIO")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(DEFAULT_LONG_JOINT_LISSAJOUS_RANGE_RATIO);
    let output = env::var("FRANKA_LISSAJOUS_TRAJ_FILE").unwrap_or_else(|_| DEFAULT_OUTPUT.into());

    let mut planner = OfflineFrankaPlanner::default();
    planner.move_path(long_joint_lissajous_closure_with_range_ratio(
        FrankaEmika::JOINT_DEFAULT,
        range_ratio,
    ))?;
    let traj = planner
        .traj
        .take()
        .ok_or_else(|| anyhow!("offline planner did not produce a trajectory"))?;

    let output_path = Path::new(&output);
    if let Some(parent) = output_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let file = File::create(output_path)?;
    serde_json::to_writer_pretty(file, &traj)?;

    println!(
        "[joint-lissajous] wrote {} samples to {}, range_ratio={:.3}",
        traj.len(),
        output_path.display(),
        range_ratio.clamp(0.0, 0.48)
    );

    Ok(())
}
