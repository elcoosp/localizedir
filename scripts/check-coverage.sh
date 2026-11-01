#!/usr/bin/env bash
# Enforces 100% line coverage for the workspace.
#
# Why not `cargo llvm-cov --fail-under-lines 100`?
# A proc-macro crate is compiled into *several* units: the dylib that runs
# inside rustc during macro expansion (uninstrumented - rustc's profraw is
# not collected) plus copies inlined into every downstream test binary.
# llvm-cov's builtin line metric counts every duplicate instance, so lines
# that ARE executed report as missed through their phantom twins. The lcov
# metric below only counts instrumented lines with their real execution
# counts - the same view Codecov consumes - and is the source of truth here.
set -euo pipefail

cd "$(dirname "$0")/.."

cargo llvm-cov --workspace --all-targets --all-features --lcov --output-path target/lcov.info

python3 - <<'PY'
import sys

total = 0
uncovered = []
current = None
for line in open("target/lcov.info", encoding="utf-8"):
    if line.startswith("SF:"):
        current = line.strip()[3:]
    elif line.startswith("DA:") and current:
        lineno, hits = line.strip()[3:].split(",")[:2]
        total += 1
        if int(hits) == 0:
            uncovered.append(f"{current}:{lineno}")

if uncovered:
    print("uncovered lines:", file=sys.stderr)
    print("\n".join(uncovered), file=sys.stderr)

pct = 100.0 if total == 0 else 100.0 * (total - len(uncovered)) / total
print(f"line coverage: {total - len(uncovered)}/{total} ({pct:.2f}%)")
sys.exit(0 if pct >= 100.0 else 1)
PY
