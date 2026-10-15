"""
Unit tests for the GSE key-provisioning tool.

The reference frame is the one software/flight/tests/test_gse_provision.c
checks, so a CRC or layout change on either side fails both suites. Keys
are TEST-ONLY patterns or freshly generated in a temporary directory.
"""

import json
import os
import stat
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "tools"))

from key_provision import (  # noqa: E402
    FRAME_SIZE,
    OP_ACTIVATE,
    OP_ARM_FLIGHT,
    OP_PROVISION,
    OP_ZEROIZE,
    RECOVERY_SLOT,
    build_gse_frame,
    generate_key_set,
    provisioning_frames,
    write_frames,
)

TEST_KEY = bytes(range(32))

PROVISION_SLOT0 = bytes.fromhex(
    "53514750 01 00 01 000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F E93364B5")


class TestFrame(unittest.TestCase):
    """Frames match the flight reference vector and layout."""

    def test_reference_vector(self):
        self.assertEqual(build_gse_frame(OP_PROVISION, 0, 1, TEST_KEY), PROVISION_SLOT0)

    def test_activate_and_zeroize_carry_no_key(self):
        for op in (OP_ACTIVATE, OP_ZEROIZE):
            frame = build_gse_frame(op, 0, 0)
            self.assertEqual(len(frame), FRAME_SIZE)
            self.assertEqual(frame[7:39], bytes(32))

    def test_arm_flight_frame(self):
        frame = build_gse_frame(OP_ARM_FLIGHT, 0, 0)
        self.assertEqual(frame[4], 0x04)
        self.assertEqual(frame[7:39], bytes(32))

    def test_bad_slot_rejected(self):
        with self.assertRaises(ValueError):
            build_gse_frame(OP_PROVISION, RECOVERY_SLOT + 1, 1, TEST_KEY)

    def test_provision_needs_full_key(self):
        with self.assertRaises(ValueError):
            build_gse_frame(OP_PROVISION, 0, 1, TEST_KEY[:16])


class TestKeySet(unittest.TestCase):
    """Generated keys are random, private and never overwritten."""

    def test_generate_writes_private_distinct_keys(self):
        with TemporaryDirectory() as tmp:
            directory = Path(tmp) / "sqso-keys"
            manifest = generate_key_set(directory, [1, 2, 3, 4], 99)
            keys = []
            for entry in manifest["keys"]:
                path = directory / entry["file"]
                if os.name == "posix":
                    self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o600)
                keys.append(bytes.fromhex(path.read_text().strip()))
            self.assertEqual(len(keys), 5)
            self.assertEqual(len(set(keys)), 5)
            self.assertTrue(all(len(k) == 32 for k in keys))
            recorded = json.loads((directory / "manifest.json").read_text())
            self.assertNotIn(keys[0].hex(), json.dumps(recorded))

    def test_generate_refuses_to_overwrite(self):
        with TemporaryDirectory() as tmp:
            directory = Path(tmp) / "sqso-keys"
            generate_key_set(directory, [1, 2, 3, 4], 99)
            with self.assertRaises(FileExistsError):
                generate_key_set(directory, [1, 2, 3, 4], 99)

    def test_frames_provision_all_then_activate(self):
        with TemporaryDirectory() as tmp:
            directory = Path(tmp) / "sqso-keys"
            generate_key_set(directory, [1, 2, 3, 4], 99)
            frames = provisioning_frames(directory, activate_slot=0)
            self.assertEqual([f[4] for f in frames], [OP_PROVISION] * 5 + [OP_ACTIVATE])
            self.assertEqual([f[5] for f in frames], [0, 1, 2, 3, RECOVERY_SLOT, 0])
            self.assertEqual([f[6] for f in frames[:5]], [1, 2, 3, 4, 99])

    def test_frames_file_is_private(self):
        with TemporaryDirectory() as tmp:
            out = Path(tmp) / "frames.bin"
            write_frames(out, [build_gse_frame(OP_PROVISION, 0, 1, TEST_KEY)])
            self.assertEqual(out.read_bytes(), PROVISION_SLOT0)
            if os.name == "posix":
                self.assertEqual(stat.S_IMODE(out.stat().st_mode), 0o600)


if __name__ == "__main__":
    unittest.main()
