# Age Verification Contract for Arbitrum Stylus

Rust-based smart contract for Arbitrum Stylus that verifies SP1 zkVM proofs onchain.

## Why Stylus?

- **Performance:** ~10-100x faster execution than EVM
- **Lower Gas Costs:** Significant gas savings
- **Rust:** Type-safe, memory-safe contract development
- **Interoperability:** Can call Solidity contracts (like SP1 verifier)

## Architecture

```
┌─────────────────────────────────────┐
│   AgeVerification (Stylus/Rust)      │
│                                      │
│   - Receives proof + public_values   │
│   - Calls SP1Verifier (Solidity)     │
│   - Stores results onchain           │
└──────────────┬──────────────────────┘
               │
               │ calls
               ▼
┌─────────────────────────────────────┐
│   SP1Verifier (Solidity Contract)   │
│                                      │
│   - Verifies ZK proof                 │
│   - Returns true/false                │
└─────────────────────────────────────┘
```

## Setup

### 1. Install Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-stylus
cargo install --force cargo-stylus

# Add WASM target
rustup target add wasm32-unknown-unknown
```

### 2. Build Contract

```bash
cd contracts-stylus
cargo stylus build --release
```

### 3. Deploy SP1 Verifier (Solidity)

First, deploy the SP1 verifier contract (Solidity):

```solidity
// Deploy SP1Verifier.sol first
SP1Verifier verifier = new SP1Verifier();
```

### 4. Deploy Age Verification Contract (Stylus)

```bash
# Get vkey from proof server
VKEY=$(curl -s http://localhost:3000/proof/vkey | jq -r '.vkey')

# Deploy Stylus contract
cargo stylus deploy \
  --constructor-args "$VKEY" "$VERIFIER_ADDRESS" \
  --network arbitrum-sepolia
```

## Contract Functions

### `verify_age_proof(proof, public_values, document_number_hash)`

Main function - verifies proof and stores result.

**Parameters:**
- `proof` (bytes): Proof from server
- `public_values` (bytes): Public values from server
- `document_number_hash` (bytes32): Hash of document number

**Returns:**
- `(bool, bool)`: `(isValid, meetsAgeRequirement)`

### `verify_proof_only(proof, public_values)`

View function - verifies proof without storing.

### `get_proof_record(document_number_hash)`

View function - retrieves stored proof record.

### `is_document_number_used(document_number_hash)`

View function - checks if document number already verified.

### `init(vkey, verifier_address)`

Constructor - sets vkey and SP1 verifier address.

## Deployment Steps

### Step 1: Deploy SP1 Verifier

The SP1 verifier is a Solidity contract. Deploy it first:

```bash
# Using Hardhat/Foundry
forge install succinctlabs/sp1-sdk
forge build
forge create SP1Verifier --rpc-url $ARBITRUM_RPC
```

### Step 2: Deploy Stylus Contract

```bash
# Build
cargo stylus build --release

# Deploy
cargo stylus deploy \
  --constructor-args "$VKEY" "$VERIFIER_ADDRESS" \
  --network arbitrum-sepolia \
  --private-key $PRIVATE_KEY
```

### Step 3: Activate Contract

After deployment, activate the contract:

```bash
cargo stylus activate --address $CONTRACT_ADDRESS --network arbitrum-sepolia
```

Activation compiles WASM to native code for optimal performance.

## Frontend Integration

```javascript
import { StylusContract } from '@arbitrum/sdk';

// Connect to Stylus contract
const contract = new StylusContract(
  CONTRACT_ADDRESS,
  ABI,
  signer
);

// Generate proof (offchain)
const proof = await generateProofFromServer(passportData);

// Submit to Stylus contract
const tx = await contract.verifyAgeProof(
  proof.proof,
  proof.public_values,
  documentNumberHash
);

await tx.wait();
```

## Gas Comparison

| Operation | Solidity | Stylus | Savings |
|-----------|----------|--------|---------|
| Proof Verification | ~200K gas | ~20-40K gas | 80-90% |
| Storage | ~20K gas | ~5-10K gas | 50-75% |
| **Total** | **~220K gas** | **~25-50K gas** | **~75-85%** |

## Advantages of Stylus

1. **Lower Gas Costs:** 75-85% cheaper than Solidity
2. **Faster Execution:** Native WASM execution
3. **Type Safety:** Rust's type system prevents bugs
4. **Better Tooling:** Cargo, rust-analyzer, etc.
5. **Interoperability:** Can call Solidity contracts

## Limitations

1. **SP1 Verifier:** Must be deployed as separate Solidity contract
2. **Activation:** Contract must be activated after deployment
3. **Network Support:** Only on Arbitrum chains (Arbitrum One, Nova, Sepolia)

## Testing

```bash
# Run tests
cargo test

# Test with Stylus test VM
cargo stylus test
```

## Documentation

- [Arbitrum Stylus Docs](https://docs.arbitrum.io/stylus)
- [Stylus SDK Reference](https://docs.arbitrum.io/stylus/reference/stylus-sdk)
- `SMART_CONTRACT_GUIDE.md` - Detailed integration guide

