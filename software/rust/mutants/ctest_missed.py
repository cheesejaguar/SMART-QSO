#!/usr/bin/env python3
"""Run the mutants that the Rust unit tests missed against the C test suites.

cargo-mutants runs `cargo test` only. Much of the flight code is exercised by
the C suites (software/flight/tests) through the C ABI instead, so a mutant
the unit tests miss may still be caught there. For each missed mutant this
applies cargo-mutants' diff to the working tree, rebuilds the CMake build
directory (which rebuilds the Rust library), runs ctest, and reverts.

Usage (from the repository root, on a clean tree):
    cargo mutants -p sqso-protocol --in-place -o OUT     # in software/rust
    python3 software/rust/mutants/ctest_missed.py OUT/mutants.out BUILD_DIR

BUILD_DIR is a configured CMake build of software/flight. Writes
ctest_outcomes.json next to outcomes.json and prints the mutants that
neither the unit tests nor the C suites catch.
"""

import json
import subprocess
import sys
from pathlib import Path

RUST = Path("software/rust")


def run(argv, **kw):
    return subprocess.run(argv, capture_output=True, text=True, **kw)


def main() -> int:
    out_dir = Path(sys.argv[1])
    build = Path(sys.argv[2])
    outcomes = json.loads((out_dir / "outcomes.json").read_text())
    missed = [o for o in outcomes["outcomes"] if o["summary"] == "MissedMutant"]
    if run(["git", "diff", "--quiet", "--", str(RUST)]).returncode != 0:
        print("software/rust has local changes; run on a clean tree", file=sys.stderr)
        return 2

    results = []
    for i, o in enumerate(missed, 1):
        name = o["scenario"]["Mutant"]["name"]
        diff = (out_dir / o["diff_path"]).read_text()
        # cargo-mutants' diffs name the file only on the "---" line, which
        # patch(1) accepts and git apply does not
        applied = run(["patch", "-s", "-p0", "-d", str(RUST)], input=diff)
        if applied.returncode != 0:
            results.append({"mutant": name, "outcome": "not_applied", "detail": applied.stderr.strip()})
            continue
        try:
            built = run(["cmake", "--build", str(build), "-j8"])
            if built.returncode != 0:
                outcome = "unviable"
            else:
                tested = run(["ctest", "--test-dir", str(build), "-j8", "--timeout", "120"])
                outcome = "caught_by_c_suites" if tested.returncode != 0 else "missed"
        finally:
            run(["patch", "-s", "-R", "-p0", "-d", str(RUST)], input=diff)
        results.append({"mutant": name, "outcome": outcome})
        print(f"[{i}/{len(missed)}] {outcome:20} {name}", flush=True)

    # Rebuild the unmutated library so the build directory is left clean
    run(["cmake", "--build", str(build), "-j8"])
    (out_dir / "ctest_outcomes.json").write_text(json.dumps(results, indent=1) + "\n")
    survivors = [r["mutant"] for r in results if r["outcome"] == "missed"]
    print(f"\n{len(missed)} missed by the unit tests; {len(survivors)} also survive the C suites:")
    for s in survivors:
        print("  " + s)
    return 0


if __name__ == "__main__":
    sys.exit(main())
