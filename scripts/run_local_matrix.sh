#!/usr/bin/env bash
set -euo pipefail

OUTPUT="${1:-results/raw/local_m2_pro_results.csv}"

mkdir -p "$(dirname "$OUTPUT")"
rm -f "$OUTPUT"

cargo build --release
cargo run --release -- benchmark-matrix \
  --workload all \
  --executor all \
  --block-sizes 1000,5000 \
  --threads 1,2,4,8 \
  --repetitions 3 \
  --accounts 10000 \
  --compute-cost 1000 \
  --seed 42 \
  --output "$OUTPUT"

echo "Wrote $OUTPUT"
