# CAPE: Conflict-Aware Parallel Executor

This folder is the complete isolated deliverable for:

**Deterministic Parallel Transaction Execution for Logos Execution Zone: A Comparative and Experimental Study**

Repository URL: <https://github.com/bitfalt/cape-lez-parallel-executor>

The root files in `../` are treated as reference material only. All implementation, results, figures, and paper files live under this `cape/` folder.

## Contents

- `src/`: Rust prototype with state model, transaction semantics, workloads, executors, metrics, and CLI.
- `tests/`: integration tests for state hashing, conflict rules, and executor equivalence.
- `scripts/plot_results.py`: generates paper figures and a summary CSV from benchmark results.
- `results/raw/results.csv`: benchmark matrix output.
- `results/raw/kabre_results.csv`: Kabré Dribe Slurm output.
- `results/figures/`: generated PDF figures.
- `results/summary/summary.csv`: aggregate results used by the paper.
- `results/summary/environment_comparison.csv`: local/Kabré comparison summary.
- `paper/main.tex`: LNCS-style final report source.
- `paper/main.pdf`: compiled final report, if LaTeX tooling is available.

## Build and Test

```bash
cargo build --release
cargo test
```

## Correctness Check

```bash
cargo run --release -- check-correctness \
  --workload all \
  --block-size 1000 \
  --accounts 1000 \
  --threads 8 \
  --compute-cost 1000 \
  --seed 42
```

The command exits with a non-zero status if any parallel executor differs from the sequential reference in final state hash or per-transaction outcomes.

## Benchmark Matrix

The matrix used for the paper can be reproduced with:

```bash
rm -f results/raw/results.csv
cargo run --release -- benchmark-matrix \
  --workload all \
  --executor all \
  --block-sizes 1000,5000 \
  --threads 1,2,4,8 \
  --repetitions 3 \
  --accounts 10000 \
  --compute-cost 1000 \
  --seed 42 \
  --output results/raw/results.csv
```

Each row records executor, workload, seed, block size, thread count, timing, throughput, correctness, accepted/rejected transactions, conflicts, re-executions, and final state hash.

For convenience, the same local run is wrapped by:

```bash
bash scripts/run_local_matrix.sh results/raw/local_m2_pro_results.csv
```

## Kabré / HPC Run

Kabré can be used as a second environment with more CPU threads. See:

```text
docs/kabre_run_guide.md
scripts/slurm/cape_kabre.sbatch
scripts/compare_environments.py
```

The intended interpretation is not that decentralized blockchain nodes need HPC hardware. The local M2 Pro run represents commodity personal hardware, while Kabré represents institutional/high-throughput hardware for scalability stress testing.

Actual Kabré run included in this folder:

- Slurm job: `578276`
- Partition/node: `dribe` / `dribe-00.cnca`
- CPUs allocated: 36
- Thread matrix: `1,2,4,8,16,32,36`
- Output rows: 630 benchmark rows
- Correctness failures: 0
- Best Kabré run: optimistic no-conflict, block 5,000, 16 threads, 583,276.55 tx/s

Compare the local and Kabré results with:

```bash
python3 scripts/compare_environments.py \
  results/raw/results.csv \
  results/raw/kabre_results.csv \
  results/figures/comparison \
  results/summary/environment_comparison.csv
```

## Generate Figures

```bash
python3 scripts/plot_results.py results/raw/results.csv results/figures results/summary/summary.csv
```

Generated figures:

- `throughput_by_threads.pdf`
- `speedup_by_threads.pdf`
- `parallel_efficiency_by_threads.pdf`
- `reexecutions_by_workload.pdf`
- `overhead_breakdown.pdf`

## Paper

The paper source is in `paper/main.tex` and uses only generated CSV/figure data. Compile from this directory with:

```bash
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/main.tex
```

If `latexmk` is not installed, use the bundled LaTeX compile helper from Codex or any TeX Live/MacTeX installation.
