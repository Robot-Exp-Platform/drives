# Independent drivers and optional roplat integration

Driver submodules retain complete manifests. They do not inherit dependencies
from `drives/Cargo.toml`, and cloning a driver does not require a sibling roplat
checkout. Application examples inside drives may deliberately use integration
paths; `examples/jaka_roplat_multilang` is one such example.

## Source policy for this internal baseline

The published `roplat` 0.2.2 archive predates the current `Execution` /
`ExecutionContext` API. The crates.io `robot_behavior` releases available during
this audit end at 0.5.4. A matching version number alone therefore does not provide
the APIs used by this work.

The manifests pin full Git commit hashes from these sources:

- `ssh://git@github.com/Robot-Exp-Platform/roplat.git`, core revision
  `b47f7b6aa54a230417331e4b85cefcc8eff9e128`;
- `ssh://git@github.com/Robot-Exp-Platform/robot_behavior.git`, `79a3820af143a3ab75dfd3b308469482aac534c0`;
- `roplat_rerun` also pins `ssh://git@github.com/Robot-Exp-Platform/rsbullet.git`
  at `804518bacad7fec80e065e5f95e1c34524cf8fdc`. Its `rerun_urdf`
  dependency uses the published 0.1.0 API, with Rerun 0.26 and nalgebra 0.34.

The core source baseline remains `c595393df441056a25b2af8bedfcbd0f598b3fec`.
Revision `b47f7b6aa54a230417331e4b85cefcc8eff9e128` only removes the unused
`cmake-gen` gitlink whose missing submodule URL prevented Cargo Git checkouts;
it does not change core Rust source or execution semantics.

This is an internal Git baseline, not a new crates.io release. The `version` key
on a Git dependency checks the package version; the full `rev` identifies the
actual code. Do not replace these dependencies with `"0.2.2"` / `"0.6.0"` until a
compatible registry release exists. Do not replace pinned revisions with moving
branches merely to resolve a build failure.

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
checkouts. Root registry patches remain for compatible dependencies and local
URDF integration. Those patches apply only when drives is the workspace root.
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

1. Validate and push the behavior revision whose APIs downstream drivers use.
2. Update every direct behavior dependency to the same full revision, then verify
   the dependency matrix. Commit and push the driver revisions.
3. Pin the matching RsBullet revision in roplat_rerun and validate that adapter.
4. Update drives' lockfile and gitlinks only after all referenced commits are
   available remotely. Run the integration matrix and the independent checks.

Do not introduce `workspace = true` into independently cloned submodules. If a
submodule later chooses its own internal workspace dependencies, its own root
must carry those definitions; it cannot require the external drives root.
