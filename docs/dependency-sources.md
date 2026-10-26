# Independent drivers and optional roplat integration

Driver submodules retain complete manifests. They do not inherit dependencies
from `drives/Cargo.toml`, and cloning a driver does not require a sibling roplat
checkout. Application examples inside drives may deliberately use integration
paths; `examples/jaka_roplat_multilang` is one such example.

## Pending: roplat_behavior 0.9 (2026-10-26, unpublished)

`robot_behavior` is renamed `roplat_behavior` and split into `roplat_behavior`,
`roplat_kinematics`, `roplat_control` and `roplat_trajectory` (all 0.9.0, a
sibling repository instead of a drives submodule). Its pose and sample types
come from the core's new `roplat_msgs`. On the migration branches the drivers
require these crates (`franka_rust` / `libjaka` / `libhans` / `libaubo` /
`roplat_exrobot` / `roplat_rerun` 0.5.0, `rsbullet` / `rsbullet-core` 0.7.0).
None of them is on crates.io yet, so until they are released an independent
driver checkout does not resolve from the registry; build at this integration
root, whose `[patch.crates-io]` points at `../roplat` and `../roplat_behavior`.
The table below remains the last published state. Migration notes:
[MIGRATION.md](https://github.com/Robot-Exp-Platform/roplat_behavior/blob/main/MIGRATION.md).

## Published sources

The drivers depend on the crates.io releases published on 2026-10-22. No driver
manifest pins roplat, robot_behavior or another released driver to a Git
revision. An independent checkout resolves these packages from the registry and
needs neither GitHub SSH access nor a sibling roplat checkout.

| Package | crates.io version | Packaged from |
| --- | --- | --- |
| `roplat`, `roplat_macros`, `roplat_system`, `roplat_launch`, `roplat_build`, `roplat_cpp`, `roplat_cpp_msg`, `roplat_py`, `cargo-roplat` | 0.5.0 | roplat `8ca570d3f8be7eef269ab315c7b2a579310454a3` (tag `v0.5.0`) |
| `robot_behavior` | 0.8.0 | `8694e5b6dfa81c97136d9e01f36445131c9b06d1` |
| `rsbullet`, `rsbullet-core` | 0.6.0 | `d3c0eddd134f92e858a438eb23da7bb120903d1b` |
| `franka_rust` | 0.4.0 | `09953e4fec64264c0a3731f9d7e9ed54bd02cd35` |
| `libjaka` | 0.4.0 | `18cfa30b0984f38586145b6a30dc20873891ddd0` |
| `libaubo` | 0.4.0 | `71e63d09174f3e794ed68f925ac9685e0e1770dc` |
| `libhans` | 0.4.0 | `55501fea06325e037352523281333565c251df46` |
| `roplat_exrobot` | 0.4.0 | `3efb153021f0a4e5400f76a982bc1f74bfb2cbf9` |
| `roplat_rerun` | 0.4.0 | `0834001d0b3ef2c7471ae78b17deb757b54c004a` |

Packages whose version did not change were not republished: `rsbullet_sys`
0.3.2, `libhans_derive` 0.1.3, `rerun_urdf` 0.1.1 and `libk1` 0.1.0 resolve to
their existing registry releases. `libgo2` 0.3.0 is not published; install it
from Git as its README describes. Its optional roplat dependency uses the
registry release. The older registry releases (roplat 0.4 and earlier,
robot_behavior 0.7 and earlier) do not provide the contract below.

## Core contract in these versions

Node and Rhythm declare a separate `Resources` tuple. Graph bindings use
`&mut robot >> user.resources.0`; the call receives a tuple of mutable references
whose lifetime follows that process/drive. Business Input/Output no longer carry
borrowed resource bodies. `ResourceSlot`, `reclaim_input`, and return edges are
removed. Control rhythms have Input/Output `()` and Resources `(R,)`;
`MotionNode` takes a target, borrows `(R,)`, and returns `RobotResult<()>`.
Go2 and Bullet rhythms retain their internally owned devices/engines and declare
Resources `()`. Resource replacement through a mutable reference is allowed.

Users of the same resource are ordered only by data and `.trigger` dependencies,
including transitive ones, never by binding source order; two users without a
dependency path are rejected at compile time. Crossed resource chains are
completed with ordering edges, and a borrow that still cannot stay in one
execution unit is handed across a join without locks, clones or spawns. The
`robot_behavior` tests declare their resource dependencies explicitly.

Driver versions move with their public dependency types: franka_rust, libjaka,
libaubo, libhans, roplat_exrobot and roplat_rerun 0.4.0; libgo2 0.3.0.
`roplat_rerun` requires RsBullet 0.6.0. Rerun 0.26, nalgebra 0.34, pure
helper/sys packages and application versions remain unchanged. See the
[resource borrow update record](review/2026-10-13-resource-borrow.md) for
validation and scope. Historical reports preserve their original versions,
commit pins and measurements.

The current libraries require a Rust nightly toolchain (`generic_const_exprs` /
`adt_const_params`); these checks use the caller's selected toolchain and do not
claim stable-Rust support. Builds and checks skip optional asset export with:

```sh
export ROPLAT_SKIP_ASSET_EXPORT=1
export BULLET_SKIP_ASSET_EXPORT=1
```

The drives root keeps `[patch.crates-io]` entries that replace roplat,
robot_behavior, RsBullet and rerun_urdf with the local integration checkouts, so
changes to a local public interface are used immediately. Those patches apply
only when drives is the workspace root; an independent checkout uses the
registry releases. Cargo.lock and `cargo metadata` checks prevent accidental
multiple copies of the core traits.

## Feature boundaries

| Package | Default | Optional `roplat` feature |
| --- | --- | --- |
| `robot_behavior` | Robot capabilities, sync/async control contracts | Node adapters, ControlRhythm and AsyncControlRhythm |
| `libgo2` | Native driver and mock support | Go2ControlRhythm and Go2SportRhythm |
| `rsbullet` | Simulator and simulated robot APIs | SimRhythm |
| Franka, JAKA, Hans, Aubo, ExRobot | Native robot_behavior capabilities | No empty forwarding feature is required |
| `roplat_rerun` | Visualization using behavior and simulator APIs | Does not require the roplat core |

`robot_behavior::roplat` exports its adapters, not a replacement re-export of the
whole roplat crate. Applications using `#[roplat::system]`, `ExecutionContext`, or
other core APIs directly declare the same `roplat` 0.5 dependency themselves.

Here, “default without roplat” describes normal dependency edges and
`cargo check --lib`. Franka deliberately enables `robot_behavior/roplat` and a
direct core dependency in `dev-dependencies` for System integration tests;
`cargo test` or `--all-targets` may therefore enable those development features.

RsBullet retains Tokio for asynchronous motion timers even with `roplat` disabled.
This feature separation does not change simulator timing or stopping policies.

## Finite validation

From drives, use the committed lockfile and local integration sources:

```sh
python3 scripts/check-dependency-boundaries.py --target-dir /tmp/drives-boundary-target
```

The current-host matrix checks both default and `roplat` configurations of
robot_behavior, Go2, and RsBullet, verifies that the core disappears from default
normal dependencies, requires roplat and robot_behavior to be declared as
crates.io dependencies with one version requirement each, and rejects Git
sources or multiple identities of framework/behavior packages. It compiles
libraries only; it never runs devices, a simulator, or a GUI. Add `--offline`
after dependencies have been fetched. Add `--skip-build` for just the dependency
assertions.

Validate the same source files without drives' parent manifest or sibling paths:

```sh
python3 scripts/check-dependency-boundaries.py --independent \
  --target-dir /tmp/drives-independent-target
```

This creates temporary source snapshots of robot_behavior, Go2, and ExRobot.
Manifests are copied unchanged, so dependencies resolve from crates.io. Heavy
assets and hardware SDK references are omitted because this default-feature
matrix does not use them. These checks prove ordinary library/source resolution
independence; they do not validate packaging, all native SDK features, or device
behavior. Repeat `--package` to select additional native drivers.

Each Cargo command has a 300-second timeout (`--command-timeout` changes it).
To reuse already verified registry dependency versions, add `--seed-lockfile
Cargo.lock`. This copies the integration lockfile into each temporary snapshot
before Cargo updates only what its independent manifest requires. Record this
option in validation results: it is an independent source check with seeded
dependency versions, not a fresh unlocked dependency resolution. It never
modifies the input lockfile.

RsBullet is itself a workspace containing a Bullet Git submodule. Validate an
independent full checkout separately after initializing its vendor source:

```sh
# Run inside a standalone RsBullet checkout.
git submodule update --init --recursive rsbullet-sys/bullet3
BULLET_SKIP_ASSET_EXPORT=1 ROPLAT_SKIP_ASSET_EXPORT=1 \
  cargo check -p rsbullet --no-default-features --lib
BULLET_SKIP_ASSET_EXPORT=1 ROPLAT_SKIP_ASSET_EXPORT=1 \
  cargo check -p rsbullet --no-default-features --features roplat --lib
```

## Updating the baseline

1. Publish the core release first. Driver manifests may only require versions
   that already exist on crates.io; a GitHub push does not satisfy a registry
   dependency.
2. Update robot_behavior's core requirement, validate it independently, and
   publish it. Then update all direct behavior users to that release.
3. Publish independent prerequisites such as rerun_urdf and the matching RsBullet
   release before raising their requirements in roplat_rerun.
4. In each driver repository, update the manifest and its own Cargo.lock outside
   drives, validate, and push. Update drives' lockfile and gitlinks only after all
   referenced commits are available remotely. Run the integration matrix and the
   independent checks.

Do not reintroduce Git revision pins for published packages to work around a
build failure; fix the release or publish a new version instead. Do not introduce
`workspace = true` into independently cloned submodules. If a submodule later
chooses its own internal workspace dependencies, its own root must carry those
definitions; it cannot require the external drives root.
