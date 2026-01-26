# Complete MID252 Configuration

## Summary
The `mid252_config.yaml` file now contains **all 45 registers** from the original MID252 mockbus configuration.

## Register Breakdown

### RTU -> Controller (Read) - 8 registers
- **Address 0**: Status Register 1 (Bitfield with 5 bits)
- **Address 1**: Status Register 2 (U16)
- **Addresses 2-9**: Asset 1-4 Active Power Setpoint % (4x I32)
- **Addresses 84-87**: Reactive Power control registers (2x I32)

### Controller -> RTU (Write) - 37 registers
- **Address 1000**: Status Register 1 (Bitfield with 5 bits)
- **Address 1001**: Status Register 2 (U16)
- **Addresses 1002-1009**: Asset 1-4 Telemetry Active Power Directive % (4x I32)
- **Addresses 1022-1029**: Asset 1-4 Telemetry Active Power (4x I32)
- **Addresses 1042-1049**: Asset 1-4 Available Active Power (4x I32)
- **Addresses 1062-1069**: Asset 1-4 Telemetry Reactive Power (4x I32)
- **Addresses 1082-1089**: Asset 1-4 Available Reactive Power Underexcited (4x I32)
- **Addresses 1102-1109**: Asset 1-4 Available Reactive Power Overexcited (4x I32)
- **Addresses 1122-1129**: Asset 1-4 Available Generators (4x I32)
- **Addresses 1142-1149**: Asset 1-4 External Active Power Reduction (4x I32)
- **Addresses 1164-1167**: Reactive Power control telemetry (2x I32)
- **Address 1192**: Global Irradiance (I32)

## Bitfield Registers (2 total)

### RTU Status Register 1 (Address 0)
- Bit 0: Toggle bit
- Bit 1: IEC101/IEC104 Connection OK
- Bit 8: Reactive Power Voltage Control Mode
- Bit 9: Reserved
- Bit 10: Reactive Power Limited Voltage Control Mode

### Controller Status Register 1 (Address 1000)
- Bit 0: Toggle bit
- Bit 1: IEC101/IEC104 Connection OK
- Bit 8: Reactive Power Voltage Control Mode
- Bit 9: Reserved
- Bit 10: Reactive Power Limited Voltage Control Mode

## Data Types Used
- **Bitfield**: 2 registers (status registers)
- **U16**: 2 registers (status register 2)
- **I32**: 41 registers (all power, control, and telemetry values)

## Usage
```bash
./target/release/datapoint_tui --config mid252_config.yaml
```

## Verification
✅ Original config: 45 register definitions
✅ TUI config: 45 datapoint definitions
✅ All addresses match
✅ All data types appropriate (I32 for multi-register values)
✅ Bitfield support for status registers
