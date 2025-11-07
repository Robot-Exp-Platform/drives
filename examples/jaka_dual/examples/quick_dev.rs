use anyhow::Result;
use libjaka::JakaMini2;
use robot_behavior::{MotionType, behavior::*};
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

    // 设计机器人具备的行为
    //
    // 样例：运动到某目标点
    // ```rust
    // robot
    //     .with_velocity(&[5.; 6])
    //     .with_acceleration(&[2.; 6])
    //     .move_joint(&[0.; 6])?;
    // ```
    robot.move_cartesian(&nalgebra::Isometry3::identity().into())?;
    robot.move_with_closure(|_state, _dt| {
        // 设计实时规划逻辑，根据当前状态 state 计算下一步的运动指令
        // let motion = MotionType::Joint([0.5; 6]);
        let motion = MotionType::Cartesian(nalgebra::Isometry3::identity().into());
        let done = false; // 是否结束该行为
        (motion, done)
    })?;

    loop {
        physics.step()?;
        sleep(Duration::from_secs_f64(0.01));
    }
}
