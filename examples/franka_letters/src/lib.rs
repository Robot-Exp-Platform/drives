//! Cartesian-space letter trajectories for a 7-DOF Franka arm.
//!
//! Letters are laid out on a vertical "whiteboard" plane in front of the
//! robot (normal along base +X). Each 2D letter point is converted to a
//! 6-DOF end-effector pose, then resolved to joint angles via the
//! `ArmInverseKinematics` (DLS) pipeline. Solutions are warm-started from
//! the previous joint vector to keep waypoints smooth.

#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use franka_rust::FrankaEmika;
use nalgebra as na;
use robot_behavior::{
    CommonStop, IKMethod, MotionType, Pose,
    behavior::{ArmInverseKinematics, ArmParam},
};

/// Whiteboard plane geometry (metres).
pub const PLANE_X: f64 = 0.45;
pub const PLANE_Z0: f64 = 0.25;
pub const TEXT_Y0: f64 = -0.18;
/// Physical size of one normalised letter unit.
pub const LETTER_W: f64 = 0.05;
pub const LETTER_H: f64 = 0.07;
/// Horizontal stride between letters (`LETTER_W` + gap).
pub const LETTER_STRIDE: f64 = 0.07;

/// Pen-down end-effector orientation: ee z-axis points along base +X
/// (into the whiteboard). Achieved by rotating -π/2 about base Y.
fn pen_orientation() -> na::UnitQuaternion<f64> {
    na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), -std::f64::consts::FRAC_PI_2)
}

/// Build the EE pose for a normalised letter point at character offset `x_off`.
pub fn pen_pose(u: f64, v: f64, x_off: f64) -> Pose {
    let trans = na::Translation3::new(
        PLANE_X,
        TEXT_Y0 + x_off + u * LETTER_W,
        PLANE_Z0 + v * LETTER_H,
    );
    Pose::Quat(na::Isometry3::from_parts(trans, pen_orientation()))
}

/// Resolve a target pose to joint angles via DLS IK, warm-started from `seed`.
pub fn solve_ik(seed: &[f64; 7], target: &Pose) -> [f64; 7] {
    let q0 = na::SVector::<f64, 7>::from_column_slice(seed);
    let method = IKMethod::DLS {
        lambda: 0.05,
        stop: CommonStop { pos_tol: 5e-4, rot_tol: 5e-3, max_iters: 200, step_clip: 0.2 },
    };
    let q = FrankaEmika::ik_solve(&q0, target, method);
    let mut out = [0.0f64; 7];
    out.copy_from_slice(q.as_slice());
    out
}

/// Block-letter polylines in normalised `[0, 1]^2`.
fn block_letter(c: char) -> Option<&'static [[f64; 2]]> {
    match c {
        'H' => Some(&[
            [0.0, 0.0],
            [0.0, 1.0],
            [0.0, 0.5],
            [1.0, 0.5],
            [1.0, 1.0],
            [1.0, 0.0],
        ]),
        'E' => Some(&[
            [1.0, 0.0],
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [0.0, 0.5],
            [0.7, 0.5],
        ]),
        'L' => Some(&[[0.0, 1.0], [0.0, 0.0], [1.0, 0.0]]),
        'O' => Some(&[
            [0.5, 1.0],
            [1.0, 0.7],
            [1.0, 0.3],
            [0.5, 0.0],
            [0.0, 0.3],
            [0.0, 0.7],
            [0.5, 1.0],
        ]),
        'I' => Some(&[[0.5, 0.0], [0.5, 1.0]]),
        'T' => Some(&[[0.0, 1.0], [1.0, 1.0], [0.5, 1.0], [0.5, 0.0]]),
        ' ' => Some(&[]),
        _ => None,
    }
}

/// **正体**: discrete joint-space waypoints (after IK) for `move_waypoints`.
pub fn block_text_waypoints(text: &str, seed: &[f64; 7]) -> Vec<MotionType<7>> {
    let mut waypoints = Vec::new();
    let mut warm = *seed;
    for (i, c) in text.chars().enumerate() {
        let strokes = match block_letter(c) {
            Some(s) => s,
            None => continue,
        };
        let x_off = i as f64 * LETTER_STRIDE;
        for [u, v] in strokes {
            let target = pen_pose(*u, *v, x_off);
            warm = solve_ik(&warm, &target);
            waypoints.push(MotionType::Joint(warm));
        }
    }
    waypoints
}

/// **花体**: continuous parametric "S" closure for `move_path`, with an
/// internal warm-start cell so successive IK calls stay smooth.
pub fn cursive_s_closure(seed: [f64; 7]) -> impl Fn(f64) -> Option<MotionType<7>> {
    use std::cell::RefCell;
    let last = RefCell::new(seed);
    move |s| {
        if !(0.0..=1.0).contains(&s) {
            return None;
        }
        let u = s;
        let v = 0.5 + 0.4 * (std::f64::consts::TAU * s).sin();
        let target = pen_pose(u, v, 0.0);
        let warm = *last.borrow();
        let q = solve_ik(&warm, &target);
        *last.borrow_mut() = q;
        Some(MotionType::Joint(q))
    }
}

/// **花体**: a more flourished parametric loop (Lissajous-like).
pub fn cursive_loop_closure(seed: [f64; 7]) -> impl Fn(f64) -> Option<MotionType<7>> {
    use std::cell::RefCell;
    let last = RefCell::new(seed);
    move |s| {
        if !(0.0..=1.0).contains(&s) {
            return None;
        }
        let t = std::f64::consts::TAU * s;
        let u = 0.5 + 0.45 * t.sin();
        let v = 0.5 + 0.225 * (2.0 * t).sin();
        let target = pen_pose(u, v, 0.0);
        let warm = *last.borrow();
        let q = solve_ik(&warm, &target);
        *last.borrow_mut() = q;
        Some(MotionType::Joint(q))
    }
}

pub const DEFAULT_LONG_JOINT_LISSAJOUS_RANGE_RATIO: f64 = 0.40;

/// Long joint-space Lissajous path for `move_path` demos.
///
/// Each joint draws a different rhythm in the `(s, q_i)` rectangle. The smooth
/// envelope returns the arm to `center` at both ends, while per-joint amplitudes
/// are clipped against Franka joint limits with a small margin.
pub fn long_joint_lissajous_closure(center: [f64; 7]) -> impl Fn(f64) -> Option<MotionType<7>> {
    long_joint_lissajous_closure_with_range_ratio(center, DEFAULT_LONG_JOINT_LISSAJOUS_RANGE_RATIO)
}

pub fn long_joint_lissajous_closure_with_range_ratio(
    center: [f64; 7],
    range_ratio: f64,
) -> impl Fn(f64) -> Option<MotionType<7>> {
    move |s| {
        if !(0.0..=1.0).contains(&s) {
            return None;
        }
        Some(MotionType::Joint(long_joint_lissajous_at_with_range_ratio(
            center,
            s,
            range_ratio,
        )))
    }
}

pub fn long_joint_lissajous_at(center: [f64; 7], s: f64) -> [f64; 7] {
    long_joint_lissajous_at_with_range_ratio(center, s, DEFAULT_LONG_JOINT_LISSAJOUS_RANGE_RATIO)
}

pub fn long_joint_lissajous_at_with_range_ratio(
    center: [f64; 7],
    s: f64,
    range_ratio: f64,
) -> [f64; 7] {
    const MAIN_FREQ: [f64; 7] = [19.0, 25.0, 29.0, 17.0, 31.0, 37.0, 41.0];
    const SIDE_FREQ: [f64; 7] = [7.0, 11.0, 13.0, 19.0, 23.0, 29.0, 31.0];
    const PHASE: [f64; 7] = [0.0, 0.7, 1.4, 2.1, 2.8, 3.5, 4.2];
    const LIMIT_MARGIN: f64 = 0.06;

    let tau = std::f64::consts::TAU;
    let envelope = (std::f64::consts::PI * s).sin().powi(2);
    let range_ratio = range_ratio.clamp(0.0, 0.48);
    let mut q = center;
    for i in 0..7 {
        let joint_range = FrankaEmika::JOINT_MAX[i] - FrankaEmika::JOINT_MIN[i];
        let range_limited_amp = range_ratio * joint_range;
        let lower_room = (center[i] - FrankaEmika::JOINT_MIN[i] - LIMIT_MARGIN).max(0.0);
        let upper_room = (FrankaEmika::JOINT_MAX[i] - center[i] - LIMIT_MARGIN).max(0.0);
        let amp = range_limited_amp.min(lower_room.min(upper_room));
        let profile = envelope
            * (0.68 * (tau * MAIN_FREQ[i] * s + PHASE[i]).sin()
                + 0.22 * (tau * SIDE_FREQ[i] * s + 0.5 * PHASE[i]).sin()
                + 0.10 * (tau * (MAIN_FREQ[i] - SIDE_FREQ[i]).abs() * 0.5 * s).sin());
        q[i] += amp * profile;
    }
    q
}
