#!/usr/bin/env python3
"""Compare dependency feature sets of two already-built Franka test executables.

Reads Cargo implementation-detail fingerprints; does not invoke Cargo. Supports
traditional .fingerprint and nightly build-dir-new-layout directory layouts.
Pass the exact test-lib-franka_rust.json belonging to each selected executable.
"""
import argparse
from collections import Counter
import json
from pathlib import Path


def graph(target, unit):
    index = {}
    paths = list(target.glob("release/.fingerprint/*/*.json"))
    paths += list(target.glob("release/build/*/*/fingerprint/*.json"))
    for path in paths:
        fingerprint = path.with_suffix("")
        if fingerprint.is_file():
            try:
                value = int.from_bytes(bytes.fromhex(fingerprint.read_text().strip()), "little")
                index[value] = (path, json.loads(path.read_text()))
            except (ValueError, UnicodeDecodeError):
                continue
    visited = set()
    records = []
    missing = []

    def visit(path, data):
        if path in visited:
            return
        visited.add(path)
        raw = data.get("features", "")
        features = json.loads(raw) if raw else []
        # Build-script execution units do not compile code independently.
        if path.stem.startswith(("lib-", "test-lib-", "build-script-")):
            records.append({"target": path.stem, "features": sorted(features)})
        for dep in data.get("deps", []):
            if dep[-1] not in index:
                missing.append({"from": path.stem, "dependency": dep[1]})
            else:
                visit(*index[dep[-1]])

    visit(unit, json.loads(unit.read_text()))
    if missing:
        raise SystemExit("Incomplete fingerprint graph: " + json.dumps(missing))
    return sorted(records, key=lambda r: (r["target"], r["features"]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for prefix in ["baseline", "current"]:
        parser.add_argument("--" + prefix + "-target", type=Path, required=True)
        parser.add_argument("--" + prefix + "-unit", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    old = graph(args.baseline_target, args.baseline_unit)
    new = graph(args.current_target, args.current_unit)
    a = Counter((r["target"], tuple(r["features"])) for r in old)
    b = Counter((r["target"], tuple(r["features"])) for r in new)
    report = {
        "baseline": old, "current": new, "feature_sets_equal": a == b,
        "baseline_only": [{"target": k[0], "features": k[1], "count": n}
                          for k, n in sorted((a - b).items())],
        "current_only": [{"target": k[0], "features": k[1], "count": n}
                         for k, n in sorted((b - a).items())],
        "scope": "Compiled dependency feature sets, not equal source or binary hashes. Registry versions audited separately.",
    }
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"feature_sets_equal": a == b, "baseline_units": len(old), "current_units": len(new)}))
    if a != b:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
