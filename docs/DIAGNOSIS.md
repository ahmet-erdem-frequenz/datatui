# Debug Log Analysis: MID252 Connection Issue

## What the Log Shows

```
[2026-01-22T14:27:08Z INFO] Connecting to Modbus server at ::1:2525
[2026-01-22T14:27:09Z INFO] Connecting to Modbus server at ::1:2525
...repeating every second...
```

## Problem Identified

**The connection is timing out after 5 seconds.**

The log shows:
1. ✅ Config is being read (host: 127.0.0.1)
2. ❌ Connection attempts timeout (no "Connected successfully" message)
3. ❌ Address shows as `::1` (IPv6) instead of `127.0.0.1` (IPv4)
4. 🔄 Retrying every scan_interval (1000ms)

## Root Causes (Most Likely)

### 1. Device is Not on Localhost
**Your MID252 device is probably NOT on 127.0.0.1**

The config says:
```yaml
server:
  host: 127.0.0.1  # localhost
  port: 2525
```

But if you copied this from the mockbus config, **that's for the MOCK server**, not the real device!

**Solution:** Update the IP to your actual MID252 device:
```yaml
server:
  host: 192.168.1.100  # Your actual device IP
  port: 502            # Standard Modbus port (might be 2525)
```

### 2. Wrong Port
Standard Modbus TCP uses port **502**, not 2525.
Port 2525 is what the mock server uses.

**Solution:** Try port 502:
```yaml
server:
  port: 502
```

### 3. IPv6 vs IPv4
The log shows `::1` (IPv6) but your config has `127.0.0.1` (IPv4).
This might be a DNS resolution issue.

**Solution:** If device is on localhost, try:
```yaml
server:
  host: "0.0.0.0"  # or explicit 127.0.0.1
```

## How to Fix

### Step 1: Find Your Device IP

Ask yourself:
- Where is the MID252 device?
- Is it on this computer (localhost)?
- Is it on the network?

If you can read it with `modbus-cli`, check what IP/port you use:
```bash
modbus-cli --ip WHAT_IP --port WHAT_PORT ...
```

### Step 2: Update Config

Update `mid252_config.yaml`:
```yaml
server:
  protocol: modbus
  host: YOUR_DEVICE_IP  # NOT 127.0.0.1 unless device is local
  port: YOUR_DEVICE_PORT # Usually 502, not 2525
  unit_id: 1
  endianness: big
```

### Step 3: Test Connection

Before running the full config, test if you can connect:
```bash
# Test with your CLI tool
modbus-cli --ip YOUR_DEVICE_IP --port YOUR_DEVICE_PORT \
  --unit-id 1 --start-register 0 read --fn-code 3

# If that works, update mid252_config.yaml with same IP and port
```

### Step 4: Run TUI Again

```bash
./target/release/datapoint_tui --config mid252_config.yaml --debug
```

Now the log should show:
```
INFO  Connecting to Modbus server at YOUR_DEVICE_IP:YOUR_PORT
DEBUG Connected successfully
DEBUG Reading RTU Status Register 1 at address 0 (length 1)
```

## Quick Checklist

- [ ] Is the MID252 device powered on?
- [ ] Can you ping the device IP?
- [ ] Does `modbus-cli` work with the same IP/port?
- [ ] Did you update the config with the correct IP (not 127.0.0.1)?
- [ ] Did you update the config with the correct port (probably 502, not 2525)?
- [ ] Is there a firewall blocking the connection?

## Expected Working Log

When fixed, you should see:
```
[INFO] Connecting to Modbus server at 192.168.1.100:502
[DEBUG] Connected successfully
[DEBUG] Reading RTU Status Register 1 at address 0 (length 1)
[DEBUG] Successfully read 1 registers: [65]
[DEBUG] Reading RTU Status Register 2 at address 1 (length 1)
[DEBUG] Successfully read 1 registers: [0]
```

## TL;DR

**Your config has `host: 127.0.0.1` which is localhost (this computer).**
**Your MID252 device is probably NOT on localhost - it's on the network somewhere.**

**Find the real IP address and port of your MID252 device and update the config!**

