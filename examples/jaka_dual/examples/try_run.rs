use libjaka::{
    JakaMini2,
    types::{TioVout, TioVoutMode},
};
use robot_behavior::{RobotResult, behavior::*};

fn main() -> RobotResult<()> {
    let mut robot = JakaMini2::new("10.5.5.100");

    robot._power_on()?;
    robot._enable()?;

    robot.move_joint(&[
        1.3237359251204017,
        0.24551094556528436,
        2.020892599208774,
        -0.05861559141024878,
        0.9230434626038143,
        1.268438314891961,
    ])?;

    robot.set_tio_vout(TioVout::Enable(TioVoutMode::V24V))?;

    robot.move_joint(&[
        1.3368226986771132,
        0.1831809920227407,
        2.0050015170327664,
        -0.05389380681194803,
        1.01234990632599,
        -0.00013982436826040205,
    ])?;

    robot.move_joint(&[
        1.5688852290362394,
        0.16482115952376425,
        2.0812691240977057,
        -0.0649672305907041,
        -0.588502851646126,
        -9.188746878526859e-5,
    ])?;

    robot.set_tio_vout(TioVout::Disable)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use libjaka::JakaMini2;
    use robot_behavior::{RobotResult, behavior::*};

    #[test]
    fn move_to_default() -> RobotResult<()> {
        let mut robot = JakaMini2::new("10.5.5.100");
        robot.move_joint(&[0.; _])?;

        Ok(())
    }

    #[test]
    fn read_state() -> RobotResult<()> {
        let mut robot = JakaMini2::new("10.5.5.100");
        let q = robot.state()?.joint.unwrap();
        let pose = robot.state()?.pose_o_to_ee.unwrap().euler();
        println!("q:{q:?}\npose:{pose:?}");
        Ok(())
    }
}
