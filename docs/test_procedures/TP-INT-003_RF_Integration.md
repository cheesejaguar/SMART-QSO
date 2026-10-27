# Test Procedure: RF Subsystem Integration
## TP-INT-003

**Document Number:** SQSO-TP-INT-003
**Version:** 1.2
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-200, SYS-201, SYS-203, SYS-206, SYS-210, SYS-211, SYS-212, SYS-322

---

## 1. PURPOSE

This procedure defines the integration test for the RF communication subsystem.

---

## 2. SCOPE

- RF board integration
- Antenna cable routing
- RF power verification
- Beacon transmission test (into dummy load)

---

## 3. APPLICABLE DOCUMENTS

1. `hardware/electrical/rf/RF_BOARD_DESIGN.md` (SQSO-RF-BRD-001), the build-to design; `hardware/rf/TRANSMITTER_DESIGN.md` is earlier background
2. `hardware/rf/ANTENNA_DESIGN.md`; the antennas are the bought ISIS AntS (`docs/BASELINE_PARAMETERS.md` §7)
3. TP-RF-001 (prerequisite)
4. TP-INT-002 (prerequisite)

---

## 4. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Spectrum analyzer | 100 MHz - 1 GHz |
| Power meter | VHF capable |
| Dummy load | 50Ω, 5W |
| RF cables and adapters | MMCX to the test connector, loss characterized (the board's antenna ports are MMCX, VI-RFB-13) |
| Attenuator | 20 dB |

---

## 5. PRECONDITIONS

- [ ] OBC integration complete (TP-INT-002)
- [ ] RF board tested standalone (TP-RF-001)
- [ ] Antenna NOT deployed (use dummy load)
- [ ] RF shielded environment

---

## 6. PROCEDURE

### 6.1 RF Board Installation

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Install RF board | Secure mounting | |
| 2 | Connect RF cable | MMCX mated | |
| 3 | Connect power harness | Proper polarity | |
| 4 | Connect the RF board harness | To OBC J2 (USART2, readbacks; SQSO-ICX-001) | |

### 6.2 RF Power Test [SYS-200, SYS-201, SYS-206, SYS-322]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Connect dummy load | 50Ω terminated | |
| 2 | Command beacon TX | RF output | |
| 3 | Measure power | 27 dBm ±1 dB (0.5 W nominal); never above 30 dBm (1 W, SYS-201) | |
| 4 | Measure frequency | The IARU-coordinated downlink frequency ±5 kHz (TBD until coordination, TBD-SYS-001; SYS-200, SYS-206) | |

### 6.3 Beacon Test [SYS-210, SYS-211, SYS-212]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Enable beacon mode | Periodic TX | |
| 2 | Verify timing | 60 s in ACTIVE, 120 s in IDLE, 180 s in SAFE (SYS-211) | |
| 3 | Decode beacon | Correct format | |
| 4 | Verify callsign | KK6GDB-11 (`docs/BASELINE_PARAMETERS.md` §1) | |

---

## 7. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| RF power | 27 dBm ±1 dB (0.5 W nominal); ≤ 30 dBm (SYS-201) |
| Frequency | The IARU-coordinated downlink frequency ±5 kHz (TBD until coordination, TBD-SYS-001; SYS-200, SYS-206) |
| Beacon decode | 100% success |
| Spurious | ≤ −46 dBc and ≤ 25 µW at 1 W (SYS-203; 47 CFR 97.307(e)); third harmonic ≤ −70 dBc at the antenna port (SYS-203 Rev G; `docs/BASELINE_PARAMETERS.md` §7); measured in TP-RF-003 |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
| 1.2 | 2026-10-16 | Conflicting steps corrected and cited: RF power 27 dBm ±1 dB, never above 30 dBm (SYS-201; was 30 dBm ±1 dB); frequency the IARU-coordinated downlink frequency (SYS-200, TBD-SYS-001; was 145.825 MHz); beacon timing 60/120/180 s (SYS-211; was 30 s); spurious limit restated as SYS-203 plus the 25 µW of 47 CFR 97.307(e) (finding VV-06) |
