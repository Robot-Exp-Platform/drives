use anyhow::Result;
use libjaka::JakaMini2;
use robot_behavior::behavior::*;

fn main() -> Result<()> {
    // let mut physics = RsBullet::new(Mode::Gui)?;
    // physics
    //     .add_search_path("./asserts")?
    //     .set_gravity([0., 0., -10.])?
    //     .set_step_time(Duration::from_secs_f64(1. / 240.))?;

    // let mut robot: RsBulletRobot<JakaMini2> = physics.robot_builder("robot").load()?;

    let mut robot = JakaMini2::new("10.5.5.100");

    robot.move_to::<JointSpace<6>>([
        0.43642145501756757,
        0.043303908409149966,
        -1.9405468867228346,
        -1.4748536326574774e-05,
        -1.2443031821789015,
        0.4364556645834642,
    ])?;

    robot.move_traj_from_file::<JointSpace<6>>("E:\\yixing\\code\\Robot-Exp\\drives\\examples\\jaka_dual\\data\\step08_optimized_trajectory_rust.json",
    )?;

    // loop {
    //     physics.step()?;
    //     sleep(Duration::from_secs_f64(1. / 240.));
    // }

    Ok(())
}
