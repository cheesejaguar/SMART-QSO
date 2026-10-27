# Software Configuration Index
## SMART-QSO Flight Software

**Document Number:** SQSO-SW-006
**Revision:** C
**Date:** 2026-10-27
**Reference:** NPR 7150.2, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document provides an index of all configurable parameters in the SMART-QSO flight software, including default values, valid ranges, and configuration methods.

## 2. Scope

This document covers:
- OBC flight software configuration
- Build-time configuration
- Runtime configuration
- Hardware-dependent configuration

**Revision B (2026-10-24).** The flight software is Rust (`software/rust/README.md`). The parameter values in §4 to §9
are unchanged. `docs/BASELINE_PARAMETERS.md` governs where they differ, and `scripts/check_parameters.py` checks the key
ones against the code. In the code, a parameter is a constant in its C header (`software/flight/include`), mirrored by a
Rust constant. `software/flight/tests/test_rust_contract.c` checks that the two agree. §3 is updated to the build as it
is. The preprocessor defines in §3.2, the stack and heap sizes in §3.3, the storage map in §10.1 and the configuration
commands in §10.2 are not in the code. The storage map as built is `software/flight/docs/MEMORY_POLICY.md` §8.1, and the
telecommands are the ones in `software/flight/command_dictionary.json`.

**Revision C (2026-10-27).** The values of §4 to §9 that Revision A took from an earlier spacecraft (a 2S battery, a
2 W transmitter, 145.825 MHz, 20/30/60 s beacons, preemptive tasks) are replaced by the baseline and the code, each row
with its source. A row marked "not in the code" names a Revision A parameter that the flight software does not have.

---

## 3. Build-Time Configuration

### 3.1 CMake Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| FLIGHT_BUILD | BOOL | OFF | Enable flight configuration |
| ENABLE_COVERAGE | BOOL | OFF | Enable code coverage |
| ENABLE_SANITIZERS | BOOL | OFF | Enable address sanitizers |
| TARGET_PLATFORM | STRING | LINUX | Target: LINUX, STM32L4, FREERTOS |

Revision B, as built (`software/flight/CMakeLists.txt`): `FLIGHT_BUILD`, `ENABLE_COVERAGE`, `ENABLE_SANITIZERS` and
`ENABLE_WERROR` (ON). There is no `TARGET_PLATFORM`. The host build always targets Linux or macOS, and the STM32L4R5ZI
image is built by cargo in `software/rust/firmware` (`docs/software/BUILD_CONFIGURATIONS.md` §2.1). Other build-time
settings:

| Setting | Where | Effect |
|---------|-------|--------|
| cargo feature `flight` (crate `sqso-sim`) | set by `FLIGHT_BUILD` | The host simulation runs until stopped instead of 1000 passes |
| `target_os = "none"` | the target build | Selects the firmware HAL, FRAM storage and the debug-UART console instead of the host's files, termios and stderr |
| Toolchain | `software/rust/rust-toolchain.toml` | rustc 1.97.0 |

Host-only run-time settings (environment variables of the simulation and the tests): `SENSORS_YAML` (sensor table
file), `SMART_QSO_FAULT_LOG`, `SMART_QSO_EPS_CONFIG` and `SMART_QSO_MISSION_DATA` (storage files), and `UART_DEVICE`,
`UART_BAUDRATE` and `TELEMETRY_INTERVAL_MS` (Jetson serial line). The target has none of these.

### 3.2 Preprocessor Defines

| Define | Default | Description |
|--------|---------|-------------|
| DEBUG_ENABLED | 1 (dev) / 0 (flight) | Enable debug output |
| WATCHDOG_ENABLED | 0 (dev) / 1 (flight) | Enable hardware watchdog |
| SIMULATION_MODE | 1 (dev) / 0 (flight) | Enable HAL simulation |
| LOG_LEVEL | LOG_DEBUG / LOG_INFO | Logging verbosity |

### 3.3 Memory Configuration

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| STACK_SIZE_MAIN | 4096 | 2048-8192 | bytes |
| STACK_SIZE_SENSOR | 1024 | 512-2048 | bytes |
| STACK_SIZE_BEACON | 2048 | 1024-4096 | bytes |
| HEAP_SIZE | 32768 | 16384-65536 | bytes |
| TLM_BUFFER_COUNT | 8 | 4-16 | count |
| FAULT_LOG_SIZE | 100 (fixed; `sqso-fsw/src/fault_log.rs` `MAX_ENTRIES`) | — | entries |

---

## 4. Power Management Configuration

### 4.1 Power Mode Thresholds

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| ACTIVE allowed (sunlit) | SOC ≥ 55 | fixed | % |
| ACTIVE → IDLE | SOC < 40 | fixed | % |
| → SAFE | SOC < 25 | fixed | % |
| SAFE → EMERGENCY (beacon shed) | SOC < 15 | fixed | % |

Modes are set by state of charge, not by voltage (BASELINE §5; `eps_control.h`, `sm_power_event()`); back to SAFE at
25 %, to IDLE above 40 %. Revision A's voltage thresholds (6.5 to 7.8 V) were for a 2S pack and do not apply to the 4S
pack (14.4 V nominal, charged to 16.4 V).

### 4.2 Power Limits

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| MAX_SYSTEM_POWER | Not in the code (load allocations: BASELINE §5) | — | W |
| JETSON_POWER_LIMIT | 8.0 input (latch-up trip 12 W; `payload_guard.rs`) | fixed | W |
| RF_POWER_HIGH | 1.0 (maximum; `SET_POWER` 1) | fixed | W |
| RF_POWER_LOW | 0.5 (nominal; `SET_POWER` 0) | fixed | W |

### 4.3 Battery Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| BATTERY_CAPACITY | ≈ 3500 (4S1P 18650, ≈ 50 Wh; BASELINE §5) | — | mAh |
| MAX_DOD | 10 nominal per eclipse, 30 contingency (SYS-112) | — | % |
| CHARGE_TEMP_MIN | 0 | -10 to +10 | C |
| CHARGE_TEMP_MAX | 45 | 40-50 | C |
| DISCHARGE_TEMP_MIN | -20 | -30 to -10 | C |
| DISCHARGE_TEMP_MAX | 60 | 50-70 | C |

---

## 5. Communication Configuration

### 5.1 Beacon Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| BEACON_INTERVAL_SAFE | 180 | — | seconds |
| BEACON_INTERVAL_IDLE | 120 | — | seconds |
| BEACON_INTERVAL_ACTIVE | 60 (± 5 s jitter) | `SET_BEACON` 10-120 | seconds |
| BEACON_MAX_LENGTH | 256 (information field; text ≤ 176) | fixed | bytes |
| BEACON_CALLSIGN | "KK6GDB-11" (`station_id.h`) | fixed | string |

Intervals: `BEACON_INTERVAL_*_MS` in `beacon.h`, BASELINE §8, `docs/BEACON_SPEC.md` §7.

### 5.2 RF Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| TX_FREQUENCY | One IARU-coordinated frequency in 145.800-146.000 MHz, not 145.825 MHz (BASELINE §7); set in the RF board, not by the flight software | — | Hz |
| TX_DEVIATION | Not in the code (RF board, `hardware/electrical/rf/RF_BOARD_DESIGN.md`) | — | Hz |
| TX_POWER_LEVEL | 0 (0.5 W) or 1 (1 W), `SET_POWER` | 0-1 | enum |
| BAUD_RATE | 1200 (AFSK, in the TNC) | fixed | bps |
| PREAMBLE_FLAGS | TXDELAY 300 ms, a TNC parameter (`downlink.h`, TBR-SRS-003) | — | ms |

### 5.3 UART Configuration

| Interface | Baud | Data | Parity | Stop |
|-----------|------|------|--------|------|
| DEBUG (USART3) | 115200 | 8 | None | 1 |
| JETSON (USART1) | 115200 | 8 | None | 2 on the target (1 on the host) |
| TNC (USART2, KISS) | 9600 | 8 | None | 1 |

The EPS controller is on I2C2 (address 0x48), not a UART (`software/rust/README.md`, The firmware).

---

## 6. ADCS Configuration

### 6.1 Sensor Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| MAG_SAMPLE_RATE | 10 | 1-50 | Hz |
| MAG_FILTER_ALPHA | 0.3 | 0.1-0.9 | - |
| SUN_SAMPLE_RATE | 5 | 1-20 | Hz |
| SUN_THRESHOLD | 0.1 | 0.05-0.5 | V |

### 6.2 Control Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| BDOT_GAIN | 1.0e4 | 3e3-3e4 | A·m²·s/T |
| DETUMBLE_THRESHOLD | 0.5 (rate across +Y) | 0.2-2.0 | deg/s |
| SPIN_RATE | 8.0 (about +Y; `adcs.rs` `SPIN_RATE`, BASELINE §8 [D9]) | fixed (the spin axis and inertia presets for a one-wing deployment: `ADCS_SET_GEOMETRY`) | deg/s |
| MTQ_MAX_DIPOLE | 0.2 | 0.1-0.5 | Am^2 |
| CONTROL_PERIOD | 1000 | 100-5000 | ms |

### 6.3 Calibration Data

| Sensor | Offset X | Offset Y | Offset Z | Unit |
|--------|----------|----------|----------|------|
| Magnetometer | 0.0 | 0.0 | 0.0 | uT |
| Sun Sensor +X | 0.0 | - | - | V |
| Sun Sensor -X | 0.0 | - | - | V |
| Sun Sensor +Y | 0.0 | - | - | V |
| Sun Sensor -Y | 0.0 | - | - | V |
| Sun Sensor +Z | 0.0 | - | - | V |
| Sun Sensor -Z | 0.0 | - | - | V |

---

## 7. Timing Configuration

### 7.1 Task Periods

| Task | Period | Priority | Stack |
|------|--------|----------|-------|
| Main loop (one Embassy task, every stage) | 20 ms wait after each pass | — | one 8 KB stack |

There are no separate tasks (`software/flight/docs/RTOS_ARCHITECTURE.md` §4); slower activities are timed inside the
pass (health check 10 s, Jetson telemetry 60 s, beacon by mode).

### 7.2 Watchdog Timeouts

| Watchdog | Timeout | Action |
|----------|---------|--------|
| Hardware WDT | 30000 ms | Reset |
| Stage supervision | 1 s (health 20 s); 3 misses stop the hardware refresh | Reset by the IWDG |
| Jetson telemetry | 5 s without telemetry (`payload_guard.rs`) | Jetson powered off |
| Loss of contact | 30 days at launch, 14 days after commissioning (`SET_CONTACT_TIMEOUT`, 1-90) | Transmit inhibit, receiver on (BASELINE §7) |

### 7.3 Deployment Timers

| Timer | Value | Valid Range | Unit |
|-------|-------|-------------|------|
| DEPLOY_DELAY (after separation) | 1800 (`deployment.rs` `WAIT_TIME_MS`; CDS 2.4.4) | fixed | seconds |
| SOLAR_BURN_TIME (per resistor, one at a time) | 10 (decision C-17) | fixed | seconds |
| ANTENNA_BURN_TIME (AntS sequence) | 45 | fixed | seconds |
| DEPLOY_RETRY_DELAY | 3000 (50 min) | fixed | seconds |
| MAX_DEPLOY_ATTEMPTS | 3, then `DEPLOY_RETRY` by ground | fixed | count |

No RF before 45 min (CDS 2.4.5; `rf_silence.rs`). Wings first, then the antennas (BASELINE §4).

---

## 8. Fault Management Configuration

### 8.1 Fault Thresholds

| Fault | Threshold | Count | Window | Action |
|-------|-----------|-------|--------|--------|
| Low SOC | < 25 % / < 15 % | — | — | SAFE / EMERGENCY (BASELINE §5) |
| Cell over-voltage | > 4.25 V/cell | — | - | EPS hardware protection (BASELINE §5) |
| Jetson over-temperature | spreader > 70 °C, junction > 95 °C | — | — | Jetson off, 10 min cool-down (`payload_guard.rs`) |
| Transmitter over-temperature | > 70 °C | — | — | Downlink held until < 60 °C (`downlink.rs`) |
| Loss of contact | 30 d / 14 d | 1 | - | Transmit inhibit (BASELINE §7) |
| Watchdog | - | 3 | 1 hr | Reset + log |

### 8.2 Recovery Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| AUTO_RECOVERY_DELAY | 3600 | 600-86400 | seconds |
| MAX_RECOVERY_ATTEMPTS | 3 | 1-10 | count |
| SAFE_MODE_DURATION | 86400 | 3600-604800 | seconds |

---

## 9. AI Payload Configuration

### 9.1 Jetson Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| JETSON_BOOT_TIMEOUT | 30 | 20-60 | seconds |
| JETSON_INFERENCE_TIMEOUT | 10 | 5-30 | seconds |
| JETSON_HEALTH_INTERVAL | 5 | 1-30 | seconds |
| JETSON_POWER_MODE | 7 W (nvpmodel 1, original flash configuration; `software/payload/platform_config.py`) | fixed | - |
| MAX_BURST_DURATION | 600 including boot, then ≥ 100 min off (BASELINE §6; `payload_schedule.rs`) | fixed | seconds |

### 9.2 Model Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| MODEL_PATH | `/models/SmolLM2-360M-Instruct-Q4_K_M.gguf` (`LLAMA_MODEL`; gated experiment, D5) | Model location |
| MAX_TOKENS | 32 | Max output tokens (`software/payload/llama_runner.py`) |
| TEMPERATURE | 0.3 | Generation temperature |
| TOP_K | 3 | Sampling (no nucleus sampling) |

---

## 10. Configuration Storage

### 10.1 Non-Volatile Storage Map

| Address | Parameter Block | Size | CRC |
|---------|-----------------|------|-----|
| 0x0000 | System config | 256 B | Yes |
| 0x0100 | Power config | 128 B | Yes |
| 0x0180 | Comm config | 128 B | Yes |
| 0x0200 | ADCS config | 256 B | Yes |
| 0x0300 | Timing config | 128 B | Yes |
| 0x0380 | Fault config | 128 B | Yes |
| 0x0400 | Calibration | 256 B | Yes |

### 10.2 Configuration Commands

| Command | Description |
|---------|-------------|
| CFG_LOAD | Load config from NVM |
| CFG_SAVE | Save config to NVM |
| CFG_DEFAULT | Reset to defaults |
| CFG_DUMP | Output all config |

---

## 11. Flight vs. Test Configuration

| Parameter | Flight | Test | Flatsat |
|-----------|--------|------|---------|
| WATCHDOG_ENABLED | 1 | 0 | 0 |
| DEBUG_ENABLED | 0 | 1 | 1 |
| TX_POWER_LEVEL | 1-3 | 0 | 0 |
| SIMULATION_MODE | 0 | 0 | 1 |
| LOG_LEVEL | INFO | DEBUG | DEBUG |
| DEPLOYMENT_ENABLED | 1 | 0 | 0 |

---

## 12. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-24 | The flight software is Rust: parameters are C header constants mirrored in Rust and checked by the contract test; §3 build-time settings as built (CMake options, cargo feature, host environment variables); items not in the code identified. No parameter value changed | SMART-QSO Team |
| C | 2026-10-27 | Documentation audit (post-CDR 2026): §4 to §9 values replaced from the baseline and the code (SOC mode thresholds, 1 W maximum, beacon intervals, call sign, UARTs, spin rate, watchdog and contact timeouts, deployment timers, Jetson and model settings); FAULT_LOG_SIZE 100 | SMART-QSO Team |
