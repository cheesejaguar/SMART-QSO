"""Tests of the ground decoder (software/ground/tools/beacon_decoder.py).

The frames come from the flight formatters: fixtures/flight_frames.txt is
written by software/rust/sqso-fsw/tests/ground_frames.rs, whose test fails
when the file is not what the formatters make now. So these tests decode
what the flight software sends, and a change to a flight format reaches them.
"""

import io
import json
import os
import socket
import sys
import threading

import pytest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(os.path.dirname(HERE), "tools"))

import sqso_protocol  # noqa: E402

import beacon_decoder as bd  # noqa: E402

FIXTURE = os.path.join(HERE, "fixtures", "flight_frames.txt")


def fixture_frames():
    """{label: frame bytes} in file order"""
    out = {}
    with open(FIXTURE, encoding="utf-8") as fh:
        for line in fh:
            if line.strip() and not line.startswith("#"):
                label, *hexed = line.split()
                out[label] = bytes.fromhex("".join(hexed))
    return out


FRAMES = fixture_frames()


def hk_frames(name):
    return [f for label, f in FRAMES.items() if label.startswith(name + ".")]


def decode_all(frames, **kw):
    dec = bd.Decoder(**kw)
    out = []
    for f in frames:
        out.extend(dec.feed(f, now=0.0))
    return out + dec.flush()


def kiss_body(frame):
    """What a KISS TNC delivers: no flags, no FCS"""
    return frame.strip(b"\x7e")[:-2]


# --- The beacon -------------------------------------------------------------


def test_beacon_every_field():
    f = bd.decode_frame(FRAMES["beacon_typical"])
    assert (f.kind, f.dest, f.source, f.fcs, f.errors) == ("beacon", "CQ", "KK6GDB-11", "ok", [])
    b = f.beacon
    assert b.callsign == "KK6GDB-11"
    assert b.text == "Amateur satellite beacon - 73 de KK6GDB-11"
    assert list(b.fields) == ["T", "M", "SOC", "BV", "SUN", "RF", "QSO"]
    assert b.values == {
        "mission_time_s": 1792115511,
        "power_mode": "ACTIVE",
        "soc_pct": 83,
        "battery_v": 14.4,
        "sunlit": 1,
        "rf_power": 1,
        "qso_count": None,  # "-": not measured, never zero
    }
    assert b.unknown_keys == []


def test_beacon_unmeasured_fields_are_none():
    b = bd.decode_frame(FRAMES["beacon_unmeasured"]).beacon
    assert b.values["power_mode"] == "SAFE"
    assert b.values["mission_time_s"] == 42
    for name in ("soc_pct", "battery_v", "sunlit", "rf_power", "qso_count"):
        assert b.values[name] is None


def test_beacon_idle_and_rounded_voltage():
    b = bd.decode_frame(FRAMES["beacon_idle"]).beacon
    assert (b.values["power_mode"], b.values["battery_v"], b.values["mission_time_s"]) == ("IDLE", 14.4, 0)


def test_beacon_widest():
    f = bd.decode_frame(FRAMES["beacon_widest"])
    assert len(f.info) == 255  # 176 text bytes and the telemetry at its widest
    b = f.beacon
    assert len(b.text) == 176 and b.text.startswith("Hello from orbit: a long")  # ": " inside the text
    assert b.values == {
        "mission_time_s": 4294967295,
        "power_mode": "ACTIVE",
        "soc_pct": 255,
        "battery_v": 65.5,
        "sunlit": 255,
        "rf_power": 255,
        "qso_count": 65535,
    }


@pytest.mark.parametrize(
    "info, message",
    [
        ("hello", "no 'de ' prefix"),
        ("de KK6GDB-11 hello", "without 'de CALL: '"),
        ("de KK6GDB-11: hello T=1", "separator"),
        ("de KK6GDB-11: x | T=1 M=SAFE SOC=1 BV=1.0 SUN=1 RF=1", "missing: QSO"),
        ("de KK6GDB-11: x | T=1 M=BURST SOC=1 BV=1.0 SUN=1 RF=1 QSO=0", "M=BURST"),
        ("de KK6GDB-11: x | T=1 M=SAFE SOC=1 BV=14 SUN=1 RF=1 QSO=0", "one decimal"),
        ("de KK6GDB-11: x | T=1 M=SAFE SOC=x SUN=1 BV=1.0 RF=1 QSO=0", "not an integer"),
        ("de KK6GDB-11: x | T=1 M=SAFE SOC=1 SOC=2 BV=1.0 SUN=1 RF=1 QSO=0", "twice"),
        ("de KK6GDB-11: x | T=1 M SOC=1", "KEY=value"),
    ],
)
def test_beacon_malformed(info, message):
    with pytest.raises(bd.DecodeError, match=message):
        bd.parse_beacon(info)


def test_beacon_empty_text_and_unknown_field():
    b = bd.parse_beacon("de KK6GDB-11:  | T=0 M=SAFE SOC=- BV=- SUN=- RF=- QSO=- NEW=5")
    assert b.text == ""
    assert b.unknown_keys == ["NEW"] and b.fields["NEW"] == "5"


# --- Housekeeping -----------------------------------------------------------


def test_hk1_one_frame():
    (hk,) = decode_all(hk_frames("hk1_typical"))
    assert (hk["kind"], hk["seq"], hk["frames"], hk["complete"]) == ("hk1", 7, 1, True)
    v = hk["values"]
    assert (v["uptime_s"], v["boots"], v["reset_cause"], v["mode"]) == (3600, 3, 2, 3)
    assert (v["commissioning_hold"], v["transponder_enabled"], v["tx_permitted"]) == (1, 0, 1)
    assert (v["soc_pct"], v["battery_v"], v["battery_a"], v["array_w"]) == (87.3, 15.812, -0.12, 2.41)
    assert (v["battery_temp_c"], v["obc_temp_c"], v["pa_temp_c"], v["jetson_temp_c"]) == (12.3, 21.5, 30.1, None)
    assert (v["deployment_state"], v["ants_deployed_mask"], v["wing_sense"]) == (8, 15, 1)
    assert (v["rf_silence_state"], v["rf_silence_s"], v["contact_timeout_d"], v["since_contact_h"]) == (3, 2700, 30, 2)
    assert (v["fault_count"], v["last_fault_type"], v["last_fault_severity"]) == (4, 17, 2)
    assert (v["rate_x_dps"], v["rate_y_dps"], v["rate_z_dps"], v["adcs_mode"]) == (0.12, -0.05, 6.0, 2)
    assert (v["gnss_fix_age_s"], v["gnss_sats"], v["gnss_sessions"], v["gnss_sessions_with_fix"]) == (1200, 7, 5, 4)
    assert (v["passivation_phase"], v["eol_state"], v["eol_fire_attempts"], v["eol_fire_reasons"]) == (1, 4, 0, 3)
    assert (v["rx_recovery_cycles"], v["rx_failed"], v["contacts"]) == (2, 0, 3)
    assert (v["stored_summary_next"], v["stored_detail_next"], v["fram_state"]) == (12, 60, 0)
    assert (v["ants_temp_deferrals"], v["ants_temp_c"]) == (2, -19)


def test_hk1_widest_is_two_frames_merged():
    frames = hk_frames("hk1_widest")
    assert len(frames) == 2
    texts = [bd.decode_frame(f).info.decode() for f in frames]
    assert all(t.startswith("HK1 N=65535 ") for t in texts) and all(len(t) <= 250 for t in texts)
    (hk,) = decode_all(frames)
    assert (hk["seq"], hk["frames"], hk["complete"], hk["unknown_keys"]) == (65535, 2, True, [])
    assert list(hk["fields"]) == list(bd.HK1_FIELDS)  # every field, in the flight order
    v = hk["values"]
    assert (v["soc_pct"], v["battery_v"], v["battery_a"], v["array_w"]) == (100.0, 16.8, -3.5, 9.999)
    assert v["rate_x_dps"] == -327.68 and v["jetson_temp_c"] == -40.0
    assert (v["mag_rejected"], v["stored_detail_next"], v["fram_failed_checks"]) == (4000000000, 2**32 - 1, 2**32 - 1)
    assert (v["gnss_sessions"], v["eol_state"], v["ants_temp_c"]) == (99999, 255, -60)


def test_hk1_unmeasured():
    (hk,) = decode_all(hk_frames("hk1_unmeasured"))
    v = hk["values"]
    assert hk["complete"]
    for name in ("soc_pct", "battery_v", "battery_a", "array_w", "battery_temp_c", "ants_deployed_mask",
                 "last_fault_type", "last_fault_severity", "gnss_fix_age_s", "gnss_sats", "eol_state", "ants_temp_c"):
        assert v[name] is None, name
    assert v["deployment_state"] == 0 and v["eol_fire_attempts"] == 0


def test_hk2_one_frame():
    (hk,) = decode_all(hk_frames("hk2_typical"))
    assert (hk["kind"], hk["seq"], hk["complete"]) == ("hk2", 7, True)
    v = hk["values"]
    assert (v["commands_executed"], v["commands_refused"], v["auth_failures"], v["watchdog_resets"]) == (12, 3, 1, 2)
    assert (v["transponder_enables"], v["repeater_keyed_s"], v["exciter_keyed_s"]) == (4, 3600, 4100)
    assert (v["array_a"], v["hard_limit_violations"]) == (0.412, 0)
    assert (v["jetson_bursts"], v["last_burst_j"], v["last_burst_lines"], v["energy_gate_ends"]) == (6, 1210, 10, 0)
    assert (v["jetson_off_clean"], v["jetson_off_cut"], v["jetson_pg_faults"], v["jetson_readback_faults"]) == (5, 1, 0, 0)
    assert (v["eol_state"], v["eol_fire_attempts"], v["rtc_unix_s"], v["last_reset_unix_s"]) == (4, 0, 1900000000, None)
    assert (v["mcu_clock_ppm"], v["trips_gyro"], v["gyro_in_use"], v["mag_samples_gated"]) == (-12, 1, 0, 37)
    assert (v["sram2_parity"], v["separation_latched"], v["stowed_readings"]) == (1, 1, 1)
    assert (v["tx_temp_faulted"], v["tx_temp_faults"], v["eps_supv"], v["heartbeats_refused"]) == (0, 2, 33, 0)
    assert v["chg_inhibit"] == 64
    assert (v["tnc_version"], v["tnc_criteria"], v["tnc_floor_dbm"], v["tnc_floor_cal_dbm"]) == (3, 5, -121, -119)
    assert (v["tnc_carrier_count"], v["tnc_frames"], v["tnc_uptime_s"]) == (42, 17, 86400)
    assert (v["rx_fault_selftest_ok"], v["tnc_status_unanswered"]) == (1, 0)
    assert [v[k] for k in ("selector_enabled", "h1_decisions", "h1_abstentions", "h2_decisions", "h2_abstentions",
                           "h3_decisions", "h3_abstentions")] == [1, 1, 0, 0, 0, 0, 1]
    assert (v["geometry_preset"], v["sun_x"], v["sun_y"], v["sun_z"]) == (1, -0.138, 0.989, 0.044)


def test_hk2_widest_is_three_frames_merged():
    frames = hk_frames("hk2_widest")
    assert len(frames) == 3
    (hk,) = decode_all(frames)
    assert (hk["seq"], hk["frames"], hk["complete"], hk["unknown_keys"]) == (65535, 3, True, [])
    assert list(hk["fields"]) == list(bd.HK2_FIELDS)
    v = hk["values"]
    assert v["last_burst_lines"] == 2**64 - 1 and v["last_burst_j"] == 2_000_000_000
    assert (v["array_a"], v["mcu_clock_ppm"], v["tnc_floor_dbm"]) == (-2.6, -1000000, -128)
    assert (v["sun_x"], v["sun_y"], v["sun_z"], v["geometry_preset"]) == (-1.0, -1.0, -1.0, 255)


def test_hk2_unmeasured():
    (hk,) = decode_all(hk_frames("hk2_unmeasured"))
    v = hk["values"]
    for name in ("array_a", "eol_state", "rtc_unix_s", "last_reset_unix_s", "mcu_clock_ppm", "sram2_parity",
                 "eps_supv", "chg_inhibit", "tnc_version", "tnc_uptime_s", "sun_x", "sun_z"):
        assert v[name] is None, name


def test_field_tables_have_unique_names():
    for spec in (bd.HK1_FIELDS, bd.HK2_FIELDS):
        names = [p.name for parts in spec.values() for p in parts]
        assert len(names) == len(set(names))


def test_all_fixture_frames_decode_without_errors():
    records = decode_all(FRAMES.values())
    assert [r["kind"] for r in records] == ["beacon"] * 4 + ["hk1"] * 3 + ["hk2"] * 3
    assert all(not r["errors"] for r in records)
    assert all(r.get("complete", True) for r in records)


# --- Merging the frames of one text ----------------------------------------


def test_parts_shown_when_asked():
    records = decode_all(hk_frames("hk1_widest"), show_parts=True)
    assert [r["kind"] for r in records] == ["hk1_part", "hk1_part", "hk1"]
    assert records[1]["seq"] == 65535 and "GS" in records[1]["fields"]


def test_hk1_then_hk2_of_one_sequence_merge_separately():
    frames = hk_frames("hk1_widest") + hk_frames("hk2_widest")
    records = decode_all(frames)
    assert [(r["kind"], r["frames"], r["complete"]) for r in records] == [("hk1", 2, True), ("hk2", 3, True)]


def test_lost_last_frame_gives_an_incomplete_text_on_flush():
    dec = bd.Decoder()
    first = hk_frames("hk1_widest")[0]
    assert dec.feed(first, now=0.0) == []
    (hk,) = dec.flush()
    assert hk["complete"] is False and "AD" in hk["missing_keys"] and hk["values"]["ants_temp_c"] is None
    assert hk["values"]["uptime_s"] == 99999999


def test_lost_first_frame_still_gives_the_rest():
    (hk,) = decode_all(hk_frames("hk2_widest")[1:])
    assert hk["complete"] is False and hk["missing_keys"][:2] == ["CA", "WD"]
    assert hk["values"]["sun_x"] == -1.0


def test_incomplete_text_expires():
    asm = bd.HkAssembler(max_age_s=30)
    first = bd.decode_frame(hk_frames("hk1_widest")[0]).hk
    assert asm.add(first, now=0.0) == []
    assert asm.expire(now=10.0) == []
    (hk,) = asm.expire(now=31.0)
    assert not hk.complete and asm.flush() == []


def test_new_text_with_same_head_pushes_out_the_old():
    # The sequence restarts at 0 at every reset: a first frame of N=65535
    # again while one is waiting gives the waiting one out
    first = hk_frames("hk1_widest")[0]
    records = decode_all([first, first] + hk_frames("hk1_widest")[1:])
    assert [(r["frames"], r["complete"]) for r in records] == [(1, False), (2, True)]


def test_conflicting_repeat_is_an_error():
    asm = bd.HkAssembler()
    asm.add(bd.parse_hk_frame("HK2 N=3 RT=1/2"), now=0)
    (hk,) = asm.add(bd.parse_hk_frame("HK2 N=3 RT=1/3 SN=-"), now=0)
    assert any("RT sent twice" in e for e in hk.errors) and not hk.complete


def test_single_line_text():
    hk = bd.parse_housekeeping(bd.decode_frame(hk_frames("hk1_typical")[0]).info.decode())
    assert hk.complete and hk.values["soc_pct"] == 87.3


@pytest.mark.parametrize(
    "text, message",
    [("HK3 N=1", "not a housekeeping"), ("HK1 UP=1", "'N=' head"), ("HK1 N=x UP=1", "not an integer")],
)
def test_hk_malformed(text, message):
    with pytest.raises(bd.DecodeError, match=message):
        bd.parse_hk_frame(text)


def test_hk_wrong_part_count_is_reported():
    hk = bd.decode_housekeeping("HK1", 1, {"W": "1/2"})
    assert any("W=1/2" in e for e in hk.errors) and hk.values["rate_x_dps"] is None


# --- AX.25 forms, other frames -----------------------------------------------


def test_frame_forms():
    on_air = FRAMES["beacon_typical"]
    assert on_air[0] == 0x7E and on_air[-1] == 0x7E
    without_flags = on_air.strip(b"\x7e")
    for raw, fcs in ((on_air, "ok"), (without_flags, "ok"), (kiss_body(on_air), "absent")):
        f = bd.decode_frame(raw)
        assert (f.fcs, f.kind, f.beacon.values["soc_pct"]) == (fcs, "beacon", 83)


def test_bad_fcs_on_the_air_is_refused():
    bad = bytearray(FRAMES["beacon_typical"])
    bad[30] ^= 0x01
    with pytest.raises(bd.DecodeError, match="FCS"):
        bd.decode_frame(bytes(bad))
    (record,) = bd.Decoder().feed(bytes(bad))
    assert record["kind"] == "error"


def test_not_ax25():
    with pytest.raises(bd.DecodeError):
        bd.decode_frame(b"\x01\x02")


def test_other_station_and_other_frames():
    frame = sqso_protocol.ax25_ui_frame("CQ", "N0CALL-1", b"de N0CALL-1: hi | T=1 M=SAFE SOC=- BV=- SUN=- RF=- QSO=-")
    f = bd.decode_frame(frame)
    assert f.kind == "beacon" and any("not KK6GDB-11" in e for e in f.errors)
    ack = bd.decode_frame(sqso_protocol.ax25_ui_frame("CQ", "KK6GDB-11", b"ack KK6GDB-11: C=41 R=0 N=0"))
    assert (ack.kind, ack.decoded) == ("ack", {"counter": 41, "result": 0, "days_since_contact": 0})
    hkd = bd.decode_frame(sqso_protocol.ax25_ui_frame("CQ", "KK6GDB-11", b"HKD\x00\x01\xff"))
    assert hkd.kind == "hkd" and hkd.to_dict()["info"] == "484B440001FF"
    other = bd.decode_frame(sqso_protocol.ax25_ui_frame("CQ", "KK6GDB-11", b"hello"))
    assert other.kind == "other" and not other.errors


def test_gnss_frame_through_gnss_frame_py():
    info = (b"GNS N=17 WK=452 TOW=345600000 X=-4349115 Y=4803225 Z=2239564 VX=-4581234 VY=-1912078 "
            b"VZ=5870431 VDT=30 Q=1 SV=8 PD=18 AGE=0")
    f = bd.decode_frame(sqso_protocol.ax25_ui_frame("CQ", "KK6GDB-11", info))
    assert f.kind == "gns" and f.decoded["week"] == 452 and f.decoded["seq"] == 17
    json.dumps(f.to_dict())


# --- Input: logs, KISS --------------------------------------------------------


def test_read_lines_forms():
    hexed = FRAMES["beacon_typical"].hex()
    spaced = " ".join(FRAMES["beacon_typical"].hex()[i:i + 2] for i in range(0, len(hexed), 2))
    lines = ["# comment", "", hexed, f"2026-11-03T10:00:00Z {hexed}", f"beacon_typical {spaced}",
             json.dumps({"rx_utc": "2026-11-03T10:00:05Z", "frame_hex": hexed})]
    got = list(bd.read_lines(lines))
    assert [rx for _, rx in got] == [None, None, None, "2026-11-03T10:00:05Z"]
    assert all(raw == FRAMES["beacon_typical"] for raw, _ in got)
    with pytest.raises(bd.DecodeError, match="line 1"):
        list(bd.read_lines(["zz"]))


def test_cli_file(tmp_path, capsys):
    out = tmp_path / "out.jsonl"
    assert bd.main(["-f", FIXTURE, "-o", str(out)]) == 0
    records = [json.loads(line) for line in out.read_text().splitlines()]
    assert len(records) == 10 and records[-3]["kind"] == "hk2"
    assert bd.main([FRAMES["beacon_idle"].hex()]) == 0
    assert json.loads(capsys.readouterr().out)["beacon"]["values"]["power_mode"] == "IDLE"
    assert bd.main(["nothex"]) == 2


def kiss_stream():
    """The fixture as a KISS TNC passes it on: no flags or FCS, KISS-framed"""
    return b"".join(sqso_protocol.kiss_frame(kiss_body(f) if f[0] == 0x7E else f[:-2]) for f in FRAMES.values())


def test_kiss_frames_from_a_stream():
    frames = list(bd.kiss_frames(io.BytesIO(kiss_stream()), chunk=7))
    assert len(frames) == len(FRAMES)
    records = decode_all(frames)
    assert [r["kind"] for r in records] == ["beacon"] * 4 + ["hk1"] * 3 + ["hk2"] * 3
    assert all(not r["errors"] for r in records)


def test_cli_kiss_file_with_log(tmp_path):
    stream = tmp_path / "capture.kiss"
    stream.write_bytes(kiss_stream())
    log, out = tmp_path / "frames.jsonl", tmp_path / "out.jsonl"
    assert bd.main(["--kiss-file", str(stream), "--log", str(log), "-o", str(out)]) == 0
    assert len(out.read_text().splitlines()) == 10
    logged = log.read_text().splitlines()
    assert len(logged) == len(FRAMES) and json.loads(logged[0])["rx_utc"].endswith("Z")
    # The log reads back with -f
    again = tmp_path / "again.jsonl"
    assert bd.main(["-f", str(log), "-o", str(again)]) == 0
    strip = lambda text: [{k: v for k, v in json.loads(x).items() if k != "rx_utc"} for x in text.splitlines()]
    assert strip(again.read_text()) == strip(out.read_text())


def test_cli_kiss_tcp(tmp_path):
    server = socket.socket()
    server.bind(("127.0.0.1", 0))
    server.listen(1)
    port = server.getsockname()[1]

    def serve():
        conn, _ = server.accept()
        conn.sendall(kiss_stream())
        conn.close()

    t = threading.Thread(target=serve)
    t.start()
    out = tmp_path / "out.jsonl"
    try:
        assert bd.main(["--kiss-tcp", f"127.0.0.1:{port}", "-o", str(out)]) == 0
    finally:
        t.join()
        server.close()
    kinds = [json.loads(x)["kind"] for x in out.read_text().splitlines()]
    assert kinds == ["beacon"] * 4 + ["hk1"] * 3 + ["hk2"] * 3
