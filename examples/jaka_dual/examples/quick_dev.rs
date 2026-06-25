use anyhow::Result;
use libjaka::JakaMini2;
use robot_behavior::behavior::*;
use roplat_rerun::RerunHost;
use rsbullet::{Mode, RsBullet};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    let mut physics = RsBullet::new(Mode::Gui)?;
    let mut renderer = RerunHost::new("quick_dev")?;
    physics
        .add_search_path("E:\\yixing\\code\\Robot-Exp\\drives\\asserts")?
        .set_gravity([0., 0., -10.])?
        .set_step_time(Duration::from_secs_f64(1. / 240.))?;

    let mut robot = physics.robot_builder::<JakaMini2>("robot_1").load()?;
    renderer
        .robot_builder::<JakaMini2>("robot_1")
        .load()?
        .attach_from(&mut robot)?;

    // Example: move to a target in an explicit motion space.
    // ```rust
    // robot.move_to::<JointSpace<6>>([0.; 6])?;
    // ```
    robot.move_to::<FlangeSpace>(nalgebra::Isometry3::identity().into())?;
    robot.control_with::<JointPositionControl<6>, _>(|_state, _dt| {
        // Compute the next command from the current state.
        let joint = [0.5; 6];
        let done = false;
        (joint, done)
    })?;

    loop {
        physics.step()?;
        sleep(Duration::from_secs_f64(0.01));
    }
}
