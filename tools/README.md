# Utility Tools

This directory contains utility tools for processing passport validation data.

## Why Preprocess Outside the Circuit?

**Yes, preprocessing is better!** See [PREPROCESSING.md](./PREPROCESSING.md) for details.

Key benefits:
- ⚡ **Faster**: Parse once, use many times
- 💰 **Cheaper**: Fewer cycles in zkVM = lower proof costs
- 🔧 **Flexible**: Easy to update and filter certificates
- ✅ **Testable**: Test parser independently

## CSCA Certificate Parser

Parses the ICAO PKD LDIF file and extracts CSCA (Country Signing Certificate Authority) certificates for use in certificate chain verification.

### Quick Test

```bash
cd tools
./test_parser.sh
```

### Full Usage

```bash
cd tools
cargo run --release --bin parse_csca_certs -- \
    ../certificates/icaopkd-001-complete-009494.ldif \
    ../certificates/trusted_csca_certs.json
```

This will:
1. Parse the LDIF file
2. Extract all CSCA certificates
3. Calculate SHA-256 hashes for each certificate
4. Save to JSON format for use in the zkVM circuit

## Output Format

The output JSON contains an array of `TrustedCSCACert` objects:
```json
[
  {
    "country_code": "EC",
    "certificate_hash": [32-byte array],
    "certificate_der": [DER-encoded certificate bytes],
    "serial_number": "42E575AF",
    "common_name": "OU=Identity Services Passport CA,..."
  },
  ...
]
```

## Using in Circuit

The trusted certificates can be loaded and passed to the circuit via the `AgeProofInput.trusted_csca_certs` field for certificate chain verification.

