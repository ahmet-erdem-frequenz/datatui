# Black Screen Fix - Technical Details

## Root Cause
The application was getting stuck with a black screen because the scanner task held a mutex lock indefinitely, blocking the UI thread from accessing datapoints.

## The Problem Chain

1. **Scanner Task Holds Lock Forever**
   ```rust
   // OLD CODE - WRONG
   tokio::spawn(async move {
       let mut scanner = scanner_clone.lock().await;  // Lock acquired here
       let _ = scanner.run().await;  // Never returns, lock never released
   });
   ```
   The `run()` method contains an infinite loop, so the lock was never released.

2. **UI Thread Blocks Waiting for Lock**
   ```rust
   // UI trying to get datapoints
   let scanner_guard = scanner.lock().await;  // Blocks forever
   scanner_guard.get_datapoints().to_vec()
   ```
   
3. **Terminal in Raw Mode = Black Screen**
   - Terminal enters alternate screen and raw mode
   - UI can't draw because it's blocked
   - Keys don't work because event loop never runs
   - Result: Black screen with no response

## The Solution

### 1. Release Lock Between Scans
```rust
// NEW CODE - CORRECT
tokio::spawn(async move {
    tokio::time::sleep(Duration::from_millis(100)).await;  // Let UI init
    
    loop {
        {
            let mut scanner = scanner_clone.lock().await;  // Lock
            let _ = scanner.scan_once().await;             // Scan
        }  // Lock released here automatically
        
        tokio::time::sleep(Duration::from_millis(scan_interval)).await;
    }
});
```

### 2. Add Timeout to Lock Acquisition
```rust
let datapoints_result = tokio::time::timeout(
    Duration::from_millis(50),
    async {
        let scanner_guard = scanner.lock().await;
        scanner_guard.get_datapoints().to_vec()
    }
).await;

if let Ok(datapoints) = datapoints_result {
    app.update_datapoints(datapoints);
}
```

### 3. Always Draw UI
```rust
// Draw initial UI immediately
terminal.draw(|f| ui::draw(f, app))?;

// Always redraw even if we couldn't get data
terminal.draw(|f| ui::draw(f, app))?;
```

### 4. Check Keys First
```rust
// Check for key events at start of loop (non-blocking)
if event::poll(Duration::from_millis(0))? {
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
            KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => return Ok(()),
            // ...
        }
    }
}
```

### 5. Add Connection/Read Timeouts
```rust
// 5 second connection timeout
let connect_result = tokio::time::timeout(
    Duration::from_secs(5),
    tcp::connect_slave(...)
).await;

// 2 second read timeout per register
let read_result = tokio::time::timeout(
    Duration::from_secs(2),
    ctx.read_holding_registers(...)
).await;
```

## Testing
To test with a non-existent server:
```bash
cargo run -- --config test_config.yaml
```

Expected behavior:
- UI appears immediately with empty/waiting datapoints
- After 5 seconds, datapoints show "Connection timeout" errors
- 'q', 'Esc', or 'Ctrl+C' exits immediately at any time
- Scanner keeps retrying in background

## Key Improvements
1. ✅ UI draws immediately
2. ✅ Keys respond instantly
3. ✅ No blocking on connection failures
4. ✅ Lock contention eliminated
5. ✅ Graceful degradation (UI works even if scanner is stuck)
