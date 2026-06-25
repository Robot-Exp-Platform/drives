fn main() {
    // This example demonstrates robot behavior implementations.
    println!("Robot behavior example running...");
}

#[cfg(test)]
mod tests {
    use std::fs;

    use robot_behavior::behavior::{FlangeSpace, JointSpace, Joints, MotionSpace, Pose};

    struct DemoRobot;

    impl Joints<3> for DemoRobot {
        const JOINT_MIN: [f64; 3] = [-1.0; 3];
        const JOINT_MAX: [f64; 3] = [1.0; 3];
    }

    #[test]
    fn serialize_typed_motion_targets_to_json() {
        let joint_target: <JointSpace<3> as MotionSpace<DemoRobot>>::Target = [1.0, 0.0, -1.0];
        let flange_target: <FlangeSpace as MotionSpace<DemoRobot>>::Target =
            Pose::from(([0.0, 0.0, 0.5], [0.0, 0.0, 0.0]));
        let motions = serde_json::json!([
            {
                "space": "JointSpace<3>",
                "target": joint_target,
            },
            {
                "space": "FlangeSpace",
                "target_homo": flange_target.homo(),
            },
        ]);

        fs::write(
            "typed_motion_targets.json",
            serde_json::to_string_pretty(&motions).unwrap(),
        )
        .unwrap();
        let written = fs::read_to_string("typed_motion_targets.json").unwrap();
        assert!(written.contains("JointSpace"));
        assert!(written.contains("FlangeSpace"));
    }
}
