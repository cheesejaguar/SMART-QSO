#!/usr/bin/env bash
# Line coverage of the flight software (software/rust), from the C test
# suites and the Rust unit tests together.
#
# Usage: rust_coverage.sh <cmake build dir> <minimum line %> [report dir]
#
# The build dir must be configured with -DENABLE_COVERAGE=ON (the Rust
# library is then instrumented) and its tests run with
#   LLVM_PROFILE_FILE=<build dir>/profraw/%p-%m.profraw ctest ...
# This script runs the Rust unit tests instrumented as well, merges both
# profiles, prints the per-file report and fails below the minimum.
#
# Counted: every source file of the flight crates (sqso-protocol, sqso-fsw,
# sqso-ffi), including lines no test reaches. Not counted: the Python
# binding (sqso-py, tested by the payload and ground suites), the fuzz
# targets, the firmware (target only) and third-party crates.
set -euo pipefail

build=$(cd "$1" && pwd)
min=$2
report=${3:-$build/rust-coverage}
rust_dir="$(cd "$(dirname "$0")/../../rust" && pwd)"

host=$(cd "$rust_dir" && rustc --print host-tuple)
tools="$(cd "$rust_dir" && rustc --print sysroot)/lib/rustlib/$host/bin"
for tool in llvm-profdata llvm-cov; do
    [ -x "$tools/$tool" ] || { echo "::error::$tool missing: rustup component add llvm-tools"; exit 1; }
done

shopt -s nullglob
ctest_profiles=("$build"/profraw/*.profraw)
[ "${#ctest_profiles[@]}" -gt 0 ] || { echo "::error::no profiles in $build/profraw (run ctest with LLVM_PROFILE_FILE)"; exit 1; }

# The Rust unit tests, instrumented, in their own target directory
unit_dir="$build/cargo-unit-cov"
rm -rf "$build/profraw-unit"
mkdir -p "$build/profraw-unit"
mapfile -t unit_tests < <(
    cd "$rust_dir" &&
    RUSTFLAGS="-C instrument-coverage" cargo test --workspace --exclude sqso-py --locked --no-run \
        --target-dir "$unit_dir" --message-format=json 2>/dev/null |
    python3 -c 'import json, sys
for line in sys.stdin:
    m = json.loads(line)
    if m.get("reason") == "compiler-artifact" and m.get("profile", {}).get("test") and m.get("executable"):
        print(m["executable"])'
)
[ "${#unit_tests[@]}" -gt 0 ] || { echo "::error::no Rust unit-test binaries built"; exit 1; }
for t in "${unit_tests[@]}"; do
    (cd "$rust_dir" && LLVM_PROFILE_FILE="$build/profraw-unit/%p-%m.profraw" "$t" -q >/dev/null)
done

"$tools/llvm-profdata" merge -sparse "${ctest_profiles[@]}" "$build"/profraw-unit/*.profraw -o "$build/rust.profdata"

# One C test binary carries the whole library (it is linked as one LTO
# object); the unit-test binaries carry the crates' own test builds
c_object=$(find "$build/tests" -maxdepth 1 -type f -name 'test_*' -perm -u+x | sort | head -1)
objects=(-object "$c_object")
for t in "${unit_tests[@]}"; do objects+=(-object "$t"); done

ignore='/\.cargo/registry/|/rustc/|/library/|sqso-py/|/fuzz/|/firmware/|/tests?/|/target/'
mkdir -p "$report"
"$tools/llvm-cov" report "${objects[@]}" -instr-profile="$build/rust.profdata" \
    -ignore-filename-regex="$ignore" | tee "$report/report.txt"
"$tools/llvm-cov" export "${objects[@]}" -instr-profile="$build/rust.profdata" \
    -ignore-filename-regex="$ignore" -format=lcov > "$report/lcov.info"

total=$(awk '/^TOTAL/ { print $(NF-3) }' "$report/report.txt" | tr -d '%')
echo "Rust line coverage: ${total}% (minimum ${min}%)"
awk -v t="$total" -v m="$min" 'BEGIN { exit !(t + 0 >= m + 0) }' || {
    echo "::error::Rust line coverage ${total}% is below ${min}%"
    exit 1
}
