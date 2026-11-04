#!/usr/bin/env python3
"""Uso: perfilado/comparar.py <binario-antes> <binario-después> [programa.rabi] [corridas]"""

import os
import statistics
import subprocess
import sys
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def performance_core():
    try:
        with open("/sys/devices/cpu_core/cpus") as f:
            return f.read().strip().split("-")[0].split(",")[0]
    except OSError:
        return None


def run(binary, backend, program, core):
    command = [binary, "--backend", backend, program]
    if core is not None:
        command = ["taskset", "-c", core] + command
    start = time.perf_counter()
    subprocess.run(command, stdout=subprocess.DEVNULL, check=True)
    return (time.perf_counter() - start) * 1000


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    before, after = sys.argv[1], sys.argv[2]
    program = sys.argv[3] if len(sys.argv) > 3 else os.path.join(
        REPO, "tests/benchmark/benchmark.rabi"
    )
    runs = int(sys.argv[4]) if len(sys.argv) > 4 else 10
    backend = os.environ.get("BACKEND", "vm")
    core = performance_core()

    times = {before: [], after: []}
    for binary in (before, after):
        run(binary, backend, program, core)
    for _ in range(runs):
        for binary in (before, after):
            times[binary].append(run(binary, backend, program, core))

    print(f"{os.path.relpath(program, REPO)}, backend {backend}, {runs} corridas intercaladas"
          + (f", CPU {core}" if core is not None else ""))
    print(f"{'':<10} {'mínimo':>9} {'mediana':>9} {'promedio':>9}")
    for label, binary in (("antes", before), ("después", after)):
        t = times[binary]
        print(f"{label:<10} {min(t):>6.0f} ms {statistics.median(t):>6.0f} ms {statistics.mean(t):>6.0f} ms")
    change = statistics.median(times[after]) / statistics.median(times[before]) - 1
    print(f"cambio de la mediana: {100 * change:+.1f} %")


if __name__ == "__main__":
    main()
