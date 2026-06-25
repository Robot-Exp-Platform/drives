fn main() {}

#[cfg(test)]
mod tests {
    use std::{io::Write, net::TcpStream};

    use libjaka::{
        JakaA5,
        types::{EndMoveData, MoveLData},
    };
    use robot_behavior::{Coord, Pose, behavior::*};

    #[test]
    fn init() -> anyhow::Result<()> {
        let mut robot = JakaA5::new("10.5.5.100");
        robot.init()?;

        Ok(())
    }

    #[test]
    fn enable() -> anyhow::Result<()> {
        let mut robot = JakaA5::new("10.5.5.100");

        robot.enable()?;
        Ok(())
    }

    #[test]
    fn move_to_packed() -> anyhow::Result<()> {
        let mut robot = JakaA5::new("10.5.5.100");

        robot.move_to::<JointSpace<6>>(JakaA5::JOINT_PACKED)?;

        Ok(())
    }

    #[test]
    fn move_joint() -> anyhow::Result<()> {
        let mut robot = JakaA5::new("10.5.5.100");
        robot.with_coord(Coord::Relative);
        robot.move_to::<JointSpace<6>>([-0.1; 6])?;
        Ok(())
    }

    #[test]
    fn move_cartesian() -> anyhow::Result<()> {
        let mut robot = JakaA5::new("10.5.5.100");

        robot.with_coord(Coord::Relative);
        robot.move_to::<FlangeSpace>(Pose::Position([0., 0., 0.05]))?;

        Ok(())
    }

    #[test]
    fn move_cartesian_rel() -> anyhow::Result<()> {
        let mut robot = JakaA5::new("10.5.5.100");

        robot
            .with_cartesian_acceleration(0.05)
            .with_cartesian_velocity(0.02)
            .with_coord(Coord::Relative);
        robot
            .move_to::<FlangeSpace>(Pose::Euler([0., 0., 1.], [0.; 3]))
            .unwrap();

        Ok(())
    }

    #[test]
    fn end_move() -> anyhow::Result<()> {
        let mut robot = JakaA5::new("10.5.5.100");
        robot.robot_impl._end_move(EndMoveData {
            end_position: [100., 200.1, 200.5, 0., 0., 0.],
            speed: 21.5,
            accel: 31.5,
        })?;

        Ok(())
    }

    #[test]
    fn tcp_test() -> anyhow::Result<()> {
        let mut tcp = TcpStream::connect("10.5.5.100:10001")?;

        let msg = r#"{"cmdName":"moveL","relFlag":1,"cartPosition":[0,0,50,0,0,0],"speed":20,"accel":50,"tol":0.5}"#.as_bytes();
        // let msg = r#"{"cmdName":"joint_move","relFlag":0,"jointPosition":[0,90.5,90.5,0,90.5,0],"speed":20.5,"accel":20.5}
        // "#.as_bytes();

        // let msg = r#"{"cmdName":"movc","relFlag":move_mode,"pos_mid":[100,200,300,0,0,0],"pos_end":[300,200,100,0,0,0],"speed":20,"accel":50,"tol":0.5,"executing_line_id":0,"end_cond":{"di_type":0,"di_index":0,"di_state":1},"circle_mode":0,"circle_cnt":0}
        // "#.as_bytes();
        tcp.write(msg)?;

        Ok(())
    }
}
