use anyhow::Result;
use franka_rust::FrankaEmika;
use robot_behavior::behavior::*;
use rsbullet::{Mode, RsBullet};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    let mut physics = RsBullet::new(Mode::Gui)?;
    physics
        .add_search_path("./asserts")?
        .set_gravity([0., 0., -10.])?
        .set_step_time(Duration::from_secs_f64(1. / 240.))?;

    let mut robots = Vec::new();
    for i in 0..5 {
        let mut row = Vec::new();
        for j in 0..5 {
            let robot = physics
                .robot_builder::<FrankaEmika>(format!("robot_{}_{}", i, j))
                .base_fixed(true)
                .base([i as f64, j as f64, 0.])
                .load()?;
            row.push(robot);
        }
        robots.push(row);
    }

    for robot_l in robots {
        for mut robot in robot_l {
            let mut t = 0.0_f64;
            let dt = 1.0_f64 / 240.0_f64;

            robot.control_with::<JointPositionControl<7>, _>(move |_, _| {
                t += dt;
                ([t.sin(); 7], false)
            })?;

            // robot.move_to::<JointSpace<7>>(FrankaEmika::JOINT_DEFAULT)?;
        }
    }

    loop {
        physics.step()?;
        sleep(Duration::from_secs_f64(0.01));
    }
}
