#!/usr/bin/env python3
"""
Check that repository documentation only references files that exist.

Two reference styles are checked in every tracked Markdown file:
  * relative Markdown links: [text](path/to/file.md#anchor)
  * backtick-quoted repository paths: `docs/...`, `software/...`,
    `hardware/...`, `mission/...`, `scripts/...`

A document set that funders and regulators read must not point at artifacts
that do not exist. Exit status is 1 if any reference is broken.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

MD_LINK = re.compile(r"\[[^\]]*\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")
REPO_PATH = re.compile(
    r"`((?:docs|software|hardware|mission|scripts)/[A-Za-z0-9_./-]+"
    r"\.(?:md|c|h|py|yaml|yml|txt|toml|ipynb|sh|cff))`"
)
FENCE = re.compile(r"```.*?```", re.S)
EXTERNAL = re.compile(r"^(?:[a-z][a-z0-9+.-]*:|#)", re.I)


def tracked_markdown() -> list[Path]:
    out = subprocess.run(
        ["git", "ls-files", "*.md"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return [REPO_ROOT / line for line in out.splitlines() if line]


def broken_references(md: Path) -> list[str]:
    text = md.read_text(encoding="utf-8", errors="replace")
    problems = []
    for match in MD_LINK.finditer(FENCE.sub("", text)):
        target = match.group(1)
        if EXTERNAL.match(target):
            continue
        path = target.split("#", 1)[0]
        if path and not (md.parent / path).exists():
            problems.append(f"link -> {target}")
    for match in REPO_PATH.finditer(text):
        path = match.group(1)
        if not (REPO_ROOT / path).exists() and not (md.parent / path).exists():
            problems.append(f"path -> {path}")
    return problems


def main() -> int:
    failures = 0
    files = tracked_markdown()
    for md in files:
        for problem in broken_references(md):
            print(f"{md.relative_to(REPO_ROOT)}: broken {problem}")
            failures += 1
    print(f"Checked {len(files)} Markdown files: {failures} broken reference(s)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
