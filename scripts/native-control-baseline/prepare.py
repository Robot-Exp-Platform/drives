#!/usr/bin/env python3
"""Export the pre-change Franka/behavior sources and add test-only loopback tooling.

Python 3.11+. No Cargo command, network access, build, test, or timing is run.
The destination must not exist. Generated sources/audits belong outside drives.
"""
import argparse
import difflib
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import tarfile
import tomllib

FRANKA = "70e0e022f9d85ce02021d672501ff8d0bf309560"
BEHAVIOR = "38094b239f9d03a9b0977ed98e2d2969ed60808b"
FILES = Path(__file__).resolve().parent


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args])


def digest(data):
    return hashlib.sha256(data).hexdigest()


def diff(before, after, name):
    return "".join(difflib.unified_diff(
        before.splitlines(True), after.splitlines(True),
        fromfile="old/" + name, tofile="baseline/" + name,
    ))


def export(repo, revision, destination):
    # Keep only files needed for Rust validation, never history or model assets.
    paths = ["Cargo.toml", "build.rs", "src", "tests", "README.md"]
    if repo.name == "robot_behavior":
        paths.append("benches")
    archive = git(repo, "archive", "--format=tar", revision, *paths)
    destination.mkdir(parents=True)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        for member in tar:
            name = PurePosixPath(member.name)
            if name.is_absolute() or ".." in name.parts:
                raise ValueError("Unsafe archive path: " + member.name)
            target = destination.joinpath(*name.parts)
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
            elif member.isfile():
                target.parent.mkdir(parents=True, exist_ok=True)
                source = tar.extractfile(member)
                if source is None:
                    raise ValueError("Missing archive data: " + member.name)
                target.write_bytes(source.read())
            else:
                raise ValueError("Unexpected archive link/device: " + member.name)


def audit_sources(repo, revision, destination, reversals=None):
    checks = {}
    reversals = reversals or {}
    names = git(repo, "ls-tree", "-r", "--name-only", revision, "src", "build.rs")
    for name in names.decode().splitlines():
        expected = git(repo, "show", revision + ":" + name)
        actual = (destination / name).read_bytes()
        if name in reversals:
            added = reversals[name].encode()
            if actual.count(added) != 1:
                raise ValueError("Test-only reversal is ambiguous: " + name)
            actual = actual.replace(added, b"", 1)
        if actual != expected:
            raise ValueError("Unexpected production source difference: " + name)
        checks[name] = digest(expected)
    return checks


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--drives-root", type=Path, default=FILES.parents[1])
    parser.add_argument("--output", type=Path, required=True,
                        help="Fresh output directory, normally below /tmp")
    parser.add_argument("--roplat-root", type=Path,
                        help="Core checkout root; defaults to sibling roplat")
    parser.add_argument("--lock-seed", type=Path,
                        help="Matching current integration Cargo.lock; copied, never updated here")
    args = parser.parse_args()
    drives = args.drives_root.resolve()
    output = args.output.resolve()
    core = (args.roplat_root or drives.parent / "roplat").resolve()
    seed_path = (args.lock_seed or drives / "Cargo.lock").resolve()
    if output == drives or drives in output.parents:
        parser.error("Keep generated baselines outside the drives checkout")
    if output.exists():
        parser.error("Refuse to overwrite an existing output directory")
    if not (core / "roplat/Cargo.toml").is_file():
        parser.error("--roplat-root must contain roplat/Cargo.toml")
    seed = seed_path.read_bytes()
    tomllib.loads(seed.decode())
    for repo, revision in [("franka-rust", FRANKA), ("robot_behavior", BEHAVIOR)]:
        git(drives / repo, "cat-file", "-e", revision + "^{commit}")

    workspace = output / "workspace"
    for repo, revision in [("franka-rust", FRANKA), ("robot_behavior", BEHAVIOR)]:
        export(drives / repo, revision, workspace / repo)

    behavior_manifest = workspace / "robot_behavior/Cargo.toml"
    original_behavior = behavior_manifest.read_text()
    old_path = 'path = "../../roplat/roplat"'
    if original_behavior.count(old_path) != 1:
        raise ValueError("Unexpected behavior core dependency spelling")
    behavior_manifest.write_text(original_behavior.replace(
        old_path, "path = " + json.dumps(str(core / "roplat"))))
    # Only relocate the already-optional core dependency; do not enable it.
    (workspace / "Cargo.toml").write_text('''[workspace]
members = ["robot_behavior", "franka-rust"]
resolver = "3"

[patch.crates-io]
robot_behavior = { path = "robot_behavior" }
''')
    (workspace / "Cargo.lock").write_bytes(seed)
    (output / "integration-lock-seed.lock").write_bytes(seed)

    destination = workspace / "franka-rust"
    franka_manifest = destination / "Cargo.toml"
    original_franka = franka_manifest.read_text()
    if "[dev-dependencies]" in original_franka:
        raise ValueError("Unexpected old Franka dev-dependencies")
    # Align the current test build through dev dependencies only. Production
    # source and ordinary dependency declarations retain the historical form.
    franka_manifest.write_text(original_franka + (
        '\n# Test-only parity with the current Franka protocol-test build.\n'
        '[dev-dependencies]\n'
        'robot_behavior = { version = "0.6.0", features = ["roplat"] }\n'
        'roplat = { version = "0.2.2", path = ' + json.dumps(str(core / "roplat")) + ' }\n'
        'tokio = { version = "1.48.0", features = ["io-util"] }\n'
    ))
    factory = (FILES / "from_test_impl.rs").read_text()
    robot = destination / "src/robot.rs"
    original_robot = robot.read_text()
    anchor = "    pub fn connect(&mut self, ip: &str) {"
    if original_robot.count(anchor) != 1 or not factory.lstrip().startswith("#[cfg(test)]"):
        raise ValueError("Unexpected test factory insertion")
    robot.write_text(original_robot.replace(anchor, factory + "\n" + anchor))
    tests = destination / "src/realtime/tests.rs"
    original_tests = tests.read_text()
    tests.write_text(original_tests + "\nmod native;\n")
    (destination / "src/realtime/tests/native").mkdir(parents=True)
    for source, target in [("native.rs", "src/realtime/tests/native.rs"),
                           ("performance.rs", "src/realtime/tests/native/performance.rs")]:
        shutil.copyfile(FILES / source, destination / target)

    patch = diff(original_robot, robot.read_text(), "franka-rust/src/robot.rs")
    patch += diff(original_tests, tests.read_text(), "franka-rust/src/realtime/tests.rs")
    for target in ["src/realtime/tests/native.rs", "src/realtime/tests/native/performance.rs"]:
        patch += diff("", (destination / target).read_text(), "franka-rust/" + target)
    (output / "test-only-graft.patch").write_text(patch)
    (output / "manifest-adjustments.patch").write_text(diff(
        original_behavior, behavior_manifest.read_text(), "robot_behavior/Cargo.toml") + diff(
            original_franka, franka_manifest.read_text(), "franka-rust/Cargo.toml"))

    report = {
        "franka_revision": FRANKA,
        "behavior_revision": BEHAVIOR,
        "core_checkout_head": git(core, "rev-parse", "HEAD").decode().strip(),
        "factory_is_cfg_test_only": True,
        "production_code_equal_after_removing_test_graft": True,
        "franka_source_sha256": audit_sources(drives / "franka-rust", FRANKA, destination, {
            "src/robot.rs": factory + "\n", "src/realtime/tests.rs": "\nmod native;\n",
        }),
        "behavior_source_sha256": audit_sources(
            drives / "robot_behavior", BEHAVIOR, workspace / "robot_behavior"),
        "integration_lock_seed_sha256": digest(seed),
        "graft_sha256": {name: digest((FILES / name).read_bytes())
                         for name in ["from_test_impl.rs", "native.rs", "performance.rs"]},
        "cargo_run": False,
        "timing_run": False,
        "feature_scope": "Franka default; test-only behavior/roplat + core + Tokio io-util align current protocol-test features. Audit compiled units with features.py.",
    }
    (output / "source-audit.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"workspace": str(workspace), "source_audit": "PASS", "cargo_run": False}))


if __name__ == "__main__":
    main()
