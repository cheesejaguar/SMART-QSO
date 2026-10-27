#!/bin/bash
# Runs the CDR power/thermal scenario matrix (run_matrix.txt: name scenario key=value ...) with the
# mission simulation, four at a time, into OUT (default ./sil-cdr), then the power closure.
# Usage: run_matrix.sh [OUT]   (from anywhere; builds sqso_sil first). About 1 h on 4 cores.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../../.." && pwd)"
out="${1:-$PWD/sil-cdr}"
mkdir -p "$out"
(cd "$repo/software/rust" && cargo build --profile sil -p sqso-sil)
sil="$repo/software/rust/target/sil/sqso_sil"
run1() {
  local name=$1 scen=$2; shift 2
  local args=(); for kv in "$@"; do args+=(--set "$kv"); done
  "$sil" --scenario "$scen" --out "$out/$name" "${args[@]}" > "$out/$name.log" 2>&1 || true
}
export -f run1; export sil out
grep -v '^\s*#' "$here/run_matrix.txt" | xargs -P 4 -L 1 bash -c 'run1 "$@"' _
runs=()
while read -r name rest; do [ -n "$name" ] && runs+=("$out/$name=$name"); done < <(grep -v '^\s*#' "$here/run_matrix.txt")
python3 "$here/power_closure.py" --runs "${runs[@]}" --json "$out/closure.json" > "$out/closure.md"
echo "closure: $out/closure.md"
