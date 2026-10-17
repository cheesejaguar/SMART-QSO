#!/usr/bin/env bash
# Quality ratchet for CI gates that cannot be zero yet (clang-tidy, MISRA).
#
# Usage: ratchet.sh <name> <count>
#
# Fails unless <count> equals the committed baseline in
# software/flight/quality/<name>.baseline: new findings block a merge, and a
# change that removes findings must lower the baseline with it, so the
# baseline is always the exact count. Counts depend on the tool version, so
# the tools are pinned by runner image (ubuntu-24.04) and checked.
set -euo pipefail

name=$1
count=${2//[[:space:]]/}   # BSD wc -l pads its count
file="$(cd "$(dirname "$0")/.." && pwd)/quality/${name}.baseline"

if ! [[ "$count" =~ ^[0-9]+$ ]]; then
    echo "::error::${name}: '${count}' is not a count of findings"
    exit 1
fi

if [ ! -f "$file" ]; then
    echo "::error::${name}: ${count} findings and no baseline file ${file}"
    exit 1
fi

base=$(tr -cd '0-9' < "$file")
# An empty report is what a failed tool run produces, so a drop to zero
# is only accepted once the baseline itself says zero
if [ "$count" -eq 0 ] && [ "$base" -gt 0 ]; then
    echo "::error::${name}: 0 findings against baseline ${base}. Check the tool ran; if the findings are really gone, set ${file} to 0."
    exit 1
fi
if [ "$count" -gt "$base" ]; then
    echo "::error::${name}: ${count} findings, baseline ${base}. Fix the new findings."
    exit 1
fi
if [ "$count" -lt "$base" ]; then
    echo "::error::${name}: ${count} findings, below baseline ${base}. Lower ${file} to ${count} in this change, so the fixed findings cannot come back."
    exit 1
fi
echo "${name}: ${count} findings (baseline ${base})"
