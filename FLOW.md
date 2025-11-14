# System Flow Documentation

## Current Flow: Offchain Proof Generation → Frontend → Smart Contract

Yes, this is the current flow! Here's how it works:

## Step-by-Step Flow

### 1. Frontend → Server (Proof Generation - Offchain)

**What happens:**
- User scans passport in frontend app
- Frontend extracts DG1 and SOD data
- Frontend sends data to proof server
- Server generates ZK proof (2-5 minutes, offchain, no gas cost)

**Request:**
```javascript
POST /proof/generate
{
  "dg1_base64": "...",
  "sod_base64": "...",
  "age_to_verify": 18,
  "current_date": [2024, 1, 15],
  "proof_system": "groth16"
}
```

### 2. Server → Frontend (Proof Response)

**What happens:**
- Server returns proof data ready for smart contract
- Frontend receives: `proof`, `public_values`, `vkey`, `output`

**Response:**
```json
{
  "success": true,
  "proof": {
    "proof": "0x...",           // For smart contract
    "public_values": "0x...",   // For smart contract
    "vkey": "...",              // For smart contract
    "output": {
      "is_valid": true,
      "meets_age_requirement": true,
      "document_number_hash": [...]
    }
  }
}
```

### 3. Frontend → Smart Contract (Onchain Verification)

**What happens:**
- Frontend validates `output.is_valid` (optional, saves gas)
- Frontend submits proof to smart contract
- Contract verifies proof cryptographically (~200K gas)
- Contract emits event with result

**Transaction:**
```javascript
contract.verifyAgeProof(proof, public_values, vkey)
```

## Why This Architecture?

### ✅ Benefits

1. **Cost Efficient**
   - Proof generation: FREE (offchain)
   - Only verification costs gas: ~$0.10-0.50

2. **Fast User Experience**
   - Proof generation happens in background
   - User can see validation result immediately (`output.is_valid`)
   - Onchain verification is optional (for permanent record)

3. **Privacy Preserving**
   - Passport data never leaves server
   - Only proof + public values go to blockchain
   - Public values contain only validation result

4. **Scalable**
   - Server can handle many requests
   - Can use prover network for faster generation
   - Smart contract verification is fast

## Current Implementation

✅ **What's Working:**
- Server generates proofs offchain
- Returns proof + public_values + vkey
- EVM-compatible proofs (Groth16/PLONK)
- Frontend can receive and validate proof

⏳ **What's Next:**
- Deploy smart contract
- Frontend submits proof to contract
- Contract verifies and stores result

## Example Frontend Code

See `server/frontend_example.js` for complete implementation.

**Quick Example:**
```javascript
// 1. Generate proof (offchain)
const proof = await generateProof(passportData);

// 2. Validate output (optional, saves gas)
if (!proof.output.is_valid) {
  throw new Error('Invalid passport');
}

// 3. Submit to contract (onchain)
await contract.verifyAgeProof(
  proof.proof,
  proof.public_values,
  proof.vkey
);
```

## Trust Model

- **Server:** Generates proofs (trusted to generate correctly)
- **Smart Contract:** Verifies proofs (trustless, cryptographic verification)

The contract doesn't trust the server - it verifies the cryptographic proof. If the server generates an invalid proof, the contract will reject it.

## Performance

| Step | Location | Time | Cost |
|------|----------|------|------|
| Proof Generation | Server (offchain) | 2-5 min | FREE |
| Proof Validation | Frontend | <1 sec | FREE |
| Contract Verification | Blockchain | ~15 sec | ~200K gas |

**Total:** ~2-5 minutes, ~$0.10-0.50 (only gas for verification)

