# Test Procedure: RF Subsystem Integration
## TP-INT-003

**Document Number:** SQSO-TP-INT-003
**Version:** 1.1
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-203, SYS-206, SYS-210, SYS-212, SYS-322

**Not cited until corrected:** §6.2 step 3 and §7 (30 dBm ±1 dB allows more than the 1 W maximum of SYS-201); the 145.825 MHz nominal in §6.2 step 4 and §7 (SYS-200 excludes it; the ±5 kHz tolerance is SYS-206); §6.3 step 2 (30 s; SYS-211 sets 60, 120 and 180 s).

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

1. hardware/rf/TRANSMITTER_DESIGN.md
2. hardware/rf/ANTENNA_DESIGN.md
3. TP-RF-001 (prerequisite)
4. TP-INT-002 (prerequisite)

---

## 4. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Spectrum analyzer | 100 MHz - 1 GHz |
| Power meter | VHF capable |
| Dummy load | 50Ω, 5W |
| SMA cables | Low loss |
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
| 2 | Connect RF cable | SMA mated | |
| 3 | Connect power harness | Proper polarity | |
| 4 | Connect UART cable | To OBC | |

### 6.2 RF Power Test [SYS-206, SYS-322]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Connect dummy load | 50Ω terminated | |
| 2 | Command beacon TX | RF output | |
| 3 | Measure power | 30 dBm ±1 dB | |
| 4 | Measure frequency | 145.825 MHz ±5 kHz | |

### 6.3 Beacon Test [SYS-210, SYS-212]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Enable beacon mode | Periodic TX | |
| 2 | Verify timing | 30s interval | |
| 3 | Decode beacon | Correct format | |
| 4 | Verify callsign | Correct callsign | |

---

## 7. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| RF power | 30 dBm ±1 dB |
| Frequency | 145.825 MHz ±5 kHz |
| Beacon decode | 100% success |
| Spurious | < -43 dBc (SYS-203: at least 40 dB below carrier) |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
