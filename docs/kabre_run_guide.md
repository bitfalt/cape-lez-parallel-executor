# Kabré Dual-Environment Run Guide

This project can be run on Kabré as a second experimental environment.

The local results answer: how does CAPE behave on personal/decentralization-friendly hardware?

The Kabré results answer: how does CAPE behave with a research/HPC allocation and more CPU threads?

Do not merge these into one claim. They should be presented as two environments with different hardware assumptions.

## Important Scope Note

CAPE uses Rayon shared-memory parallelism. It can use many CPU threads on one allocated node, but it does not distribute one block across multiple nodes. Using all of Kabré would require an MPI/distributed version, which is outside the current proposal.

## 1. Copy Project To Kabré

Use Open OnDemand file upload, Git, or `scp`.

Example:

```bash
scp -r /Users/bitfalt/Developer/proyecto-paralela/cape USER@KABRE_LOGIN_HOST:~/cape
```

Replace `USER@KABRE_LOGIN_HOST` with the login information provided by Kabré/CeNAT.

## 2. Check Rust

On Kabré:

```bash
cd ~/cape
cargo --version
rustc --version
```

If Rust is not available, try:

```bash
module avail rust
module load rust
```

or install Rust in the user account with `rustup` if permitted by cluster policy.

In the completed run for this project, Kabré's system Rust module was too old for the generated `Cargo.lock` (`rustc 1.15.0`). The working setup was a user-level rustup installation sourced by `scripts/slurm/cape_kabre.sbatch` from:

```bash
$HOME/.cargo/env
```

## 3. Submit Batch Job

Default job:

```bash
sbatch scripts/slurm/cape_kabre.sbatch
```

If the cluster requires a partition/account, pass it to `sbatch` without editing the script:

```bash
sbatch --partition=PARTITION --account=ACCOUNT scripts/slurm/cape_kabre.sbatch
```

For a smaller first run:

```bash
CAPE_THREADS=1,2,4,8,16 CAPE_BLOCK_SIZES=1000 CAPE_REPETITIONS=1 \
sbatch scripts/slurm/cape_kabre.sbatch
```

For a larger run if the allocation supports 64 CPU threads:

```bash
CAPE_THREADS=1,2,4,8,16,32,64 \
sbatch --cpus-per-task=64 scripts/slurm/cape_kabre.sbatch
```

Completed project run:

```bash
CAPE_THREADS=1,2,4,8,16,32,36 \
CAPE_BLOCK_SIZES=1000,5000 \
CAPE_REPETITIONS=3 \
CAPE_OUTPUT=results/raw/kabre_results.csv \
sbatch --job-name=cape-kabre --partition=dribe --cpus-per-task=36 --time=00:30:00 \
  scripts/slurm/cape_kabre.sbatch
```

This produced Slurm job `578276` on `dribe-00.cnca`, completed in 23 seconds with 36 allocated CPUs.

## 4. Outputs

The job writes:

```text
results/raw/kabre_results.csv
results/summary/kabre_summary.csv
results/figures/kabre/
results/raw/kabre-<jobid>.out
results/raw/kabre-<jobid>.err
```

If `matplotlib` is not installed on the compute node, the Slurm script skips remote plot generation and still leaves the raw CSV ready for local plotting.

## 5. Compare Local And Kabré

After copying `kabre_results.csv` back to the local machine:

```bash
python3 scripts/compare_environments.py \
  results/raw/results.csv \
  results/raw/kabre_results.csv \
  results/figures/comparison \
  results/summary/environment_comparison.csv
```

This produces:

```text
results/figures/comparison/environment_throughput_by_threads.pdf
results/summary/environment_comparison.csv
```

## Interpretation Language

Use this framing:

```text
The local M2 Pro experiment represents commodity personal hardware, which is closer to a decentralization-friendly validator/operator setting. The Kabré experiment represents institutional HPC hardware, useful for stress-testing scalability with more threads. The comparison is not meant to say that all blockchain nodes should use HPC hardware. It separates semantic correctness from hardware scalability and shows how much extra performance is available when the execution environment has more CPU parallelism.
```
