#!/usr/bin/env python3
"""Record cold-process startup evidence for the ordinary Hara JVM launcher.

This benchmark is evidence, not a threshold. Every sample starts a new Java
process through ``core/hara`` and verifies the displayed result before the
measurement is accepted.
"""

from __future__ import annotations

import argparse
import csv
import json
import math
import os
import platform
import statistics
import subprocess
import sys
import time
from pathlib import Path
from typing import Sequence


def run_text(command: Sequence[str], cwd: Path) -> str:
    completed = subprocess.run(
        command,
        cwd=cwd,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    return completed.stdout.strip()


def percentile_nearest_rank(values: Sequence[int], percentile: float) -> int:
    ordered = sorted(values)
    rank = max(1, math.ceil(percentile * len(ordered)))
    return ordered[rank - 1]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--launcher", default="core/hara")
    parser.add_argument("--expression", default="(+ 19 23)")
    parser.add_argument("--expected", default="42")
    parser.add_argument("--label", default="builtin-only")
    parser.add_argument("--samples", type=int, default=20)
    parser.add_argument(
        "--output",
        default="core/target/truffle-cli-startup.csv",
        help="CSV destination; a .summary.json sidecar is also written",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.samples < 1:
        raise SystemExit("--samples must be at least 1")

    repository = Path(__file__).resolve().parents[2]
    launcher = (repository / args.launcher).resolve()
    if not launcher.is_file():
        raise SystemExit(f"launcher does not exist: {launcher}")

    output = (repository / args.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    summary_output = output.with_suffix(output.suffix + ".summary.json")

    commit = run_text(["git", "rev-parse", "HEAD"], repository)
    java = os.environ.get("HARA_JAVA") or "java"
    java_version = run_text([java, "-version"], repository).splitlines()[0]
    system = platform.uname()

    command = [str(launcher), "eval", args.expression]
    samples: list[dict[str, object]] = []
    durations: list[int] = []

    for sample in range(1, args.samples + 1):
        started = time.perf_counter_ns()
        completed = subprocess.run(
            command,
            cwd=repository,
            env=os.environ.copy(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        elapsed = time.perf_counter_ns() - started
        displayed = completed.stdout.strip()
        if completed.returncode != 0:
            raise SystemExit(
                f"sample {sample} failed with exit {completed.returncode}:\n"
                f"stdout:\n{completed.stdout}\nstderr:\n{completed.stderr}"
            )
        if displayed != args.expected:
            raise SystemExit(
                f"sample {sample} displayed {displayed!r}; expected {args.expected!r}"
            )
        durations.append(elapsed)
        samples.append(
            {
                "label": args.label,
                "sample": sample,
                "wall_ns": elapsed,
                "wall_ms": f"{elapsed / 1_000_000:.6f}",
                "display": displayed,
                "commit": commit,
                "java_version": java_version,
                "system": system.system,
                "release": system.release,
                "machine": system.machine,
                "launcher": str(launcher.relative_to(repository)),
                "expression": args.expression,
            }
        )

    fieldnames = list(samples[0].keys())
    with output.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(samples)

    summary = {
        "schema": "hara.truffle.cli-startup.v1",
        "label": args.label,
        "samples": len(durations),
        "expression": args.expression,
        "expected": args.expected,
        "launcher": str(launcher.relative_to(repository)),
        "commit": commit,
        "java_version": java_version,
        "system": system.system,
        "release": system.release,
        "machine": system.machine,
        "minimum_ns": min(durations),
        "median_ns": int(statistics.median(durations)),
        "mean_ns": int(statistics.fmean(durations)),
        "p95_ns": percentile_nearest_rank(durations, 0.95),
        "maximum_ns": max(durations),
    }
    summary["minimum_ms"] = summary["minimum_ns"] / 1_000_000
    summary["median_ms"] = summary["median_ns"] / 1_000_000
    summary["mean_ms"] = summary["mean_ns"] / 1_000_000
    summary["p95_ms"] = summary["p95_ns"] / 1_000_000
    summary["maximum_ms"] = summary["maximum_ns"] / 1_000_000

    summary_output.write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(json.dumps(summary, indent=2, sort_keys=True))
    print(f"csv={output.relative_to(repository)}")
    print(f"summary={summary_output.relative_to(repository)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as error:
        print(error.stdout or "", file=sys.stderr)
        raise
