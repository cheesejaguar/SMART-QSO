#!/usr/bin/env bash
# Line coverage of the flight software (software/rust), from the C test
# suites and the Rust unit tests together.
#
# Usage: rust_coverage.sh <cmake build dir> <minimum line %> [report dir]
#
# Branch coverage: with RUST_COVERAGE_MIN_BRANCH set, the branch column
# must reach it too. Branches are only counted in a build made with
# -Zcoverage-options=branch (or =condition, which also counts each condition)
# on a nightly toolchain: configure with RUSTUP_TOOLCHAIN=<nightly> and
# -DSQSO_RUST_COVERAGE_FLAGS=-Zcoverage-options=condition, and run this script
# with the same RUSTUP_TOOLCHAIN and RUST_COVERAGE_FLAGS (the unit tests' flag).
# scripts/safety_coverage.py then gates the safety-critical files on its lcov.
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
    RUSTFLAGS="-C instrument-coverage ${RUST_COVERAGE_FLAGS:-}" cargo test --workspace --exclude sqso-py --locked --no-run \
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
# (test_rust_contract: every C suite but the integration drivers links it the
# same way; named so a driver that links no Rust can never be picked)
c_object="$build/tests/test_rust_contract"
[ -x "$c_object" ] || { echo "::error::$c_object missing (configure with -DENABLE_COVERAGE=ON and build)"; exit 1; }
objects=(-object "$c_object")
# The host simulation (the executive and sqso-sim), run by ctest Simulation_Run
[ -x "$build/smart_qso_flight" ] && objects+=(-object "$build/smart_qso_flight")
for t in "${unit_tests[@]}"; do objects+=(-object "$t"); done

ignore='/\.cargo/registry/|/rustc/|/library/|sqso-py/|/fuzz/|/firmware/|/tests?/|/target/|_tests\.rs$'
mkdir -p "$report"
"$tools/llvm-cov" report "${objects[@]}" -instr-profile="$build/rust.profdata" \
    -ignore-filename-regex="$ignore" | tee "$report/report.txt"
"$tools/llvm-cov" export "${objects[@]}" -instr-profile="$build/rust.profdata" \
    -ignore-filename-regex="$ignore" -format=lcov > "$report/lcov.info"
# The unit-test binaries alone: llvm-cov keeps one record per function name,
# and for a #[no_mangle] function the C test object's record (another build,
# another hash) hides the unit tests' counts. safety_coverage.py takes the
# union of the two exports.
unit_objects=()
for t in "${unit_tests[@]}"; do unit_objects+=(-object "$t"); done
"$tools/llvm-cov" export "${unit_objects[@]}" -instr-profile="$build/rust.profdata" \
    -ignore-filename-regex="$ignore" -format=lcov > "$report/lcov-unit.info"

total=$(awk '/^TOTAL/ { print $(NF-3) }' "$report/report.txt" | tr -d '%')
echo "Rust line coverage: ${total}% (minimum ${min}%)"
awk -v t="$total" -v m="$min" 'BEGIN { exit !(t + 0 >= m + 0) }' || {
    echo "::error::Rust line coverage ${total}% is below ${min}%"
    exit 1
}
if [ -n "${RUST_COVERAGE_MIN_BRANCH:-}" ]; then
    branch=$(awk '/^TOTAL/ { print $NF }' "$report/report.txt" | tr -d '%')
    [ "$branch" != "-" ] || { echo "::error::no branches counted (build with -Zcoverage-options=branch)"; exit 1; }
    echo "Rust branch coverage: ${branch}% (minimum ${RUST_COVERAGE_MIN_BRANCH}%)"
    awk -v t="$branch" -v m="$RUST_COVERAGE_MIN_BRANCH" 'BEGIN { exit !(t + 0 >= m + 0) }' || {
        echo "::error::Rust branch coverage ${branch}% is below ${RUST_COVERAGE_MIN_BRANCH}%"
        exit 1
    }
fi
