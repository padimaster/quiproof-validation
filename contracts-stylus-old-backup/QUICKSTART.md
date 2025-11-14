# Quick Start: Arbitrum Stylus Deployment

Get your Age Verification contract deployed on Arbitrum Stylus in 5 minutes.

## Prerequisites

```bash
# Install Rust 1.80
rustup default 1.80

# Install cargo-stylus
cargo install --force cargo-stylus

# Add WASM target
rustup target add wasm32-unknown-unknown --toolchain 1.80
```

## Step 1: Deploy SP1 Verifier

The SP1 verifier must be deployed as a Solidity contract first.

**Option A: Use Existing Deployment**

If SP1 verifier is already deployed, use that address.

**Option B: Deploy New Verifier**

```bash
# Using Foundry
forge install succinctlabs/sp1-sdk
forge create SP1Verifier --rpc-url $ARBITRUM_RPC --private-key $PRIVATE_KEY

# Save address
export VERIFIER_ADDRESS="0x..."
```

## Step 2: Get Verification Key

```bash
# From proof server
curl http://localhost:3000/proof/vkey | jq -r '.vkey_hex'

# Or manually
export VKEY="0x00a43d8424d369ae4eb7740d7a006ce78458458a9185c58932d79ddf4b366052"
```

## Step 3: Deploy Stylus Contract

```bash
cd contracts-stylus

# Create .env file
cat > .env << EOF
VKEY=0x00a43d8424d369ae4eb7740d7a006ce78458458a9185c58932d79ddf4b366052
VERIFIER_ADDRESS=0x...
PRIVATE_KEY=0x...
NETWORK=arbitrum-sepolia
EOF

# Deploy using script
./scripts/deploy.sh

# Or manually
cargo stylus build --release
cargo stylus deploy \
  --constructor-args "$VKEY" "$VERIFIER_ADDRESS" \
  --network arbitrum-sepolia \
  --private-key $PRIVATE_KEY
```

## Step 4: Activate Contract

```bash
# Get contract address from deployment output
export CONTRACT_ADDRESS="0x..."

# Activate
cargo stylus activate \
  --address $CONTRACT_ADDRESS \
  --network arbitrum-sepolia \
  --private-key $PRIVATE_KEY
```

## Step 5: Test Contract

```javascript
// Using ethers.js
const contract = new ethers.Contract(
  CONTRACT_ADDRESS,
  [
    'function verifyAgeProof(bytes calldata proof, bytes calldata publicValues, bytes32 documentNumberHash) external returns (bool, bool)',
    'function isDocumentNumberUsed(bytes32 documentNumberHash) external view returns (bool)',
  ],
  signer
);

// Check if document number is used
const isUsed = await contract.isDocumentNumberUsed(documentNumberHash);

// Verify proof
const [isValid, meetsAge] = await contract.verifyAgeProof(
  proof,
  publicValues,
  documentNumberHash
);
```

## Troubleshooting

### Build Errors

```bash
# Ensure correct Rust version
rustup default 1.80
rustup target add wasm32-unknown-unknown --toolchain 1.80

# Clean and rebuild
cargo clean
cargo stylus build --release
```

### Deployment Errors

- **"Insufficient funds"**: Fund your account with ETH
- **"Invalid vkey"**: Check VKEY format (should be 0x + 64 hex chars)
- **"Invalid verifier address"**: Ensure SP1 verifier is deployed

### Activation Errors

- **"Contract not found"**: Wait for deployment confirmation
- **"Already activated"**: Contract is already active (this is OK)

## Next Steps

1. ✅ Contract deployed and activated
2. Update frontend with contract address
3. Test with real proofs
4. Deploy to Arbitrum One (mainnet)

## Resources

- [Arbitrum Stylus Docs](https://docs.arbitrum.io/stylus)
- [cargo-stylus CLI](https://github.com/OffchainLabs/cargo-stylus)
- [SP1 SDK](https://github.com/succinctlabs/sp1)

