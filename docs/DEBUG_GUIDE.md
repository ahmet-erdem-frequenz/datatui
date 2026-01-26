# Debug Guide - Finding Why Values Don't Arrive

## Quick Start

```bash
# 1. Run with debug logging
./target/release/datapoint_tui --config mid252_config.yaml --debug

# 2. In another terminal, watch the log
tail -f datapoint_tui_debug.log

# 3. Look for errors and compare with CLI tool
```

## The Problem

**Symptoms:**
- Status shows "WAITING" forever
- No values appear
- CLI tool (`modbus-cli`) can read the values successfully

**Most Common Causes:**
1. Wrong register addresses
2. Wrong register type (rare, since function code 3 works)
3. Configuration syntax error
4. Data type mismatch

## Debug Process

### Step 1: Enable Debug Logging

The `--debug` flag creates `datapoint_tui_debug.log` with detailed information:

```bash
./target/release/datapoint_tui --config mid252_config.yaml --debug
```

### Step 2: Read the Debug Log

```bash
# Watch live
tail -f datapoint_tui_debug.log

# Or check full log
cat datapoint_tui_debug.log
```

**What to look for:**

✅ **Success looks like:**
```
INFO  Connecting to Modbus server at 127.0.0.1:2525
DEBUG Connected successfully
DEBUG Reading RTU Status Register 1 at address 0 (length 1)
DEBUG Successfully read 1 registers: [65]
```

❌ **Problems look like:**
```
WARN  Modbus exception for RTU Status Register 1: IllegalAddress
WARN  Read timeout for Asset 1
ERROR Connection failed: Connection refused
```

### Step 3: Interpret Error Messages

| Error Message | Meaning | Solution |
|---------------|---------|----------|
| `IllegalAddress` | Register address doesn't exist | Check address in config, compare with CLI |
| `IllegalFunction` | Wrong register type (holding/input) | Try `register_type: input` |
| `Connection refused` | Can't connect to device | Check IP, port, firewall |
| `Connection timeout` | Device not responding | Check network, device power |
| `Read timeout` | Device too slow | Increase `scan_interval_ms` |

### Step 4: Compare with CLI Tool

If CLI works but TUI doesn't, check exact parameters:

```bash
# What works with CLI?
modbus-cli --ip 127.0.0.1 --port 2525 --unit-id 1 \
  --start-register 0 --dtype u16 read --fn-code 3 --quantity 1

# Match these in your config:
# - IP and port in server section
# - unit_id in server section  
# - start-register → address
# - dtype → data_type
# - fn-code 3 → register_type: holding (default)
# - quantity → length
```

## Common Scenarios

### Scenario 1: "IllegalAddress" Error

**Debug log shows:**
```
WARN  Modbus exception for RTU Status Register 1: IllegalAddress
```

**Problem:** The address doesn't exist on the device.

**Solutions:**
1. Check if CLI uses `--offset-zero` flag (addresses might be off by 1)
2. Verify address in device documentation
3. Try reading address-1 or address+1

**Example:** If CLI uses:
```bash
modbus-cli --offset-zero --start-register 1 read
```

Then in your config, use `address: 0` (not 1).

### Scenario 2: All Registers Show "WAITING"

**Debug log shows:**
```
INFO  Connecting to Modbus server at 127.0.0.1:2525
DEBUG Connected successfully
```
But no register reads.

**Problem:** Likely a configuration parsing error.

**Solutions:**
1. Check YAML syntax (indentation, colons, dashes)
2. Ensure all required fields are present
3. Check for typos in field names

### Scenario 3: Some Work, Some Don't

**Debug log shows:**
```
DEBUG Successfully read 1 registers: [65]  ← This works
WARN  Modbus exception: IllegalAddress      ← This doesn't
```

**Problem:** Mixed address ranges or device limitations.

**Solutions:**
1. Group working addresses together
2. Check device documentation for valid address ranges
3. Some devices have gaps in their address maps

## Example Debug Session

```bash
# Terminal 1: Run TUI with debug
$ ./target/release/datapoint_tui --config mid252_config.yaml --debug

# Terminal 2: Watch debug log
$ tail -f datapoint_tui_debug.log
INFO  Connecting to Modbus server at 192.168.1.100:2525
DEBUG Connected successfully
DEBUG Reading RTU Status Register 1 at address 0 (length 1)
WARN  Modbus exception for RTU Status Register 1: IllegalAddress
DEBUG Reading RTU Status Register 2 at address 1 (length 1)
DEBUG Successfully read 1 registers: [0]

# Terminal 3: Test with CLI
$ modbus-cli --ip 192.168.1.100 --port 2525 --start-register 0 read
Error: Modbus exception: IllegalAddress

$ modbus-cli --ip 192.168.1.100 --port 2525 --start-register 1 read
1  # Success!
```

**Conclusion:** Address 0 doesn't exist, address 1 works. Remove or fix address 0 in config.

## Quick Fixes

### Fix 1: Test with Minimal Config

Create `test_one.yaml`:
```yaml
server:
  protocol: modbus
  host: 192.168.1.100
  port: 2525
  unit_id: 1
  endianness: big

scan_interval_ms: 1000

datapoints:
  - name: "Test"
    address: 0  # Use address that works with CLI
    length: 1
    data_type: u16
```

Run: `./target/release/datapoint_tui --config test_one.yaml --debug`

### Fix 2: Match CLI Parameters Exactly

If this CLI command works:
```bash
modbus-cli --ip 192.168.1.100 --port 2525 --unit-id 1 \
  --start-register 1000 --dtype i32 --endianness big \
  read --fn-code 3 --quantity 2
```

Use this config:
```yaml
datapoints:
  - name: "My Register"
    address: 1000
    length: 2
    data_type: i32
    # register_type defaults to holding (fn-code 3)
```

## Still Stuck?

1. Share the debug log file
2. Share the CLI command that works
3. Share your config file
4. Check that device firmware supports all addresses in your config

