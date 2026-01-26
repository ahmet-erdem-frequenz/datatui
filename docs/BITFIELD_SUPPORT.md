# Bitfield Support Documentation

## Overview
The datapoint TUI now supports monitoring individual bits within Modbus registers, which is useful for status registers and control words where each bit has a specific meaning.

## Configuration

### Basic Bitfield Configuration
```yaml
datapoints:
  - name: "Status Register"
    address: 0
    length: 1
    data_type: bitfield
    description: "System status flags"
    bitfields:
      - bit: 0
        name: "Toggle bit"
        description: "Optional description"
      - bit: 1
        name: "IEC Connection OK"
      - bit: 8
        name: "Voltage Control Mode"
      - bit: 10
        name: "Limited Voltage Control"
```

### Key Points:
- **`data_type: bitfield`** - Marks the register as a bitfield
- **`bitfields`** - Array of bit definitions
- **`bit`** - Bit number (0-15 for 16-bit register)
- **`name`** - Display name for the bit
- **`description`** - Optional description (not currently displayed but stored)

## Display Format

### In Main Table:
- **Type column**: Shows "bits" 
- **Value column**: Shows hexadecimal format (e.g., `0x00A1`)

### Bitfield Details Panel:
When you navigate to (select) a bitfield datapoint, a details panel appears showing each configured bit:

```
┌─ Bitfield Details ──────────────────┐
│  Bit  0: ✓ Toggle bit               │
│  Bit  1: ✗ IEC Connection OK        │
│  Bit  8: ✓ Voltage Control Mode     │
│  Bit 10: ✗ Limited Voltage Control  │
└─────────────────────────────────────┘
```

- ✓ (green) = Bit is SET (1)
- ✗ (gray) = Bit is CLEAR (0)

## Example Use Cases

### 1. Status Registers
Monitor system status flags from PLCs, inverters, or RTUs:
```yaml
- name: "Inverter Status"
  address: 1000
  length: 1
  data_type: bitfield
  bitfields:
    - bit: 0
      name: "Running"
    - bit: 1
      name: "Fault"
    - bit: 2
      name: "Warning"
```

### 2. Control Words
Monitor control registers:
```yaml
- name: "Mode Control Word"
  address: 19050
  length: 1
  data_type: bitfield
  bitfields:
    - bit: 0
      name: "Enable"
    - bit: 1
      name: "Reset"
    - bit: 2
      name: "Auto Mode"
```

### 3. Communication Status
```yaml
- name: "Comm Status"
  address: 0
  length: 1
  data_type: bitfield
  bitfields:
    - bit: 0
      name: "Toggle bit"
    - bit: 1
      name: "IEC101/IEC104 Connection OK"
```

## Implementation Details

### Files Modified:
1. **`src/config.rs`**
   - Added `BitfieldConfig` struct
   - Added `Bitfield` to `DataType` enum
   - Added optional `bitfields` field to `DatapointConfig`

2. **`src/datapoint.rs`**
   - Added `Bitfield(u16)` variant to `DataValue`
   - Added `bitfield_names` to `Datapoint` struct
   - Added `get_bitfield_status()` method to extract bit states

3. **`src/scanner.rs`**
   - Modified scanner initialization to handle bitfield configs
   - Added `DataType::Bitfield` case in register reading

4. **`src/ui.rs`**
   - Added `draw_bitfield_details()` function
   - Modified layout to show bitfield panel when selected
   - Displays bits in order with status indicators

## Limitations
- Currently supports 16-bit registers only
- Maximum 16 bits can be defined (bit 0-15)
- Only displays when datapoint is selected (navigate with ↑/↓)

## Example Configs
- **`mid252_config.yaml`** - Full MID252/ABC Klinker configuration
- **`config.yaml`** - Original test configuration

## Testing
```bash
# Start mockbus server on port 2525 with mid252 config
# Then run:
./target/release/datapoint_tui --config mid252_config.yaml

# Navigate to "Status Register 1" with arrow keys
# The bitfield panel will show individual bit states
```
