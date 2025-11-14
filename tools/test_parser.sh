#!/bin/bash
# Test script for the CSCA certificate parser
# This tests the parser with a small sample before processing the full file

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=== Testing CSCA Certificate Parser ==="
echo ""

# Create a test sample (first 200 lines should contain at least one certificate)
echo "Creating test sample..."
head -200 "$PROJECT_ROOT/certificates/icaopkd-001-complete-009494.ldif" > /tmp/test_csca_sample.ldif

# Build the tool
echo "Building parser tool..."
cd "$SCRIPT_DIR"
cargo build --release

# Run with test sample
echo ""
echo "Running parser on test sample..."
cargo run --release --bin parse_csca_certs -- \
    /tmp/test_csca_sample.ldif \
    /tmp/test_output.json

# Check results
if [ -f /tmp/test_output.json ]; then
    echo ""
    echo "✓ Test successful! Output file created."
    echo "  File: /tmp/test_output.json"
    echo ""
    echo "Sample output (first certificate):"
    head -20 /tmp/test_output.json
    echo ""
    echo "Certificate count:"
    cat /tmp/test_output.json | grep -c '"country_code"' || echo "0"
else
    echo "✗ Test failed - output file not created"
    exit 1
fi

echo ""
echo "=== Test Complete ==="
echo ""
echo "To process the full LDIF file, run:"
echo "  cd tools"
echo "  cargo run --release --bin parse_csca_certs -- \\"
echo "    ../certificates/icaopkd-001-complete-009494.ldif \\"
echo "    ../certificates/trusted_csca_certs.json"

