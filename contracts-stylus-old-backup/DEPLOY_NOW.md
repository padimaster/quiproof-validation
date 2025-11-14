# Deploy Your Stylus Contract - Quick Guide

Follow these steps to deploy your contract right now.

## Prerequisites Check

Before deploying, make sure you have:

1. ✅ **VERIFIER_ADDRESS** - Already set to canonical address
2. ⚠️ **VKEY** - Need to get this
3. ⚠️ **PRIVATE_KEY** - Need to set this
4. ⚠️ **NETWORK** - Set to `arbitrum-one` for mainnet

## Step 1: Get Your Verification Key (VKEY)

The VKEY must match the one used by your proof server. Get it using one of these methods:

### Option A: From Proof Server (If Running)

```bash
# If your proof server is running on localhost:3000
curl -s http://localhost:3000/proof/vkey | jq -r '.vkey_hex'
```

### Option B: From Circuit

```bash
cd ../circuits/script
cargo run --release --bin vkey 2>/dev/null | grep -o '0x[0-9a-f]\{64\}' || echo "Run: cargo run --release --bin vkey"
```

### Option C: Use Default (If You Know It)

If you've deployed before and know your VKEY, use it directly.

## Step 2: Update Your .env File

Edit `contracts-stylus/.env` and ensure these are set:

```bash
# Network (arbitrum-one for mainnet, arbitrum-sepolia for testnet)
NETWORK=arbitrum-one

# Verification Key (get from Step 1)
VKEY=0x...

# SP1 Verifier Address (already set)
VERIFIER_ADDRESS=0x3B6041173B80E77f038f3F2C0f9744f04837185e

# Your private key (NEVER commit to git!)
PRIVATE_KEY=0x...

# Arbitrum RPC (optional, defaults to public RPC)
ARBITRUM_RPC=https://arb1.arbitrum.io/rpc
```

## Step 3: Build the Contract

```bash
cd contracts-stylus
cargo stylus build --release
```

This compiles your Rust code to WASM. Takes a few minutes the first time.

## Step 4: Deploy the Contract

### Option A: Using Deployment Script (Recommended)

```bash
cd contracts-stylus
source .env  # Load environment variables
./scripts/deploy.sh
```

The script will:
- Check prerequisites
- Build the contract
- Deploy to Arbitrum One
- Activate the contract
- Save the contract address to `.env`

### Option B: Manual Deployment

```bash
cd contracts-stylus
source .env

# Deploy
cargo stylus deploy \
  --constructor-args "$VKEY" "$VERIFIER_ADDRESS" \
  --network arbitrum-one \
  --private-key "$PRIVATE_KEY"
```

**Save the contract address from the output!**

## Step 5: Activate the Contract

Activation is **required** - the contract won't work until activated.

```bash
# Get contract address from deployment output
export CONTRACT_ADDRESS="0x..."  # From Step 4

# Activate
cargo stylus activate \
  --address "$CONTRACT_ADDRESS" \
  --network arbitrum-one \
  --private-key "$PRIVATE_KEY"
```

Activation can take a few minutes. Wait for confirmation.

## Step 6: Verify Deployment

```bash
# Check contract status
cargo stylus check \
  --address "$CONTRACT_ADDRESS" \
  --network arbitrum-one

# Test a view function
cargo stylus call \
  --address "$CONTRACT_ADDRESS" \
  --function "is_document_number_used" \
  --args "0x0000000000000000000000000000000000000000000000000000000000000000" \
  --network arbitrum-one
```

## Quick Deploy Command (All-in-One)

If you have everything set up in `.env`:

```bash
cd contracts-stylus
source .env

# Build
cargo stylus build --release

# Deploy
cargo stylus deploy \
  --constructor-args "$VKEY" "$VERIFIER_ADDRESS" \
  --network "$NETWORK" \
  --private-key "$PRIVATE_KEY"

# Save CONTRACT_ADDRESS from output, then activate:
cargo stylus activate \
  --address "$CONTRACT_ADDRESS" \
  --network "$NETWORK" \
  --private-key "$PRIVATE_KEY"
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

**Insufficient Funds:**
- Check your Arbitrum One balance
- You need ~0.1-0.2 ETH for deployment + activation
- Bridge ETH: https://bridge.arbitrum.io/

**Network Connection:**
```bash
# Test RPC connection
curl -X POST "$ARBITRUM_RPC" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

**Invalid VKEY:**
- Ensure VKEY matches your proof server
- Get fresh VKEY: `curl http://localhost:3000/proof/vkey | jq -r '.vkey_hex'`

### Activation Errors

- Wait for deployment confirmation (~12 blocks)
- Ensure contract address is correct
- Try again after a few minutes

## What You'll Get

After successful deployment:

- ✅ **Contract Address**: `0x...` (save this!)
- ✅ **Transaction Hash**: `0x...` (for verification)
- ✅ **Contract Activated**: Ready to use

## Next Steps

1. ✅ Contract deployed and activated
2. Update frontend with contract address
3. Test with real proofs
4. Monitor on Arbiscan: https://arbiscan.io/address/YOUR_CONTRACT_ADDRESS

## Cost Estimate

- **Deployment**: ~0.05-0.1 ETH
- **Activation**: ~0.01-0.02 ETH
- **Total**: ~0.1-0.2 ETH (varies with gas prices)

---

**Ready to deploy?** Start with Step 1 above! 🚀

