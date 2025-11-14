# Test Client

Rust-based test client for the Age Verification Proof Server.

## Usage

### Run all tests
```bash
cargo run --release
```

### Run specific tests
```bash
# Health check only
cargo run --release -- health

# Verification key only
cargo run --release -- vkey

# Valid certificate test only
cargo run --release -- valid

# Invalid certificate test only
cargo run --release -- invalid
```

## Prerequisites

1. Server must be running on `http://localhost:3000`
2. `payload.json` must exist in the project root
3. Trusted certificate files must exist:
   - `trusted_csca_certs_ecuador.json`
   - `trusted_csca_certs_invalid_ecuador.json`

## Test Coverage

- ✅ Health check endpoint
- ✅ Verification key endpoint
- ✅ Proof generation with valid certificate
- ✅ Proof generation with invalid certificate
- ✅ Response validation and assertions

## Alternative: Bash Script

For simpler testing, use the bash script:
```bash
./test_server.sh
```

