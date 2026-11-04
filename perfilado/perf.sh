#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

backend="${BACKEND:-vm}"
programa="${1:-tests/benchmark/benchmark.rabi}"
lineas="${LINEAS:-30}"
salida="target/perfilado"
binario="$salida/release/rabi"
nombre="$(basename "$programa" .rabi)-$backend"

CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release --quiet --target-dir "$salida"

evento=()
nucleo=()
if [ -r /sys/devices/cpu_core/cpus ]; then
    evento=(-e cpu_core/cycles/)
    nucleo=(taskset -c "$(cut -d- -f1 /sys/devices/cpu_core/cpus | cut -d, -f1)")
fi

"${nucleo[@]}" perf record "${evento[@]}" -F 20000 --quiet \
    --output "$salida/$nombre.data" \
    "$binario" --backend "$backend" "$programa" > /dev/null

echo "== funciones ($programa, $backend)"
perf report --input "$salida/$nombre.data" --no-children --stdio \
    --sort symbol --percent-limit 1 2> /dev/null | grep '%' | cut -c1-110

echo
echo "== líneas de src/"
python3 perfilado/lineas.py "$salida/$nombre.data" "$binario" "$lineas"
