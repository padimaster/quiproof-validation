#!/bin/bash
# Deploy Age Verification Contract to Arbitrum Stylus

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Deploying Age Verification Contract to Arbitrum Stylus${NC}"

# Check prerequisites
if ! command -v cargo-stylus &> /dev/null; then
    echo -e "${RED}❌ cargo-stylus not found. Install with: cargo install --force cargo-stylus${NC}"
    exit 1
fi

# Load environment variables
if [ -f .env ]; then
    source .env
fi

# Required variables
if [ -z "$VKEY" ]; then
    echo -e "${YELLOW}⚠️  VKEY not set. Getting from proof server...${NC}"
    VKEY=$(curl -s http://localhost:3000/proof/vkey 2>/dev/null | jq -r '.vkey_hex' 2>/dev/null || echo "")
    if [ -z "$VKEY" ]; then
        echo -e "${RED}❌ Could not get VKEY. Please set VKEY environment variable or ensure proof server is running.${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Got VKEY from server: ${VKEY:0:20}...${NC}"
fi

if [ -z "$VERIFIER_ADDRESS" ]; then
    echo -e "${RED}❌ VERIFIER_ADDRESS not set. Please deploy SP1Verifier contract first.${NC}"
    echo -e "${YELLOW}   Deploy SP1Verifier.sol and set VERIFIER_ADDRESS in .env${NC}"
    exit 1
fi

if [ -z "$PRIVATE_KEY" ]; then
    echo -e "${RED}❌ PRIVATE_KEY not set. Please set PRIVATE_KEY in .env${NC}"
    exit 1
fi

# Network (default to arbitrum-sepolia)
NETWORK=${NETWORK:-arbitrum-sepolia}

echo -e "${GREEN}📋 Deployment Configuration:${NC}"
echo "   Network: $NETWORK"
echo "   VKey: ${VKEY:0:20}..."
echo "   Verifier: $VERIFIER_ADDRESS"
echo ""

# Build happens automatically during deploy, but we can check first
echo -e "${GREEN}🔨 Checking contract...${NC}"
cargo stylus check

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Contract check failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Contract check passed${NC}"

# Deploy contract (builds automatically)
echo -e "${GREEN}📤 Deploying contract...${NC}"
DEPLOY_OUTPUT=$(cargo stylus deploy \
    --constructor-args "$VKEY" "$VERIFIER_ADDRESS" \
    --network "$NETWORK" \
    --private-key "$PRIVATE_KEY" 2>&1)

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Deployment failed${NC}"
    echo "$DEPLOY_OUTPUT"
    exit 1
fi

# Extract contract address from output
CONTRACT_ADDRESS=$(echo "$DEPLOY_OUTPUT" | grep -oP '0x[a-fA-F0-9]{40}' | head -1)

if [ -z "$CONTRACT_ADDRESS" ]; then
    echo -e "${RED}❌ Could not extract contract address from deployment output${NC}"
    echo "$DEPLOY_OUTPUT"
    exit 1
fi

echo -e "${GREEN}✅ Contract deployed: $CONTRACT_ADDRESS${NC}"

# Activate contract
echo -e "${GREEN}⚡ Activating contract...${NC}"
cargo stylus activate \
    --address "$CONTRACT_ADDRESS" \
    --network "$NETWORK" \
    --private-key "$PRIVATE_KEY"

if [ $? -ne 0 ]; then
    echo -e "${YELLOW}⚠️  Activation failed (contract may still work, but not optimized)${NC}"
else
    echo -e "${GREEN}✅ Contract activated${NC}"
fi

# Save to .env
if [ -f .env ]; then
    # Update or add CONTRACT_ADDRESS
    if grep -q "CONTRACT_ADDRESS=" .env; then
        sed -i.bak "s|CONTRACT_ADDRESS=.*|CONTRACT_ADDRESS=$CONTRACT_ADDRESS|" .env
    else
        echo "CONTRACT_ADDRESS=$CONTRACT_ADDRESS" >> .env
    fi
else
    echo "CONTRACT_ADDRESS=$CONTRACT_ADDRESS" > .env
fi

echo ""
echo -e "${GREEN}🎉 Deployment Complete!${NC}"
echo ""
echo "Contract Address: $CONTRACT_ADDRESS"
echo "Network: $NETWORK"
echo "Verifier: $VERIFIER_ADDRESS"
echo ""
echo "Next steps:"
echo "  1. Verify contract on Arbiscan (if on testnet/mainnet)"
echo "  2. Update frontend with contract address"
echo "  3. Test with real proofs"

