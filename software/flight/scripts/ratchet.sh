#!/usr/bin/env bash
# Quality ratchet for CI gates that cannot be zero yet (clang-tidy, MISRA).
#
# Usage: ratchet.sh <name> <count>
#
# Fails when <count> exceeds the committed baseline in
# software/flight/quality/<name>.baseline, so new findings block a merge
# while existing ones are worked down. When the count drops, the step passes
# and asks for the baseline to be lowered. Counts depend on the tool version,
# so the tools are pinned by runner image (ubuntu-24.04) and checked.
set -euo pipefail

name=$1
count=$2
file="$(cd "$(dirname "$0")/.." && pwd)/quality/${name}.baseline"

if [ ! -f "$file" ]; then
    echo "::error::${name}: ${count} findings and no baseline file ${file}"
    exit 1
fi

base=$(tr -cd '0-9' < "$file")
if [ "$count" -gt "$base" ]; then
    echo "::error::${name}: ${count} findings, baseline ${base}. Fix the new findings."
    exit 1
fi
if [ "$count" -lt "$base" ]; then
    echo "::notice::${name}: ${count} findings, below baseline ${base}. Lower ${file} to ${count}."
fi
echo "${name}: ${count} findings (baseline ${base})"
