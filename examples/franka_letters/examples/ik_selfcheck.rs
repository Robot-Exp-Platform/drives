//! IK self-check.
//!
//! Sample random joint vectors, FK them to a target pose, run IK (DLS) from a
//! perturbed seed, then FK the IK result and report the residual.
//! A healthy IK should give pos_err < 1e-3 m and rot_err < 1e-2 rad on >95%
//! of trials inside the workspace.

#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use franka_rust::FrankaEmika;
use nalgebra as na;
use robot_behavior::{
    CommonStop, IKMethod, Pose,
    behavior::{ArmForwardKinematics, ArmInverseKinematics, ArmParam},
};

const DOF: usize = 7;
const N_TRIALS: usize = 200;

fn rand_in(min: f64, max: f64, state: &mut u64) -> f64 {
    // tiny LCG, deterministic
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let u = ((*state >> 33) as u32) as f64 / u32::MAX as f64;
    min + (max - min) * u
}

fn main() {
    let mut rng = 0x1234_5678_9abc_def0u64;
    let mut pos_errs = Vec::with_capacity(N_TRIALS);
    let mut rot_errs = Vec::with_capacity(N_TRIALS);
    let mut bad = 0usize;

    for trial in 0..N_TRIALS {
        // 1) sample a feasible q_truth
        let mut q_truth = [0.0f64; DOF];
        for i in 0..DOF {
            let lo = FrankaEmika::JOINT_MIN[i] + 0.1;
            let hi = FrankaEmika::JOINT_MAX[i] - 0.1;
            q_truth[i] = rand_in(lo, hi, &mut rng);
        }

        // 2) FK -> target pose
        let target = FrankaEmika::fk_end_pose(&q_truth);

        // 3) seed = q_truth + small perturbation, IK
        let mut seed_arr = q_truth;
        for v in seed_arr.iter_mut() {
            *v += rand_in(-0.3, 0.3, &mut rng);
        }
        let seed = na::SVector::<f64, DOF>::from_column_slice(&seed_arr);
        let method = IKMethod::DLS {
            lambda: 0.05,
            stop: CommonStop { pos_tol: 1e-5, rot_tol: 1e-5, max_iters: 500, step_clip: 0.2 },
        };
        let q_ik = FrankaEmika::ik_solve(&seed, &target, method);

        // 4) FK back, measure residual
        let mut q_ik_arr = [0.0f64; DOF];
        q_ik_arr.copy_from_slice(q_ik.as_slice());
        let pose_back = FrankaEmika::fk_end_pose(&q_ik_arr);

        let tgt_iso = target.quat();
        let back_iso = pose_back.quat();
        let dp = tgt_iso.translation.vector - back_iso.translation.vector;
        let drot = (tgt_iso.rotation * back_iso.rotation.inverse()).scaled_axis();
        let pos_err = dp.norm();
        let rot_err = drot.norm();

        pos_errs.push(pos_err);
        rot_errs.push(rot_err);
        if pos_err > 1e-3 || rot_err > 1e-2 {
            bad += 1;
            if bad <= 5 {
                println!(
                    "[trial {trial:>3}] pos_err = {pos_err:.4e} m, rot_err = {rot_err:.4e} rad  (BAD)"
                );
            }
        }
    }

    pos_errs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    rot_errs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = |v: &[f64]| v[v.len() / 2];
    let p95 = |v: &[f64]| v[(v.len() as f64 * 0.95) as usize];
    let pmax = |v: &[f64]| *v.last().unwrap();

    println!();
    println!("=== IK self-check: {N_TRIALS} trials ===");
    println!(
        "pos_err  median = {:.3e}  p95 = {:.3e}  max = {:.3e}",
        median(&pos_errs),
        p95(&pos_errs),
        pmax(&pos_errs)
    );
    println!(
        "rot_err  median = {:.3e}  p95 = {:.3e}  max = {:.3e}",
        median(&rot_errs),
        p95(&rot_errs),
        pmax(&rot_errs)
    );
    println!(
        "fail rate (pos>1e-3 or rot>1e-2): {} / {}  ({:.1}%)",
        bad,
        N_TRIALS,
        100.0 * bad as f64 / N_TRIALS as f64
    );
}
