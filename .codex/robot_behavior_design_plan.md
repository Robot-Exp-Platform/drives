# robot_behavior Design Plan

Date: 2026-06-21

Goal: prepare `robot_behavior` for arm, humanoid, quadruped, mobile base, and
other robot categories while keeping the current `Arm<N>` design unchanged for
now.

## Implementation Status

Implemented on 2026-06-21:

- Added reusable state fragments in `robot_behavior/src/robot/state.rs`.
- Added category trait sketches in `robot_behavior/src/robot/category.rs`.
- Added non-arm operation spaces in `robot_behavior/src/robot/spaces.rs`.
- Added non-arm realtime control channels in `robot_behavior/src/robot/control.rs`.
- Exported the new API through `robot_behavior/src/robot/mod.rs` and the
  `robot_behavior::behavior` prelude.
- Verified with `cargo check -p robot_behavior`.
- Implemented mock consumers in `roplat_exrobot/src/exrobot.rs`:
  - `ExMobileBase`
  - `ExQuadruped<N>`
  - `ExHumanoid<N>`
- Verified with `cargo check -p roplat_exrobot`.
- Relaxed category traits so `MobileBase`, `Quadruped<N>`, and `Humanoid<N>`
  no longer constrain `Robot::State`. Like `Arm<N>`, they expose category state
  as an explicit view method and allow drivers to keep native SDK states.
- Added generic model/mapping layer in `robot_behavior/src/robot/model.rs`:
  - `SpaceMap<From, To>`
  - `TypedSpaceMap<From, To>`
  - `ForwardKinematics<N>`
  - `InverseKinematics<N>`
  - `JacobianModel<N>`
  - `DynamicsModel<N>`
  - model spaces/inputs such as `JacobianSpace`, `MassMatrixSpace`,
    `CoriolisInput`, and `GravityInput`.
- Added function-style dynamics controller constructors in
  `robot_behavior/src/utils/controller/dynamics.rs`:
  - `gravity_compensation_control`
  - `computed_torque_control`
- Added more function-style controller constructors:
  - `joint_pd_control`
  - `joint_pid_control`
  - `whole_body_joint_pd_control`
  - `whole_body_joint_pid_control`
  - `base_velocity_pid_control`
  - `cartesian_impedance_control`
- Verified again with `cargo check -p robot_behavior` and
  `cargo check -p roplat_exrobot`.
- Reorganized robot module boundaries so file names match concepts more tightly:
  - `spaces.rs` owns type-level space markers, including joint, endpoint,
    whole-body, gait, and model spaces.
  - `joint.rs` owns only the `Joints<N>` capability trait.
  - `endpoint.rs` owns only the `EndPoint` capability trait.
  - `model.rs` owns `SpaceMap`, typed mapping traits, and model input structs.
  - `kinematics_dynamatics.rs` was renamed to `kinematics_dynamics.rs`.
- Landed the new abstractions into downstream API/examples:
  - Removed type-specific convenience methods over typed spaces/channels, such
    as `move_joint`, `move_cartesian`, `move_joint_traj`,
    `control_torque_with`, `control_joint_position_with`,
    `control_joint_velocity_with`, `control_cartesian_velocity_with`, and
    `control_cartesian_pose_with`.
  - Split public preludes by audience:
    - `robot_behavior::behavior::*` is the application-facing prelude. It keeps
      blanket call traits such as `Motion` and `Control`, plus spaces,
      channels, states, categories, world/render traits, and model helpers.
    - `robot_behavior::driver::*` is the driver-implementation prelude. It
      re-exports `behavior::*` plus implementation traits `MoveTo`,
      `MoveTraj`, and `RealtimeControl`.
  - `behavior::*` intentionally does not export `MoveTo`, `MoveTraj`, or
    `RealtimeControl`, so method syntax such as
    `robot.move_to::<JointSpace<N>>(target)` resolves to `Motion` without
    conflicting with the driver-level `MoveTo::move_to`.
  - Standardized examples on application-facing method calls such as
    `robot.move_to::<JointSpace<N>>(target)`,
    `robot.move_traj::<FlangeSpace>(traj)`, and
    `robot.control_with_closure::<TorqueControl<N>, _>(controller)`.
  - Added `CartesianPoseControl<N>` as a realtime Cartesian pose channel.
  - Adapted `FrankaModel` to `SpaceMap`, `ForwardKinematics`,
    `JacobianModel`, and `DynamicsModel` via its model library.
  - Added Franka realtime support for joint velocity, Cartesian velocity, and
    Cartesian pose channels in addition to existing joint position and torque.
  - Updated Franka examples away from old `robot_behavior::MotionType` /
    `ControlType` usage and toward typed spaces/channels.
  - Updated `jaka_dual` examples to use explicit typed motion/control calls.
  - Updated `jaka_roplat_multilang` examples to manually run generated
    Python/C++ roplat nodes into a typed joint trajectory, then execute via
    `robot.move_traj::<JointSpace<N>>(trajectory)`, instead of old
    `ArmMotionRhythm`.
  - Generalized controller constructors around reusable target sources:
    - Added `constant_joint_target_fn`, `joint_traj_target_fn`,
      `joint_path_target_fn`, `constant_pose_target_fn`,
      `pose_traj_target_fn`, and `pose_path_target_fn` in
      `utils/path_generate.rs`.
    - Expanded impedance controllers into fixed-target, trajectory,
      target-function, and handle/session variants.
    - Added target-function variants for joint PD/PID, whole-body joint
      PD/PID, base velocity PID, and computed-torque control.
    - Kept fixed-target controller constructors as convenience wrappers around
      the more general target-function forms.
  - Added `roplat_exrobot/examples/capability_showcase.rs` covering arm,
    mobile base, quadruped, humanoid, and realtime control capabilities.

## Downstream Review Notes

Reviewed `franka-rust` and `libjaka-rs` for behavior that may deserve future
shared traits:

- Motion context / override policy:
  - Both Franka and JAKA keep persistent and one-shot motion settings such as
    coordinate mode, velocity scale, joint velocity/acceleration bounds, and
    Cartesian bounds.
  - Candidate future traits:
    - `MotionContext`
    - `MotionScale`
    - `CoordinateMode`
    - `LimitOverride`
  - Do not extract yet; current `Arm<N>` already carries enough of this surface
    for arms, and premature splitting would hurt simplicity.

- Trajectory planning adapter:
  - Franka uses `utils::trajectory` / `copp` path planning for path and waypoint
    APIs.
  - JAKA uses Ruckig to interpolate waypoints into dense joint trajectories.
  - Candidate future trait:
    - `TrajectoryPlanner<S>` or `WaypointInterpolator<S>`.
  - This is worth extracting only when a second backend wants to share the same
    planner object rather than just a helper function.

- Realtime command loop shape:
  - Franka and JAKA both adapt closures into vendor realtime/servo loops.
- Existing `RealtimeControl<S>` captures the public part well.
- Possible future helper: reusable loop driver utilities, not necessarily a
  public trait.
- Roplat rhythm adapters are intentionally deferred. The next design pass should
  focus on middleware-independent driver capabilities first.

- Dynamics/model access:
  - Franka exposes model-level operations such as frame pose, Jacobian, mass,
    coriolis, and gravity.
  - Candidate future traits:
    - `FrameKinematics`
    - `JacobianModel`
    - `DynamicsModel`
  - This is a strong candidate for `robot_behavior` because humanoid and
    quadruped whole-body control will also need Jacobian and dynamics queries.

- Impedance handles:
  - Franka has joint/cartesian impedance async handles with stiffness, damping,
    target, and completion flag.
  - Candidate future traits:
    - `JointImpedanceControl<N>`
    - `CartesianImpedanceControl`
  - Keep out of the core until `RealtimeControl<S>` proves insufficient.

- Gripper:
  - Franka gripper has homing, grasp, move, stop, and state readback.
  - Candidate future category:
    - `Gripper`
  - This is separate from `Arm<N>` and fits the category-trait pattern.

## Non-Goals For This Phase

- Do not split or redesign `Arm<N>` yet.
- Do not rewrite FFI yet.
- Do not replace or downgrade `Joints<N>`.
- Do not force every robot category into `ArmState<N>`.

## Design Principles

- `Robot` remains the lifecycle root.
- Robot categories are capability bundles, not inheritance roots.
- `Arm<N>` is one category bundle. Future `Humanoid<N>`, `Quadruped<N>`,
  `MobileBase`, `Gripper`, etc. should follow the same style.
- Reusable concepts should live below category traits only when they are clearly
  shared by multiple categories.
- Rust core should use typed spaces/channels. Runtime command enums can return
  later in FFI or serialization layers.

## State Model Plan

Keep `ArmState<N>` as the arm-specific state model.

Add reusable state fragments under a future `robot/state.rs` or
`robot/state/` module:

```rust
pub struct JointState<const N: usize> {
    pub position: Option<[f64; N]>,
    pub velocity: Option<[f64; N]>,
    pub acceleration: Option<[f64; N]>,
    pub effort: Option<[f64; N]>,
}

pub struct BaseState {
    pub pose: Option<Pose>,
    pub velocity: Option<[f64; 6]>,
    pub acceleration: Option<[f64; 6]>,
}

pub struct EndEffectorState {
    pub pose: Option<Pose>,
    pub velocity: Option<[f64; 6]>,
    pub wrench: Option<[f64; 6]>,
}

pub struct ContactState {
    pub active: bool,
    pub force: Option<[f64; 3]>,
    pub point: Option<[f64; 3]>,
    pub normal: Option<[f64; 3]>,
}
```

Then add category-specific states by composition:

```rust
pub struct MobileBaseState {
    pub base: BaseState,
}

pub struct QuadrupedState<const N: usize> {
    pub joints: JointState<N>,
    pub base: BaseState,
    pub feet: [EndEffectorState; 4],
    pub contacts: [ContactState; 4],
}

pub struct HumanoidState<const N: usize> {
    pub joints: JointState<N>,
    pub base: BaseState,
    pub hands: [EndEffectorState; 2],
    pub feet: [EndEffectorState; 2],
    pub contacts: Vec<ContactState>,
}
```

Open design choice:

- Keep fixed-size arrays where the count is intrinsic and stable, such as four
  feet for a quadruped.
- Use `Vec` where the count may vary by model, sensor availability, or optional
  hardware.

## Operation Space Plan

Keep current spaces:

- `JointSpace<N>`
- `FlangeSpace`
- `TcpSpace`
- `EndSpace`
- `Relative<S>`
- `Inertial<S>`

Add non-arm spaces gradually:

```rust
pub struct BasePoseSpace;
pub struct BaseVelocitySpace;

pub struct WholeBodyJointSpace<const N: usize>;
pub struct WholeBodyVelocitySpace<const N: usize>;
pub struct WholeBodyTorqueSpace<const N: usize>;

pub struct CenterOfMassSpace;
pub struct GaitSpace;
pub struct FootSpace<const LEG: usize>;
pub struct HandSpace<const HAND: usize>;
```

Suggested target mappings:

```rust
impl<R> MotionSpace<R> for BasePoseSpace {
    type Target = Pose;
}

impl<R> MotionSpace<R> for BaseVelocitySpace {
    type Target = [f64; 6];
}

impl<const N: usize, R: Joints<N>> MotionSpace<R> for WholeBodyJointSpace<N> {
    type Target = [f64; N];
}

impl<const N: usize, R: Joints<N>> MotionSpace<R> for WholeBodyVelocitySpace<N> {
    type Target = [f64; N];
}

impl<R> MotionSpace<R> for CenterOfMassSpace {
    type Target = [f64; 3];
}

impl<R> MotionSpace<R> for GaitSpace {
    type Target = GaitCommand;
}

impl<const LEG: usize, R> MotionSpace<R> for FootSpace<LEG> {
    type Target = Pose;
}
```

`GaitCommand` should start minimal and backend-neutral:

```rust
pub enum GaitCommand {
    Stop,
    Stand,
    Walk {
        linear: [f64; 3],
        angular: [f64; 3],
    },
}
```

Avoid encoding one vendor's gait controller as the general gait abstraction.

## Realtime Control Channel Plan

Keep current channels:

- `TorqueControl<N>`
- `JointPositionControl<N>`
- `JointVelocityControl<N>`
- `CartesianVelocityControl<N>`

Add general channels later:

```rust
pub struct BaseVelocityControl;
pub struct WholeBodyTorqueControl<const N: usize>;
pub struct WholeBodyPositionControl<const N: usize>;
pub struct WholeBodyVelocityControl<const N: usize>;
pub struct BalanceControl;
```

Suggested mappings:

```rust
impl<R> ControlSpace<R> for BaseVelocityControl {
    type Obs = BaseState;
    type Command = [f64; 6];
}

impl<const N: usize, R> ControlSpace<R> for WholeBodyTorqueControl<N> {
    type Obs = JointState<N>;
    type Command = [f64; N];
}
```

For humanoid/quadruped, observations may need category states rather than
`JointState<N>` only. Do not finalize this until the first real consumer exists.

## Category Trait Sketches

Keep `Arm<N>` unchanged for now.

Future mobile base:

```rust
pub trait MobileBase: Robot + MoveTo<BasePoseSpace> + RealtimeControl<BaseVelocityControl> {
    fn base_state(&mut self) -> RobotResult<BaseState>;
}
```

Future quadruped:

```rust
pub trait Quadruped<const N: usize>:
    Robot
    + Joints<N>
    + MoveTo<GaitSpace>
    + MoveTo<WholeBodyJointSpace<N>>
    + RealtimeControl<WholeBodyTorqueControl<N>>
{
    fn state(&mut self) -> RobotResult<QuadrupedState<N>>;
}
```

Future humanoid:

```rust
pub trait Humanoid<const N: usize>:
    Robot
    + Joints<N>
    + MoveTo<WholeBodyJointSpace<N>>
    + MoveTo<CenterOfMassSpace>
    + RealtimeControl<WholeBodyTorqueControl<N>>
{
    fn state(&mut self) -> RobotResult<HumanoidState<N>>;
}
```

Note: these are sketches. Avoid adding all of them to public API until at least
one consumer needs them.

## Suggested Implementation Order

1. Add reusable state fragments.
2. Add low-risk spaces:
   - `BasePoseSpace`
   - `BaseVelocitySpace`
   - `WholeBodyJointSpace<N>`
   - `WholeBodyVelocitySpace<N>`
   - `WholeBodyTorqueSpace<N>`
3. Add category states only when there is a concrete target robot or simulator.
4. Add first non-arm category trait, preferably `MobileBase` or `Quadruped`,
   because their command spaces are clearer than full humanoid control.
5. Update examples or `roplat_exrobot` with a minimal mock implementation.
6. Run compile checks after each layer.

## Verification Strategy

- Start with `cargo check -p robot_behavior`.
- Then check the first consumer:
  - mock category implementation in `roplat_exrobot`, or
  - simulation adapter in `rsbullet`.
- Run full workspace only after the core and one consumer compile.

## Design Risks

- Over-abstracting before there is a real non-arm consumer.
- Making `GaitSpace` too vendor-specific.
- Making state models too rigid for partially observed robots.
- Tying Rust API shape to old FFI concepts.

## Next Driver-Line Priorities

Defer roplat downstream rhythm design until the middleware-independent driver
surface is healthier.

Recommended next capabilities:

1. Dynamics/model access:
   - `FrameKinematics`
   - `JacobianModel`
   - `DynamicsModel`
   - Start from Franka's model API: frame pose, body/zero Jacobian, mass,
     coriolis, and gravity.
   - Implemented initial generic form as `SpaceMap`-based traits. Next step is
     to adapt Franka's model object to these traits and see whether the generic
     input/output shapes feel natural.

2. Controller-constructor pattern:
   - Keep `RealtimeControl<S>` as the execution entry point.
   - Add controller objects/functions that build closures or closure state for
     `control_with_closure`.
   - This keeps impedance, balance, force, and other controllers independent of
     roplat.
   - Prefer free functions returning closures, like `joint_impedance_control`,
     rather than requiring every controller to be a struct implementing a trait.

3. Impedance control traits only after the constructor pattern is clear:
   - `JointImpedanceControl<N>`
   - `CartesianImpedanceControl`
   - Prefer these as optional driver conveniences, not replacements for
     `RealtimeControl<S>`.

Still deferred controller families:

- Admittance control: needs a clear observed wrench source. `EndEffectorState`
  has `wrench`, but `ArmState` does not yet expose external wrench/contact
  consistently.
- Force and hybrid force-position control: needs wrench/contact frame semantics,
  force targets, axis selection, and safety limits.
- Whole-body QP/task-space control: needs task stack, constraints, contact model,
  and a QP/optimization backend.
- Gait control: needs robot-specific gait phase/contact scheduling semantics; do
  not freeze a universal trait from one mock.
- Safe-stop/watchdog/filter wrappers: useful, but should be designed as command
  wrappers once the controller closure surface is stable.
