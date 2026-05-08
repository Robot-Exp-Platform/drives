//! Same demo as `sim_letters`, but on a real Franka Emika arm over FCI.
//!
//! ⚠️  Make sure the workspace around the writing plane is clear and the
//! user-stop is reachable. Targets live on a vertical plane at base x=0.45 m.

use anyhow::Result;
use franka_letters::{block_text_waypoints, cursive_s_closure, writing_seed};
use franka_rust::FrankaEmika;
use robot_behavior::behavior::*;

fn main() -> Result<()> {
    let mut robot = FrankaEmika::new("172.16.0.3");

    let seed = writing_seed();
    robot.move_joint(&seed)?;

    let waypoints = block_text_waypoints("HI", &seed);
    println!("[block] {} waypoints", waypoints.len());
    robot.move_waypoints(waypoints)?;

    println!("[cursive] tracing parametric S");
    robot.move_path(cursive_s_closure(seed))?;

    Ok(())
}
