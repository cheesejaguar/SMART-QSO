# On-Board Computer (OBC) Design Document

**Document Number**: SMART-QSO-OBC-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary Design

**Governing Standards**:
- NPR 7120.5 Rev F - NASA Space Flight Program and Project Management Requirements
- NPR 7150.2 Rev B - NASA Software Engineering Requirements
- ECSS-E-ST-50C - Communications

**Heritage**: NASA Ames Research Center Small Satellite Division (GeneSat, PharmaSat, O/OREOS, EcAMSat)

---

## 1. Introduction

### 1.1 Purpose

This document defines the detailed design of the On-Board Computer (OBC) subsystem for the SMART-QSO 1U CubeSat. The OBC serves as the central processing unit for all spacecraft operations.

### 1.2 Scope

The OBC encompasses:
- Central processor and memory
- Peripheral interfaces (I2C, SPI, UART, GPIO)
- Real-time operating system (FreeRTOS)
- Flight software execution
- Watchdog and safe mode management

### 1.3 Design Philosophy

The OBC implements a dual-processor architecture:
- **Primary OBC**: Ultra-low-power MCU for continuous operations
- **AI Payload**: Jetson Orin Nano Super for AI inference bursts

---

## 2. OBC Requirements

### 2.1 Functional Requirements

| ID | Requirement | Verification |
|----|-------------|--------------|
| OBC-F001 | Execute flight software in FreeRTOS | Test |
| OBC-F002 | Interface with EPS via I2C | Test |
| OBC-F003 | Interface with Jetson via UART | Test |
| OBC-F004 | Control subsystem power via GPIO | Test |
| OBC-F005 | Store and retrieve persistent data | Test |
| OBC-F006 | Implement hardware watchdog | Test |
| OBC-F007 | Boot to safe mode on watchdog reset | Test |
| OBC-F008 | Provide telemetry collection and formatting | Test |
| OBC-F009 | Generate beacon at configurable intervals | Test |
| OBC-F010 | Detect and log faults | Test |

### 2.2 Performance Requirements

| ID | Requirement | Threshold | Goal |
|----|-------------|-----------|------|
| OBC-P001 | Main loop execution | <20 ms | <10 ms |
| OBC-P002 | Boot time | <30 s | <15 s |
| OBC-P003 | Flash endurance | >10,000 cycles | >100,000 cycles |
| OBC-P004 | RAM availability | >32 KB | >64 KB |
| OBC-P005 | Flash availability | >128 KB | >256 KB |
| OBC-P006 | Power consumption (active) | <200 mW | <150 mW |
| OBC-P007 | Power consumption (sleep) | <10 mW | <5 mW |

### 2.3 Interface Requirements

| ID | Interface | Specification |
|----|-----------|---------------|
| OBC-I001 | I2C Bus 1 | 100 kHz, EPS + sensors |
| OBC-I002 | I2C Bus 2 | 400 kHz, ADCS |
| OBC-I003 | UART 1 | 115200 bps, Jetson |
| OBC-I004 | UART 2 | 9600 bps, Debug |
| OBC-I005 | SPI | 1 MHz, RF transceiver |
| OBC-I006 | GPIO | 16 lines, load control |
| OBC-I007 | ADC | 6 channels, 12-bit |

### 2.4 Environmental Requirements

| Parameter | Operating | Non-Operating |
|-----------|-----------|---------------|
| Temperature | -40°C to +85°C | -55°C to +125°C |
| Vibration | 14.1 Grms (random) | N/A |
| Radiation | 10 krad TID | N/A |
| SEU Rate | <10⁻⁹/bit/day | N/A |

---

## 3. OBC Architecture

### 3.1 Block Diagram

```
+------------------------------------------------------------------+
|                        OBC SUBSYSTEM                               |
|                                                                    |
|  +--------------------+         +--------------------+             |
|  |    STM32L476RG     |         |   External Flash   |             |
|  |    (Primary MCU)   |         |   (Mission Data)   |             |
|  |                    |         |                    |             |
|  | +----------------+ |   SPI   |   W25Q128JVSIQ     |             |
|  | |  ARM Cortex-M4 | |=========|   (16 MB)          |             |
|  | |  80 MHz        | |         +--------------------+             |
|  | +----------------+ |                                            |
|  |                    |         +--------------------+             |
|  | +----------------+ |         |   External FRAM    |             |
|  | |  Flash: 1 MB   | |   SPI   |   (Critical Data)  |             |
|  | |  RAM: 128 KB   | |=========|   FM25V10          |             |
|  | +----------------+ |         |   (1 Mbit)         |             |
|  |                    |         +--------------------+             |
|  | +----------------+ |                                            |
|  | |  Peripherals   | |                                            |
|  | | - I2C x2       | |                                            |
|  | | - UART x3      |-+----> To EPS, ADCS, Jetson, Debug           |
|  | | - SPI x2       | |                                            |
|  | | - GPIO x16     | |                                            |
|  | | - ADC x6       | |                                            |
|  | | - IWDG         | |                                            |
|  | +----------------+ |                                            |
|  +--------------------+                                            |
|                                                                    |
+------------------------------------------------------------------+
           |              |              |              |
           v              v              v              v
      +--------+     +--------+     +--------+     +--------+
      |  EPS   |     |  ADCS  |     | Jetson |     |   RF   |
      | (I2C)  |     | (I2C)  |     | (UART) |     | (SPI)  |
      +--------+     +--------+     +--------+     +--------+
```

### 3.2 Processor Selection

#### 3.2.1 Primary Option: STM32L476RG

| Parameter | Specification |
|-----------|---------------|
| Core | ARM Cortex-M4F |
| Frequency | 80 MHz |
| Flash | 1 MB |
| RAM | 128 KB |
| Power (Run) | 100 μA/MHz |
| Power (Stop2) | 1.1 μA |
| I2C | 3 modules |
| UART | 5 modules |
| SPI | 3 modules |
| ADC | 16-bit, 3 ADCs |
| Package | LQFP64 |
| Temp Range | -40°C to +125°C |

#### 3.2.2 Alternate Option: Ambiq Apollo4 Blue Plus

| Parameter | Specification |
|-----------|---------------|
| Core | ARM Cortex-M4F |
| Frequency | 192 MHz |
| Flash | 2 MB |
| RAM | 2 MB |
| Power (Run) | 3 μA/MHz |
| Power (Sleep) | 1 μA |
| Package | BGA |
| Notes | Ultra-low power, TinyML capable |

### 3.3 Memory Architecture

#### 3.3.1 Memory Map (STM32L476)

| Region | Address | Size | Usage |
|--------|---------|------|-------|
| Flash | 0x0800_0000 | 1 MB | Flight software |
| SRAM1 | 0x2000_0000 | 96 KB | Runtime data |
| SRAM2 | 0x1000_0000 | 32 KB | Stack, critical data |
| EEPROM | 0x0808_0000 | 16 KB | Configuration |
| Option | 0x1FFF_7800 | 1 KB | Boot config |

#### 3.3.2 External Memory

| Device | Interface | Size | Usage |
|--------|-----------|------|-------|
| W25Q128 | SPI | 16 MB | Mission data, logs |
| FM25V10 | SPI | 128 KB | Critical state, fault log |

#### 3.3.3 Memory Allocation

```
+------------------+  0x0810_0000
|  Reserved        |
+------------------+  0x080E_0000
|  Bootloader      |  128 KB
+------------------+  0x080C_0000
|  Flight SW       |  512 KB
|  (Main Image)    |
+------------------+  0x0804_0000
|  Flight SW       |  256 KB
|  (Backup Image)  |
+------------------+  0x0800_0000

+------------------+  0x2001_8000
|  Heap            |  32 KB
+------------------+  0x2001_0000
|  Stack           |  8 KB
+------------------+  0x2000_E000
|  FreeRTOS        |  24 KB
|  Kernel          |
+------------------+  0x2000_8000
|  Application     |  32 KB
|  Data            |
+------------------+  0x2000_0000
```

---

## 4. Peripheral Interfaces

### 4.1 I2C Bus Configuration

#### 4.1.1 I2C Bus 1 (Low Speed)

| Device | Address | Speed | Function |
|--------|---------|-------|----------|
| EPS | 0x48 | 100 kHz | Power telemetry |
| Battery | 0x55 | 100 kHz | SoC, voltage |
| Solar | 0x40 | 100 kHz | Current, MPPT |
| Temperature | 0x48 | 100 kHz | TMP117 sensors |

#### 4.1.2 I2C Bus 2 (ADCS)

| Device | Address | Speed | Function |
|--------|---------|-------|----------|
| Magnetometer | 0x1E | 400 kHz | HMC5883L |
| IMU (spare) | 0x68 | 400 kHz | MPU6050 |

### 4.2 UART Configuration

| Port | Baud | Connection | Protocol |
|------|------|------------|----------|
| UART1 | 115200 | Jetson | Custom (ICD-002) |
| UART2 | 9600 | Debug | ASCII |
| UART3 | 9600 | RF (spare) | AX.25 |

### 4.3 SPI Configuration

| Port | Speed | Connection | Mode |
|------|-------|------------|------|
| SPI1 | 1 MHz | RF Transceiver | Mode 0 |
| SPI2 | 10 MHz | External Flash | Mode 0 |
| SPI3 | 10 MHz | External FRAM | Mode 0 |

### 4.4 GPIO Allocation

| GPIO | Function | Direction | Default |
|------|----------|-----------|---------|
| PA0 | PAYLOAD_EN | Output | LOW |
| PA1 | RADIO_EN | Output | LOW |
| PA2 | ADCS_EN | Output | LOW |
| PA3 | HEATER_EN | Output | LOW |
| PA4 | SENSOR_EN | Output | HIGH |
| PA5 | SPARE_EN | Output | LOW |
| PB0 | DEPLOY_1 | Output | LOW |
| PB1 | DEPLOY_2 | Output | LOW |
| PB2 | DEPLOY_SENSE | Input | - |
| PC0 | JETSON_RDY | Input | - |
| PC1 | EPS_INT | Input | - |
| PC2 | WDT_KICK | Output | - |
| PC3 | LED_STATUS | Output | LOW |

### 4.5 ADC Channels

| Channel | Signal | Range | Resolution |
|---------|--------|-------|------------|
| ADC1_IN0 | SUN_PX | 0-3.3V | 12-bit |
| ADC1_IN1 | SUN_MX | 0-3.3V | 12-bit |
| ADC1_IN2 | SUN_PY | 0-3.3V | 12-bit |
| ADC1_IN3 | SUN_MY | 0-3.3V | 12-bit |
| ADC1_IN4 | SUN_PZ | 0-3.3V | 12-bit |
| ADC1_IN5 | SUN_MZ | 0-3.3V | 12-bit |
| ADC1_IN6 | TEMP_INT | 0-3.3V | 12-bit |

---

## 5. Clock Configuration

### 5.1 Clock Tree

```
                   +--------+
    HSE (8 MHz) -->|  PLL   |--> SYSCLK (80 MHz)
                   +--------+
                       |
         +-------------+-------------+
         |             |             |
         v             v             v
    +--------+    +--------+    +--------+
    | APB1   |    | APB2   |    | AHB    |
    | 80 MHz |    | 80 MHz |    | 80 MHz |
    +--------+    +--------+    +--------+
         |             |             |
         v             v             v
    UART, I2C    SPI, ADC      DMA, Flash
```

### 5.2 Clock Sources

| Source | Frequency | Usage | Accuracy |
|--------|-----------|-------|----------|
| HSE | 8 MHz | PLL input | ±10 ppm |
| LSE | 32.768 kHz | RTC | ±20 ppm |
| HSI | 16 MHz | Backup | ±1% |
| LSI | 32 kHz | IWDG | ±5% |

### 5.3 Real-Time Clock

| Parameter | Specification |
|-----------|---------------|
| Source | LSE (32.768 kHz) |
| Battery | CR1220 backup |
| Accuracy | ±2 s/day |
| Epoch | Unix time (seconds since 1970) |
| Calendar | BCD format |

---

## 6. Watchdog Design

### 6.1 Independent Watchdog (IWDG)

| Parameter | Value |
|-----------|-------|
| Clock | LSI (32 kHz) |
| Prescaler | 256 |
| Reload | 3750 |
| Timeout | 30 seconds |
| Reset Type | System reset |

### 6.2 Watchdog Kick Pattern

```c
// Main loop watchdog kick
void main_loop(void) {
    while (1) {
        // Execute tasks
        eps_task();
        beacon_task();
        telemetry_task();

        // Kick watchdog only if all critical tasks complete
        if (eps_healthy() && comms_healthy()) {
            hal_wdt_kick();
        }

        // Sleep until next tick
        vTaskDelay(pdMS_TO_TICKS(MAIN_LOOP_MS));
    }
}
```

### 6.3 Watchdog Reset Recovery

```
                   +-------------+
                   | Watchdog    |
                   | Reset       |
                   +------+------+
                          |
                          v
                   +-------------+
                   | Bootloader  |
                   | Check flags |
                   +------+------+
                          |
          +---------------+---------------+
          |                               |
          v                               v
   +-------------+                 +-------------+
   | Normal Boot |                 | Safe Mode   |
   | (reset_cnt  |                 | (reset_cnt  |
   |    < 3)     |                 |    >= 3)    |
   +-------------+                 +-------------+
```

---

## 7. Boot Sequence

### 7.1 Normal Boot

```
1. Power On
   |
   v
2. Hardware Init (HAL)
   - Clock configuration
   - GPIO defaults
   - Watchdog start
   |
   v
3. Memory Init
   - Stack check
   - Heap init
   - FRAM read
   |
   v
4. FreeRTOS Start
   - Scheduler init
   - Task creation
   |
   v
5. Subsystem Init
   - EPS interface
   - Sensor init
   - Comms init
   |
   v
6. Main Application
   - Enter SAFE mode
   - Begin telemetry
```

### 7.2 Safe Mode Boot

| Action | Description |
|--------|-------------|
| Disable Jetson | Keep payload off |
| Disable RF | TX only essential beacons |
| Minimum telemetry | Basic health only |
| Extended beacon | 180s interval |
| Wait for recovery | Power or thermal improvement |

### 7.3 Boot Time Budget

| Phase | Time | Cumulative |
|-------|------|------------|
| Reset vector | 1 ms | 1 ms |
| Clock init | 10 ms | 11 ms |
| HAL init | 50 ms | 61 ms |
| FreeRTOS init | 20 ms | 81 ms |
| Subsystem init | 200 ms | 281 ms |
| FRAM load | 50 ms | 331 ms |
| First beacon | 100 ms | 431 ms |
| **Total** | | **<500 ms** |

---

## 8. Software Architecture

### 8.1 FreeRTOS Configuration

| Parameter | Value |
|-----------|-------|
| Version | 10.4.3 |
| Tick Rate | 1000 Hz |
| Max Priorities | 8 |
| Stack Size (default) | 256 words |
| Heap Size | 32 KB |
| Heap Algorithm | heap_4 (coalesce) |

### 8.2 Task Configuration

| Task | Priority | Stack | Period |
|------|----------|-------|--------|
| Main | 4 | 512 | 100 ms |
| EPS | 5 | 256 | 1000 ms |
| Beacon | 3 | 512 | 30-180 s |
| Telemetry | 3 | 256 | 1000 ms |
| ADCS | 4 | 256 | 1000 ms |
| Jetson Comm | 2 | 512 | Event |
| Fault Monitor | 6 | 256 | 100 ms |
| Idle | 0 | 128 | N/A |

### 8.3 Inter-Task Communication

```
+------------+     +------------+
|  EPS Task  |     | Main Task  |
+-----+------+     +-----+------+
      |                  |
      | PowerMode_t      | Commands
      v                  v
+----------+        +----------+
| EPS      |        | Command  |
| Queue    |        | Queue    |
+----------+        +----------+
      |                  |
      +--------+---------+
               |
               v
         +----------+
         | Beacon   |
         | Task     |
         +----------+
```

---

## 9. Telemetry Collection

### 9.1 Telemetry Structure

```c
typedef struct {
    // Header
    uint32_t timestamp;          // Unix time
    uint16_t sequence;           // Packet sequence
    uint8_t  power_mode;         // SAFE/IDLE/ACTIVE

    // EPS Data
    uint16_t battery_voltage_mv;  // mV
    int16_t  battery_current_ma;  // mA
    uint8_t  state_of_charge;     // %
    uint16_t solar_current_ma;    // mA

    // Thermal
    int8_t   temp_obc;           // °C
    int8_t   temp_battery;       // °C
    int8_t   temp_payload;       // °C

    // ADCS
    int16_t  mag_x, mag_y, mag_z; // μT * 10
    uint16_t sun_sensors[6];      // ADC counts

    // Status
    uint8_t  fault_flags;        // Bitmask
    uint16_t reset_count;        // Total resets
    uint32_t uptime_s;           // Seconds since boot

    // CRC
    uint32_t crc32;              // Data integrity
} Telemetry_t;
```

### 9.2 Telemetry Rates

| Data Type | Collection Rate | Downlink Rate |
|-----------|-----------------|---------------|
| Housekeeping | 1 Hz | Every beacon |
| EPS | 1 Hz | Every beacon |
| ADCS | 10 Hz | Averaged per beacon |
| Thermal | 0.1 Hz | Every beacon |
| Fault Log | Event | On change |

---

## 10. Verification

### 10.1 Verification Matrix

| Requirement | Method | Status |
|-------------|--------|--------|
| OBC-F001 | Demonstration | Pending |
| OBC-F002 | Test | Pending |
| OBC-F003 | Test | Pending |
| OBC-F004 | Test | Pending |
| OBC-F005 | Test | Pending |
| OBC-F006 | Test | Pending |
| OBC-F007 | Test | Pending |
| OBC-F008 | Test | Pending |
| OBC-F009 | Test | Pending |
| OBC-F010 | Test | Pending |

### 10.2 Test Procedures

| Test | Description | Equipment |
|------|-------------|-----------|
| TP-OBC-001 | Boot sequence timing | Logic analyzer |
| TP-OBC-002 | Watchdog timeout | Oscilloscope |
| TP-OBC-003 | I2C communication | Bus analyzer |
| TP-OBC-004 | UART communication | Serial monitor |
| TP-OBC-005 | Flash endurance | Cycle tester |
| TP-OBC-006 | Power consumption | Power analyzer |
| TP-OBC-007 | Thermal cycling | Thermal chamber |

---

## 11. Risk Assessment

### 11.1 OBC-Specific Risks

| Risk ID | Description | L | C | Mitigation |
|---------|-------------|---|---|------------|
| OBC-R001 | Flash corruption | L | H | CRC, backup image |
| OBC-R002 | SEU in RAM | M | M | Error detection, restart |
| OBC-R003 | I2C bus lockup | L | M | Timeout, bus reset |
| OBC-R004 | Clock failure | L | H | Fallback to HSI |
| OBC-R005 | Watchdog failure | L | H | External WDT backup |

---

## Appendix A: Development Board

**NUCLEO-L476RG** for initial development:

| Feature | Specification |
|---------|---------------|
| MCU | STM32L476RGT6 |
| Flash | 1 MB |
| RAM | 128 KB |
| Debugger | ST-LINK/V2-1 |
| USB | Virtual COM |
| Arduino | Compatible headers |

---

## Appendix B: Programming Interface

```
+----------+     SWD      +----------+
|  Host    |==============|  STM32   |
|  (USB)   |  SWCLK/SWDIO |  L476    |
+----------+              +----------+
     |
     v
OpenOCD / STM32CubeProgrammer
```

**SWD Connections**:
| Signal | Pin |
|--------|-----|
| SWCLK | PA14 |
| SWDIO | PA13 |
| NRST | NRST |
| GND | GND |

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
