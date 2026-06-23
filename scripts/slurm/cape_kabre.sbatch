#!/usr/bin/env bash
#SBATCH --job-name=cape-kabre
#SBATCH --output=results/raw/kabre-%j.out
#SBATCH --error=results/raw/kabre-%j.err
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=64
#SBATCH --time=00:30:00

set -euo pipefail

echo "Job started at $(date)"
echo "Host: $(hostname)"
echo "SLURM_JOB_ID=${SLURM_JOB_ID:-none}"
echo "SLURM_CPUS_PER_TASK=${SLURM_CPUS_PER_TASK:-unset}"

if command -v module >/dev/null 2>&1; then
  module purge || true
fi

if [ -f "$HOME/.cargo/env" ]; then
  # Prefer rustup in the user account; Kabré's default rust module is too old for this crate.
  . "$HOME/.cargo/env"
elif command -v module >/dev/null 2>&1; then
  module load rust || true
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo was not found. Install Rust in your Kabré account or load the correct module." >&2
  exit 1
fi

THREAD_MAX="${SLURM_CPUS_PER_TASK:-64}"
THREADS="${CAPE_THREADS:-1,2,4,8,16,32,64}"
OUTPUT="${CAPE_OUTPUT:-results/raw/kabre_results.csv}"
BLOCK_SIZES="${CAPE_BLOCK_SIZES:-1000,5000}"
REPETITIONS="${CAPE_REPETITIONS:-3}"
ACCOUNTS="${CAPE_ACCOUNTS:-10000}"
COMPUTE_COST="${CAPE_COMPUTE_COST:-1000}"
SEED="${CAPE_SEED:-42}"

mkdir -p "$(dirname "$OUTPUT")"
rm -f "$OUTPUT"

echo "Rust: $(rustc --version)"
echo "Cargo: $(cargo --version)"
echo "Thread matrix: $THREADS"
echo "Output: $OUTPUT"

cargo build --release
cargo run --release -- benchmark-matrix \
  --workload all \
  --executor all \
  --block-sizes "$BLOCK_SIZES" \
  --threads "$THREADS" \
  --repetitions "$REPETITIONS" \
  --accounts "$ACCOUNTS" \
  --compute-cost "$COMPUTE_COST" \
  --seed "$SEED" \
  --output "$OUTPUT"

if python3 -c 'import matplotlib' >/dev/null 2>&1; then
  python3 scripts/plot_results.py "$OUTPUT" results/figures/kabre results/summary/kabre_summary.csv
else
  echo "Skipping remote plots because matplotlib is not available."
  echo "Copy $OUTPUT locally and run scripts/plot_results.py or scripts/compare_environments.py."
fi

echo "Job finished at $(date)"
