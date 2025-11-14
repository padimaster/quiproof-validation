# System Architecture & Flow

## Current Flow: Offchain Proof Generation → Frontend → Smart Contract

```
┌─────────────┐
│   Frontend  │
│  (External) │
└──────┬──────┘
       │
       │ 1. User submits passport data
       │    (DG1, SOD, age requirement)
       ▼
┌─────────────────────────────────────┐
│      Proof Server (Offchain)        │
│  http://localhost:3000               │
│                                      │
│  POST /proof/generate                │
│  - Receives: dg1_base64, sod_base64  │
│  - Generates: ZK proof (Groth16)    │
│  - Returns: proof + public_values    │
└──────┬──────────────────────────────┘
       │
       │ 2. Returns proof data
       │    {
       │      proof: "0x...",
       │      public_values: "0x...",
       │      vkey: "...",
       │      output: { is_valid, ... }
       │    }
       ▼
┌─────────────┐
│   Frontend  │
│  (External) │
└──────┬──────┘
       │
       │ 3. Frontend receives proof
       │    Validates output.is_valid
       │
       │ 4. Submits to smart contract
       │    contract.verifyProof(
       │      proof,
       │      public_values,
       │      vkey
       │    )
       ▼
┌─────────────────────────────────────┐
│      Smart Contract (Onchain)       │
│                                      │
│  - Verifies proof                    │
│  - Checks public_values              │
│  - Returns: true/false               │
│  - Emits event if valid              │
└─────────────────────────────────────┘
```

## Detailed Flow

### Step 1: Frontend → Server (Proof Generation)

**Frontend sends:**
```javascript
POST http://localhost:3000/proof/generate
{
  "dg1_base64": "...",           // Passport DG1 data
  "sod_base64": "...",           // Passport SOD data
  "age_to_verify": 18,
  "current_date": [2024, 1, 15],
  "trusted_csca_certs_file": "trusted_csca_certs_ecuador.json",
  "proof_system": "groth16"      // EVM-compatible
}
```

**Server generates proof (offchain):**
- Parses passport data
- Verifies certificate chain
- Calculates age
- Generates ZK proof (2-5 minutes for Groth16)
- Returns proof data

### Step 2: Server → Frontend (Proof Response)

**Server returns:**
```json
{
  "success": true,
  "proof": {
    "proof": "0x...",              // Hex-encoded proof (for smart contract)
    "public_values": "0x...",      // Hex-encoded public values (for smart contract)
    "vkey": "...",                 // Verification key (for smart contract)
    "output": {
      "is_valid": true,
      "meets_age_requirement": true,
      "document_number_hash": [...],
      "debug_age": 24,
      ...
    },
    "proof_system": "groth16"
  }
}
```

### Step 3: Frontend → Smart Contract (Onchain Verification)

**Frontend submits to contract:**
```javascript
// Using ethers.js or web3.js
const tx = await contract.verifyAgeProof(
  proof.proof,           // "0x..."
  proof.public_values,   // "0x..."
  proof.vkey            // "..."
);

await tx.wait();
```

**Smart contract verifies:**
- Verifies ZK proof on-chain
- Checks public values match
- Returns true if valid
- Emits event with validation result

## Why This Flow?

### ✅ Advantages

1. **Cost Efficiency**
   - Proof generation is expensive (2-5 minutes CPU time)
   - Done offchain = no gas costs
   - Only verification happens onchain (cheap)

2. **Privacy**
   - Passport data never leaves the server
   - Only proof + public values go to blockchain
   - Public values contain only: `is_valid`, `meets_age_requirement`, `document_number_hash`

3. **Scalability**
   - Server can handle multiple requests
   - Can use prover network for faster generation
   - Smart contract verification is fast (~200K gas)

4. **Security**
   - Proof cryptographically proves age without revealing passport data
   - Certificate verification happens offchain (too expensive onchain)
   - Smart contract only needs to verify the proof

### ⚠️ Trust Model

- **Trusted:** Proof server (generates proofs correctly)
- **Trustless:** Smart contract (verifies proofs cryptographically)

The smart contract doesn't trust the server - it verifies the cryptographic proof. If the server generates an invalid proof, the contract will reject it.

## Smart Contract Integration

### Example Solidity Contract

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {SP1Verifier} from "@succinctlabs/sp1-sdk/contracts/SP1Verifier.sol";

contract AgeVerification {
    SP1Verifier public verifier;
    bytes32 public vkey;
    
    event AgeVerified(
        address indexed user,
        bool isValid,
        bool meetsAgeRequirement,
        bytes32 documentNumberHash
    );
    
    constructor(bytes32 _vkey) {
        verifier = new SP1Verifier();
        vkey = _vkey;
    }
    
    function verifyAgeProof(
        bytes calldata proof,
        bytes calldata publicValues
    ) external returns (bool) {
        // Verify the ZK proof
        bool isValid = verifier.verify(vkey, proof, publicValues);
        
        if (isValid) {
            // Decode public values to get output
            // (implementation depends on your public values encoding)
            // For now, just emit the verification result
            emit AgeVerified(
                msg.sender,
                true,  // Would decode from publicValues
                true,  // Would decode from publicValues
                bytes32(0)  // Would decode from publicValues
            );
        }
        
        return isValid;
    }
}
```

### Frontend Integration Example

See `server/example_client.js` for a complete example.

## Current Implementation Status

✅ **Implemented:**
- Offchain proof generation server
- REST API for proof generation
- Returns proof + public_values + vkey
- EVM-compatible proofs (Groth16, PLONK)

⏳ **Next Steps:**
- Deploy smart contract with vkey
- Frontend integration (submit proofs to contract)
- Event handling for validation results

## Performance

- **Proof Generation:** 2-5 minutes (offchain, no gas)
- **Contract Verification:** ~200K gas (~$0.10-0.50 depending on network)
- **Total Cost:** Only gas for verification (proof generation is free offchain)

## Security Considerations

1. **Server Trust:** Server must generate proofs correctly
   - Mitigation: Open source server code, audits
   - Future: Decentralized prover network

2. **Frontend Validation:** Frontend should validate `output.is_valid` before submitting
   - Prevents wasting gas on invalid proofs
   - Better UX (immediate feedback)

3. **Replay Attacks:** Use `document_number_hash` to prevent reuse
   - Store used hashes in contract
   - Reject if hash already used

4. **Certificate Trust:** Trusted CSCA certificates loaded on server
   - Must keep certificate list updated
   - Consider onchain certificate registry (future)

