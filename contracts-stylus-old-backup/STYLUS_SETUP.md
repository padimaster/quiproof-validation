# Arbitrum Stylus Setup Guide

Complete guide for setting up and deploying the Age Verification contract on Arbitrum Stylus.

## Prerequisites

1. **Rust 1.80+**
   ```bash
   rustup default 1.80
   ```

2. **cargo-stylus CLI**
   ```bash
   cargo install --force cargo-stylus
   ```

3. **WASM Target**
   ```bash
   rustup target add wasm32-unknown-unknown --toolchain 1.80
   ```

4. **Arbitrum Account**
   - Funded with ETH for deployment
   - Private key or wallet connection

## Project Structure

```
contracts-stylus/
├── Cargo.toml          # Rust dependencies
├── src/
│   └── lib.rs          # Main contract code
├── README.md
└── STYLUS_SETUP.md
```

## Step-by-Step Deployment

### Step 1: Deploy SP1 Verifier (Solidity)

The SP1 verifier must be deployed as a Solidity contract first, as it's not available in Rust/WasM.

**Option A: Use Existing Deployment**

If SP1 verifier is already deployed on Arbitrum, use that address.

**Option B: Deploy New Verifier**

```bash
# Using Foundry
forge install succinctlabs/sp1-sdk
forge create SP1Verifier --rpc-url $ARBITRUM_RPC --private-key $PRIVATE_KEY

# Save the deployed address
export VERIFIER_ADDRESS="0x..."
```

### Step 2: Get Verification Key

```bash
# From proof server
VKEY=$(curl -s http://localhost:3000/proof/vkey | jq -r '.vkey')

# Or from circuit
cd ../circuits/script
VKEY=$(cargo run --release --bin vkey | grep -o '0x[0-9a-f]\{64\}')

export VKEY
```

### Step 3: Build Stylus Contract

```bash
cd contracts-stylus

# Build for release (optimized)
cargo stylus build --release
```

This compiles the Rust code to WASM.

### Step 4: Deploy Contract

```bash
# Set network (arbitrum-sepolia for testnet)
export NETWORK=arbitrum-sepolia
export PRIVATE_KEY=your_private_key

# Deploy
cargo stylus deploy \
  --constructor-args "$VKEY" "$VERIFIER_ADDRESS" \
  --network $NETWORK \
  --private-key $PRIVATE_KEY
```

**Constructor Arguments:**
- `vkey`: Verification key (bytes32)
- `verifier_address`: SP1 verifier contract address

### Step 5: Activate Contract

After deployment, activate the contract to compile WASM to native code:

```bash
export CONTRACT_ADDRESS="0x..." # From deployment output

cargo stylus activate \
  --address $CONTRACT_ADDRESS \
  --network $NETWORK \
  --private-key $PRIVATE_KEY
```

**Why Activation?**
- Compiles WASM to native machine code
- Optimizes performance
- Required before contract can be called

### Step 6: Verify Deployment

```bash
# Check contract is active
cargo stylus check --address $CONTRACT_ADDRESS --network $NETWORK

# Test a view function
cargo stylus call \
  --address $CONTRACT_ADDRESS \
  --function "is_document_number_used" \
  --args "0x0000000000000000000000000000000000000000000000000000000000000000" \
  --network $NETWORK
```

## Environment Variables

Create `.env` file:

```bash
# Arbitrum RPC
ARBITRUM_RPC=https://sepolia-rollup.arbitrum.io/rpc

# Private key (for deployment)
PRIVATE_KEY=0x...

# Contract addresses (after deployment)
VERIFIER_ADDRESS=0x...
CONTRACT_ADDRESS=0x...

# Verification key
VKEY=0x...
```

## Testing Locally

### Using Stylus Test VM

```bash
# Run tests
cargo test

# Test with Stylus VM
cargo stylus test
```

### Manual Testing

```bash
# Deploy to local Stylus node (if available)
cargo stylus deploy --network localhost

# Or use Arbitrum Sepolia testnet
cargo stylus deploy --network arbitrum-sepolia
```

## Frontend Integration

### Using ethers.js / viem

```javascript
import { createPublicClient, createWalletClient, http } from 'viem';
import { arbitrumSepolia } from 'viem/chains';

// Stylus contracts work like regular contracts
const contract = {
  address: CONTRACT_ADDRESS,
  abi: ABI, // Generated from Rust contract
};

// Call contract
const result = await publicClient.readContract({
  ...contract,
  functionName: 'is_document_number_used',
  args: [documentNumberHash],
});
```

### Using Stylus SDK

```javascript
import { StylusContract } from '@arbitrum/sdk';

const contract = new StylusContract(
  CONTRACT_ADDRESS,
  ABI,
  signer
);

const tx = await contract.verifyAgeProof(
  proof,
  publicValues,
  documentNumberHash
);
```

## Gas Optimization

Stylus contracts are significantly cheaper:

- **Proof Verification:** ~20-40K gas (vs ~200K in Solidity)
- **Storage Operations:** ~5-10K gas (vs ~20K in Solidity)
- **Total Savings:** 75-85% cheaper

## Troubleshooting

### Build Errors

```bash
# Ensure correct Rust version
rustup default 1.80

# Clean and rebuild
cargo clean
cargo stylus build --release
```

### Deployment Errors

```bash
# Check network connection
curl $ARBITRUM_RPC -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Check account has funds
# Check private key is correct
```

### Activation Errors

```bash
# Ensure contract is deployed first
# Check contract address is correct
# Wait for deployment confirmation before activating
```

## Network Configuration

### Arbitrum Sepolia (Testnet)

```bash
export NETWORK=arbitrum-sepolia
export RPC_URL=https://sepolia-rollup.arbitrum.io/rpc
```

### Arbitrum One (Mainnet)

```bash
export NETWORK=arbitrum-one
export RPC_URL=https://arb1.arbitrum.io/rpc
```

### Arbitrum Nova

```bash
export NETWORK=arbitrum-nova
export RPC_URL=https://nova.arbitrum.io/rpc
```

## Next Steps

1. Deploy SP1 verifier contract
2. Deploy Stylus contract
3. Activate contract
4. Test with real proofs
5. Integrate with frontend
6. Deploy to mainnet

## Resources

- [Arbitrum Stylus Docs](https://docs.arbitrum.io/stylus)
- [Stylus Examples](https://github.com/OffchainLabs/stylus-hello-world)
- [cargo-stylus CLI](https://github.com/OffchainLabs/cargo-stylus)

