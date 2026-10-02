#!/usr/bin/env python3
"""Prepare a disposable, combined Cargo publication rehearsal without running Cargo.

Requires Python 3.11+. Selected internal Git dependencies and the selected
roplat_rerun -> rerun_urdf registry edge are changed to path+the original version
in this snapshot. No [patch] is used. Cargo's `package --workspace` subsequently
strips paths from real .crate manifests and verifies the batch through its
temporary registry. This is not proof that
these versions can be uploaded to crates.io, nor an edit of the source repos.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import tomllib


CORE = [
    "roplat_launch", "roplat_system", "roplat_macros", "roplat", "roplat_build",
    "roplat_cpp", "roplat_cpp_msg", "roplat_py", "cargo-roplat",
]
DRIVES = [
    "robot_behavior", "franka-rust", "libaubo-rs", "libhans-rs",
    "libhans-rs/src/libhans_derive", "libjaka-rs", "libk1", "unitree-go2-rs",
    "roplat_exrobot", "roplat_rerun", "rsbullet/rsbullet", "rsbullet/rsbullet-core",
    "rsbullet/rsbullet-sys", "utils/rerun_urdf",
]
SKIP_PARTS = {".git", "target", "__pycache__", ".venv", "node_modules"}


def run_git(directory: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-C", str(directory), *args], text=True)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def copy_git_files(source: Path, destination: Path, inventory: list[dict]) -> None:
    # Read current bytes for tracked files, including uncommitted fixes. Untracked
    # files are included only when Git does not ignore them. Gitlinks are skipped;
    # Bullet is supplied by its already packaged archive instead.
    listed = run_git(source, "ls-files", "--cached", "--others", "--exclude-standard", "-z")
    for rel in sorted(set(listed.split("\0")) - {""}):
        relative = Path(rel)
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError(f"Unsafe Git file path: {rel}")
        if SKIP_PARTS.intersection(relative.parts):
            continue
        src, dst = source / relative, destination / relative
        if not src.is_file():
            continue
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dst, follow_symlinks=True)
        inventory.append({"source": str(src), "snapshot": str(dst), "sha256": digest(src)})


def dependency_tables(manifest: dict):
    for name in ("dependencies", "dev-dependencies", "build-dependencies"):
        yield name, manifest.get(name, {})
    for target, table in manifest.get("target", {}).items():
        for name in ("dependencies", "dev-dependencies", "build-dependencies"):
            yield f"target.{target}.{name}", table.get(name, {})


def normalize_selected_git(manifest_path: Path, selected: dict[str, Path]) -> list[dict]:
    original = manifest_path.read_text()
    parsed = tomllib.loads(original)
    expected = []
    for table_name, table in dependency_tables(parsed):
        for alias, spec in table.items():
            if not isinstance(spec, dict) or "git" not in spec:
                continue
            name = spec.get("package", alias)
            if name not in selected:
                continue
            if "version" not in spec:
                raise ValueError(f"Selected dependency lacks a publish version: {manifest_path}:{alias}")
            expected.append((table_name, alias, name, spec))

    # The current project declares all Git dependencies as inline tables. Reject
    # other representations instead of silently changing unrelated manifest data.
    changes = []
    updated = original
    for table_name, alias, name, spec in expected:
        pattern = re.compile(r"(?m)^(\s*" + re.escape(alias) + r"\s*=\s*)\{([^}]+)\}")
        relative = Path(os.path.relpath(selected[name], manifest_path.parent)).as_posix()
        matches = [m for m in pattern.finditer(updated) if re.search(r'\bgit\s*=', m[2])]
        if not matches:
            # A repeated normal/dev dependency was already normalized below.
            continue
        for match in reversed(matches):
            body = match[2]
            for key in ("git", "rev", "branch", "tag"):
                body = re.sub(r"\b" + key + r'\s*=\s*"[^"\n]*"\s*,?\s*', "", body)
            body = body.strip().rstrip(",").strip()
            replacement = match[1] + "{ " + body + ', path = ' + json.dumps(relative) + " }"
            updated = updated[:match.start()] + replacement + updated[match.end():]

    normalized = tomllib.loads(updated)
    before_tables = dict(dependency_tables(parsed))
    after_tables = dict(dependency_tables(normalized))
    for table_name, alias, name, spec in expected:
        actual = after_tables[table_name][alias]
        wanted = {k: v for k, v in spec.items() if k not in {"git", "rev", "branch", "tag"}}
        wanted["path"] = Path(os.path.relpath(selected[name], manifest_path.parent)).as_posix()
        if actual != wanted:
            raise AssertionError(f"Unexpected manifest rewrite at {manifest_path}:{table_name}.{alias}")
        changes.append({"manifest": str(manifest_path), "table": table_name,
                        "dependency": alias, "package": name,
                        "before": dict(spec), "after": dict(actual)})
        before_tables[table_name][alias] = wanted
    if parsed != normalized:
        raise AssertionError(f"Unrelated manifest contents changed: {manifest_path}")
    if updated != original:
        manifest_path.write_text(updated)
    return changes


def normalize_registry_batch_edge(manifest_path: Path, selected: dict[str, Path]) -> list[dict]:
    """Bind this one owned registry dependency to the same staged batch.

    Keeping this dependency registry-only while simultaneously staging a local
    rerun_urdf with the already-published version mixes checksums in Cargo's
    generated locks. The published .crate still contains the original version
    requirement; only the disposable pre-publication package identity changes.
    """
    original = manifest_path.read_text()
    parsed = tomllib.loads(original)
    if parsed["package"]["name"] != "roplat_rerun":
        return []
    alias = "rerun_urdf"
    spec = parsed["dependencies"][alias]
    wanted = {"version": spec} if isinstance(spec, str) else dict(spec)
    if "path" in wanted:
        return []
    if "git" in wanted or "version" not in wanted:
        raise ValueError("Unexpected rerun_urdf dependency representation")
    wanted["path"] = Path(os.path.relpath(selected[alias], manifest_path.parent)).as_posix()
    value = "{ " + ", ".join(key + " = " + json.dumps(value) for key, value in wanted.items()) + " }"
    section = re.search(r"(?ms)^\[dependencies\]\n.*?(?=^\[|\Z)", original)
    if section is None:
        raise ValueError("Missing dependencies section")
    changed, count = re.subn(r"(?m)^rerun_urdf\s*=.*$", "rerun_urdf = " + value, section[0])
    if count != 1:
        raise ValueError("Expected exactly one rerun_urdf registry dependency")
    updated = original[:section.start()] + changed + original[section.end():]
    actual = tomllib.loads(updated)
    parsed["dependencies"][alias] = wanted
    if parsed != actual:
        raise AssertionError("Unexpected registry batch identity change")
    manifest_path.write_text(updated)
    return [{"manifest": str(manifest_path), "table": "dependencies", "dependency": alias,
             "package": alias, "before": spec, "after": wanted,
             "reason": "Bind the registry dependency to the same owned publication batch; avoid mixed checksums."}]


def extract_bullet(archive: Path, destination: Path) -> dict:
    with tarfile.open(archive) as tar:
        roots = {Path(x.name).parts[0] for x in tar.getmembers() if x.name}
        if len(roots) != 1:
            raise ValueError("Expected one root directory in the Bullet crate archive")
        root = roots.pop()
        if f"{root}/LICENSE" not in tar.getnames():
            raise ValueError("Use the refreshed rsbullet_sys crate containing its root LICENSE")
        for member in tar.getmembers():
            rel = Path(member.name).relative_to(root)
            if not rel.parts:
                continue
            if member.issym() or member.islnk() or member.isdev() or ".." in rel.parts:
                raise ValueError(f"Unsafe archive member: {member.name}")
            target = destination / rel
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
            elif member.isfile():
                target.parent.mkdir(parents=True, exist_ok=True)
                with tar.extractfile(member) as stream, target.open("wb") as output:
                    shutil.copyfileobj(stream, output)
                target.chmod(member.mode)
    # Reuse the original manifest, so this batch tests the same build and exclude
    # choices as the source crate rather than a previously normalized manifest.
    shutil.copy2(destination / "Cargo.toml.orig", destination / "Cargo.toml")
    (destination / "Cargo.toml.orig").unlink()
    (destination / ".cargo_vcs_info.json").unlink(missing_ok=True)
    return {"archive": str(archive), "sha256": digest(archive), "root": root}


def verify_bullet_sources(source: Path, snapshot_root: Path, inventory: list[dict]) -> int:
    """Reject stale archives, including changes in native build inputs.

    extract_bullet has restored Cargo.toml.orig as Cargo.toml. Cargo generated
    the lockfile and copied the readme from the parent RsBullet directory.
    """
    count = 0
    for snapshot in sorted(snapshot_root.rglob("*")):
        if not snapshot.is_file():
            continue
        relative = snapshot.relative_to(snapshot_root)
        if relative == Path("Cargo.lock"):
            continue
        original = source.parent / "README.md" if relative == Path("README.md") else source / relative
        if not original.is_file() or digest(original) != digest(snapshot):
            raise AssertionError(f"Bullet archive source differs from current source: {relative}")
        inventory.append({"source": str(original), "snapshot": str(snapshot), "sha256": digest(original)})
        count += 1
    return count


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--core-source", type=Path, required=True)
    parser.add_argument("--drives-source", type=Path, required=True)
    parser.add_argument("--bullet-archive", type=Path, required=True)
    parser.add_argument("--lock-seed", type=Path,
                        help="Optionally copy an existing lock into the disposable workspace; records a seeded, not fresh, resolution")
    parser.add_argument("--output", type=Path, required=True,
                        help="A new, disposable directory; existing directories are refused")
    args = parser.parse_args()
    output = args.output.resolve()
    if output.exists():
        raise ValueError(f"Refusing to reuse an existing rehearsal directory: {output}")
    for source in (args.core_source.resolve(), args.drives_source.resolve()):
        if output == source or source in output.parents:
            raise ValueError("The rehearsal must be outside the source repositories")
    output.mkdir(parents=True)
    inventory, changes, source_states = [], [], []
    roots = {"core": args.core_source.resolve(), "drives": args.drives_source.resolve()}
    for kind, paths in (("core", CORE), ("drives", DRIVES)):
        for relative in paths:
            if relative == "rsbullet/rsbullet-sys":
                continue
            source, destination = roots[kind] / relative, output / kind / relative
            copy_git_files(source, destination, inventory)
            source_states.append({"path": str(source),
                                  "head": run_git(source, "rev-parse", "HEAD").strip(),
                                  "status": run_git(source, "status", "--short")})
    # Root-relative package metadata is intentionally preserved, not rewritten.
    for source, destination in (
        (roots["core"] / "LICENSE", output / "core/LICENSE"),
        (roots["core"] / "README.md", output / "core/README.md"),
        (roots["drives"] / "rsbullet/README.md", output / "drives/rsbullet/README.md"),
    ):
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
        inventory.append({"source": str(source), "snapshot": str(destination), "sha256": digest(source)})
    bullet = extract_bullet(args.bullet_archive.resolve(), output / "drives/rsbullet/rsbullet-sys")
    sys_source = roots["drives"] / "rsbullet/rsbullet-sys"
    sys_destination = output / "drives/rsbullet/rsbullet-sys"
    bullet["source_files_byte_verified"] = verify_bullet_sources(sys_source, sys_destination, inventory)
    members = ["core/" + p for p in CORE] + ["drives/" + p for p in DRIVES]
    selected = {}
    for member in members:
        manifest = tomllib.loads((output / member / "Cargo.toml").read_text())
        selected[manifest["package"]["name"]] = output / member
    for member in members:
        changes.extend(normalize_selected_git(output / member / "Cargo.toml", selected))
        changes.extend(normalize_registry_batch_edge(output / member / "Cargo.toml", selected))
    (output / "Cargo.toml").write_text(
        '[workspace]\nresolver = "3"\nmembers = ' + json.dumps(members, indent=2) + "\n"
    )
    lock_seed = None
    if args.lock_seed is not None:
        seed_path = args.lock_seed.resolve()
        shutil.copy2(seed_path, output / "Cargo.lock")
        lock_seed = {"source": str(seed_path), "sha256": digest(seed_path),
                     "destination": str(output / "Cargo.lock"),
                     "scope": "Initial dependency-version seed; Cargo may resolve additional or changed package identities."}
    rust_files = set()
    for entry in inventory:
        path = Path(entry["snapshot"])
        if path.suffix == ".rs":
            if digest(path) != entry["sha256"]:
                raise AssertionError(f"Rust source bytes changed: {path}")
            rust_files.add(str(path))
    rust_count = len(rust_files)
    manifest_summary = []
    for name, directory in selected.items():
        data = tomllib.loads((directory / "Cargo.toml").read_text())
        manifest_summary.append({"name": name, "version": data["package"]["version"],
                                 "manifest": str(directory / "Cargo.toml")})
    summary = {"purpose": "Combined publication-batch rehearsal; not direct registry upload readiness",
               "workspace": str(output), "source_states": source_states,
               "manifests": manifest_summary, "manifest_changes": changes,
               "bullet_archive": bullet, "rust_files_byte_verified": rust_count,
               "input_files": inventory, "cargo_ran": False, "lock_seed": lock_seed,
               "limitations": ["Formal repository manifests and versions were not changed.",
                               "Internal Git identities and the roplat_rerun -> rerun_urdf registry edge are replaced by temporary path+version identities.",
                               "No registry source patch is used; only Cargo's own multi-package staging is intended.",
                               "Existing registry versions still cannot be republished.",
                               ("An initial lockfile is seeded; this does not verify fresh unlocked resolution."
                                if lock_seed else "No lockfile is seeded; the rehearsal records its own dependency resolution.")]}
    (output / "rehearsal-provenance.json").write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps({"workspace": str(output), "packages": len(selected),
                      "manifest_changes": len(changes), "rust_files_byte_verified": rust_count}, indent=2))


if __name__ == "__main__":
    main()
