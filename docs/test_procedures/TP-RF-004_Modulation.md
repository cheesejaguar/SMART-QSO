# Test Procedure: Modulation Quality
## TP-RF-004

**Document Number:** SQSO-TP-RF-004
**Version:** 1.1
**Date:** 2026-10-27
**Status:** Draft; not yet run (no hardware exists)

**Requirements verified:** SYS-202, SYS-212
**Verification item:** VI-RF-03 (`docs/verification/VI_REGISTER.md`): packet error rate against input level, uplink and downlink, at 0 and 3.5 kHz offset

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

---

## 1. PURPOSE

Verify AFSK modulation parameters and AX.25 frame encoding.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| SDR receiver | RTL-SDR or better |
| Modulation analyzer | FM deviation |
| AX.25 decoder | Direwolf or equivalent |

---

## 3. PROCEDURE

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Transmit 1200 Hz tone | Deviation ±3 kHz | |
| 2 | Transmit 2200 Hz tone | Deviation ±3 kHz | |
| 3 | Transmit AX.25 packet | Decode successful | |
| 4 | Verify callsign | KK6GDB (`docs/BASELINE_PARAMETERS.md` §1) | |
| 5 | Verify SSID | 11 (KK6GDB-11; `software/flight/include/station_id.h`) | |
| 6 | Verify data content | Matches source | |
| 7 | Downlink: flight transmitter into a Dire Wolf ground demodulator, input level stepped, at 0 and 3.5 kHz offset | 90 % frame success at C/N₀ ≤ 52.9 dB-Hz (VI-RF-03) | |
| 8 | Uplink: telecommand frames into the flight receiver and TNC, input level stepped, at 0 and 3.5 kHz offset | 90 % frame success at C/N₀ ≤ 53.0 dB-Hz (VI-RF-03) | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| FM deviation | ±3 kHz ±0.5 kHz (the 10K4F2D emission of `docs/BASELINE_PARAMETERS.md` §7: 2 × (3.0 + 2.2) kHz) |
| Packet error rate | VI-RF-03 thresholds (steps 7 and 8) |
| AX.25 decode | 100% success rate |
| Callsign | Correct encoding |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-27 | Documentation audit (post-CDR 2026): VI-RF-03 packet-error-rate steps added; callsign KK6GDB-11; deviation tied to 10K4F2D; status and banner |
