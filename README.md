# Datapoint TUI

A terminal-based dashboard for real-time monitoring of Modbus server datapoints with support for multiple data types including bitfields.

## Features

- **Real-time Monitoring** - Live updates with configurable scan intervals
- **Multiple Data Types** - U16, I16, U32, I32, F32, and Bitfield support
- **Bitfield Visualization** - Individual bit status with named labels
- **IPv4/IPv6 Support** - Full support for both IPv4 and IPv6 addresses
- **Configurable Register Types** - Holding registers (FC3) and Input registers (FC4)
- **Endianness Control** - Big-endian or Little-endian for multi-register values
- **Error Handling** - Clear status indicators and error messages
- **Connection Resilience** - Auto-retry with configurable timeouts
- **Debug Logging** - Optional detailed logging for troubleshooting

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/datapoint_tui`.

## Quick Start

### Basic Usage

```bash
# Run with default config.yaml
./target/release/datapoint_tui

# Run with custom config
./target/release/datapoint_tui --config my_config.yaml

# Enable debug logging
./target/release/datapoint_tui --config my_config.yaml --debug
```

### Configuration Example

```yaml
server:
  protocol: modbus
  host: 192.168.1.100  # IPv4 or IPv6 (e.g., ::1)
  port: 502
  unit_id: 1
  endianness: big      # or 'little'

scan_interval_ms: 1000

datapoints:
  # Simple register
  - name: "Temperature"
    address: 1000
    length: 1
    data_type: i16
    register_type: holding  # or 'input'
    description: "Room temperature sensor"
  
  # Multi-register value
  - name: "Power Output"
    address: 1010
    length: 2
    data_type: i32
    register_type: holding
  
  # Bitfield with named bits
  - name: "Status Register"
    address: 0
    length: 1
    data_type: bitfield
    register_type: holding
    bitfields:
      - bit: 0
        name: "System Ready"
      - bit: 1
        name: "Fault Active"
      - bit: 8
        name: "Manual Mode"
```

## Configuration Reference

### Server Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `protocol` | string | - | Protocol type (currently only "modbus") |
| `host` | string | - | Server IP address (IPv4 or IPv6) |
| `port` | integer | - | Server port (standard Modbus: 502) |
| `unit_id` | integer | 1 | Modbus unit ID |
| `endianness` | string | big | Byte order: "big" or "little" |

### Datapoint Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | - | Display name |
| `address` | integer | - | Register address |
| `length` | integer | - | Number of registers to read |
| `data_type` | string | u16 | Data type (see below) |
| `register_type` | string | holding | Register type: "holding" or "input" |
| `description` | string | - | Optional description |
| `bitfields` | array | - | Bit definitions (for bitfield type only) |

### Data Types

| Type | Registers | Description |
|------|-----------|-------------|
| `u16` | 1 | Unsigned 16-bit integer |
| `i16` | 1 | Signed 16-bit integer |
| `u32` | 2 | Unsigned 32-bit integer |
| `i32` | 2 | Signed 32-bit integer |
| `f32` | 2 | 32-bit floating point |
| `bitfield` | 1 | Bit flags (see bitfield section) |

## Bitfield Support

Bitfields allow monitoring individual bits within a register with custom names:

```yaml
datapoints:
  - name: "System Status"
    address: 0
    length: 1
    data_type: bitfield
    bitfields:
      - bit: 0
        name: "Running"
      - bit: 1
        name: "Fault"
      - bit: 2
        name: "Warning"
      - bit: 8
        name: "Auto Mode"
```

**Display:**
- Main table shows hex value: `0x00A5`
- Select the row to see bit details:
  - ✓ (green) = bit is SET
  - ✗ (gray) = bit is CLEAR

## UI Controls

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate through datapoints |
| `q` | Quit application |
| `Esc` | Quit application |
| `Ctrl+C` | Quit application |

## Status Indicators

| Status | Meaning |
|--------|---------|
| **OK** (green) | Value read successfully |
| **WAITING** (gray) | No data yet |
| **TIMEOUT** (red) | Connection timeout |
| **CONN FAIL** (red) | Connection failed |
| **READ TMO** (red) | Read timeout |
| **MODBUS ERR** (red) | Modbus exception |

## Examples

### Example 1: Simple Monitoring

Monitor a few registers from a local Modbus server:

```yaml
server:
  protocol: modbus
  host: 127.0.0.1
  port: 502
  unit_id: 1

scan_interval_ms: 1000

datapoints:
  - name: "Voltage"
    address: 0
    length: 1
    data_type: u16
  
  - name: "Current"
    address: 1
    length: 1
    data_type: u16
```

### Example 2: Industrial Device (MID252)

Complete configuration for ABC Klinker MID252 device available in `mid252_config.yaml`.

### Example 3: IPv6 Device

```yaml
server:
  protocol: modbus
  host: ::1  # IPv6 localhost
  port: 2525
  unit_id: 1

datapoints:
  - name: "Register 0"
    address: 0
    length: 1
    data_type: u16
```

## Debug Mode

Enable debug logging to troubleshoot connection issues:

```bash
./target/release/datapoint_tui --config config.yaml --debug
```

Creates `datapoint_tui_debug.log` with detailed information:
- Connection attempts and results
- Register read operations
- Raw register values
- Error messages

## Tips

- **IPv6 addresses** are automatically handled (no brackets needed in config)
- **Endianness** only affects multi-register values (U32, I32, F32)
- **Register type** determines Modbus function code (FC3 for holding, FC4 for input)
- **Scan interval** should be at least 100ms; adjust based on device response time
- Use **debug mode** if values don't appear or connections fail

## License

All rights reserved.

## Additional Documentation

Detailed troubleshooting guides and technical documentation available in `docs/` directory.
