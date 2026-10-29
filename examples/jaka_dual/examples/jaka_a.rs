use libjaka::JakaA5;
use roplat_behavior::{Pose, behavior::*};

fn main() -> anyhow::Result<()> {
    let mut robot = JakaA5::new("10.5.5.100");
    robot.robot_impl._power_on()?;
    robot.enable()?;
    robot.move_to::<JointSpace<6>>(JakaA5::JOINT_DEFAULT)?;
    robot.move_to::<JointSpace<6>>(JakaA5::JOINT_PACKED)?;
    robot.move_to::<Relative<FlangeSpace>>(Pose::from_position([-10., 0.1, 0.]))?;

    // robot.disable()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use libjaka::JakaA5;

    #[test]
    fn power_off() -> anyhow::Result<()> {
        let mut robot = JakaA5::new("10.5.5.100");

        robot._disable()?;
        robot._power_off()?;
        Ok(())
    }
}
