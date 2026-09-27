# Independent drivers and optional roplat integration

Driver submodules retain complete manifests. They do not inherit dependencies
from `drives/Cargo.toml`, and cloning a driver does not require a sibling roplat
checkout. Application examples inside drives may deliberately use integration
paths; `examples/jaka_roplat_multilang` is one such example.

## Source policy for this release preparation

The prepared core version is `roplat` 0.3.0; `robot_behavior` remains at the
previously selected, unpublished 0.6.0. Neither has been uploaded to crates.io.
The published core 0.2.2 lacks the current Execution / Lifecycle API, and the
published behavior line ends at 0.5.4 as of the 2026-09-27 registry check. Current
Git sources are therefore still necessary for independent driver builds.

The manifests retain full commit pins and declare the corresponding version
requirement. The confirmed upstream revisions for this update are:

| Source | Prepared package version | Git revision |
| --- | --- | --- |
| `ssh://git@github.com/Robot-Exp-Platform/roplat.git` | 0.3.0 | `9bbdc7d235f5a1b07fc8b9eb0eba016fe2d001be` |
| `ssh://git@github.com/Robot-Exp-Platform/robot_behavior.git` | 0.6.0 | `781245ca4d662a693cfb24955328a5a610859591` |
| `ssh://git@github.com/Robot-Exp-Platform/rsbullet.git` | rsbullet / rsbullet-core 0.4.0; rsbullet_sys 0.3.2 | `ed8dcd86d80426f0cc0e425e310e57be8194d43d` |
| `ssh://git@github.com/Robot-Exp-Platform/rerun_urdf.git` | 0.1.1 | `016e0964f9dd2463025b4140460af84259cad199` |

`roplat_rerun` requires the prepared RsBullet 0.4.0 at the revision above. Its
direct `rerun_urdf` dependency now uses a Git pin as well: a registry-only requirement
for unpublished 0.1.1 would otherwise break the independent checkout. Rerun 0.26
and nalgebra 0.34 remain unchanged.

See the [version update record](review/2026-09-27-release-versions.md) for the
complete package matrix. The earlier core packaging fix removed a broken unused
`cmake-gen` gitlink; that fix remains included. Historical reports preserve the
old version numbers, commit pins and measurements.

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
