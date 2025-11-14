# Proof Systems Guide

## Available Proof Systems

### Groth16 (Recommended for EVM) ✅

**Use for:** Smart contract integration

- **Proof Size:** ~2KB
- **Verification Gas:** ~200K gas
- **Generation Time:** 2-5 minutes
- **Onchain Verification:** ✅ Supported
- **Default:** Yes (for smart contract use)

**Usage:**
```json
{
  "proof_system": "groth16"
}
```

### PLONK ✅

**Use for:** Smart contract integration (alternative to Groth16)

- **Proof Size:** ~3KB
- **Verification Gas:** ~250K gas
- **Generation Time:** 3-6 minutes
- **Onchain Verification:** ✅ Supported

**Usage:**
```json
{
  "proof_system": "plonk"
}
```

### Core ❌ (Not for Smart Contracts)

**Use for:** Testing, offchain verification only

- **Proof Size:** ~100KB
- **Verification Gas:** N/A (not EVM-compatible)
- **Generation Time:** 30-60 seconds
- **Onchain Verification:** ❌ NOT SUPPORTED

**Note:** Core proofs cannot be verified onchain. If you request a Core proof, the server will return an error explaining that only Groth16 and PLONK proofs can be used for smart contract integration.

## Default Behavior

**If `proof_system` is not specified:**
- Defaults to `"groth16"` for EVM compatibility
- This ensures proofs can be used with smart contracts

## Error Handling

If you request a Core proof, you'll get:

```json
{
  "success": false,
  "proof": null,
  "error": "Core proofs are not supported for onchain verification. Please use 'groth16' or 'plonk' for smart contract integration."
}
```

## Recommendations

### For Smart Contract Integration
```json
{
  "proof_system": "groth16"  // Best choice: smallest proof, fastest verification
}
```

### For Testing (Offchain Only)
If you need faster proof generation for testing, you can temporarily use Core proofs, but they cannot be submitted to smart contracts. For production, always use Groth16 or PLONK.

## Performance Comparison

| System | Generation | Verification | Proof Size | Gas Cost |
|--------|-----------|--------------|------------|----------|
| Groth16 | 2-5 min | ~15 sec | ~2KB | ~200K gas |
| PLONK | 3-6 min | ~15 sec | ~3KB | ~250K gas |
| Core | 30-60 sec | N/A | ~100KB | N/A |

## Migration from Core to Groth16

If you were using Core proofs for testing:

**Before:**
```json
{
  "proof_system": "core"  // ❌ Won't work for smart contracts
}
```

**After:**
```json
{
  "proof_system": "groth16"  // ✅ Works for smart contracts
}
```

The only difference is longer proof generation time (2-5 minutes vs 30-60 seconds), but this is necessary for onchain verification.

