#!/usr/bin/env python3
"""
Function length check (JPL Power of Ten rule 4: no function longer than about
60 lines).

Counts each C function's physical lines, from the line of its opening brace
to its closing brace, with comments and strings blanked so braces inside
them are ignored.

    function_length.py [--limit N] [--count] files...

Prints the functions over the limit, longest first; with --count prints only
how many there are (CI compares it with quality/function-length.baseline
through ratchet.sh).
"""

import argparse
import re
import sys


def functions(path):
    """Yield (lines, name, first_line) for each function in a C file."""
    src = open(path, encoding="utf-8", errors="replace").read()
    blank = re.sub(r'/\*.*?\*/|//[^\n]*|"(\\.|[^"\\])*"|\'(\\.|[^\'\\])*\'',
                   lambda m: re.sub(r"[^\n]", " ", m.group(0)), src, flags=re.S)
    depth = 0
    start = None
    name = "?"
    line = 1
    for idx, ch in enumerate(blank):
        if ch == "\n":
            line += 1
        elif ch == "{":
            if depth == 0:
                before = blank[max(0, idx - 300):idx].rstrip()
                if before.endswith(")"):
                    match = re.search(r"(\w+)\s*\([^()]*(\([^()]*\)[^()]*)*\)\s*$", before)
                    name = match.group(1) if match else "?"
                    start = line
                else:
                    start = None
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0 and start is not None:
                yield line - start + 1, name, start
                start = None


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[1])
    parser.add_argument("--limit", type=int, default=60)
    parser.add_argument("--count", action="store_true")
    parser.add_argument("files", nargs="+")
    args = parser.parse_args()

    long_ones = []
    for path in args.files:
        for lines, name, first in functions(path):
            if lines > args.limit:
                long_ones.append((lines, path, first, name))
    long_ones.sort(reverse=True)

    if args.count:
        print(len(long_ones))
    else:
        for lines, path, first, name in long_ones:
            print(f"{path}:{first}: {name} is {lines} lines (limit {args.limit})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
