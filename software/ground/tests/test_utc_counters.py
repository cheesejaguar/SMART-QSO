"""
UTC-derived telecommand counters for two or more stations (SQSO-ICD-013
§5.2.1, RID GND-04; VI-GND-04 is the same exercise against the EM).

The frames are built by the ground tool (the Rust protocol code through
sqso_protocol) and checked by FlightCounterRule, a model of the onboard
acceptance written from software/rust/sqso-fsw/src/keys.rs (KeyTable::verify
and KeyTable::commit) with an independent HMAC (Python's hmac):

  - the tag must verify (HMAC-SHA-256 over "SQSO-TC-v1" | cmd | N | counter | args, 16 bytes);
  - exhausted if the last accepted counter >= 0xFFFFFFFE or the counter > 0xFFFFFFFE;
  - replay unless counter > last accepted (strictly), per key;
  - a rejected frame does not consume its counter; an accepted one is committed.

The key is the TEST-ONLY key 00 01 .. 1F.
"""

import hashlib
import hmac
import os
import sys
from datetime import datetime, timezone

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "tools"))

import telecommand_builder as tb  # noqa: E402

TEST_KEY = bytes(range(32))
EPOCH_2028 = datetime(2028, 1, 1, tzinfo=timezone.utc).timestamp()
LAUNCH = datetime(2028, 11, 1, 18, 0, tzinfo=timezone.utc).timestamp()
SYNCED = tb.ClockStatus(True, 0.012, "test")


class FlightCounterRule:
    """The onboard rule for one key (keys.rs verify/commit)."""

    LIMIT = 0xFFFFFFFE

    def __init__(self, key: bytes, last: int = 0):
        self.key, self.last = key, last

    def admit(self, frame: bytes) -> str:
        assert frame[:2] == b"\xAA\x55"
        cmd, length = frame[2], frame[3]
        payload = frame[4:4 + length]
        counter = int.from_bytes(payload[:4], "big")
        args, tag = payload[4:-16], payload[-16:]
        mac = hmac.new(self.key, b"SQSO-TC-v1" + bytes([cmd, len(args)]) + payload[:4] + args,
                       hashlib.sha256).digest()[:16]
        if not hmac.compare_digest(mac, tag):
            return "tag"
        if self.last >= self.LIMIT or counter > self.LIMIT:
            return "exhausted"
        if counter <= self.last:
            return "replay"
        self.last = counter
        return "accepted"


class FakeClock:
    """A station clock: true time plus an offset; sleeping advances true time."""

    def __init__(self, world, offset_s=0.0, status=SYNCED):
        self.world, self.offset, self.status = world, offset_s, status

    def now(self):
        return self.world[0] + self.offset

    def sleep(self, seconds):
        self.world[0] += seconds

    def reserve(self, path, index, **kw):
        return tb.next_utc_counter(path, index, clock=lambda: self.status, now=self.now, sleep=self.sleep, **kw)


@pytest.fixture
def world():
    return [LAUNCH]


class TestFormula:
    def test_epoch_and_index(self):
        assert tb.UTC_COUNTER_EPOCH == EPOCH_2028
        assert tb.utc_counter(EPOCH_2028, 0) == 0
        assert tb.utc_counter(EPOCH_2028 + 1.999, 3) == 7
        assert tb.decode_utc_counter(tb.utc_counter(LAUNCH + 0.5, 2)) == (LAUNCH, 2)

    def test_capacity(self):
        # Five years after launch is far inside the 32-bit space; 34 years in all
        five_years = tb.utc_counter(LAUNCH + 5 * 365.25 * 86400, 3)
        assert five_years < 0x30000000
        assert (tb.COUNTER_LIMIT // 4) / (365.25 * 86400) > 34

    def test_refusals(self):
        with pytest.raises(ValueError):
            tb.utc_counter(EPOCH_2028 - 1, 0)
        with pytest.raises(ValueError):
            tb.utc_counter(LAUNCH, 4)


class TestTwoStations:
    def test_interleaved_commands_all_accepted(self, tmp_path, world):
        """VI-GND-04 in simulation: 50 commands alternating between two
        stations whose clocks are 0.4 s apart, with gaps from seconds (both
        on the same pass) to hours; each station has its own counter file."""
        flight = FlightCounterRule(TEST_KEY)
        a, b = FakeClock(world, +0.2), FakeClock(world, -0.2)
        gaps = [3.0, 2.0, 5400.0, 1.5, 36000.0, 2.5] * 9
        counters = []
        for n in range(50):
            station, index, path = (a, 0, tmp_path / "a.ctr") if n % 2 == 0 else (b, 1, tmp_path / "b.ctr")
            counter = station.reserve(path, index)
            frame = tb.build_frame(TEST_KEY, 0x00, counter)
            assert flight.admit(frame) == "accepted", n
            counters.append(counter)
            world[0] += gaps[n]
        assert counters == sorted(counters) and len(set(counters)) == 50
        assert {c % 4 for c in counters[0::2]} == {0} and {c % 4 for c in counters[1::2]} == {1}

    def test_same_second_waits(self, tmp_path, world):
        station = FakeClock(world)
        first = station.reserve(tmp_path / "a.ctr", 0)
        second = station.reserve(tmp_path / "a.ctr", 0)
        assert second == first + 4 and world[0] > LAUNCH + 0.9

    def test_other_station_same_second_differs_in_low_bits(self, tmp_path, world):
        a = FakeClock(world).reserve(tmp_path / "a.ctr", 0)
        b = FakeClock(world).reserve(tmp_path / "b.ctr", 1)
        assert b == a + 1

    def test_replay_rejected(self, tmp_path, world):
        flight = FlightCounterRule(TEST_KEY)
        frame = tb.build_frame(TEST_KEY, 0x31, FakeClock(world).reserve(tmp_path / "a.ctr", 0))
        assert flight.admit(frame) == "accepted"
        assert flight.admit(frame) == "replay"
        # Another station's later command, then the first station's frame
        # again: still a replay
        world[0] += 3600
        other = tb.build_frame(TEST_KEY, 0x00, FakeClock(world).reserve(tmp_path / "b.ctr", 1))
        assert flight.admit(other) == "accepted"
        assert flight.admit(frame) == "replay"

    def test_frames_sent_in_build_order(self, tmp_path, world):
        """Frames built before a pass must be sent in the order built, and
        none built before another station's later command is valid after it
        (SQSO-ICD-013 §5.2.1, station rules)."""
        flight = FlightCounterRule(TEST_KEY)
        station = FakeClock(world)
        early = tb.build_frame(TEST_KEY, 0x00, station.reserve(tmp_path / "a.ctr", 0))
        late = tb.build_frame(TEST_KEY, 0x09, station.reserve(tmp_path / "a.ctr", 0))
        assert flight.admit(late) == "accepted"
        assert flight.admit(early) == "replay"

    def test_tampered_frame_refused(self, tmp_path, world):
        frame = bytearray(tb.build_frame(TEST_KEY, 0x00, FakeClock(world).reserve(tmp_path / "a.ctr", 0)))
        frame[7] ^= 1  # a counter byte
        assert FlightCounterRule(TEST_KEY).admit(bytes(frame)) == "tag"

    def test_after_key_rotation(self, tmp_path, world):
        # KEY_ROTATE starts the new key at counter 0: the next UTC counter is valid at once
        flight = FlightCounterRule(bytes(range(32, 64)), last=0)
        counter = FakeClock(world).reserve(tmp_path / "new.ctr", 1)
        assert flight.admit(tb.build_frame(bytes(range(32, 64)), 0x00, counter)) == "accepted"

    def test_compatible_with_a_shared_file_sequence_before(self, tmp_path, world):
        # Counters used before the switch (a shared file at integration) are small
        flight = FlightCounterRule(TEST_KEY, last=1234)
        counter = FakeClock(world).reserve(tmp_path / "a.ctr", 0)
        assert flight.admit(tb.build_frame(TEST_KEY, 0x00, counter)) == "accepted"


class TestClockGuards:
    def test_unsynchronised_clock_refused(self, tmp_path, world):
        path = tmp_path / "a.ctr"
        station = FakeClock(world, status=tb.ClockStatus(False, 16.0, "kernel (adjtimex)"))
        with pytest.raises(tb.ClockError, match="not synchronised"):
            station.reserve(path, 0)
        assert not path.exists()

    def test_large_clock_error_refused(self, tmp_path, world):
        station = FakeClock(world, status=tb.ClockStatus(True, 0.8, "kernel (adjtimex)"))
        with pytest.raises(tb.ClockError, match="maximum error"):
            station.reserve(tmp_path / "a.ctr", 0)
        assert FakeClock(world, status=tb.ClockStatus(True, 0.8, "k")).reserve(
            tmp_path / "a.ctr", 0, max_clock_error_s=1.0) > 0

    def test_unknown_error_refused(self, tmp_path, world):
        with pytest.raises(tb.ClockError):
            FakeClock(world, status=tb.ClockStatus(True, None, "k")).reserve(tmp_path / "a.ctr", 0)

    def test_clock_stepped_back(self, tmp_path, world):
        path = tmp_path / "a.ctr"
        FakeClock(world).reserve(path, 0)
        before = path.read_text()
        with pytest.raises(tb.ClockError, match="behind this station's last counter"):
            FakeClock(world, offset_s=-30.0).reserve(path, 0)
        assert path.read_text() == before

    def test_other_station_fast_detected_from_ack(self, tmp_path, world):
        # Station B's clock ran 30 s fast; its acknowledged counter is ahead of A's clock
        heard = tb.utc_counter(world[0] + 30.0, 1)
        with pytest.raises(tb.ClockError, match=r"newest acknowledged counter \(station 1\)"):
            FakeClock(world).reserve(tmp_path / "a.ctr", 0, newest_heard=heard)

    def test_small_lead_waited_out(self, tmp_path, world):
        heard = tb.utc_counter(world[0] + 1.0, 1)
        counter = FakeClock(world).reserve(tmp_path / "a.ctr", 0, newest_heard=heard)
        assert counter > heard

    def test_counter_file_of_another_station_refused(self, tmp_path, world):
        path = tmp_path / "b.ctr"
        FakeClock(world).reserve(path, 1)
        with pytest.raises(ValueError, match="station 1"):
            FakeClock(world).reserve(path, 0)

    def test_kernel_status_reads(self):
        status = tb.kernel_clock_status()
        assert isinstance(status.synchronised, bool) and status.source


class TestCommandLine:
    def _key(self, tmp_path):
        key = tmp_path / "op.key"
        key.write_text(TEST_KEY.hex())
        os.chmod(key, 0o600)
        return key

    def test_utc_counter_frame(self, tmp_path, monkeypatch, capsys):
        monkeypatch.setattr(tb, "kernel_clock_status", lambda: SYNCED)
        monkeypatch.setattr(tb.time, "time", lambda: LAUNCH + 0.25)
        config = tmp_path / "station.json"
        config.write_text('{"station": "KK6GDB", "station_index": 2}')
        code = tb.main(["NOP", "--key-file", str(self._key(tmp_path)), "--counter-file", str(tmp_path / "op.ctr"),
                        "--counter-from-utc", "--station-config", str(config)])
        assert code == 0
        expected = tb.utc_counter(LAUNCH, 2)
        assert capsys.readouterr().out.strip() == tb.build_frame(TEST_KEY, 0x00, expected).hex().upper()
        assert (tmp_path / "op.ctr").read_text().strip() == str(expected)

    def test_refused_when_clock_unsynchronised(self, tmp_path, monkeypatch, capsys):
        monkeypatch.setattr(tb, "kernel_clock_status", lambda: tb.ClockStatus(False, 16.0, "kernel (adjtimex)"))
        code = tb.main(["NOP", "--key-file", str(self._key(tmp_path)), "--counter-file", str(tmp_path / "op.ctr"),
                        "--counter-from-utc", "--station-index", "0"])
        assert code == 1 and "not synchronised" in capsys.readouterr().err
        assert not (tmp_path / "op.ctr").exists()

    def test_station_index_required(self, tmp_path):
        assert tb.main(["NOP", "--key-file", str(self._key(tmp_path)), "--counter-file", str(tmp_path / "c"),
                        "--counter-from-utc"]) == 2

    def test_conflicting_index(self, tmp_path):
        config = tmp_path / "station.json"
        config.write_text('{"station_index": 1}')
        assert tb.main(["NOP", "--key-file", str(self._key(tmp_path)), "--counter-file", str(tmp_path / "c"),
                        "--counter-from-utc", "--station-index", "0", "--station-config", str(config)]) == 2
