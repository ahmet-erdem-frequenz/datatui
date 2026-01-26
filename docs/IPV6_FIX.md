# IPv6 Support Fix

## Problem Identified

**The application was not handling IPv6 addresses correctly.**

### The Issue

When the config has an IPv6 address like `::1`, the code was building the socket address as:
```
::1:2525  ❌ INVALID
```

But IPv6 addresses in socket address format MUST be wrapped in square brackets:
```
[::1]:2525  ✅ VALID
```

### What Was Failing

```rust
// Old code - BROKEN for IPv6
let socket_addr = format!("{}:{}", host, port);
// With host="::1" and port=2525 → "::1:2525"
// socket_addr.parse() fails!
```

### The Fix

```rust
// New code - WORKS for both IPv4 and IPv6
let socket_addr = if host.contains(':') {
    // IPv6 - wrap in brackets
    format!("[{}]:{}", host, port)
} else {
    // IPv4 or hostname - no brackets needed
    format!("{}:{}", host, port)
};
```

### Results

| Host | Old Output | Old Result | New Output | New Result |
|------|-----------|------------|------------|------------|
| `127.0.0.1` | `127.0.0.1:2525` | ✅ Works | `127.0.0.1:2525` | ✅ Works |
| `::1` | `::1:2525` | ❌ Parse Error | `[::1]:2525` | ✅ Works |
| `2001:db8::1` | `2001:db8::1:502` | ❌ Parse Error | `[2001:db8::1]:502` | ✅ Works |
| `localhost` | `localhost:502` | ✅ Works | `localhost:502` | ✅ Works |

## Testing

### Before Fix
```
[INFO] Connecting to Modbus server at ::1:2525
(connection never completes - parse error swallowed by timeout)
```

### After Fix
```
[INFO] Connecting to Modbus server at [::1]:2525
[DEBUG] Connected successfully
```

## Configuration

No changes needed! The same config works now:

```yaml
server:
  host: ::1        # IPv6 localhost
  port: 2525
```

or

```yaml
server:
  host: 127.0.0.1  # IPv4 localhost
  port: 2525
```

Both work correctly now!

## Impact

This fix enables:
- ✅ IPv6 localhost (`::1`)
- ✅ IPv6 addresses (`2001:db8::1`, etc.)
- ✅ IPv4 addresses (unchanged, still works)
- ✅ Hostnames (unchanged, still works)

## Files Modified

- `src/scanner.rs` - Added IPv6 bracket wrapping logic in `scan_modbus()` function

## Related Debug Log Analysis

The original debug log showed:
```
[INFO] Connecting to Modbus server at ::1:2525
```

This was being built from config correctly, but the parse was failing silently.
The 5-second connection timeout would expire, but no error was logged because
the parse error happened inside the connection attempt.

Now it correctly formats as `[::1]:2525` and can connect!
