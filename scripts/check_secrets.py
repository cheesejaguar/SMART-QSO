#!/usr/bin/env python3
"""
Repository secret scan.

Fails if any tracked file is a key or certificate file, contains a PEM
private-key block, or contains a 256-bit hex string (the form of a
telecommand key, docs/interfaces/TELECOMMAND_AUTHENTICATION.md) that is not
in the allowlist. Allowlisted strings are public test vectors and the
documented TEST-ONLY key; each entry carries its reason.

Usage: python3 scripts/check_secrets.py [paths...]   (default: git ls-files)
"""

import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Set

REPO = Path(__file__).resolve().parents[1]
ALLOWLIST = REPO / "scripts" / "secret_scan_allowlist.txt"

KEY_FILE = re.compile(r"(\.(key|pem|p12|pfx|jks|keystore)|^id_(rsa|dsa|ecdsa|ed25519)(\.pub)?)$", re.IGNORECASE)
PRIVATE_KEY = re.compile(r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----")
HEX_256 = re.compile(r"(?<![0-9A-Fa-f])[0-9A-Fa-f]{64,}(?![0-9A-Fa-f])")
#: Cargo lockfiles pin each crate by the SHA-256 of its published archive
#: (public); only those checksum lines are exempt, not the whole file
LOCK_CHECKSUM = re.compile(r'^checksum = "[0-9a-f]{64}"$')


@dataclass(frozen=True)
class Finding:
    path: str
    line: int
    reason: str


def load_allowlist(path: Path) -> Set[str]:
    """Read allowlisted hex strings (one per line, '#' starts a comment)."""
    entries = set()
    for raw in path.read_text().splitlines():
        value = raw.split("#", 1)[0].strip()
        if not value:
            continue
        if not re.fullmatch(r"[0-9A-Fa-f]{64,}", value):
            raise ValueError(f"malformed allowlist entry: {raw!r}")
        entries.add(value.lower())
    return entries


def scan_paths(paths: Iterable[Path], allowlist: Set[str]) -> List[Finding]:
    """Return every finding in the given files."""
    findings = []
    for path in paths:
        name = str(path)
        if KEY_FILE.search(path.name):
            findings.append(Finding(name, 0, "key or certificate file"))
            continue
        try:
            data = path.read_bytes()
        except OSError:
            continue
        if b"\x00" in data:
            continue  # binary
        text = data.decode("utf-8", errors="replace")
        lockfile = path.name == "Cargo.lock"
        for number, line in enumerate(text.splitlines(), start=1):
            if lockfile and LOCK_CHECKSUM.match(line):
                continue
            if PRIVATE_KEY.search(line):
                findings.append(Finding(name, number, "private key block"))
            for match in HEX_256.finditer(line):
                if match.group(0).lower() not in allowlist:
                    findings.append(Finding(name, number, "256-bit hex string (possible key)"))
    return findings


def tracked_files() -> List[Path]:
    out = subprocess.run(["git", "ls-files", "-z"], cwd=REPO, check=True,
                         capture_output=True).stdout
    return [REPO / p for p in out.decode().split("\0") if p]


def main(argv=None) -> int:
    args = sys.argv[1:] if argv is None else argv
    paths = [Path(a) for a in args] if args else tracked_files()
    findings = scan_paths(paths, load_allowlist(ALLOWLIST))
    for f in findings:
        print(f"{f.path}:{f.line}: {f.reason}")
    print(f"Scanned {len(paths)} files: {len(findings)} finding(s)")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
