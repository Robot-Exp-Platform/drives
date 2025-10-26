use libjaka::{JakaMini2, types::SetTioVoutParamData};
use robot_behavior::behavior::*;
fn main() {
    // let (mut robot_1, mut robot_2) = (JakaMini2::new(""), JakaMini2::new(""));
    // robot_1.enable().unwrap();
    // robot_2.enable().unwrap();
    let mut robot_1 = JakaMini2::new("192.168.1.100");
    let _ = robot_1.enable();
    // let _ = robot_1.disable();
    // robot_1.move_joint_rel(&[0.1; 6]).unwrap();
    robot_1
        ._set_tio_vout_param(SetTioVoutParamData {
            tio_vout_ena: 1,
            tio_vout_vol: 0,
        })
        .unwrap();
    let tio = robot_1._get_tio_vout_param().unwrap();
    println!("{tio:?}");
}
