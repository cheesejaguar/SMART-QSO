# Test Procedure: RF Subsystem Integration
## TP-INT-003

**Document Number:** SQSO-TP-INT-003
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

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

### 6.2 RF Power Test

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Connect dummy load | 50Ω terminated | |
| 2 | Command beacon TX | RF output | |
| 3 | Measure power | 30 dBm ±1 dB | |
| 4 | Measure frequency | 145.825 MHz ±5 kHz | |

### 6.3 Beacon Test

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
| Spurious | < -43 dBc |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
