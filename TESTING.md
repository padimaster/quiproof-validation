# Testing Guide

This document describes how to test the Age Verification Proof Server.

## Test Options

### Option 1: Bash Script (Recommended for Quick Testing)

The `test_server.sh` script provides a comprehensive test suite similar to `test_both_certs.sh`:

```bash
./test_server.sh
```

**What it tests:**
- ✅ Server health check
- ✅ Verification key retrieval
- ✅ Proof generation with valid certificate
- ✅ Proof generation with invalid certificate
- ✅ Response validation and assertions

**Prerequisites:**
- Server must be running on `http://localhost:3000`
- `payload.json` must exist in project root
- Trusted certificate files must exist

### Option 2: Rust Test Client (Recommended for CI/CD)

The `test-client` provides a more robust, programmatic testing approach:

```bash
cd test-client
cargo run --release
```

**Run specific tests:**
```bash
cargo run --release -- health    # Health check only
cargo run --release -- vkey      # Verification key only
cargo run --release -- valid     # Valid certificate test
cargo run --release -- invalid   # Invalid certificate test
```

**Advantages:**
- ✅ Type-safe assertions
- ✅ Better error handling
- ✅ Easier to integrate in CI/CD
- ✅ More detailed output

### Option 3: Quick Health Check

For simple server availability checks:

```bash
./test_server_quick.sh
```

## Running Tests

### Step 1: Start the Server

```bash
cd server
cargo run --release
```

Or run in background:
```bash
cd server
cargo run --release > /tmp/server.log 2>&1 &
```

### Step 2: Run Tests

**Bash script:**
```bash
./test_server.sh
```

**Rust client:**
```bash
cd test-client
cargo run --release
```

## Expected Results

### Test 1: Health Check
```
✅ Status: ok
✅ Version: 0.1.0
```

### Test 2: Verification Key
```
✅ VKey retrieved successfully
✅ VKey hex format: 0x...
```

### Test 3: Valid Certificate
```
✅ Proof generated successfully
Is valid: true
Meets age requirement: true
SOD verification passed: true
Passport not expired: true
Calculated age: 24
```

### Test 4: Invalid Certificate
```
✅ Proof generated successfully
Is valid: false
SOD verification passed: false
✅ Correctly rejected invalid certificate
```

## Troubleshooting

### Server Not Running
```
❌ Server is not running!
```
**Solution:** Start the server first (see Step 1 above)

### Connection Refused
```
Error: Connection refused
```
**Solution:** 
- Check if server is running: `curl http://localhost:3000/health`
- Check if port 3000 is available
- Check firewall settings

### Proof Generation Timeout
```
Error: Request timeout
```
**Solution:**
- Core proofs take 30-60 seconds
- EVM proofs (Groth16/PLONK) take 2-5 minutes
- Increase timeout in test client if needed

### Certificate File Not Found
```
Error: Failed to load trusted certificates
```
**Solution:**
- Ensure `trusted_csca_certs_ecuador.json` exists
- Ensure `trusted_csca_certs_invalid_ecuador.json` exists
- Check file paths are correct

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Test Server

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Start server
        run: |
          cd server
          cargo run --release &
          sleep 10  # Wait for server to start
      
      - name: Run tests
        run: |
          cd test-client
          cargo run --release
```

## Comparison: Bash vs Rust Client

| Feature | Bash Script | Rust Client |
|---------|------------|-------------|
| Setup | Simple (just bash) | Requires Rust |
| Speed | Fast | Fast |
| Type Safety | No | Yes |
| Error Handling | Basic | Advanced |
| CI/CD Integration | Easy | Easy |
| Detailed Output | Good | Excellent |
| Assertions | Basic | Comprehensive |

**Recommendation:**
- Use **bash script** for quick manual testing
- Use **Rust client** for automated testing and CI/CD

