#!/usr/bin/env python3
import csv
import statistics
import sys
from collections import defaultdict
from pathlib import Path

import matplotlib.pyplot as plt


def load(path, environment):
    with open(path, newline="") as handle:
        rows = list(csv.DictReader(handle))
    for row in rows:
        row["environment"] = environment
        row["threads"] = int(row["threads"])
        row["block_size"] = int(row["block_size"])
        row["elapsed_ns"] = int(row["elapsed_ns"])
        row["throughput_tps"] = float(row["throughput_tps"])
        row["correct"] = row["correct_vs_sequential"].lower() == "true"
    return rows


def mean(values):
    return statistics.fmean(values) if values else 0.0


def grouped(rows, fields, value):
    data = defaultdict(list)
    for row in rows:
        data[tuple(row[field] for field in fields)].append(row[value])
    return {key: mean(values) for key, values in data.items()}


def plot_environment_throughput(rows, outdir):
    data = grouped(rows, ["environment", "executor", "threads"], "throughput_tps")
    for env in sorted({row["environment"] for row in rows}):
        for executor in ["sequential", "static", "optimistic"]:
            xs = sorted(key[2] for key in data if key[0] == env and key[1] == executor)
            ys = [data[(env, executor, x)] for x in xs]
            if xs:
                plt.plot(xs, ys, marker="o", label=f"{env} {executor}")
    plt.xlabel("Threads")
    plt.ylabel("Mean throughput (tx/s)")
    plt.title("Throughput comparison by environment")
    plt.legend(fontsize=7)
    plt.grid(alpha=0.25)
    plt.tight_layout()
    plt.savefig(outdir / "environment_throughput_by_threads.pdf")
    plt.close()


def write_summary(rows, path):
    fields = [
        "environment",
        "executor",
        "workload",
        "block_size",
        "threads",
        "mean_throughput_tps",
        "mean_elapsed_ms",
        "runs",
        "all_correct",
    ]
    data = defaultdict(list)
    for row in rows:
        key = (
            row["environment"],
            row["executor"],
            row["workload"],
            row["block_size"],
            row["threads"],
        )
        data[key].append(row)
    with open(path, "w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        for key in sorted(data):
            values = data[key]
            writer.writerow(
                {
                    "environment": key[0],
                    "executor": key[1],
                    "workload": key[2],
                    "block_size": key[3],
                    "threads": key[4],
                    "mean_throughput_tps": f"{mean([v['throughput_tps'] for v in values]):.2f}",
                    "mean_elapsed_ms": f"{mean([v['elapsed_ns'] / 1_000_000 for v in values]):.4f}",
                    "runs": len(values),
                    "all_correct": all(v["correct"] for v in values),
                }
            )


def print_takeaways(rows):
    failures = sum(not row["correct"] for row in rows)
    print(f"rows={len(rows)} correctness_failures={failures}")
    for env in sorted({row["environment"] for row in rows}):
        env_rows = [row for row in rows if row["environment"] == env]
        best = max(env_rows, key=lambda row: row["throughput_tps"])
        print(
            f"best_{env}={best['throughput_tps']:.2f} "
            f"executor={best['executor']} workload={best['workload']} "
            f"threads={best['threads']} block={best['block_size']}"
        )


def main():
    if len(sys.argv) != 5:
        raise SystemExit(
            "usage: compare_environments.py LOCAL_CSV KABRE_CSV OUT_DIR SUMMARY_CSV"
        )
    local_csv, kabre_csv, out_dir, summary_csv = sys.argv[1:]
    outdir = Path(out_dir)
    outdir.mkdir(parents=True, exist_ok=True)
    rows = load(local_csv, "local_m2_pro") + load(kabre_csv, "kabre")
    plot_environment_throughput(rows, outdir)
    write_summary(rows, Path(summary_csv))
    print_takeaways(rows)


if __name__ == "__main__":
    main()
