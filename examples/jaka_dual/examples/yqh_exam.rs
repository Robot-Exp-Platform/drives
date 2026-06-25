use std::{
    f64::consts::{FRAC_PI_2, PI},
    time::Duration,
};

use franka_rust::FrankaEmika;
use nalgebra as na;
use robot_behavior::{Pose, behavior::*};
use roplat_rerun::RerunHost;
use rsbullet::RsBullet;

fn main() -> anyhow::Result<()> {
    let mut renderer = RerunHost::new("jaka_calibration")?;
    let mut physics_engine = RsBullet::new(rsbullet::Mode::Gui)?;

    physics_engine
        .add_search_path("./asserts")?
        .set_gravity([0., 0., -9.81])?
        .set_step_time(Duration::from_secs_f64(1. / 240.))?;
    renderer.add_search_path("./asserts")?;

    let mut robot = physics_engine
        .robot_builder::<FrankaEmika>("robot_1")
        .base([0.0, 0.0, 0.0])
        .base_fixed(true)
        .load()?;

    let robot_renderer = renderer
        .robot_builder::<FrankaEmika>("robot_1")
        .base([0.0, 0.0, 0.0])
        .base_fixed(true)
        .load()?;

    robot_renderer.attach_from(&mut robot)?;

    for _ in 0..100 {
        physics_engine.step()?;
    }

    robot.move_to::<JointSpace<7>>([0.0, -FRAC_PI_2, 0.0, 0.0, -FRAC_PI_2, 0.0, 0.0])?;

    for _ in 0..200 {
        physics_engine.step()?;
    }

    let translation = na::Translation3::new(0.16, -0.20, -0.0);
    let rotation = na::UnitQuaternion::from_euler_angles(PI, 0.0, 0.0);
    let target_pose = na::Isometry3::from_parts(translation, rotation);

    robot.move_to::<FlangeSpace>(Pose::Quat(target_pose))?;
    // robot.move_to::<JointSpace<6>>([FRAC_PI_2; 6])?;

    for _ in 0..300 {
        physics_engine.step()?;
    }

    // println!(">>> Drawing a line along +Y axis...");
    // let line_end_pose = na::Isometry3::from_parts(
    //     na::Translation3::new(0.4, 0.1, 0.2),
    //     rotation,
    // );
    // robot.move_to::<FlangeSpace>(Pose::Quat(line_end_pose))?;

    loop {
        physics_engine.step()?;
    }
}
