//! Same demo as `sim_letters`, but on a real Franka Emika arm over FCI.
//!
//! ⚠️  Make sure the workspace around the writing plane is clear and the
//! user-stop is reachable. Targets live on a vertical plane at base x=0.45 m.

use anyhow::Result;
use franka_letters::{block_text_waypoints, cursive_s_closure};
use franka_rust::{FrankaEmika, types::robot_types::SetCollisionBehaviorData};
use robot_behavior::behavior::*;

fn main() -> Result<()> {
    let mut robot = FrankaEmika::new("172.16.0.3");

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

    let waypoints = block_text_waypoints("HELLO", &seed);
    println!("[block] {} waypoints", waypoints.len());
    robot.move_waypoints(waypoints)?;

    println!("[cursive] tracing parametric S");
    robot.move_path(cursive_s_closure(seed))?;

    Ok(())
}
