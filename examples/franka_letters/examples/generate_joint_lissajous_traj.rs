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
use robot_behavior::{ArmState, Coord, LoadState, MotionType, RobotResult, behavior::*};
use std::{env, fs::File, path::Path};

const DOF: usize = 7;
const DEFAULT_OUTPUT: &str = "examples/franka_letters/data/joint_lissajous_traj.json";

struct OfflineFrankaPlanner {
    traj: Option<Vec<MotionType<DOF>>>,
    scale: f64,
}

impl Default for OfflineFrankaPlanner {
    fn default() -> Self {
        Self { traj: None, scale: 1.0 }
    }
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

impl Arm<DOF> for OfflineFrankaPlanner {
    fn state(&mut self) -> RobotResult<ArmState<DOF>> {
        Ok(ArmState::default())
    }

    fn set_load(&mut self, _load: LoadState) -> RobotResult<()> {
        Ok(())
    }

    fn set_coord(&mut self, _coord: Coord) -> RobotResult<()> {
        Ok(())
    }

    fn set_scale(&mut self, scale: f64) -> RobotResult<()> {
        self.scale = scale;
        Ok(())
    }

    fn get_scale(&self) -> f64 {
        self.scale
    }

    fn with_coord(&mut self, _coord: Coord) -> &mut Self {
        self
    }

    fn with_scale(&mut self, scale: f64) -> &mut Self {
        self.scale = scale;
        self
    }

    fn with_velocity(&mut self, _joint_vel: &[f64; DOF]) -> &mut Self {
        self
    }

    fn with_acceleration(&mut self, _joint_acc: &[f64; DOF]) -> &mut Self {
        self
    }

    fn with_jerk(&mut self, _joint_jerk: &[f64; DOF]) -> &mut Self {
        self
    }

    fn with_cartesian_velocity(&mut self, _cartesian_vel: f64) -> &mut Self {
        self
    }

    fn with_cartesian_acceleration(&mut self, _cartesian_acc: f64) -> &mut Self {
        self
    }

    fn with_cartesian_jerk(&mut self, _cartesian_jerk: f64) -> &mut Self {
        self
    }

    fn with_rotation_velocity(&mut self, _rotation_vel: f64) -> &mut Self {
        self
    }

    fn with_rotation_acceleration(&mut self, _rotation_acc: f64) -> &mut Self {
        self
    }

    fn with_rotation_jerk(&mut self, _rotation_jerk: f64) -> &mut Self {
        self
    }
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
    let scale = env::var("FRANKA_LISSAJOUS_PLAN_SCALE")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.5);
    let output = env::var("FRANKA_LISSAJOUS_TRAJ_FILE").unwrap_or_else(|_| DEFAULT_OUTPUT.into());

    let mut planner = OfflineFrankaPlanner::default();
    planner.set_scale(scale)?;
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
        "[joint-lissajous] wrote {} samples to {}, range_ratio={:.3}, plan_scale={:.3}",
        traj.len(),
        output_path.display(),
        range_ratio.clamp(0.0, 0.48),
        scale
    );

    Ok(())
}
