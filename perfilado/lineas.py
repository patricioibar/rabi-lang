#!/usr/bin/env python3
"""Uso: perfilado/lineas.py <perf.data> <binario> [cantidad]"""

import collections
import os
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(REPO, "src") + os.sep


def symbol_addresses(binary):
    output = subprocess.run(
        ["nm", binary], capture_output=True, text=True, check=True
    ).stdout
    addresses = {}
    for line in output.splitlines():
        parts = line.split()
        if len(parts) == 3:
            addresses[parts[2]] = int(parts[0], 16)
    return addresses


def samples(perf_data, binary):
    output = subprocess.run(
        ["perf", "script", "-i", perf_data, "-F", "ip,sym,symoff,dso", "--no-demangle"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    binary = os.path.realpath(binary)
    for line in output.splitlines():
        parts = line.split()
        if len(parts) < 3:
            continue
        symoff, dso = parts[1], parts[2].strip("()")
        if os.path.realpath(dso) != binary or "+0x" not in symoff:
            yield None, f"[{os.path.basename(dso)}] {symoff.split('+')[0]}"
            continue
        symbol, offset = symoff.rsplit("+0x", 1)
        yield symbol, int(offset, 16)


def resolve(binary, addresses):
    query = "\n".join(hex(a) for a in addresses)
    output = subprocess.run(
        ["addr2line", "-f", "-i", "-C", "-a", "-e", binary],
        input=query,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.splitlines()

    chains, current = {}, None
    i = 0
    while i < len(output):
        line = output[i]
        if line.startswith("0x"):
            current = int(line, 16)
            chains[current] = []
            i += 1
            continue
        chains[current].append((line, output[i + 1]))
        i += 2
    return chains


def location(chain):
    for function, place in reversed(chain):
        path = place.split(" (")[0]
        if path.startswith(SRC):
            return os.path.relpath(path, REPO), function
    function, place = chain[0]
    return f"(fuera de src/) {function}", function


def source_line(location_text):
    path, _, number = location_text.rpartition(":")
    try:
        with open(os.path.join(REPO, path)) as f:
            return f.readlines()[int(number) - 1].strip()
    except (OSError, ValueError, IndexError):
        return ""


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    perf_data, binary = sys.argv[1], sys.argv[2]
    limit = int(sys.argv[3]) if len(sys.argv) > 3 else 30

    symbols = symbol_addresses(binary)
    per_address = collections.Counter()
    outside = collections.Counter()
    total = 0
    for symbol, value in samples(perf_data, binary):
        total += 1
        if symbol is None:
            outside[value] += 1
        elif symbol in symbols:
            per_address[symbols[symbol] + value] += 1
        else:
            outside[f"(sin símbolo) {symbol}"] += 1

    chains = resolve(binary, per_address)
    per_line = collections.Counter(outside)
    for address, count in per_address.items():
        where, _ = location(chains.get(address, [("??", "??:0")]))
        per_line[where] += count

    print(f"{total} muestras\n")
    for where, count in per_line.most_common(limit):
        text = source_line(where) if where.startswith("src/") else ""
        print(f"{100 * count / total:6.2f} %  {where:<38} {text[:70]}")


if __name__ == "__main__":
    main()
