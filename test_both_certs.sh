#!/bin/bash

# Test script to verify both valid and invalid certificates
# Run from project root directory

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR" || exit 1

echo "=========================================="
echo "Testing Certificate Validation"
echo "=========================================="
echo ""

# Test 1: Valid Ecuador Certificate
echo "🧪 TEST 1: Valid Ecuador Certificate"
echo "-----------------------------------"
python3 << 'PYEOF'
import json
import os

# Get script directory
script_dir = os.path.dirname(os.path.abspath(__file__)) if '__file__' in globals() else os.getcwd()
payload_path = os.path.join(script_dir, 'payload.json')

# Update payload to use valid certificate
with open(payload_path, 'r') as f:
    payload = json.load(f)

payload['trusted_csca_certs_file'] = 'trusted_csca_certs_ecuador.json'

with open(payload_path, 'w') as f:
    json.dump(payload, f, indent=2)

print("✓ Updated payload.json to use: trusted_csca_certs_ecuador.json")
PYEOF

echo ""
echo "Running circuit with VALID certificate..."
echo "Expected: ✅ Is valid: true"
echo ""
cd circuits/script && RUST_LOG=info cargo run --release -- --execute --payload ../../payload.json 2>&1 | grep -A 20 "Validation Results\|Is valid\|SOD verification\|Debug Information" || echo "Run completed. Check output above."
cd ../..
echo ""

# Test 2: Invalid Ecuador Certificate
echo ""
echo "🧪 TEST 2: Invalid Ecuador Certificate"
echo "-----------------------------------"
python3 << 'PYEOF'
import json
import os

# Get script directory
script_dir = os.path.dirname(os.path.abspath(__file__)) if '__file__' in globals() else os.getcwd()
payload_path = os.path.join(script_dir, 'payload.json')

# Update payload to use invalid certificate
with open(payload_path, 'r') as f:
    payload = json.load(f)

payload['trusted_csca_certs_file'] = 'trusted_csca_certs_invalid_ecuador.json'

with open(payload_path, 'w') as f:
    json.dump(payload, f, indent=2)

print("✓ Updated payload.json to use: trusted_csca_certs_invalid_ecuador.json")
PYEOF

echo ""
echo "Running circuit with INVALID certificate..."
echo "Expected: ❌ Is valid: false (certificate verification should fail)"
echo ""
cd circuits/script && RUST_LOG=info cargo run --release -- --execute --payload ../../payload.json 2>&1 | grep -A 20 "Validation Results\|Is valid\|SOD verification\|Debug Information" || echo "Run completed. Check output above."
cd ../..
echo ""

echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo "✓ Valid certificate should PASS validation"
echo "✓ Invalid certificate should FAIL validation"
echo ""
echo "If invalid certificate passes, there's a security issue!"

