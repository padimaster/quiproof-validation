# Preprocessing vs Circuit Processing

## Why Preprocess Outside the Circuit?

**Yes, preprocessing outside the circuit is definitely better!** Here's why:

### 1. **Performance & Cost**
- The LDIF file is **1.2+ million lines** - parsing this in a zkVM would be extremely expensive
- Certificate parsing involves complex ASN.1/DER decoding - computationally heavy
- Preprocessing once saves thousands of cycles in every proof generation

### 2. **Separation of Concerns**
- **Preprocessing**: Extract and prepare data (one-time operation)
- **Circuit**: Verify and prove (runs for every proof)

### 3. **Reusability**
- Parse certificates once, use in multiple proofs
- Update certificate list without rebuilding circuit
- Filter by country if needed

### 4. **Practical Workflow**

```
┌─────────────────────────────────────────┐
│ 1. Preprocessing (Outside Circuit)     │
│    - Parse LDIF file                    │
│    - Extract CSCA certificates          │
│    - Calculate hashes                   │
│    - Save to JSON                       │
│    Time: ~seconds                       │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│ 2. Circuit Execution (zkVM)            │
│    - Load pre-processed certificates   │
│    - Verify certificate chain           │
│    - Verify passport data               │
│    - Generate proof                     │
│    Time: ~minutes (proof generation)    │
└─────────────────────────────────────────┘
```

## Recommended Workflow

### Step 1: Preprocess Certificates (One-time)

```bash
cd tools
cargo run --release --bin parse_csca_certs -- \
    ../certificates/icaopkd-001-complete-009494.ldif \
    ../certificates/trusted_csca_certs.json
```

This creates a JSON file with all trusted CSCA certificates.

### Step 2: Use in Circuit

The preprocessed certificates are loaded when running the circuit:

```json
{
  "dg1_base64": "...",
  "sod_base64": "...",
  "age_to_verify": 18,
  "current_date": [2025, 11, 13],
  "trusted_csca_certs_file": "certificates/trusted_csca_certs.json"
}
```

### Step 3: Circuit Verification

The circuit:
1. Loads the preprocessed certificates
2. Verifies certificate chain against trusted list
3. Verifies passport data integrity
4. Generates zero-knowledge proof

## Benefits

✅ **Faster**: No need to parse LDIF in every proof  
✅ **Cheaper**: Fewer cycles = lower proof generation cost  
✅ **Flexible**: Easy to update certificate list  
✅ **Testable**: Can test parser independently  
✅ **Scalable**: Can filter by country or date range  

## Testing the Parser

Test with a small sample first:

```bash
cd tools
./test_parser.sh
```

Or manually:

```bash
# Create sample
head -200 ../certificates/icaopkd-001-complete-009494.ldif > /tmp/test.ldif

# Run parser
cargo run --release --bin parse_csca_certs -- \
    /tmp/test.ldif \
    /tmp/test_output.json

# Check results
cat /tmp/test_output.json | jq '.[0]'
```

## Filtering Certificates

You can also create filtered versions:

```bash
# All certificates
cargo run --release --bin parse_csca_certs -- \
    ../certificates/icaopkd-001-complete-009494.ldif \
    ../certificates/trusted_csca_certs_all.json

# Filter by country (using jq)
cat ../certificates/trusted_csca_certs_all.json | \
    jq '[.[] | select(.country_code == "EC")]' > \
    ../certificates/trusted_csca_certs_ecuador.json
```

This allows you to use only relevant certificates for specific passports, further reducing circuit size.

