fn main() {
    // This example demonstrates robot behavior implementations.
    println!("Robot behavior example running...");
}

#[cfg(test)]
mod tests {
    use std::fs;

    use robot_behavior::{MotionType, Pose};

    #[test]
    fn serialize_motion_types_to_json() {
        const N: usize = 3;
        let motions: [MotionType<N>; 6] = [
            MotionType::Joint([1.0, 2.0, 3.0]),
            MotionType::JointVel([0.1, 0.2, 0.3]),
            MotionType::Cartesian(Pose::from(([0.0, 0.0, 0.5], [0.0, 0.0, 0.0]))),
            MotionType::CartesianVel([0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
            MotionType::Position([0.5, 0.5, 0.5]),
            MotionType::Stop,
        ];
        fs::write(
            "motion_types.json",
            serde_json::to_string_pretty(&motions).unwrap(),
        )
        .unwrap();
        let written = fs::read_to_string("motion_types.json").unwrap();
        assert!(written.contains("Joint"));
        assert!(written.contains("Cartesian"));
    }
}
