//! Manual TOPP3-SOCP pipeline (no `move_path` / `move_waypoints` defaults,
//! no inverse kinematics).
//!
//! We sidestep IK entirely: the geometric path is a small sinusoidal wiggle
//! in **joint space** around `JOINT_DEFAULT`. That makes the path trivially
//! kinematically feasible, so any oddness left in the executed motion is
//! attributable to the planner / driver, not to bad IK seeds.
//!
//! Pipeline:
//!   1. Sample sinusoidal joint waypoints around `JOINT_DEFAULT`.
//!   2. Hermite-spline through them via copp.
//!   3. TOPP2-RA → TOPP3-SOCP (2 SCP iters).
//!   4. Re-sample `q(t)` at `dt = CONTROL_PERIOD`.
//!   5. Push through `ArmPreplannedPath::move_traj`.

#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use anyhow::{Result, anyhow};
use copp::InterpolationMode;
use copp::path::{Path, SplineConfig};
use copp::robot::Robot;
use copp::solver::topp2_ra::{ReachSet2OptionsBuilder, Topp2ProblemBuilder, topp2_ra};
use copp::solver::topp3_socp::{
    ClarabelOptionsBuilder, Topp3ProblemBuilder, s_to_t_topp3, t_to_s_topp3, topp3_socp,
};
use franka_rust::FrankaEmika;
use nalgebra as na;
use robot_behavior::{MotionType, behavior::*, roplat_data_dir};
use rsbullet::{Mode, RsBullet, RsBulletRobot};
use std::{thread::sleep, time::Duration};

const DOF: usize = 7;
/// Number of `s` samples used to discretise the geometric path.
const N_SAMPLES: usize = 1001;

/// Build a list of sinusoidal joint waypoints around `JOINT_DEFAULT`.
///
/// Each joint i is perturbed by `amp_i * sin(2π * k * f_i / (n - 1))` where
/// `f_i` are coprime small integers so the path is a multi-frequency
/// Lissajous in joint space — visibly "wiggling" but bounded.
fn sinusoidal_waypoints(n: usize) -> Vec<[f64; DOF]> {
    let amps = [0.30, 0.20, 0.30, 0.20, 0.30, 0.20, 0.40];
    let freqs = [1.0, 2.0, 3.0, 1.0, 2.0, 3.0, 4.0];
    let q0 = FrankaEmika::JOINT_DEFAULT;
    let mut waypoints = Vec::with_capacity(n);
    for k in 0..n {
        let s = k as f64 / (n - 1) as f64;
        let mut q = q0;
        for i in 0..DOF {
            q[i] += amps[i] * (std::f64::consts::TAU * freqs[i] * s).sin();
        }
        waypoints.push(q);
    }
    waypoints
}

/// Convert a list of 7-DOF joint waypoints into a uniform-time joint
/// trajectory via the manual copp pipeline.
fn plan_traj(waypoints: &[[f64; DOF]]) -> Result<Vec<MotionType<DOF>>> {
    if waypoints.len() < 2 {
        return Err(anyhow!("need at least 2 waypoints"));
    }

    // 1) Build a Hermite spline through the waypoints.
    let n_pts = waypoints.len();
    let wp_mat = na::DMatrix::<f64>::from_fn(DOF, n_pts, |i, j| waypoints[j][i]);
    let path = Path::from_waypoints(&wp_mat, SplineConfig::default())
        .map_err(|e| anyhow!("Path::from_waypoints: {e}"))?;

    // 2) Sample derivatives on a uniform s grid.
    let s: Vec<f64> = (0..N_SAMPLES)
        .map(|j| j as f64 / (N_SAMPLES - 1) as f64)
        .collect();
    let derivs = path
        .evaluate_up_to_3rd(&s)
        .map_err(|e| anyhow!("evaluate_up_to_3rd: {e}"))?;

    let mut robot = Robot::with_capacity(DOF, N_SAMPLES);
    robot
        .with_s(s.as_slice())
        .map_err(|e| anyhow!("with_s: {e}"))?;
    robot
        .with_q(
            &derivs.q.as_view(),
            &derivs.dq.as_ref().unwrap().as_view(),
            &derivs.ddq.as_ref().unwrap().as_view(),
            derivs.dddq.as_ref().map(|m| m.as_view()).as_ref(),
            0,
        )
        .map_err(|e| anyhow!("with_q: {e}"))?;
    robot
        .with_axial_velocity(
            (FrankaEmika::JOINT_VEL_BOUND.as_slice(), N_SAMPLES),
            (
                FrankaEmika::JOINT_VEL_BOUND.map(|x| -x).as_slice(),
                N_SAMPLES,
            ),
            0,
        )
        .map_err(|e| anyhow!("with_axial_velocity: {e}"))?;
    robot
        .with_axial_acceleration(
            (FrankaEmika::JOINT_ACC_BOUND.as_slice(), N_SAMPLES),
            (
                FrankaEmika::JOINT_ACC_BOUND.map(|x| -x).as_slice(),
                N_SAMPLES,
            ),
            0,
        )
        .map_err(|e| anyhow!("with_axial_acceleration: {e}"))?;
    robot
        .with_axial_jerk(
            (FrankaEmika::JOINT_JERK_BOUND.as_slice(), N_SAMPLES),
            (
                FrankaEmika::JOINT_JERK_BOUND.map(|x| -x).as_slice(),
                N_SAMPLES,
            ),
            0,
        )
        .map_err(|e| anyhow!("with_axial_jerk: {e}"))?;

    // 4) TOPP2-RA seed.
    let idx_s_interval = (0, N_SAMPLES - 1);
    let a_boundary = (0.0, 0.0);
    let a_ra0 = {
        let prob = Topp2ProblemBuilder::new(&robot, idx_s_interval, a_boundary)
            .build()
            .map_err(|e| anyhow!("Topp2ProblemBuilder: {e}"))?;
        let opts = ReachSet2OptionsBuilder::new()
            .build()
            .map_err(|e| anyhow!("ReachSet2OptionsBuilder: {e}"))?;
        topp2_ra(&prob, &opts).map_err(|e| anyhow!("topp2_ra: {e}"))?
    };
    robot
        .constraints
        .amax_substitute(&a_ra0, 0)
        .map_err(|e| anyhow!("amax_substitute: {e}"))?;

    // 5) TOPP3-SOCP, two SCP iterations.
    let opts_socp = ClarabelOptionsBuilder::new()
        .allow_almost_solved(true)
        .build()
        .map_err(|e| anyhow!("ClarabelOptionsBuilder: {e}"))?;

    let (a_qp1, _b_qp1, _ns1) = {
        let prob =
            Topp3ProblemBuilder::new(&mut robot, idx_s_interval.0, &a_ra0, (0.0, 0.0), (0.0, 0.0))
                .build_with_linearization()
                .map_err(|e| anyhow!("Topp3 iter1: {e}"))?;
        topp3_socp(&prob, &opts_socp).map_err(|e| anyhow!("topp3_socp iter1: {e}"))?
    };
    let (a_qp2, b_qp2, ns2) = {
        let prob =
            Topp3ProblemBuilder::new(&mut robot, idx_s_interval.0, &a_qp1, (0.0, 0.0), (0.0, 0.0))
                .build_with_linearization()
                .map_err(|e| anyhow!("Topp3 iter2: {e}"))?;
        topp3_socp(&prob, &opts_socp).map_err(|e| anyhow!("topp3_socp iter2: {e}"))?
    };

    // 6) Re-sample s(t) on uniform CONTROL_PERIOD grid.
    let dt = FrankaEmika::CONTROL_PERIOD;
    let (t_final, t_s) = s_to_t_topp3(&s, &a_qp2, &b_qp2, ns2, 0.0);
    println!("[copp] traversal time = {:.3} s, dt = {:.4} s", t_final, dt);
    let s_t = t_to_s_topp3(
        &s,
        &a_qp2,
        &b_qp2,
        ns2,
        &t_s,
        InterpolationMode::UniformTimeGrid(0.0, dt, true),
    );
    if s_t.is_empty() {
        return Err(anyhow!("t_to_s_topp3 produced empty grid"));
    }

    // 7) Evaluate q(t) from the same spline.
    let q_t = path
        .evaluate_q(&s_t)
        .map_err(|e| anyhow!("evaluate_q: {e}"))?;

    let mut traj = Vec::with_capacity(q_t.q.ncols());
    for j in 0..q_t.q.ncols() {
        let mut joint = [0.0f64; DOF];
        for i in 0..DOF {
            joint[i] = q_t.q[(i, j)];
        }
        traj.push(MotionType::Joint(joint));
    }
    Ok(traj)
}

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

    // Sinusoidal joint-space waypoints — no IK involved.
    let waypoints = sinusoidal_waypoints(64);
    println!("[manual] {} input waypoints", waypoints.len());

    let start = std::time::Instant::now();

    let traj = plan_traj(&waypoints)?;

    let elapsed = start.elapsed();
    println!("[manual] planning took {:.3} s", elapsed.as_secs_f64());

    println!("[manual] {} samples in time-uniform trajectory", traj.len());

    robot.move_traj(traj)?;

    loop {
        physics.step()?;
        sleep(Duration::from_secs_f64(1. / 240.));
    }
}
