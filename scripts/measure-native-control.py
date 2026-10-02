#!/usr/bin/env python3
"""Run prebuilt control benchmarks serially and retain compact summaries.

No Cargo builds occur here. Franka binaries run only the exact ignored loopback
fixture; they do not connect to hardware. Keep the output directory outside Git.
"""
import argparse
import csv
import hashlib
import io
import json
import os
from pathlib import Path
import statistics
import subprocess
import time

NAMES = ("behavior_before", "behavior_after", "behavior_native", "franka_before", "franka_after")


def median(values):
    return statistics.median(values)


def write_csv(path, rows):
    with path.open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(rows[0]), lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def read_rows(directory, name, repeats):
    rows = []
    for repeat in range(1, repeats + 1):
        data = (directory / f"{name}-{repeat}.log").read_text()
        if name.startswith("franka"):
            if "1 passed; 0 failed" not in data:
                raise ValueError(f"{name}-{repeat}: exact benchmark did not pass")
            data = "\n".join(line for line in data.splitlines()
                             if line.startswith(("backend,", "std,", "blocking_async,", "native_async,")))
        current = list(csv.DictReader(io.StringIO(data)))
        if not current:
            raise ValueError(f"{name}-{repeat}: missing measurements")
        rows.extend(current)
    return rows


def summarize(directory, repeats):
    all_rows = {name: read_rows(directory, name, repeats) for name in NAMES}
    rows = []
    for key in sorted({(r["path"], int(r["command_bytes"])) for r in all_rows["behavior_before"]}):
        values = {}
        for version in ("before", "after"):
            # Both old binaries' CSV names compare an even older reconstructed
            # contract internally. For THIS change, compare their new_ columns.
            values[version] = [float(r["new_median_ns_per_cycle"])
                               for r in all_rows[f"behavior_{version}"]
                               if (r["path"], int(r["command_bytes"])) == key]
            assert len(values[version]) == repeats
        before, after = (median(values[v]) for v in ("before", "after"))
        rows.append(dict(path=key[0], command_bytes=key[1],
                         before_median_batch_ns_per_cycle=before,
                         after_median_batch_ns_per_cycle=after,
                         delta_percent=(after / before - 1) * 100,
                         before_min=min(values["before"]), before_max=max(values["before"]),
                         after_min=min(values["after"]), after_max=max(values["after"])))
    write_csv(directory / "behavior-regression.csv", rows)
    rows = []
    for size in sorted({int(r["command_bytes"]) for r in all_rows["behavior_native"]}):
        group = [r for r in all_rows["behavior_native"] if int(r["command_bytes"]) == size]
        assert len(group) == repeats
        blocking = median(float(r["blocking_median_batch_ns_per_cycle"]) for r in group)
        native = median(float(r["native_median_batch_ns_per_cycle"]) for r in group)
        rows.append(dict(command_bytes=size, blocking_median_batch_ns_per_cycle=blocking,
                         native_median_batch_ns_per_cycle=native,
                         delta_percent=(native / blocking - 1) * 100))
    write_csv(directory / "native-rhythm.csv", rows)
    rows = []
    for version in ("before", "after"):
        source = all_rows[f"franka_{version}"]
        for key in sorted({(r["backend"], int(r["cycles"])) for r in source}):
            group = [r for r in source if (r["backend"], int(r["cycles"])) == key]
            rows.append(dict(version=version, backend=key[0], cycles=key[1], sessions=len(group),
                             median_session_us=median(int(r["session_ns"]) for r in group) / 1000,
                             median_session_cycle_p50_us=median(int(r["cycle_p50_ns"]) for r in group) / 1000,
                             # A one-cycle session has no meaningful p99 estimate.
                             median_session_cycle_p99_us="" if key[1] == 1 else
                             median(int(r["cycle_p99_ns"]) for r in group) / 1000))
    write_csv(directory / "franka-loopback.csv", rows)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--cpu-cycles", type=int, default=1_000_000)
    parser.add_argument("--network-cycles", type=int, default=1000)
    parser.add_argument("--network-rounds", type=int, default=11)
    parser.add_argument("--suite", choices=("all", "behavior", "franka"), default="all")
    parser.add_argument("--summarize-only", action="store_true")
    for name in NAMES:
        parser.add_argument("--" + name.replace("_", "-"), type=Path)
    args = parser.parse_args()
    if min(args.repeats, args.cpu_cycles, args.network_rounds) < 1 or args.network_cycles < 2:
        parser.error("positive counts required; network-cycles must exceed one")
    args.output.mkdir(parents=True, exist_ok=True)
    if not args.summarize_only:
        selected = [n for n in NAMES if args.suite == "all" or n.startswith(args.suite)]
        binaries = {}
        for name in selected:
            path = getattr(args, name)
            if path is None or not path.is_file():
                parser.error(f"{name}: supply an existing prebuilt executable")
            binaries[name] = path.resolve()
        metadata_path = args.output / "run-metadata.json"
        metadata = json.loads(metadata_path.read_text()) if metadata_path.exists() else {"binaries": {}, "runs": []}
        metadata["runs"] = [r for r in metadata["runs"] if r["name"] not in selected]
        for name, path in binaries.items():
            metadata["binaries"][name] = dict(path=str(path), bytes=path.stat().st_size,
                                               sha256=hashlib.sha256(path.read_bytes()).hexdigest())
        env = dict(os.environ, CONTROL_BENCH_CYCLES=str(args.cpu_cycles),
                   FRANKA_PERF_ROUNDS=str(args.network_rounds), FRANKA_PERF_CYCLES=str(args.network_cycles))
        for repeat in range(1, args.repeats + 1):
            pair = ["before", "after"] if repeat % 2 else ["after", "before"]
            order = ["behavior_" + v for v in pair] + ["behavior_native"] + ["franka_" + v for v in pair]
            for name in (n for n in order if n in selected):
                command = [str(binaries[name])]
                if name.startswith("franka"):
                    test = "baseline_loopback_performance" if name.endswith("before") else "native_loopback_performance"
                    command += ["realtime::tests::native::performance::" + test, "--exact", "--ignored", "--nocapture", "--test-threads=1"]
                log = args.output / f"{name}-{repeat}.log"
                started = time.time()
                with log.open("w") as stream:
                    subprocess.run(command, stdout=stream, stderr=subprocess.STDOUT,
                                   env=env, check=True, timeout=180)
                measured = read_rows(args.output, name, repeat)
                expected = (args.network_rounds * 2 * (2 if name.endswith("before") else 3)) if name.startswith("franka") else (3 if name.endswith("native") else 9)
                assert len(measured) == repeat * expected, f"{name}: unexpected sample count"
                metadata["runs"].append(dict(name=name, repeat=repeat, started_unix=started, seconds=time.time()-started))
                metadata_path.write_text(json.dumps(metadata, indent=2) + "\n")
                print(f"{name} repeat {repeat} complete", flush=True)
    if all((args.output / f"{name}-{args.repeats}.log").exists() for name in NAMES):
        summarize(args.output, args.repeats)


if __name__ == "__main__":
    main()
