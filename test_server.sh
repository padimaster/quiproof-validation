#!/bin/bash

# Test script for the Age Verification Proof Server
# Similar to test_both_certs.sh but tests the HTTP server API
# Run from project root directory

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR" || exit 1

echo "=========================================="
echo "Server API Test Suite"
echo "=========================================="
echo ""

# Check if server is running
echo "🔍 Checking if server is running..."
if ! curl -s -f http://localhost:3000/health > /dev/null 2>&1; then
    echo "❌ Server is not running!"
    echo ""
    echo "Please start the server first:"
    echo "  cd server"
    echo "  cargo run --release"
    echo ""
    echo "Or run in background:"
    echo "  cd server"
    echo "  cargo run --release > /tmp/server.log 2>&1 &"
    echo ""
    exit 1
fi

echo "✓ Server is running"
echo ""

# Test 1: Health Check
echo "🧪 TEST 1: Health Check"
echo "-----------------------------------"
HEALTH_RESPONSE=$(curl -s http://localhost:3000/health)
if echo "$HEALTH_RESPONSE" | grep -q '"status":"ok"'; then
    echo "✅ Health check passed"
    echo "$HEALTH_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$HEALTH_RESPONSE"
else
    echo "❌ Health check failed"
    echo "$HEALTH_RESPONSE"
    exit 1
fi
echo ""

# Test 2: Verification Key
echo "🧪 TEST 2: Verification Key"
echo "-----------------------------------"
VKEY_RESPONSE=$(curl -s http://localhost:3000/proof/vkey)
if echo "$VKEY_RESPONSE" | grep -q '"vkey"'; then
    echo "✅ Verification key retrieved"
    echo "$VKEY_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$VKEY_RESPONSE"
else
    echo "❌ Failed to get verification key"
    echo "$VKEY_RESPONSE"
    exit 1
fi
echo ""

# Test 3: Valid Certificate
echo "🧪 TEST 3: Valid Ecuador Certificate"
echo "-----------------------------------"
echo "Generating proof with VALID certificate..."
echo "Expected: ✅ Is valid: true"
echo ""

# Prepare request payload
PROOF_REQUEST=$(python3 << 'PYEOF'
import json
import os

script_dir = os.path.dirname(os.path.abspath(__file__)) if '__file__' in globals() else os.getcwd()
payload_path = os.path.join(script_dir, 'payload.json')

with open(payload_path, 'r') as f:
    payload = json.load(f)

request = {
    "dg1_base64": payload["dg1_base64"],
    "sod_base64": payload["sod_base64"],
    "age_to_verify": payload["age_to_verify"],
    "current_date": payload["current_date"],
    "trusted_csca_certs_file": "trusted_csca_certs_ecuador.json",
    "proof_system": "groth16"  # Use groth16 for EVM compatibility
}

print(json.dumps(request))
PYEOF
)

echo "Sending request to server..."
PROOF_RESPONSE=$(curl -s -X POST http://localhost:3000/proof/generate \
    -H "Content-Type: application/json" \
    -d "$PROOF_REQUEST")

if echo "$PROOF_RESPONSE" | grep -q '"success":true'; then
    echo "✅ Proof generated successfully"
    
    # Extract and display key results
    echo ""
    echo "=== Validation Results ==="
    echo "$PROOF_RESPONSE" | python3 -c "
import json, sys
data = json.load(sys.stdin)
if data.get('success') and data.get('proof'):
    output = data['proof']['output']
    print(f\"Is valid: {output['is_valid']}\")
    print(f\"Meets age requirement: {output['meets_age_requirement']}\")
    print(f\"SOD verification passed: {output['debug_sod_valid']}\")
    print(f\"Passport not expired: {output['debug_not_expired']}\")
    print(f\"Calculated age: {output['debug_age']}\")
    print(f\"Document number: {output['debug_document_number']}\")
    
    # Verify expectations
    if not output['is_valid']:
        print('❌ ERROR: Expected valid certificate to pass!')
        sys.exit(1)
    if not output['debug_sod_valid']:
        print('❌ ERROR: Expected SOD verification to pass!')
        sys.exit(1)
    print('✅ All assertions passed!')
" || {
        echo "❌ Failed to parse response"
        echo "$PROOF_RESPONSE"
        exit 1
    }
else
    echo "❌ Proof generation failed"
    echo "$PROOF_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$PROOF_RESPONSE"
    exit 1
fi
echo ""

# Test 4: Invalid Certificate
echo "🧪 TEST 4: Invalid Ecuador Certificate"
echo "-----------------------------------"
echo "Generating proof with INVALID certificate..."
echo "Expected: ❌ Is valid: false (certificate verification should fail)"
echo ""

# Prepare request payload with invalid certificate
PROOF_REQUEST_INVALID=$(python3 << 'PYEOF'
import json
import os

script_dir = os.path.dirname(os.path.abspath(__file__)) if '__file__' in globals() else os.getcwd()
payload_path = os.path.join(script_dir, 'payload.json')

with open(payload_path, 'r') as f:
    payload = json.load(f)

request = {
    "dg1_base64": payload["dg1_base64"],
    "sod_base64": payload["sod_base64"],
    "age_to_verify": payload["age_to_verify"],
    "current_date": payload["current_date"],
    "trusted_csca_certs_file": "trusted_csca_certs_invalid_ecuador.json",
    "proof_system": "groth16"  # Use groth16 for EVM compatibility
}

print(json.dumps(request))
PYEOF
)

echo "Sending request to server..."
PROOF_RESPONSE_INVALID=$(curl -s -X POST http://localhost:3000/proof/generate \
    -H "Content-Type: application/json" \
    -d "$PROOF_REQUEST_INVALID")

if echo "$PROOF_RESPONSE_INVALID" | grep -q '"success":true'; then
    echo "✅ Proof generated successfully"
    
    # Extract and display key results
    echo ""
    echo "=== Validation Results ==="
    echo "$PROOF_RESPONSE_INVALID" | python3 -c "
import json, sys
data = json.load(sys.stdin)
if data.get('success') and data.get('proof'):
    output = data['proof']['output']
    print(f\"Is valid: {output['is_valid']}\")
    print(f\"Meets age requirement: {output['meets_age_requirement']}\")
    print(f\"SOD verification passed: {output['debug_sod_valid']}\")
    print(f\"Passport not expired: {output['debug_not_expired']}\")
    
    # Verify expectations - invalid certificate should fail
    if output['is_valid']:
        print('❌ ERROR: Expected invalid certificate to fail!')
        sys.exit(1)
    if output['debug_sod_valid']:
        print('❌ ERROR: Expected SOD verification to fail with invalid certificate!')
        sys.exit(1)
    print('✅ All assertions passed! (correctly rejected invalid certificate)')
" || {
        echo "❌ Failed to parse response"
        echo "$PROOF_RESPONSE_INVALID"
        exit 1
    }
else
    echo "❌ Proof generation failed"
    echo "$PROOF_RESPONSE_INVALID" | python3 -m json.tool 2>/dev/null || echo "$PROOF_RESPONSE_INVALID"
    exit 1
fi
echo ""

echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo "✅ Health check passed"
echo "✅ Verification key retrieved"
echo "✅ Valid certificate test passed"
echo "✅ Invalid certificate test passed (correctly rejected)"
echo ""
echo "All server API tests completed successfully! 🎉"

