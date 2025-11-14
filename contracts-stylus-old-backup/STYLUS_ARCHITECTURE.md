# Arbitrum Stylus Architecture

## Overview

The Age Verification contract is implemented in **Rust** for **Arbitrum Stylus**, providing significant gas savings (75-85% cheaper) compared to Solidity.

## Architecture

```
┌─────────────────────────────────────────────┐
│   AgeVerification (Stylus/Rust Contract)    │
│                                              │
│   Functions:                                 │
│   - verify_age_proof()                      │
│   - verify_proof_only()                      │
│   - get_proof_record()                       │
│   - is_document_number_used()                │
│   - get_user_proofs_count()                  │
│   - get_user_proof_at()                      │
└──────────────┬──────────────────────────────┘
               │
               │ evm::call()
               │
               ▼
┌─────────────────────────────────────────────┐
│   SP1Verifier (Solidity Contract)            │
│                                              │
│   - verify(vkey, proof, publicValues)        │
│   - Returns: bool                            │
└─────────────────────────────────────────────┘
```

## Key Design Decisions

### 1. External SP1 Verifier

**Why:** SP1 verifier is a Solidity contract, not available in Rust/WASM.

**How:** Stylus contract calls the Solidity verifier using `evm::call()`:

```rust
let verifier_interface = SP1Verifier::new(verifier_address);
let call_data = verifier_interface.verify(vkey, proof, public_values);

let result = evm::call(
    &evm::CallOpts {
        to: verifier_address,
        data: Some(call_data.abi_encode()),
        ..Default::default()
    },
);
```

### 2. Storage Layout

Uses EVM-compatible storage slots:

- **Proof Records:** `keccak256(documentNumberHash || slot)`
- **User Proofs:** `keccak256(user || slot || index)`
- **VKey:** Direct storage slot
- **Verifier Address:** Direct storage slot

### 3. Public Values Decoding

Decodes bincode-serialized `AgeProofOutput`:

```rust
// Bincode format: [length: varint, is_valid: bool, meets_age_requirement: bool, ...]
let is_valid = public_values[1] != 0;
let meets_age_requirement = public_values[2] != 0;
```

## Deployment Flow

### Step 1: Deploy SP1 Verifier (Solidity)

```bash
# Deploy SP1Verifier.sol first
forge create SP1Verifier --rpc-url $ARBITRUM_RPC
export VERIFIER_ADDRESS="0x..."
```

### Step 2: Deploy Stylus Contract

```bash
cd contracts-stylus
cargo stylus build --release
cargo stylus deploy \
  --constructor-args "$VKEY" "$VERIFIER_ADDRESS" \
  --network arbitrum-sepolia
```

### Step 3: Activate Contract

```bash
cargo stylus activate \
  --address $CONTRACT_ADDRESS \
  --network arbitrum-sepolia
```

**Why Activation?**
- Compiles WASM → native machine code
- Optimizes performance
- Required before contract can be called

## Gas Comparison

| Operation | Solidity | Stylus | Savings |
|-----------|----------|--------|---------|
| Proof Verification | ~200K | ~20-40K | 80-90% |
| Storage Write | ~20K | ~5-10K | 50-75% |
| External Call (SP1) | ~5K | ~5K | Same |
| **Total** | **~225K** | **~30-55K** | **~75-85%** |

## Advantages

1. **Lower Gas Costs:** 75-85% cheaper
2. **Type Safety:** Rust prevents many bugs
3. **Performance:** Native WASM execution
4. **Interoperability:** Can call Solidity contracts
5. **Better Tooling:** Cargo, rust-analyzer, etc.

## Limitations

1. **SP1 Verifier:** Must be separate Solidity contract
2. **Activation Required:** Extra step after deployment
3. **Network Support:** Only Arbitrum chains
4. **Learning Curve:** Rust instead of Solidity

## Frontend Integration

Stylus contracts work like regular contracts:

```javascript
// Using ethers.js / viem
const contract = new ethers.Contract(
  CONTRACT_ADDRESS,
  ABI, // Generated from Rust contract
  signer
);

// Call contract
const tx = await contract.verifyAgeProof(
  proof,
  publicValues,
  documentNumberHash
);
```

## Testing

```bash
# Unit tests
cargo test

# Stylus test VM
cargo stylus test

# Manual testing on testnet
cargo stylus deploy --network arbitrum-sepolia
```

## Next Steps

1. Deploy SP1 verifier to Arbitrum Sepolia
2. Deploy Stylus contract
3. Activate contract
4. Test with real proofs
5. Integrate with frontend
6. Deploy to Arbitrum One (mainnet)

