# UART Telemetry to Payload Jetson

## Overview

The flight software (Rust; the executive is `software/rust/sqso-ffi/src/executive.rs`) sends a `TELEMETRY` line to the
Jetson over its UART while the payload is powered, followed by a `BEACON_REQ`. The line format, CRC and the other
message types are defined by `docs/interfaces/ICD_OBC_JETSON.md` §4, which governs; this page describes the host
simulation's configuration and the fields the OBC adds. The Jetson side is `software/payload/obc_link.py` and
`agent.py` (`software/payload/README.md`).

## Features

- **Telemetry every interval** (default 60 s, `TELEMETRY_INTERVAL_MS`) while the payload is on; none while it is off,
  and the first one at once when it comes back on
- **Sealed lines**: every line ends with a CRC-16/CCITT-FALSE field and is at most 512 bytes (ICD §4.1)
- **Contents**: the fixed ICD fields, the sensor readings in table order, the EPS switches, the last fault, UART health
  and the watchdog state
- **Error recovery**: UART reinitialization after failures; periodic UART health checks

## Configuration

### Environment Variables (host simulation only)

On the host the executive reads these; on the target the Jetson link is USART1 (PA9/PA10) at 115200 baud, 8N2
(`software/rust/README.md`, The firmware).

```bash
# UART device (default: /dev/ttyUSB0)
export UART_DEVICE="/dev/ttyACM0"

# Baud rate (default: 115200)
export UART_BAUDRATE="115200"

# Telemetry interval in milliseconds (default: 60000 = 60 seconds)
export TELEMETRY_INTERVAL_MS="30000"

# Sensor configuration file (default: software/flight/sensors.yaml)
export SENSORS_YAML="custom_sensors.yaml"
```

### Supported Baud Rates

9600, 19200, 38400, 57600, 115200 (default) and 230400 (`sqso-fsw/src/uart_comm.rs`). An unsupported rate falls back
to the default.

### Telemetry Interval Range

- **Minimum**: 1 millisecond
- **Maximum**: 3,600,000 milliseconds (1 hour)
- **Default**: 60,000 milliseconds (60 seconds)

## Telemetry Format

```
TELEMETRY,<time_ms>,<SUNLIT|ECLIPSE>,<soc>,<power_mode>,<power_limit_w>,<sensor>=<value>,...,EPS_PAYLOAD=<ON|OFF>,EPS_RADIO=..,EPS_ADCS=..,EPS_BEACON=..,LAST_FAULT=..,FAULT_COUNT=..,UART_HEALTH=<OK|FAIL>,WATCHDOG=<OK|TRIGGERED>,<CRC>
```

The ICD golden line (both test suites check these bytes):

```
TELEMETRY,1704153600000,SUNLIT,0.750,2,3.0,BV=8.1,EPS_PAYLOAD=ON,9C86
```

### Fields

- **time_ms**: milliseconds of the OBC clock
- **SUNLIT / ECLIPSE**
- **soc**: battery state of charge, 0.000 to 1.000
- **power_mode**: 0 SAFE, 1 IDLE, 2 ACTIVE
- **power_limit_w**: the power limit of the mode
- **KEY=VALUE fields**: the sensors in table order, then the status fields above. If the sensor table is too long for
  the line, the status fields that do not fit are left out (tested by `Executive_Jetson_guard`,
  `software/flight/tests/README.md`)
- **CRC**: four upper-case hex digits over every byte before the last comma (ICD §4.4)

## Sensor Configuration

Sensors are configured in `sensors.yaml` and include:

- **SET**: Spacecraft Elapsed Timer
- **BV**: Battery Voltage
- **BT**: Battery Temperature
- **BUSV**: Bus Voltage
- **BDI**: Battery Discharge Current
- **JPC**: Jetson Payload Current
- **PT**: Payload Heat Spreader Temperature
- **SPV**: Solar Panel Voltage
- **SPC**: Solar Panel Current
- **ST**: Status Hex (bitmap)

## UART Communication

### Hardware Requirements

- UART interface (USB-to-Serial, GPIO UART, etc.)
- Proper wiring between OBC and Jetson
- Ground connection

### Protocol

- **Data format**: 8N1 on the host; 8N2 on the target (the module's receiver needs two stop bits)
- **Flow control**: none
- **Termination**: a single LF (`\n`)
- **Encoding**: printable ASCII (0x20 to 0x7E)

### Error Handling

- **Automatic recovery**: UART reinitialization on communication failures
- **Health monitoring**: periodic UART status checks (every 10 s)
- **Graceful degradation**: telemetry is not sent if the UART is unavailable; the OBC beacons with its templates

## Usage Examples

### Basic Operation

```bash
# From the repository root: build and run the host simulation
cmake -S software/flight -B build && cmake --build build
./build/smart_qso_flight
```

### Development/Testing

```bash
# Use a pseudo-terminal as the Jetson port and faster telemetry
export UART_DEVICE="/dev/pts/1"
export TELEMETRY_INTERVAL_MS="5000"
./build/smart_qso_flight
```

`software/flight/tests/integration/test_executive_jetson.c` does this with two pseudo-terminals and checks every line
against the ICD.

## Monitoring and Debugging

### Log Messages

The software provides detailed logging:

```
[UART] Configuration: device=/dev/ttyUSB0, baudrate=115200, interval=60000ms
[UART] Initialized /dev/ttyUSB0 at 115200 baud
[UART] Sent telemetry to Jetson (156 bytes): TELEMETRY,1234567890,SUNLIT,0.750,...
[UART] Health check failed, attempting recovery
```

### Common Issues

1. **Permission Denied**: Ensure user has access to UART device
2. **Device Not Found**: Check UART device path and connections
3. **Baud Rate Mismatch**: Verify Jetson UART configuration matches
4. **No Telemetry**: Confirm Jetson is powered on and UART initialized

### Troubleshooting

```bash
# Check UART device permissions
ls -l /dev/ttyUSB0

# Test UART communication manually
stty -F /dev/ttyUSB0 115200
echo "TEST" > /dev/ttyUSB0

# Monitor UART traffic
cat /dev/ttyUSB0
```

## Integration with Jetson

### Jetson Side Requirements

- UART driver support
- Telemetry packet parser
- Configurable baud rate matching
- Proper termination handling

### Recommended Jetson Setup

Use `software/payload/obc_link.py` (the same Rust parser as the OBC, through `sqso_protocol`), which checks the CRC and
the format; do not split lines by hand. `software/payload/agent.py` answers `BEACON_REQ` and runs on stdin/stdout or
under the supervisor (`software/payload/README.md`).

## Performance Considerations

- **Memory Usage**: one line of at most 512 bytes (`LINE_MAX`, ICD §4.1)
- **CPU Overhead**: Minimal impact on main sensor polling loop
- **UART Bandwidth**: at most 512 bytes per interval (68 bps average at 60 s)
- **Power Consumption**: UART active only during transmission

## Future Enhancements

- **Compression**: Reduce telemetry packet size
- **Selective Sensors**: Configurable sensor subset for telemetry
- **Dynamic Interval**: Adaptive telemetry rate based on mission phase
