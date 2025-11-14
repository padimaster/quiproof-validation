# Arbitrum One Mainnet Deployment Guide

Complete step-by-step guide for deploying the Age Verification contract to **Arbitrum One mainnet**.

## ⚠️ Pre-Deployment Checklist

Before deploying to mainnet, ensure:

- [ ] **Tested on Arbitrum Sepolia testnet** - All functionality verified
- [ ] **Sufficient ETH balance** - ~0.1-0.5 ETH for deployment + activation
- [ ] **Private key secured** - Use hardware wallet or secure key management
- [ ] **Verification key (VKEY) finalized** - Won't change after deployment
- [ ] **SP1 Verifier deployed** - Already on Arbitrum One or ready to deploy
- [ ] **Backup plan** - Know how to handle deployment issues

## Prerequisites

### 1. Install Required Tools

```bash
# Rust 1.80+
rustup default 1.80

# cargo-stylus CLI
cargo install --force cargo-stylus

# WASM target
rustup target add wasm32-unknown-unknown --toolchain 1.80

# Verify installation
cargo stylus --version
```

### 2. Fund Your Account

Ensure your deployment account has sufficient ETH on Arbitrum One:

```bash
# Check balance (you'll need ~0.1-0.5 ETH)
# Use Arbitrum Bridge: https://bridge.arbitrum.io/
# Or send from another Arbitrum account
```

**Estimated Costs:**
- SP1 Verifier deployment: ~0.01-0.05 ETH
- Stylus contract deployment: ~0.05-0.1 ETH
- Contract activation: ~0.01-0.02 ETH
- **Total: ~0.1-0.2 ETH** (gas prices vary)

## Step-by-Step Deployment

### Step 1: Get SP1 Verifier Address

The SP1 verifier must be deployed first as a Solidity contract.

#### Option A: Use Canonical SP1 Verifier Gateway (Recommended) ⭐

**Easiest approach** - Use the official SP1 Verifier Gateway deployed by Succinct Labs:

```bash
# Canonical SP1 Verifier Gateway on Arbitrum One
export VERIFIER_ADDRESS="0x3B6041173B80E77f038f3F2C0f9744f04837185e"
```

**Why use this?**
- ✅ Already deployed and verified on Arbiscan
- ✅ Maintained by Succinct Labs
- ✅ Recommended for production use
- ✅ No deployment needed - saves gas and time

**Verify on Arbiscan:** https://arbiscan.io/address/0x3B6041173B80E77f038f3F2C0f9744f04837185e

#### Option B: Use Existing Deployment

If you have your own SP1 verifier already deployed on Arbitrum One:

```bash
export VERIFIER_ADDRESS="0x..."  # Your existing verifier address
```

#### Option C: Deploy New Verifier

Only deploy your own if you have a specific reason. See `DEPLOY_VERIFIER.md` for detailed instructions.

**Using Foundry:**

```bash
# Install SP1 SDK
forge install succinctlabs/sp1-sdk

# Set Arbitrum One RPC
export ARBITRUM_ONE_RPC="https://arb1.arbitrum.io/rpc"
export PRIVATE_KEY="0x..."  # Your private key

# Deploy SP1 Verifier
forge create SP1Verifier \
  --rpc-url $ARBITRUM_ONE_RPC \
  --private-key $PRIVATE_KEY \
  --legacy

# Save the deployed address
export VERIFIER_ADDRESS="0x..."  # From deployment output
```

**Using Hardhat:**

```javascript
// scripts/deploy-verifier.js
const { ethers } = require("hardhat");

async function main() {
  const SP1Verifier = await ethers.getContractFactory("SP1Verifier");
  const verifier = await SP1Verifier.deploy();
  await verifier.waitForDeployment();
  
  console.log("SP1 Verifier deployed to:", await verifier.getAddress());
}

main().catch(console.error);
```

```bash
npx hardhat run scripts/deploy-verifier.js --network arbitrumOne
```

### Step 2: Get Verification Key (VKEY)

The verification key must match your proof generation setup.

**Option A: From Proof Server**

```bash
# If your proof server is running
VKEY=$(curl -s http://localhost:3000/proof/vkey | jq -r '.vkey_hex')
export VKEY
```

**Option B: From Circuit**

```bash
cd ../circuits/script
VKEY=$(cargo run --release --bin vkey | grep -o '0x[0-9a-f]\{64\}')
export VKEY
```

**Option C: Manual (if you know the vkey)**

```bash
export VKEY="0x00a43d8424d369ae4eb7740d7a006ce78458458a9185c58932d79ddf4b366052"
```

⚠️ **Important:** The VKEY must match the one used for proof generation. If you change the circuit, you'll need a new VKEY.

### Step 3: Set Up Environment Variables

Create a `.env` file in `contracts-stylus/`:

```bash
cd contracts-stylus

cat > .env << EOF
# Network
NETWORK=arbitrum-one

# Arbitrum One RPC (use your own RPC endpoint for better reliability)
ARBITRUM_RPC=https://arb1.arbitrum.io/rpc

# Private Key (NEVER commit to git!)
PRIVATE_KEY=0x...

# Verification Key
VKEY=0x00a43d8424d369ae4eb7740d7a006ce78458458a9185c58932d79ddf4b366052

# SP1 Verifier Address
VERIFIER_ADDRESS=0x...

# Contract Address (will be set after deployment)
CONTRACT_ADDRESS=
EOF
```

**Security Note:** 
- Add `.env` to `.gitignore`
- Use environment variables or secure key management in production
- Consider using hardware wallets for mainnet deployments

### Step 4: Build Contract

```bash
cd contracts-stylus

# Build for release (optimized)
cargo stylus build --release
```

This compiles your Rust code to WASM. The build should complete without errors.

### Step 5: Deploy to Arbitrum One

**Option A: Using Deployment Script**

```bash
# Ensure .env is configured
export NETWORK=arbitrum-one
./scripts/deploy.sh
```

**Option B: Manual Deployment**

```bash
# Load environment variables
source .env

# Deploy contract
cargo stylus deploy \
  --constructor-args "$VKEY" "$VERIFIER_ADDRESS" \
  --network arbitrum-one \
  --private-key "$PRIVATE_KEY"
```

**Expected Output:**
```
Deploying contract...
Contract deployed at: 0x...
Transaction hash: 0x...
```

**Save the contract address:**
```bash
export CONTRACT_ADDRESS="0x..."  # From deployment output
```

### Step 6: Activate Contract

Activation compiles WASM to native code for optimal performance. **This is required** before the contract can be used.

```bash
cargo stylus activate \
  --address "$CONTRACT_ADDRESS" \
  --network arbitrum-one \
  --private-key "$PRIVATE_KEY"
```

**Expected Output:**
```
Activating contract...
Contract activated successfully
```

⚠️ **Note:** Activation can take a few minutes. Wait for confirmation.

### Step 7: Verify Deployment

#### Check Contract Status

```bash
cargo stylus check \
  --address "$CONTRACT_ADDRESS" \
  --network arbitrum-one
```

#### Test View Function

```bash
# Test is_document_number_used (should return false for unused hash)
cargo stylus call \
  --address "$CONTRACT_ADDRESS" \
  --function "is_document_number_used" \
  --args "0x0000000000000000000000000000000000000000000000000000000000000000" \
  --network arbitrum-one
```

#### Verify on Arbiscan

1. Go to [Arbiscan](https://arbiscan.io/)
2. Search for your contract address: `$CONTRACT_ADDRESS`
3. Verify the contract code (optional but recommended)

### Step 8: Update Configuration

Update your `.env` file with the deployed address:

```bash
echo "CONTRACT_ADDRESS=$CONTRACT_ADDRESS" >> .env
```

## Post-Deployment

### 1. Update Frontend

Update your frontend configuration with the new contract address:

```javascript
// frontend/config.js
export const CONTRACT_ADDRESS = "0x...";  // Your deployed address
export const NETWORK = "arbitrum-one";
export const CHAIN_ID = 42161;
```

### 2. Test with Real Proofs

```javascript
// Test the deployed contract
const contract = new ethers.Contract(
  CONTRACT_ADDRESS,
  ABI,
  signer
);

// Generate proof from server
const proofData = await generateProofFromServer(passportData);

// Submit to contract
const tx = await contract.verifyAgeProof(
  proofData.proof,
  proofData.public_values,
  documentNumberHash
);

await tx.wait();
console.log("Proof verified on mainnet!");
```

### 3. Monitor Contract

- Monitor contract activity on Arbiscan
- Set up alerts for important events
- Track gas usage and costs

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

**Insufficient Funds:**
```bash
# Check balance
# Bridge more ETH: https://bridge.arbitrum.io/
```

**Network Connection:**
```bash
# Test RPC connection
curl -X POST $ARBITRUM_RPC \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

**Invalid VKEY:**
```bash
# Ensure VKEY matches proof generation
# Get fresh VKEY from proof server
curl http://localhost:3000/proof/vkey | jq -r '.vkey_hex'
```

### Activation Errors

```bash
# Wait for deployment confirmation (wait for ~12 block confirmations)
# Ensure contract address is correct
# Try activation again after a few minutes
```

## Security Best Practices

1. **Private Key Management**
   - Never commit private keys to git
   - Use hardware wallets for mainnet
   - Consider using multi-sig for contract ownership

2. **Verification**
   - Verify contract code on Arbiscan
   - Test thoroughly on testnet first
   - Use formal verification if possible

3. **Monitoring**
   - Set up monitoring for contract events
   - Track unusual activity
   - Have a response plan for issues

4. **Backup**
   - Save all deployment addresses
   - Document all configuration values
   - Keep deployment transaction hashes

## Network Configuration

### Arbitrum One Mainnet

```bash
NETWORK=arbitrum-one
RPC_URL=https://arb1.arbitrum.io/rpc
CHAIN_ID=42161
EXPLORER=https://arbiscan.io
```

### Alternative RPC Providers

For better reliability, use a dedicated RPC provider:

- **Alchemy:** `https://arb-mainnet.g.alchemy.com/v2/YOUR_API_KEY`
- **Infura:** `https://arbitrum-mainnet.infura.io/v3/YOUR_API_KEY`
- **QuickNode:** `https://YOUR_ENDPOINT.arbitrum-mainnet.quiknode.pro/YOUR_KEY/`

## Cost Estimates

| Operation | Estimated Gas | Cost (at 0.1 gwei) |
|-----------|--------------|-------------------|
| SP1 Verifier Deploy | ~2M gas | ~0.02 ETH |
| Stylus Contract Deploy | ~5M gas | ~0.05 ETH |
| Contract Activation | ~1M gas | ~0.01 ETH |
| **Total Deployment** | **~8M gas** | **~0.08 ETH** |
| Proof Verification | ~30-55K gas | ~0.00003-0.00005 ETH |

*Gas prices vary. Check current prices before deploying.*

## Quick Reference

```bash
# Full deployment command
cd contracts-stylus
source .env
cargo stylus build --release
cargo stylus deploy \
  --constructor-args "$VKEY" "$VERIFIER_ADDRESS" \
  --network arbitrum-one \
  --private-key "$PRIVATE_KEY"
cargo stylus activate \
  --address "$CONTRACT_ADDRESS" \
  --network arbitrum-one \
  --private-key "$PRIVATE_KEY"
```

## Next Steps

1. ✅ Contract deployed and activated
2. ✅ Test with real proofs
3. ✅ Verify on Arbiscan
4. ✅ Update frontend
5. ✅ Monitor contract activity
6. ✅ Set up alerts and monitoring

## Resources

- [Arbitrum Stylus Docs](https://docs.arbitrum.io/stylus)
- [Arbiscan Explorer](https://arbiscan.io/)
- [Arbitrum Bridge](https://bridge.arbitrum.io/)
- [cargo-stylus CLI](https://github.com/OffchainLabs/cargo-stylus)
- [SP1 SDK](https://github.com/succinctlabs/sp1)

## Support

If you encounter issues:

1. Check the [Troubleshooting](#troubleshooting) section
2. Review deployment logs
3. Verify all environment variables
4. Test on Arbitrum Sepolia first
5. Check Arbitrum Stylus documentation

---

**⚠️ Remember:** Mainnet deployments are permanent. Test thoroughly on testnet first!

