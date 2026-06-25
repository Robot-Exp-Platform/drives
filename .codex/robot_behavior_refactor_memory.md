# robot_behavior Refactor Memory

Date: 2026-06-20

Scope: notes for future AI work in `E:/yixing/code/Robot-Exp/drives`.
This file is intentionally descriptive only. It should not be treated as source
of truth over the code.

## Workspace Shape

- Root workspace is a Rust `resolver = "3"` workspace for robot drivers,
  simulation, visualization, and adapter crates.
- Root `[patch.crates-io]` redirects:
  - `roplat` to `../roplat/roplat`
  - `robot_behavior` to `./robot_behavior`
- Current root `Cargo.toml` includes `./robot_behavior` as a workspace member.
  This differs from older AGENTS notes that said it was excluded.
- Major consumers of `robot_behavior` in this workspace:
  - `franka-rust`
  - `libaubo-rs`
  - `libhans-rs`
  - `libjaka-rs`
  - `rsbullet/rsbullet`
  - `roplat_exrobot`
  - `examples/jaka_dual`
  - `examples/jaka_roplat_multilang`

## Current Git/Refactor State

- Root repo is dirty and contains many submodule/member changes. Do not revert
  unrelated changes.
- `robot_behavior` is also dirty:
  - Old files deleted: `src/robot.rs`, `src/utils.rs`, several old robot files.
  - New module tree added under `src/robot/`.
  - Old implementation copied/preserved under `src/robot_old/`.
- Treat the current `robot_behavior` state as an in-progress refactor, not a
  clean migration.

## New robot_behavior Core Model

The new public core is type-driven:

- `Robot`: lifecycle + `State` + `CONTROL_PERIOD` + `read_state`.
- `Joints<N>`: model-level joint constants and bounds.
- `EndPoint`: task-space linear/angular bounds.
- `Arm<N>`: central arm capability requiring:
  - `Robot`
  - `Joints<N>`
  - `EndPoint`
  - `MoveTo<JointSpace<N>>`
  - `MoveTo<FlangeSpace>`
- `MotionSpace<R>` maps type-level spaces to target types.
  - `JointSpace<N>` -> `[f64; N]`
  - `FlangeSpace`, `TcpSpace`, `EndSpace` -> `Pose`
  - `Relative<S>`, `Inertial<S>` wrappers preserve target type.
- Driver-implemented motion traits:
  - `MoveTo<S>`
  - `MoveTraj<S>`
- User-facing blanket traits:
  - `Motion`
  - `MotionFile`
- Control is also type-driven:
  - `ControlSpace<R>` maps channel to observation and command.
  - Channels include `TorqueControl<N>`, `JointPositionControl<N>`,
    `JointVelocityControl<N>`, `CartesianVelocityControl<N>`.
  - Driver-implemented `RealtimeControl<S>`.
  - User-facing blanket `Control`.

## Confirmed Design Direction

- `Arm<N>` is not the root model of robot behavior. It is a capability bundle:
  a combination of traits plus some arm-specific convenience API.
- Do not split or redesign the current `Arm<N>` shape yet. It is intentionally
  kept as a concise category trait for now; revisit only after the surrounding
  non-arm abstractions settle.
- Future robot categories should follow the same pattern:
  - define focused reusable capabilities;
  - combine them into category traits such as arm, humanoid, quadruped, mobile
    base, gripper, etc.;
  - provide category-specific API only where it is genuinely specific.
- Some API currently inside `Arm<N>` may later move to shared capabilities if
  that removes real duplication, but avoid premature over-splitting.
- Do not downgrade or replace `Joints<N>` for now. Keep the current joint model
  direction while preparing additional abstractions around it.
- Prepare separate state models for different robot categories rather than
  forcing every robot into `ArmState<N>`.
- Add new operation spaces beyond the current arm-centric spaces.
- Ignore current FFI during the Rust refactor. Python/C/C++ bindings will be
  rewritten after the Rust API is stable.

## Compatibility Gaps To Resolve Later

- Many downstream files still refer to old names such as:
  - `MotionType`
  - `ControlType`
  - `ArmParam`
  - `ArmPreplannedMotion*`
  - `ArmRealtimeControl`
  - `ArmMotionRhythm`
- These old names are currently present mainly under `robot_behavior/src/robot_old/`
  and old FFI macro code, but are not clearly integrated into the new public
  module tree.
- `robot_behavior/src/ffi.rs` currently declares feature-gated FFI modules but
  comments out the `pub use` re-exports. Downstream crates using FFI macros may
  break until the FFI story is settled.
- `robot_behavior/robot_behavior.pyi` and `robot_behavior.hpp` still describe
  old API concepts. They will need a deliberate compatibility or regeneration
  pass after Rust API stabilizes.
- Current FFI compatibility does not need to constrain the Rust API design.
- Some README text appears mojibake when read through the current PowerShell
  environment. Avoid rewriting docs unless encoding is explicitly handled.

## Component Integration Notes

- `roplat_exrobot/src/exrobot.rs` is the smallest concrete implementation of the
  new model. It implements `Robot`, `Joints`, `EndPoint`, `Arm`, `MoveTo`,
  `MoveTraj`, and multiple `RealtimeControl` channels for a generic
  `ExRobot<N>`. Use it as the quick API shape reference.
- `rsbullet/rsbullet/src/rsbullet_robot.rs` adapts simulation to the same model:
  - `RobotDescription::URDF` supplies the asset path.
  - `AddRobot` / `EntityBuilder` load the URDF into Bullet.
  - Joint motion and realtime joint channels enqueue callbacks into the Bullet
    stepping loop.
  - Generic cartesian IK/control currently returns explicit unsupported errors.
- Real drivers such as `libjaka-rs/src/robot.rs` implement the same traits over
  vendor SDK calls:
  - lifecycle maps to power/enable/network commands;
  - `MoveTo<JointSpace<N>>` and `MoveTo<FlangeSpace>` translate units and
    coordinate modes;
  - `MoveTraj<JointSpace<N>>` may use realtime servo loops or planners like
    Ruckig;
  - `RealtimeControl<JointPositionControl<N>>` owns the vendor realtime loop.
- `world.rs`, `physics_engine.rs`, and `renderer.rs` are intentionally small
  backend-neutral traits. They let `rsbullet`, Rerun, and examples share a
  builder-style world loading surface without depending on one concrete backend.

## Suggested Refactor Plan

1. Establish the intended public surface:
   - Decide whether old `MotionType`/`ControlType` remain as compatibility
     enums or are fully replaced by type-level spaces/channels.
   - Decide if `robot_old` is temporary private backup or a compatibility
     module.

2. Make `robot_behavior` compile alone first:
   - Run `cargo check -p robot_behavior` after any source changes.
   - Fix module exports before touching downstream drivers.

3. Migrate downstream by layer:
   - `rsbullet/rsbullet` and `roplat_exrobot` first, because they exercise the
     generic adapter/simulation path.
   - Then real drivers: `libjaka-rs`, `libhans-rs`, `libaubo-rs`,
     `franka-rust`.
   - Then examples and FFI feature builds.

4. Rebuild compatibility wrappers deliberately:
   - Rust ergonomic old-style wrappers, if needed.
   - Python/C++/C macro exports, if still supported.
   - `.pyi` and `.hpp` artifacts.

5. Verification order:
   - `cargo check -p robot_behavior`
   - `cargo check -p rsbullet`
   - `cargo check -p roplat_exrobot`
   - `cargo check --workspace`
   - feature checks only after core builds:
     - `cargo check -p robot_behavior --features to_py`
     - `cargo check -p robot_behavior --features to_cxx`

## Cautions

- The root workspace patches `roplat`, so public API changes in
  `../roplat/roplat` can break this workspace immediately.
- Real robot crates may depend on vendor SDKs or runtime services; prefer
  compile checks over hardware tests unless explicitly requested.
- Do not clean, reset, or remove dirty changes unless the user explicitly asks.
