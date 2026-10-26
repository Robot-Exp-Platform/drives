#!/usr/bin/env python3
"""Initialize only Cargo inputs; reuse checkout authentication without logging it."""
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
PATHS = [
    "examples/jaka_roplat_multilang", "franka-rust", "libaubo-rs", "libhans-rs",
    "libjaka-rs", "libk1", "unitree-go2-rs", "roplat_exrobot", "roplat_rerun",
    "rsbullet", "utils/rerun_urdf",
]


def main():
    env = os.environ.copy()
    env.update(GIT_TERMINAL_PROMPT="0", GIT_LFS_SKIP_SMUDGE="1")
    config = [("url.https://github.com/.insteadOf", "git@github.com:")]
    # actions/checkout installs this repository's token as an extraheader.
    # Inherit it for child Git processes through their environment only.
    header = subprocess.run(
        ["git", "config", "--local", "--get", "http.https://github.com/.extraheader"],
        cwd=ROOT, capture_output=True, text=True, check=False,
    )
    if header.returncode == 0 and header.stdout.strip():
        config.append(("http.https://github.com/.extraheader", header.stdout.strip()))
    offset = int(env.get("GIT_CONFIG_COUNT", "0"))
    for index, (key, value) in enumerate(config, offset):
        env[f"GIT_CONFIG_KEY_{index}"] = key
        env[f"GIT_CONFIG_VALUE_{index}"] = value
    env["GIT_CONFIG_COUNT"] = str(offset + len(config))
    subprocess.run(["git", "submodule", "sync", "--", *PATHS], cwd=ROOT, env=env, check=True)
    subprocess.run(
        ["git", "submodule", "update", "--init", "--recursive", "--depth=1", "--", *PATHS],
        cwd=ROOT, env=env, check=True,
    )


if __name__ == "__main__":
    main()
