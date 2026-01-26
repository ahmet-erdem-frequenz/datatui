# Troubleshooting Guide

## Issue: Values Never Arrive (Status Always "WAITING")

### Symptoms
- TUI shows "WAITING" status
- Values never update
- No error messages
- Your CLI tool can read the values successfully

### Possible Causes & Solutions

#### 1. Wrong Register Type (Most Common)
Modbus has two types of read registers:
- **Holding Registers** (Function Code 3) - Default
- **Input Registers** (Function Code 4)

Your device might be using **Input Registers** instead of Holding Registers.

**Solution:** Add `register_type: input` to your datapoint configs:

```yaml
datapoints:
  - name: "My Datapoint"
    address: 0
    length: 1
    data_type: u16
    register_type: input  # Add this line!
```

**Test with CLI:**
```bash
# Test with holding registers (function code 3)
modbus-cli --ip 127.0.0.1 --port 2525 --start-register 0 read --fn-code 3

# Test with input registers (function code 4)
modbus-cli --ip 127.0.0.1 --port 2525 --start-register 0 read --fn-code 4
```

If function code 4 works, use `register_type: input` in your config.

#### 2. Wrong Unit ID
Check if your device uses a different unit ID (default is 1):

```yaml
server:
  unit_id: 1  # Try different values: 0, 1, 2, etc.
```

#### 3. Wrong Endianness
For multi-register values (i32, u32, f32):

```yaml
server:
  endianness: big  # Try 'little' if values look wrong
```

#### 4. Connection Issues
- Check IP and port are correct
- Ensure device is reachable (ping test)
- Check firewall rules

#### 5. Wrong Address
Modbus addresses can be confusing:
- Some devices use 0-based addressing
- Some use 1-based addressing
- CLI tool has `--offset-zero` flag

If CLI uses `--offset-zero`, you might need to adjust addresses in the config.

## Debugging Steps

### Step 1: Enable Debug Logging
Run with the `--debug` flag to create a detailed log file:

```bash
./target/release/datapoint_tui --config mid252_config.yaml --debug
```

This creates `datapoint_tui_debug.log` in the current directory with detailed information about:
- Connection attempts
- Each register read operation
- Raw register values
- Errors and exceptions

### Step 2: Check the Log File
```bash
tail -f datapoint_tui_debug.log
```

Look for:
```
INFO  Connecting to Modbus server at 127.0.0.1:2525
DEBUG Connected successfully
DEBUG Reading RTU Status Register 1 at address 0 (length 1)
DEBUG Successfully read 1 registers: [42]
```

Or errors like:
```
WARN  Modbus exception for RTU Status Register 1: IllegalAddress
WARN  Read timeout for Asset 1
ERROR Connection failed: Connection refused
```

### Step 3: Test with CLI tool
```bash
# Read first register as holding register
modbus-cli --ip YOUR_IP --port YOUR_PORT --unit-id 1 \
  --start-register 0 read --fn-code 3

# Read first register as input register  
modbus-cli --ip YOUR_IP --port YOUR_PORT --unit-id 1 \
  --start-register 0 read --fn-code 4
```

### Step 4: Compare CLI and TUI
If CLI works but TUI doesn't:

**Check the log file** (`datapoint_tui_debug.log`) for:
1. Connection success: `Connected successfully`
2. Read attempts: `Reading [name] at address [X]`
3. Errors: Look for `WARN` or `ERROR` lines

**Common issues in logs:**
- `IllegalAddress` - Address doesn't exist on device
- `Connection timeout` - Device unreachable
- `Modbus exception` - Device rejected the request
- `Read timeout` - Device too slow to respond

### Step 5: Update Config Based on What Works
If function code 3 works → Use default (or `register_type: holding`)
If function code 4 works → Use `register_type: input`

### Step 6: Create Test Config
Create a minimal test config with just 1-2 registers:

```yaml
server:
  protocol: modbus
  host: YOUR_IP
  port: YOUR_PORT
  unit_id: 1
  endianness: big

scan_interval_ms: 1000

datapoints:
  - name: "Test Register"
    address: 0
    length: 1
    data_type: u16
    register_type: input  # or holding
```

### Step 7: Run and Observe
```bash
./target/release/datapoint_tui --config test_config.yaml
```

Watch for:
- Status changing from WAITING to OK
- Error messages in the status column
- Timeout errors

## Example Fix for MID252

If your MID252 device uses **input registers**, update `mid252_config.yaml`:

```yaml
datapoints:
  - name: "RTU Status Register 1"
    address: 0
    length: 1
    data_type: bitfield
    register_type: input  # Add this!
    bitfields:
      - bit: 0
        name: "Toggle bit"
```

Or use the provided `test_input_registers.yaml` config file.

## Still Not Working?

Check these:
1. Device is powered on and network accessible
2. No other tool is holding a connection (Modbus TCP allows limited connections)
3. Check device documentation for correct function codes
4. Try increasing `scan_interval_ms` to 2000 or 5000
5. Check if device requires specific connection settings (timeout, etc.)
