#!/usr/bin/env python3
"""Check already-generated metadata/lock against the preparation seed; no Cargo."""
import argparse
import hashlib
import json
from pathlib import Path
import tomllib


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    seed = tomllib.loads((output / "integration-lock-seed.lock").read_text())
    metadata = json.loads((output / "metadata.json").read_text())
    lock_bytes = (output / "workspace/Cargo.lock").read_bytes()
    lock = tomllib.loads(lock_bytes.decode())
    key = lambda p: (p["name"], p["version"], p.get("source", ""))
    known = {key(p): p.get("checksum") for p in seed["package"]}
    registry = []
    for package in metadata["packages"]:
        if package.get("source", "") and package["source"].startswith("registry+"):
            identity = key(package)
            if identity not in known:
                raise SystemExit("New registry version outside seed: " + repr(identity))
            registry.append({"name": identity[0], "version": identity[1], "source": identity[2]})
    for package in lock["package"]:
        if package.get("source", "").startswith("registry+"):
            if key(package) not in known or known[key(package)] != package.get("checksum"):
                raise SystemExit("Lock package/checksum differs from seed: " + repr(key(package)))
    for name, expected in [("tokio", "1.48.0"), ("nalgebra", "0.34.1")]:
        versions = {p["version"] for p in registry if p["name"] == name}
        if versions != {expected}:
            raise SystemExit(name + " must remain " + expected + ", got " + repr(versions))
    report = {
        "registry_versions_from_seed_only": True,
        "registry_packages": sorted(registry, key=lambda p: (p["name"], p["version"])),
        "baseline_lock_sha256": hashlib.sha256(lock_bytes).hexdigest(),
        "metadata_sha256": hashlib.sha256((output / "metadata.json").read_bytes()).hexdigest(),
        "feature_note": "Cargo metadata features are not a per-target compiled-feature audit.",
    }
    (output / "dependency-audit.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"registry_version_audit": "PASS", "registry_packages": len(registry)}))


if __name__ == "__main__":
    main()
