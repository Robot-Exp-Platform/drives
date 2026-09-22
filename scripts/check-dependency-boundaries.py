#!/usr/bin/env python3
"""Finite compilation and dependency checks; never runs devices or GUI programs.

Independent mode copies a working source snapshot outside drives (no manifest
rewrites, sibling repositories, root patches, or inherited workspace settings).
It omits assets/vendor references because every selected check disables their
export and uses neither hardware SDK features nor simulator execution.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = {
    "robot_behavior": ("robot_behavior", ("", "roplat")),
    "libgo2": ("unitree-go2-rs", ("", "roplat")),
    "rsbullet": ("rsbullet/rsbullet", ("", "roplat")),
    "franka_rust": ("franka-rust", ("",)),
    "libjaka": ("libjaka-rs", ("",)),
    "libhans": ("libhans-rs", ("",)),
    "libaubo": ("libaubo-rs", ("",)),
    "libk1": ("libk1", ("",)),
    "roplat_exrobot": ("roplat_exrobot", ("",)),
    "roplat_rerun": ("roplat_rerun", ("",)),
}
CORE_NAMES = {
    "robot_behavior", "roplat", "roplat_macros", "roplat_system",
    "roplat_launch", "roplat_py", "roplat_build", "rsbullet", "rsbullet-core",
    "rsbullet_sys", "rerun_urdf",
}
PINNED_NAMES = {"roplat", "robot_behavior"}
SOURCE_FILES = {"Cargo.toml", "build.rs", "rust-toolchain", "rust-toolchain.toml"}
SOURCE_DIRS = {"src", "tests", "examples", "benches", "cpp", "include", "libhans_derive"}


def command(args, cwd, env, capture=False):
    print("+", " ".join(str(arg) for arg in args), "[in", cwd, "]", flush=True)
    with subprocess.Popen(args, cwd=cwd, env=env, text=True,
                          stdout=subprocess.PIPE if capture else None,
                          start_new_session=os.name == "posix") as process:
        try:
            stdout, _ = process.communicate(timeout=int(env["DRIVES_CHECK_COMMAND_TIMEOUT"]))
        except (subprocess.TimeoutExpired, KeyboardInterrupt):
            # Git/SSH can outlive Cargo. Stop our whole process group on timeout.
            if os.name == "posix":
                os.killpg(process.pid, signal.SIGTERM)
            else:
                process.terminate()
            try:
                process.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                if os.name == "posix":
                    os.killpg(process.pid, signal.SIGKILL)
                else:
                    process.kill()
                process.communicate()
            raise
        if process.returncode:
            raise subprocess.CalledProcessError(process.returncode, args, output=stdout)
    return stdout if capture else None


def cargo_args(action, package, feature, offline):
    args = ["cargo", action, "-p", package, "--no-default-features"]
    if feature:
        args += ["--features", feature]
    if offline:
        args += ["--offline"]
    return args


def assert_unique(metadata):
    for name in sorted(CORE_NAMES):
        ids = {p["id"] for p in metadata["packages"] if p["name"] == name}
        if len(ids) > 1:
            raise RuntimeError("multiple copies of %s: %s" % (name, sorted(ids)))


def assert_git_checkouts(metadata, env):
    cache = Path(env.get("CARGO_HOME", str(Path.home() / ".cargo"))).resolve() / "git" / "checkouts"
    for package in metadata["packages"]:
        if package["name"] not in CORE_NAMES or not (package.get("source") or "").startswith("git+"):
            continue
        location = Path(package["manifest_path"]).resolve()
        if cache not in location.parents:
            raise RuntimeError("Git package resolved outside Cargo checkout cache: " + str(location))


def assert_explicit_sources(metadata):
    member_ids = set(metadata["workspace_members"])
    pins = {name: set() for name in PINNED_NAMES}
    for package in metadata["packages"]:
        if package["id"] not in member_ids or package["name"] == "jaka_roplat_multilang":
            continue  # The multilingual demo deliberately belongs to drives.
        for dep in package["dependencies"]:
            if dep["name"] not in PINNED_NAMES:
                continue
            source = dep.get("source") or ""
            if not source.startswith("git+ssh://git@github.com/Robot-Exp-Platform/"):
                raise RuntimeError("%s must use an independently accessible pinned %s source: %s"
                                   % (package["name"], dep["name"], source or dep.get("path")))
            if not re.search(r"[?&]rev=[0-9a-f]{40}(?:&|$)", source):
                raise RuntimeError("dependency revision must be a full commit hash: " + source)
            pins[dep["name"]].add(source)
    for name, sources in pins.items():
        if len(sources) > 1:
            raise RuntimeError("inconsistent %s pins: %s" % (name, sorted(sources)))


def snapshot(repo, destination):
    paths = subprocess.check_output(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=repo).decode().split("\0")
    for name in paths:
        if not name:
            continue
        path = Path(name)
        if not (path.name in SOURCE_FILES or path.parts[0] in SOURCE_DIRS
                or path.name.startswith(("README", "LICENSE"))):
            continue
        source = repo / path
        if not source.is_file():
            continue
        target = destination / path
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
    if not (destination / "Cargo.toml").is_file():
        raise RuntimeError("no standalone Cargo.toml copied from " + str(repo))


def check_package(package, cwd, features, env, offline, skip_build, locked):
    for feature in features:
        selected_feature = package + "/" + feature if locked and feature else feature
        args = cargo_args("metadata", package, selected_feature, offline)
        # cargo metadata does not accept -p; the snapshot/root selects the workspace.
        del args[2:4]
        host = next(line.split(": ", 1)[1] for line in subprocess.check_output(
            ["rustc", "-vV"], text=True).splitlines() if line.startswith("host: "))
        args += ["--format-version", "1", "--filter-platform", env.get("CARGO_BUILD_TARGET", host)]
        if locked:
            args += ["--locked"]
        metadata = json.loads(command(args, cwd, env, capture=True))
        assert_unique(metadata)
        assert_git_checkouts(metadata, env)
        assert_explicit_sources(metadata)
        tree = cargo_args("tree", package, selected_feature, offline)
        tree += ["--edges", "normal", "--prefix", "none", "--format", "{p}", "--locked"]
        names = {line.split()[0] for line in command(tree, cwd, env, capture=True).splitlines() if line}
        core_present = "roplat" in names
        if core_present != bool(feature == "roplat"):
            raise RuntimeError("%s feature %r has unexpected core dependency=%s"
                               % (package, feature or "default", core_present))
        if not skip_build:
            check = cargo_args("check", package, selected_feature, offline) + ["--lib", "--locked"]
            command(check, cwd, env)
        print("PASS:", package, feature or "default", "single dependency identities; core=", core_present)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--independent", action="store_true", help="check sibling-free working source snapshots")
    parser.add_argument("--package", action="append", choices=PACKAGES, help="repeat to select packages")
    parser.add_argument("--offline", action="store_true", help="require dependencies already cached")
    parser.add_argument("--skip-build", action="store_true", help="dependency assertions only")
    parser.add_argument("--target-dir", type=Path, help="reuse an explicit compilation cache")
    parser.add_argument("--seed-lockfile", type=Path, help="seed independent resolution with an existing lockfile; sources remain unmodified")
    parser.add_argument("--command-timeout", type=int, default=300, help="maximum seconds per Cargo command (default: 300)")
    args = parser.parse_args()
    if args.command_timeout <= 0:
        parser.error("--command-timeout must be positive")
    if args.seed_lockfile and (not args.independent or not args.seed_lockfile.is_file()):
        parser.error("--seed-lockfile requires --independent and an existing file")
    default = ["robot_behavior", "libgo2", "roplat_exrobot"] if args.independent else ["robot_behavior", "libgo2", "rsbullet"]
    packages = args.package or default
    if args.independent and "rsbullet" in packages:
        parser.error("RsBullet is a nested workspace with Bullet vendor submodules; run its independent checkout directly using docs/dependency-sources.md")
    env = os.environ.copy()
    env.update(BULLET_SKIP_ASSET_EXPORT="1", ROPLAT_SKIP_ASSET_EXPORT="1", CARGO_NET_GIT_FETCH_WITH_CLI="true",
               DRIVES_CHECK_COMMAND_TIMEOUT=str(args.command_timeout))
    if args.offline:
        env["CARGO_NET_OFFLINE"] = "true"
    if args.target_dir:
        env["CARGO_TARGET_DIR"] = str(args.target_dir.resolve())
    for package in packages:
        relative, features = PACKAGES[package]
        if args.independent:
            with tempfile.TemporaryDirectory(prefix="drives-independent-") as temporary:
                checkout = Path(temporary) / package
                checkout.mkdir()
                snapshot(ROOT / relative, checkout)
                if args.seed_lockfile:
                    shutil.copyfile(args.seed_lockfile, checkout / "Cargo.lock")
                    print("Seed lockfile:", args.seed_lockfile.resolve(), "(dependency sources still resolved from unchanged manifests)", flush=True)
                check_package(package, checkout, features, env, args.offline, args.skip_build, False)
        else:
            # Explicit root is essential for nested workspaces such as RsBullet.
            # Qualified features select only this package's roplat adapter.
            check_package(package, ROOT, features, env, args.offline, args.skip_build, True)


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
        print("FAIL:", error, file=sys.stderr)
        sys.exit(1)
