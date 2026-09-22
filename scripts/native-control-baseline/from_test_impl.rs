    #[cfg(test)]
    pub(crate) fn from_test_impl(robot_impl: FrankaRobotImpl) -> Self {
        Self {
            marker: PhantomData,
            robot_impl,
            is_moving: false,
            before_observers: control_observers(),
            after_observers: control_observers(),
            coord: OverrideOnce::new(Coord::OCS),
            scale: OverrideOnce::new(0.1),
            max_vel: OverrideOnce::new(Self::JOINT_VEL_BOUND),
            max_acc: OverrideOnce::new(Self::JOINT_ACC_BOUND),
            max_jerk: OverrideOnce::new(Self::JOINT_JERK_BOUND),
            max_cartesian_vel: OverrideOnce::new(Self::CARTESIAN_VEL_BOUND),
            max_cartesian_acc: OverrideOnce::new(Self::CARTESIAN_ACC_BOUND),
        }
    }
