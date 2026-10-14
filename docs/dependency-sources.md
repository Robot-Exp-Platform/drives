# Independent drivers and optional roplat integration

Driver submodules retain complete manifests. They do not inherit dependencies
from `drives/Cargo.toml`, and cloning a driver does not require a sibling roplat
checkout. Application examples inside drives may deliberately use integration
paths; `examples/jaka_roplat_multilang` is one such example.

## Source policy for this release preparation

The prepared core version is `roplat` 0.5.0; `robot_behavior` is 0.8.0.
Node and Rhythm declare a separate `Resources` tuple. Graph bindings use
`&mut robot >> user.resources.0`; the call receives a tuple of mutable references
whose lifetime follows that process/drive. Business Input/Output no longer carry
borrowed resource bodies. `ResourceSlot`, `reclaim_input`, and return edges are
removed. Control rhythms have Input/Output `()` and Resources `(R,)`;
`MotionNode` takes a target, borrows `(R,)`, and returns `RobotResult<()>`.
Go2 and Bullet rhythms retain their internally owned devices/engines and declare
Resources `()`. Resource replacement through a mutable reference is allowed.

These versions are Git preparation branches (`codex/resource-borrow-channel`),
not registry uploads or merged main revisions. Older registry installation
examples do not describe this new trait contract. Independent drivers require
the exact Git revisions below; a version number alone does not select this code.

| Source | Prepared package version | Git revision |
| --- | --- | --- |
| `ssh://git@github.com/Robot-Exp-Platform/roplat.git` | 0.5.0 | `d6484b1c595b7d69e8727dc04d471f4bbf57def4` |
| `ssh://git@github.com/Robot-Exp-Platform/robot_behavior.git` | 0.8.0 | `1f6bf80e6fa18cfb331ea5c18db790cc85ef8e8d` |
| `ssh://git@github.com/Robot-Exp-Platform/rsbullet.git` | rsbullet / rsbullet-core 0.6.0; rsbullet_sys 0.3.2 | `d3c0eddd134f92e858a438eb23da7bb120903d1b` |
| `ssh://git@github.com/Robot-Exp-Platform/rerun_urdf.git` | 0.1.1 | `016e0964f9dd2463025b4140460af84259cad199` |

Driver versions move with their public dependency types: franka_rust, libjaka,
libaubo, libhans, roplat_exrobot and roplat_rerun 0.4.0; libgo2 0.3.0.
`roplat_rerun` requires the RsBullet 0.6.0 revision above. The unchanged pinned
`rerun_urdf` source is still necessary for independent resolution. Rerun 0.26,
nalgebra 0.34, pure helper/sys packages and application versions remain unchanged.

See the [resource borrow update record](review/2026-10-13-resource-borrow.md)
for validation and scope. Historical reports preserve their original versions,
commit pins and measurements. The earlier packaging fix removing the unused
`cmake-gen` gitlink remains included.

Same-resource users follow resource binding source order. Crossed multi-resource
DAGs that cannot keep exclusive borrows within the current execution-unit layout
are diagnosed explicitly and deferred by author decision. This release adds no
implicit lock, clone or fallback scheduling policy for those graphs.

This is Git release preparation, not a crates.io upload. The `version` key on a
Git dependency checks the package version and becomes the registry requirement
when packaging; the full `rev` identifies the actual source. Until matching
registry packages are available, keep both. Do not replace pinned revisions with
moving branches merely to resolve a build failure.

GitHub repository access and an already configured SSH key/agent are required.
The current libraries require a Rust nightly toolchain (`generic_const_exprs` /
`adt_const_params`); these checks use the caller's selected toolchain and do not
claim stable-Rust support. Use Cargo's Git CLI integration so it uses that existing
authentication setup:

```sh
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export ROPLAT_SKIP_ASSET_EXPORT=1
export BULLET_SKIP_ASSET_EXPORT=1
```

No passwords, tokens, or private keys belong in manifests, scripts, or logs.
Private dependency source retrieval can still be necessary during resolution when
an optional feature is disabled; optional means the crate is absent from that
build graph, not that Cargo can always avoid inspecting its source manifest.

The drives root patches the exact Git source URLs to the local integration
checkouts. Root registry patches remain for compatible registry dependencies; the new
URDF Git source also has an explicit local source patch. Those patches apply only when drives is the workspace root.
An independent checkout fetches its declared revisions instead. Cargo.lock and
`cargo metadata` checks prevent accidental multiple copies of the core traits.

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
other core APIs directly declare the same pinned core dependency themselves.

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
normal dependencies, and rejects multiple identities of framework/behavior
packages. It compiles libraries only; it never runs devices, a simulator, or a GUI.
Add `--offline` after sources and dependencies have been fetched. Add
`--skip-build` for just the dependency assertions.

Validate the same source files without drives' parent manifest or sibling paths:

```sh
python3 scripts/check-dependency-boundaries.py --independent \
  --target-dir /tmp/drives-independent-target
```

This creates temporary source snapshots of robot_behavior, Go2, and ExRobot.
Manifests are copied unchanged. Heavy assets and hardware SDK references are
omitted because this default-feature matrix does not use them. These checks prove
ordinary library/source resolution independence; they do not validate packaging,
all native SDK features, or device behavior. Repeat `--package` to select additional
native drivers. Snapshot dependencies use the real declared remote revisions,
so the behavior revision must be pushed before these checks can succeed.

Each Cargo command has a 300-second timeout (`--command-timeout` changes it).
To reuse already verified registry dependency versions while independently
resolving the unchanged Git sources, add `--seed-lockfile Cargo.lock`. This copies
the integration lockfile into each temporary snapshot before Cargo updates only
what its independent manifest requires. Record this option in validation results:
it is an independent source check with seeded dependency versions, not a fresh
unlocked dependency resolution. It never modifies the input lockfile.

RsBullet is itself a workspace containing a Bullet Git submodule. Validate an
independent full checkout separately after initializing its vendor source:

```sh
# Run inside a standalone RsBullet checkout.
git submodule update --init --recursive rsbullet-sys/bullet3
CARGO_NET_GIT_FETCH_WITH_CLI=true BULLET_SKIP_ASSET_EXPORT=1 \
  ROPLAT_SKIP_ASSET_EXPORT=1 cargo check -p rsbullet --no-default-features --lib
CARGO_NET_GIT_FETCH_WITH_CLI=true BULLET_SKIP_ASSET_EXPORT=1 \
  ROPLAT_SKIP_ASSET_EXPORT=1 cargo check -p rsbullet --no-default-features --features roplat --lib
```

## Updating the baseline

1. Validate and push the core revision containing the required version and API.
2. Update behavior's core version/pin; validate and push behavior. Then update all
   direct behavior users to that same full revision.
3. Validate and push independent prerequisites such as rerun_urdf, and validate
   and push the matching RsBullet revision before pinning both in roplat_rerun.
4. Update drives' lockfile and gitlinks only after all referenced commits are
   available remotely. Run the integration matrix and the independent checks.
5. For an actual registry release, recheck availability and publish upstream
   packages before their dependents. GitHub pushes do not satisfy registry edges.

Do not introduce `workspace = true` into independently cloned submodules. If a
submodule later chooses its own internal workspace dependencies, its own root
must carry those definitions; it cannot require the external drives root.
