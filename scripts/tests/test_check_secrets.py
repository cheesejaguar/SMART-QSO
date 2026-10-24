"""Tests for the repository secret scan (scripts/check_secrets.py)."""

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from check_secrets import load_allowlist, scan_paths  # noqa: E402

RANDOM_KEY = "7f3a9c" + "e1" * 29  # 64 hex characters, not allowlisted
# Built at run time so this file does not itself trip the scan
PEM_BEGIN = "-----BEGIN " + "OPENSSH PRIVATE KEY-----"


def write(tmp_path, name, text):
    path = tmp_path / name
    path.write_text(text)
    return path


def test_clean_file_passes(tmp_path):
    path = write(tmp_path, "notes.md", "No secrets here, just a git SHA 145a162f9249b429d3aa1a50be5cdbb7ae1af30f.\n")
    assert scan_paths([path], set()) == []


def test_pem_private_key_flagged(tmp_path):
    path = write(tmp_path, "id.txt", f"{PEM_BEGIN}\nabc\n")
    findings = scan_paths([path], set())
    assert [f.reason for f in findings] == ["private key block"]


def test_hex_key_flagged(tmp_path):
    path = write(tmp_path, "config.h", f'#define KEY "{RANDOM_KEY}"\n')
    findings = scan_paths([path], set())
    assert len(findings) == 1
    assert findings[0].line == 1
    assert "256-bit hex" in findings[0].reason


def test_uppercase_hex_key_flagged(tmp_path):
    path = write(tmp_path, "key.txt", RANDOM_KEY.upper())
    assert len(scan_paths([path], set())) == 1


def test_allowlisted_hex_passes(tmp_path):
    path = write(tmp_path, "vectors.c", f'assert_digest(d, "{RANDOM_KEY}");\n')
    assert scan_paths([path], {RANDOM_KEY}) == []


def test_key_file_name_flagged(tmp_path):
    path = write(tmp_path, "sqso.key", "not even hex")
    findings = scan_paths([path], set())
    assert [f.reason for f in findings] == ["key or certificate file"]


def test_binary_file_skipped(tmp_path):
    path = tmp_path / "blob.bin"
    path.write_bytes(b"\x00\x01" + RANDOM_KEY.encode())
    assert scan_paths([path], set()) == []


def test_cargo_lock_checksums_pass_but_other_hex_does_not(tmp_path):
    lock = write(tmp_path, "Cargo.lock",
                 f'[[package]]\nname = "x"\nchecksum = "{RANDOM_KEY}"\nkey = "{RANDOM_KEY}"\n')
    findings = scan_paths([lock], set())
    assert [f.line for f in findings] == [4]
    other = write(tmp_path, "notes.txt", f'checksum = "{RANDOM_KEY}"\n')
    assert len(scan_paths([other], set())) == 1


def test_allowlist_parsing(tmp_path):
    path = write(tmp_path, "allow.txt", f"# comment\n\n{RANDOM_KEY.upper()}  # FIPS vector\n")
    assert load_allowlist(path) == {RANDOM_KEY}


def test_allowlist_rejects_malformed_entry(tmp_path):
    path = write(tmp_path, "allow.txt", "not-hex  # oops\n")
    with pytest.raises(ValueError):
        load_allowlist(path)
