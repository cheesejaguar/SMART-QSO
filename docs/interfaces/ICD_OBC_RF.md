# Interface Control Document: OBC to RF Subsystem

**Document ID**: SQSO-ICD-003
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the interface between the On-Board Computer (OBC) and the RF Subsystem for the SMART-QSO CubeSat per NPR 7123.1 SE Process 5.

## 2. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-SRS-001 | System Requirements Specification |
| SQSO-HW-RF-001 | RF Subsystem Design |
| SQSO-HW-OBC-001 | OBC Design |
| SQSO-ICD-001 | Beacon Format ICD |

## 3. Interface Summary

```
INTERFACE BLOCK DIAGRAM

┌─────────────────────────────────────────────────────────────┐
│                          OBC                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │  RF Driver  │    │  Beacon     │    │   State     │     │
│  │   Task      │    │  Generator  │    │  Manager    │     │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘     │
│         │                  │                   │            │
└─────────┼──────────────────┼───────────────────┼────────────┘
          │                  │                   │
          │ SPI              │ Data              │ Control
          │                  │                   │
┌─────────┼──────────────────┼───────────────────┼────────────┐
│         ▼                  ▼                   ▼            │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │  Si4463     │    │   FIFO      │    │   GPIO      │     │
│  │  Driver     │    │   Buffer    │    │  Control    │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│                          RF Subsystem                       │
└─────────────────────────────────────────────────────────────┘
```

## 4. Physical Interface

### 4.1 Connector Definition

| Interface | OBC Connector | RF Connector | Cable |
|-----------|---------------|--------------|-------|
| SPI/GPIO | J3 (Hirose DF12) | J1 (Hirose DF12) | HAR-003 |

### 4.2 Pin Assignment

| Pin | Signal | Direction | Description |
|-----|--------|-----------|-------------|
| 1 | GND | - | Signal ground |
| 2 | SPI_CLK | OBC→RF | SPI clock |
| 3 | SPI_MOSI | OBC→RF | SPI master out |
| 4 | SPI_MISO | RF→OBC | SPI master in |
| 5 | SPI_CS | OBC→RF | SPI chip select (active low) |
| 6 | RF_IRQ | RF→OBC | Interrupt request |
| 7 | RF_SDN | OBC→RF | Shutdown control |
| 8 | RF_GPIO0 | RF→OBC | CTS signal |
| 9 | RF_GPIO1 | RF→OBC | TX/RX indicator |
| 10 | NC | - | Reserved |

### 4.3 Cable Requirements

| Parameter | Requirement |
|-----------|-------------|
| Length | ≤150 mm |
| Type | Twisted pair, shielded |
| AWG | 28 AWG minimum |
| Shield | Connect to chassis GND |

## 5. Electrical Interface

### 5.1 SPI Characteristics

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| Clock frequency | - | 4 | 10 | MHz |
| Clock polarity | 0 (idle low) | | | |
| Clock phase | 0 (sample rising) | | | |
| CS setup time | 50 | - | - | ns |
| CS hold time | 50 | - | - | ns |
| MOSI setup time | 10 | - | - | ns |
| MISO valid time | - | - | 50 | ns |

### 5.2 GPIO Characteristics

| Signal | Logic 0 | Logic 1 | Pull |
|--------|---------|---------|------|
| RF_IRQ | <0.4V | >2.4V | Pull-up (10k) |
| RF_SDN | <0.4V | >2.4V | Pull-down (10k) |
| RF_GPIO0 | <0.4V | >2.4V | None |
| RF_GPIO1 | <0.4V | >2.4V | None |

### 5.3 Power Sequencing

```
POWER SEQUENCING

1. RF VCC applied (controlled by EPS)
2. Wait 10ms for power stable
3. Assert RF_SDN high (shutdown)
4. Wait 1ms
5. Deassert RF_SDN low (power on)
6. Wait for RF_GPIO0 high (CTS)
7. Begin SPI communication
```

## 6. Logical Interface

### 6.1 SPI Protocol

#### 6.1.1 Command Format

```
COMMAND FRAME FORMAT

Byte 0: Command ID
Byte 1-N: Command parameters

RESPONSE FRAME FORMAT

Byte 0: CTS (0xFF = ready)
Byte 1-N: Response data
```

#### 6.1.2 Command Set

| Command | ID | Parameters | Response | Description |
|---------|-----|------------|----------|-------------|
| POWER_UP | 0x02 | 7 bytes | None | Power up transceiver |
| GET_INT_STATUS | 0x20 | 0 bytes | 8 bytes | Get interrupt status |
| SET_PROPERTY | 0x11 | 5+ bytes | None | Set configuration |
| GET_PROPERTY | 0x12 | 4 bytes | Variable | Get configuration |
| START_TX | 0x31 | 5 bytes | None | Start transmission |
| START_RX | 0x32 | 7 bytes | None | Start reception |
| WRITE_TX_FIFO | 0x66 | N bytes | None | Write to TX FIFO |
| READ_RX_FIFO | 0x77 | 0 bytes | N bytes | Read from RX FIFO |
| FIFO_INFO | 0x15 | 1 byte | 2 bytes | Get FIFO status |

### 6.2 Initialization Sequence

```
RF INITIALIZATION SEQUENCE

1. Send POWER_UP command
2. Wait for CTS
3. Configure modem settings via SET_PROPERTY:
   - Frequency: 145.825 MHz
   - Modulation: GFSK
   - Data rate: 9600 bps
   - Deviation: 3.5 kHz
4. Configure GPIO via SET_PROPERTY
5. Configure interrupts via SET_PROPERTY
6. Verify configuration via GET_PROPERTY
7. RF ready for operation
```

### 6.3 Transmission Procedure

```
BEACON TRANSMISSION PROCEDURE

1. Verify CTS (RF_GPIO0 = high)
2. Build AX.25 frame
3. Write frame to TX FIFO (WRITE_TX_FIFO)
4. Start transmission (START_TX)
5. Wait for TX complete interrupt (RF_IRQ)
6. Clear interrupt (GET_INT_STATUS)
7. Verify transmission success
```

### 6.4 Reception Procedure

```
PACKET RECEPTION PROCEDURE

1. Start receiver (START_RX)
2. Wait for RX interrupt (RF_IRQ)
3. Get interrupt status (GET_INT_STATUS)
4. If packet received:
   a. Get FIFO info (FIFO_INFO)
   b. Read packet (READ_RX_FIFO)
   c. Process packet
5. Clear interrupt
6. Restart receiver
```

## 7. Data Interface

### 7.1 TX Data Format

| Field | Size | Description |
|-------|------|-------------|
| Preamble | 8 bytes | 0x7E alternating |
| Flag | 1 byte | 0x7E |
| AX.25 frame | Variable | Per ICD-001 |
| FCS | 2 bytes | CRC-16 |
| Flag | 1 byte | 0x7E |

### 7.2 FIFO Limits

| Parameter | Value |
|-----------|-------|
| TX FIFO depth | 64 bytes |
| RX FIFO depth | 64 bytes |
| Max packet size | 256 bytes |

### 7.3 Data Rate

| Mode | Data Rate | Modulation |
|------|-----------|------------|
| Beacon | 9600 bps | GFSK |
| Legacy | 1200 bps | AFSK |

## 8. Control Interface

### 8.1 Power Control

| State | RF_SDN | Power | Description |
|-------|--------|-------|-------------|
| Shutdown | High | ~10 µA | RF disabled |
| Standby | Low, idle | ~1 mA | Ready state |
| RX | Low, RX active | ~15 mA | Receiving |
| TX Low | Low, TX active | ~100 mA | 0.5W output |
| TX High | Low, TX active | ~250 mA | 2.0W output |

### 8.2 Interrupt Handling

| Interrupt | Cause | OBC Action |
|-----------|-------|------------|
| PACKET_SENT | TX complete | Log success, ready next |
| PACKET_RX | RX complete | Read FIFO, process |
| CRC_ERROR | RX CRC fail | Log error, restart RX |
| FIFO_OVERFLOW | FIFO full | Clear FIFO, restart |
| PREAMBLE_DETECT | Preamble received | Prepare for packet |

### 8.3 Error Handling

| Error | Detection | Recovery |
|-------|-----------|----------|
| SPI timeout | No CTS | Reset RF, reinitialize |
| TX timeout | No TX complete | Abort, reset TX |
| FIFO error | Overflow/underflow | Clear FIFO |
| Config error | Verify fail | Reinitialize |

## 9. Timing Requirements

### 9.1 Transaction Timing

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| SPI transaction | - | 50 | 100 | µs |
| Power up time | - | 15 | 25 | ms |
| TX startup | - | 100 | 200 | µs |
| RX startup | - | 100 | 200 | µs |
| Mode switch | - | 50 | 100 | µs |

### 9.2 Beacon Timing

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| Beacon period | 60 | 90 | 120 | s |
| Beacon duration | - | 2 | 5 | s |
| TX duty cycle | - | - | 10 | % |

## 10. Verification Requirements

### 10.1 Interface Verification Tests

| Test | Method | Criteria |
|------|--------|----------|
| SPI connectivity | Loopback | Data matches |
| Timing | Oscilloscope | Within spec |
| Power consumption | Ammeter | Within spec |
| TX power | Power meter | 0.5-2.0W |
| Frequency | Spectrum analyzer | ±1 kHz |

### 10.2 Acceptance Criteria

| Criteria | Requirement |
|----------|-------------|
| SPI error rate | <10^-6 |
| TX success rate | >99% |
| RX sensitivity | <-115 dBm |
| Power output | ±0.5 dB |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
