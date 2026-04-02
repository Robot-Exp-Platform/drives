use anyhow::Result;
use libunitree::UnitreeGo2;
use robot_behavior::{behavior::*, roplat_data_dir};
use rsbullet::{Mode, RsBullet};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    let assets_dir = roplat_data_dir().unwrap();

    let mut physics = RsBullet::new(Mode::Gui)?;
    physics
        .add_search_path(&assets_dir)?
        .set_gravity([0., 0., -10.])?
        .set_step_time(Duration::from_secs_f64(1. / 240.))?;

    let _ = physics
        .robot_builder::<UnitreeGo2>("robot_1")
        .base_fixed(true)
        .load()?;

    // robot_1.move_joint(&JakaA5::JOINT_DEFAULT)?;

    loop {
        physics.step()?;
        sleep(Duration::from_secs_f64(0.01));
    }
}
