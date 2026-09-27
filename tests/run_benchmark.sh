#!/usr/bin/env bash

set -eo pipefail

BENCH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/benchmark-scripts"
PROJECT_DIR="$(dirname "$(dirname "$BENCH_DIR")")"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

cargo build --release --quiet --manifest-path "$PROJECT_DIR/Cargo.toml"
rustc -O -o "$OUT/benchmark-rs" "$BENCH_DIR/benchmark.rs"

RUNS=3

run() {
    local name="$1"; shift
    local start end ms sum=0
    printf '%-10s' "$name"
    for ((i = 0; i < RUNS; i++)); do
        start=$(date +%s%N)
        "$@" >/dev/null
        end=$(date +%s%N)
        ms=$(( (end - start) / 1000000 ))
        sum=$(( sum + ms ))
        printf ' %6d ms' "$ms"
    done
    printf '   promedio: %6d ms\n' $(( sum / RUNS ))
}

echo "Resultados ($RUNS corridas por lenguaje):"
run python3 python3 "$BENCH_DIR/benchmark.py"
run rabi "$PROJECT_DIR/target/release/rabi" "$BENCH_DIR/benchmark.rabi"
run rust "$OUT/benchmark-rs"
