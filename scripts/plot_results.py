#!/usr/bin/env python3
import csv
import math
import statistics
import sys
from collections import defaultdict
from pathlib import Path

import matplotlib.pyplot as plt


def load_rows(path):
    with open(path, newline="") as handle:
        rows = list(csv.DictReader(handle))
    for row in rows:
        row["block_size"] = int(row["block_size"])
        row["threads"] = int(row["threads"])
        row["elapsed_ns"] = int(row["elapsed_ns"])
        row["throughput_tps"] = float(row["throughput_tps"])
        row["scheduling_ns"] = int(row["scheduling_ns"])
        row["execution_ns"] = int(row["execution_ns"])
        row["validation_ns"] = int(row["validation_ns"])
        row["commit_ns"] = int(row["commit_ns"])
        row["reexecutions"] = int(row["reexecutions"])
        row["correct"] = row["correct_vs_sequential"] in ("true", "True", "Some(true)", "1")
    return rows


def mean(values):
    return statistics.fmean(values) if values else 0.0


def grouped_mean(rows, key_fields, value_field):
    grouped = defaultdict(list)
    for row in rows:
        grouped[tuple(row[field] for field in key_fields)].append(row[value_field])
    return {key: mean(values) for key, values in grouped.items()}


def configure():
    plt.rcParams.update(
        {
            "figure.figsize": (8.0, 4.8),
            "axes.grid": True,
            "grid.alpha": 0.25,
            "axes.spines.top": False,
            "axes.spines.right": False,
            "font.size": 9,
        }
    )


def savefig(path):
    plt.tight_layout()
    plt.savefig(path)
    plt.close()


def plot_throughput(rows, outdir):
    data = grouped_mean(rows, ["executor", "threads"], "throughput_tps")
    executors = ["sequential", "static", "optimistic"]
    for executor in executors:
        xs = sorted({key[1] for key in data if key[0] == executor})
        ys = [data[(executor, x)] for x in xs]
        if xs:
            plt.plot(xs, ys, marker="o", label=executor)
    plt.xlabel("Threads")
    plt.ylabel("Mean throughput (tx/s)")
    plt.title("Throughput by executor and thread count")
    plt.legend()
    savefig(outdir / "throughput_by_threads.pdf")


def plot_speedup(rows, outdir):
    data = grouped_mean(rows, ["executor", "workload", "block_size", "threads"], "elapsed_ns")
    speedups = defaultdict(list)
    for (executor, workload, block_size, threads), elapsed in data.items():
        baseline = data.get(("sequential", workload, block_size, 1))
        if baseline and elapsed > 0:
            speedups[(executor, threads)].append(baseline / elapsed)
    for executor in ["sequential", "static", "optimistic"]:
        xs = sorted({key[1] for key in speedups if key[0] == executor})
        ys = [mean(speedups[(executor, x)]) for x in xs]
        if xs:
            plt.plot(xs, ys, marker="o", label=executor)
    plt.axhline(1.0, color="black", linewidth=0.8, linestyle="--")
    plt.xlabel("Threads")
    plt.ylabel("Mean speedup vs sequential 1-thread")
    plt.title("Speedup by executor")
    plt.legend()
    savefig(outdir / "speedup_by_threads.pdf")

    for executor in ["static", "optimistic"]:
        xs = sorted({key[1] for key in speedups if key[0] == executor})
        ys = [mean(speedups[(executor, x)]) / x for x in xs]
        if xs:
            plt.plot(xs, ys, marker="o", label=executor)
    plt.xlabel("Threads")
    plt.ylabel("Parallel efficiency")
    plt.title("Parallel efficiency by executor")
    plt.legend()
    savefig(outdir / "parallel_efficiency_by_threads.pdf")


def plot_reexecutions(rows, outdir):
    optimistic = [row for row in rows if row["executor"] == "optimistic"]
    data = grouped_mean(optimistic, ["workload"], "reexecutions")
    keys = sorted(data)
    names = [key[0] for key in keys]
    values = [data[key] for key in keys]
    plt.bar(names, values, color="#4c78a8")
    plt.xticks(rotation=25, ha="right")
    plt.ylabel("Mean re-executions per run")
    plt.title("Optimistic executor re-executions by workload")
    savefig(outdir / "reexecutions_by_workload.pdf")


def plot_overhead(rows, outdir):
    parallel = [row for row in rows if row["executor"] in ("static", "optimistic")]
    data = defaultdict(lambda: defaultdict(list))
    for row in parallel:
        label = row["executor"]
        data[label]["scheduling"].append(row["scheduling_ns"] / 1_000_000)
        data[label]["execution"].append(row["execution_ns"] / 1_000_000)
        data[label]["validation"].append(row["validation_ns"] / 1_000_000)
        data[label]["commit"].append(row["commit_ns"] / 1_000_000)

    labels = sorted(data)
    bottoms = [0.0 for _ in labels]
    colors = {
        "scheduling": "#4c78a8",
        "execution": "#f58518",
        "validation": "#54a24b",
        "commit": "#e45756",
    }
    for component in ["scheduling", "execution", "validation", "commit"]:
        values = [mean(data[label][component]) for label in labels]
        plt.bar(labels, values, bottom=bottoms, label=component, color=colors[component])
        bottoms = [left + right for left, right in zip(bottoms, values)]
    plt.ylabel("Mean time component (ms)")
    plt.title("Parallel executor overhead breakdown")
    plt.legend()
    savefig(outdir / "overhead_breakdown.pdf")


def write_summary(rows, summary_path):
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    fields = [
        "executor",
        "workload",
        "block_size",
        "threads",
        "mean_elapsed_ms",
        "mean_throughput_tps",
        "mean_reexecutions",
        "runs",
        "all_correct",
    ]
    grouped = defaultdict(list)
    for row in rows:
        grouped[(row["executor"], row["workload"], row["block_size"], row["threads"])].append(row)
    with open(summary_path, "w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        for key in sorted(grouped):
            values = grouped[key]
            writer.writerow(
                {
                    "executor": key[0],
                    "workload": key[1],
                    "block_size": key[2],
                    "threads": key[3],
                    "mean_elapsed_ms": f"{mean([v['elapsed_ns'] / 1_000_000 for v in values]):.4f}",
                    "mean_throughput_tps": f"{mean([v['throughput_tps'] for v in values]):.2f}",
                    "mean_reexecutions": f"{mean([v['reexecutions'] for v in values]):.2f}",
                    "runs": len(values),
                    "all_correct": all(v["correct"] for v in values),
                }
            )


def print_key_findings(rows):
    if not rows:
        return
    failures = [row for row in rows if not row["correct"]]
    print(f"rows={len(rows)} correctness_failures={len(failures)}")
    best = max(rows, key=lambda row: row["throughput_tps"])
    print(
        "best_throughput="
        f"{best['throughput_tps']:.2f} executor={best['executor']} "
        f"workload={best['workload']} threads={best['threads']} block={best['block_size']}"
    )


def main():
    if len(sys.argv) not in (3, 4):
        raise SystemExit(
            "usage: plot_results.py results/raw/results.csv results/figures [results/summary/summary.csv]"
        )
    input_path = Path(sys.argv[1])
    outdir = Path(sys.argv[2])
    summary_path = Path(sys.argv[3]) if len(sys.argv) == 4 else Path("results/summary/summary.csv")
    outdir.mkdir(parents=True, exist_ok=True)
    configure()
    rows = load_rows(input_path)
    plot_throughput(rows, outdir)
    plot_speedup(rows, outdir)
    plot_reexecutions(rows, outdir)
    plot_overhead(rows, outdir)
    write_summary(rows, summary_path)
    print_key_findings(rows)


if __name__ == "__main__":
    main()
