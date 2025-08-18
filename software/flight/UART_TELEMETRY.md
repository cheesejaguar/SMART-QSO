# UART Telemetry to Payload Jetson

## Overview

The SMART-QSO flight software automatically reports all sensor values via UART to the payload Jetson when it is powered on. This provides real-time telemetry data for mission monitoring and payload operations.

## Features

- **Automatic Telemetry**: All sensor values are sent every 60 seconds (configurable)
- **Conditional Transmission**: Telemetry only sent when Jetson is powered on
- **Comprehensive Data**: Includes all sensor readings, mission state, and UART health
- **Configurable Parameters**: UART device, baud rate, and telemetry interval
- **Error Recovery**: Automatic UART reinitialization on communication failures
- **Health Monitoring**: Periodic UART health checks and status reporting

## Configuration

### Environment Variables

The UART telemetry can be configured using environment variables:

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

- 9600
- 19200
- 38400
- 57600
- 115200 (default)
- 230400

### Telemetry Interval Range

- **Minimum**: 1 millisecond
- **Maximum**: 3,600,000 milliseconds (1 hour)
- **Default**: 60,000 milliseconds (60 seconds)

## Telemetry Format

Each telemetry packet contains:

```
TELEMETRY,<timestamp>,<sunlit_state>,<battery_soc>,<sensor1>=<value1><units1>,<sensor2>=<value2><units2>,...,UART_HEALTH=<status>
```

### Example Output

```
TELEMETRY,1234567890,SUNLIT,0.750,BV=12.450V,BT=22.500C,BUSV=5.000V,BDI=0.050A,JPC=0.000A,SPV=7.500V,SPC=0.600A,ST=03hex,UART_HEALTH=OK
```

### Fields

- **timestamp**: Milliseconds since program start
- **sunlit_state**: "SUNLIT" or "ECLIPSE"
- **battery_soc**: Battery state of charge (0.0 to 1.0)
- **sensor values**: All configured sensors with ID, value, and units
- **UART_HEALTH**: "OK" or "FAIL"

## Sensor Configuration

Sensors are configured in `sensors.yaml` and include:

- **SET**: Spacecraft Elapsed Timer
- **BV**: Battery Voltage
- **BT**: Battery Temperature
- **BUSV**: Bus Voltage
- **BDI**: Battery Discharge Current
- **JPC**: Jetson Payload Current
- **SPV**: Solar Panel Voltage
- **SPC**: Solar Panel Current
- **ST**: Status Hex (bitmap)

## UART Communication

### Hardware Requirements

- UART interface (USB-to-Serial, GPIO UART, etc.)
- Proper wiring between OBC and Jetson
- Ground connection

### Protocol

- **Data Format**: 8N1 (8 data bits, no parity, 1 stop bit)
- **Flow Control**: None (hardware flow control disabled)
- **Termination**: Newline character (`\n`)
- **Encoding**: ASCII

### Error Handling

- **Automatic Recovery**: UART reinitialization on communication failures
- **Health Monitoring**: Periodic UART status checks
- **Graceful Degradation**: Telemetry disabled if UART unavailable
- **Retry Logic**: One retry attempt on write failures

## Usage Examples

### Basic Operation

```bash
# Run with default settings
cd software/flight
make
./smart_qso_flight
```

### Custom Configuration

```bash
# Custom UART device and faster telemetry
export UART_DEVICE="/dev/ttyACM0"
export TELEMETRY_INTERVAL_MS="30000"
./smart_qso_flight
```

### Development/Testing

```bash
# Use virtual serial port for testing
export UART_DEVICE="/dev/pts/1"
export TELEMETRY_INTERVAL_MS="5000"
./smart_qso_flight
```

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

```python
import serial
import time

# Configure UART
ser = serial.Serial(
    port='/dev/ttyTHS1',  # Jetson UART port
    baudrate=115200,
    bytesize=serial.EIGHTBITS,
    parity=serial.PARITY_NONE,
    stopbits=serial.STOPBITS_ONE,
    timeout=1
)

# Read telemetry
while True:
    if ser.in_waiting:
        line = ser.readline().decode('ascii').strip()
        if line.startswith('TELEMETRY'):
            # Parse telemetry data
            parts = line.split(',')
            timestamp = int(parts[1])
            sunlit = parts[2]
            soc = float(parts[3])
            # Process sensor data...
    time.sleep(0.1)
```

## Performance Considerations

- **Memory Usage**: Telemetry buffer limited to 1024 bytes
- **CPU Overhead**: Minimal impact on main sensor polling loop
- **UART Bandwidth**: ~156 bytes every 60 seconds = ~21 bps average
- **Power Consumption**: UART active only during transmission

## Future Enhancements

- **Compression**: Reduce telemetry packet size
- **Selective Sensors**: Configurable sensor subset for telemetry
- **Binary Protocol**: More efficient data encoding
- **Acknowledgment**: Jetson confirmation of received data
- **Dynamic Interval**: Adaptive telemetry rate based on mission phase
