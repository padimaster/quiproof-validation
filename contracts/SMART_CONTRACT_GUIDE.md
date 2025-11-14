# Smart Contract Integration Guide

## Overview

This guide explains how the smart contract works and how to integrate it with your frontend application.

## Contract Architecture

```
┌─────────────────────────────────────────┐
│      AgeVerification Contract           │
│                                          │
│  1. Receives: proof + public_values     │
│  2. Verifies: SP1Verifier.verify()       │
│  3. Decodes: public_values → output     │
│  4. Stores: ProofRecord onchain          │
│  5. Emits: AgeProofVerified event        │
└─────────────────────────────────────────┘
```

## Key Components

### 1. SP1 Verifier

The contract uses SP1's onchain verifier to cryptographically verify proofs:

```solidity
SP1Verifier public immutable verifier;
bool isValid = verifier.verify(vkey, proof, publicValues);
```

**What it verifies:**
- Proof is cryptographically valid
- Public values match the proof
- Program was executed correctly

**What it doesn't verify:**
- The actual passport data (that's done offchain)
- Certificate validity (done offchain)
- Age calculation (done offchain, result in public values)

### 2. Verification Key (vkey)

The `vkey` is set at contract deployment and never changes. It identifies the specific program being verified.

**Get vkey:**
```bash
curl http://localhost:3000/proof/vkey
```

**Deploy with vkey:**
```solidity
bytes32 vkey = 0x00a43d8424d369ae4eb7740d7a006ce78458458a9185c58932d79ddf4b366052;
AgeVerification contract = new AgeVerification(vkey);
```

### 3. Proof Record Storage

Each verified proof is stored onchain:

```solidity
struct ProofRecord {
    address user;                    // Who submitted
    bytes32 documentNumberHash;      // Unique identifier
    bool isValid;                    // Validation result
    bool meetsAgeRequirement;        // Age check result
    uint256 timestamp;               // When verified
    bool exists;                     // Record exists
}
```

**Why store onchain?**
- Permanent record
- Prevents replay attacks
- Queryable by anyone
- Transparent verification history

### 4. Replay Attack Prevention

The contract tracks `documentNumberHash` to prevent the same passport from being verified twice:

```solidity
if (proofRecords[documentNumberHash].exists) {
    revert("Document number already verified");
}
```

**Frontend should check first:**
```javascript
const isUsed = await contract.isDocumentNumberUsed(documentNumberHash);
if (isUsed) {
  // Don't submit, already verified
}
```

## Function Reference

### `verifyAgeProof(proof, publicValues, documentNumberHash)`

**Main function** - Verifies proof and stores result.

**Parameters:**
- `proof` (bytes): Hex-encoded proof from server (e.g., `"0x1234..."`)
- `publicValues` (bytes): Hex-encoded public values from server
- `documentNumberHash` (bytes32): Keccak256 hash of document number

**Returns:**
- `isValid` (bool): Whether passport is valid
- `meetsAgeRequirement` (bool): Whether age requirement is met

**Gas Cost:** ~220-270K gas

**Example:**
```javascript
const tx = await contract.verifyAgeProof(
  "0x1234...",  // proof
  "0x5678...",  // publicValues
  "0xabcd..."   // documentNumberHash
);
```

### `verifyProofOnly(proof, publicValues)`

**View function** - Verifies proof without storing (no gas cost for caller).

**Use case:** Check if proof is valid before submitting (saves gas if invalid).

**Example:**
```javascript
const isValid = await contract.verifyProofOnly(proof, publicValues);
if (!isValid) {
  throw new Error('Proof is invalid');
}
// Now submit with confidence
```

### `getProofRecord(documentNumberHash)`

**View function** - Retrieve stored proof record.

**Returns:** `ProofRecord` struct with all details.

**Example:**
```javascript
const record = await contract.getProofRecord(documentNumberHash);
console.log(record.isValid);
console.log(record.meetsAgeRequirement);
console.log(record.timestamp);
```

### `isDocumentNumberUsed(documentNumberHash)`

**View function** - Check if document number already verified.

**Use case:** Prevent wasting gas on already-verified documents.

**Example:**
```javascript
const isUsed = await contract.isDocumentNumberUsed(documentNumberHash);
if (isUsed) {
  // Already verified, show existing record
  const record = await contract.getProofRecord(documentNumberHash);
}
```

### `getUserProofs(userAddress)`

**View function** - Get all proof hashes for a user.

**Returns:** Array of `bytes32` document number hashes.

**Example:**
```javascript
const myAddress = await signer.getAddress();
const myProofs = await contract.getUserProofs(myAddress);
// Get details for each
for (const hash of myProofs) {
  const record = await contract.getProofRecord(hash);
  console.log(record);
}
```

## Frontend Integration Flow

### Step 1: Generate Proof (Offchain)

```javascript
// Call your proof server
const response = await fetch('http://localhost:3000/proof/generate', {
  method: 'POST',
  body: JSON.stringify({
    dg1_base64: '...',
    sod_base64: '...',
    age_to_verify: 18,
    current_date: [2024, 1, 15],
    proof_system: 'groth16'
  })
});

const { proof } = await response.json();
// proof.proof = "0x..."
// proof.public_values = "0x..."
// proof.output.document_number_hash = [190, 154, ...]
```

### Step 2: Prepare Contract Data

```javascript
// Convert document number hash to bytes32
const docHashArray = proof.output.document_number_hash;
const docHashHex = '0x' + docHashArray.map(b => 
  b.toString(16).padStart(2, '0')
).join('');
const documentNumberHash = ethers.keccak256(docHashHex);

// Convert proof and public values to bytes
const proofBytes = ethers.getBytes(proof.proof);
const publicValuesBytes = ethers.getBytes(proof.public_values);
```

### Step 3: Check if Already Used (Optional)

```javascript
const isUsed = await contract.isDocumentNumberUsed(documentNumberHash);
if (isUsed) {
  // Show existing record instead of submitting
  const record = await contract.getProofRecord(documentNumberHash);
  return record;
}
```

### Step 4: Submit to Contract

```javascript
// Estimate gas
const gasEstimate = await contract.verifyAgeProof.estimateGas(
  proofBytes,
  publicValuesBytes,
  documentNumberHash
);

// Submit transaction
const tx = await contract.verifyAgeProof(
  proofBytes,
  publicValuesBytes,
  documentNumberHash,
  { gasLimit: gasEstimate * 120n / 100n } // 20% buffer
);

// Wait for confirmation
const receipt = await tx.wait();
```

### Step 5: Handle Events

```javascript
// Listen for events
contract.on("AgeProofVerified", (user, docHash, isValid, meetsAge, timestamp) => {
  if (user === myAddress) {
    console.log("My proof verified!");
    // Update UI
  }
});

// Or parse from receipt
const event = receipt.logs
  .map(log => contract.interface.parseLog(log))
  .find(e => e.name === "AgeProofVerified");
```

## Public Values Decoding

The `publicValues` contain the `AgeProofOutput` struct serialized with bincode.

**Current Implementation:**
The contract has a simplified decoder that extracts:
- `is_valid` (bool, byte 1)
- `meets_age_requirement` (bool, byte 2)

**For Production:**
Consider one of these approaches:

1. **Decode on Frontend:**
   ```javascript
   // Frontend already has the decoded output from server
   // Pass decoded values as separate parameters
   ```

2. **Use Bincode Decoder:**
   - Implement proper bincode decoding in Solidity
   - Or use a library if available

3. **Simplified Approach (Current):**
   - Contract only extracts essential fields
   - Full output available from server response

## Gas Optimization Tips

1. **Check Before Submitting:**
   ```javascript
   // Saves gas if already verified
   if (await contract.isDocumentNumberUsed(docHash)) {
     return; // Don't submit
   }
   ```

2. **Validate Offchain First:**
   ```javascript
   // Check proof output before submitting
   if (!proof.output.is_valid) {
     return; // Don't waste gas
   }
   ```

3. **Use View Functions:**
   ```javascript
   // Verify proof without storing (no gas)
   const isValid = await contract.verifyProofOnly(proof, publicValues);
   ```

4. **Batch Operations (Future):**
   - Verify multiple proofs in one transaction
   - Reduces per-proof overhead

## Error Handling

### Common Errors

1. **"Document number already verified"**
   - Solution: Check `isDocumentNumberUsed()` first
   - Or retrieve existing record

2. **"Invalid proof"**
   - Solution: Verify proof is valid before submitting
   - Check server response

3. **"Invalid public values length"**
   - Solution: Ensure public values are correctly formatted
   - Check server response format

4. **Out of Gas**
   - Solution: Increase gas limit
   - Use `estimateGas()` to get accurate estimate

## Testing

### Local Testing (Hardhat)

```javascript
// test/AgeVerification.test.js
describe("AgeVerification", function() {
  it("Should verify a proof", async function() {
    // Get proof from server
    const proof = await getProofFromServer();
    
    // Deploy contract
    const contract = await deployContract();
    
    // Submit proof
    const tx = await contract.verifyAgeProof(
      proof.proof,
      proof.public_values,
      proof.documentNumberHash
    );
    
    // Check result
    const receipt = await tx.wait();
    expect(receipt.status).to.equal(1);
  });
});
```

## Deployment Checklist

- [ ] Get vkey from proof server
- [ ] Deploy contract with vkey
- [ ] Verify contract on Etherscan/Blockscout
- [ ] Test with real proof
- [ ] Update frontend with contract address
- [ ] Test end-to-end flow
- [ ] Monitor gas usage
- [ ] Set up event listeners

## Security Best Practices

1. **Verify vkey matches server:**
   ```javascript
   const serverVkey = await fetch('/proof/vkey').then(r => r.json());
   const contractVkey = await contract.vkey();
   assert(serverVkey.vkey === contractVkey);
   ```

2. **Validate proof output before submitting:**
   ```javascript
   if (!proof.output.is_valid) {
     // Don't submit
   }
   ```

3. **Check for replay attacks:**
   ```javascript
   if (await contract.isDocumentNumberUsed(docHash)) {
     // Already verified
   }
   ```

4. **Handle errors gracefully:**
   ```javascript
   try {
     await contract.verifyAgeProof(...);
   } catch (error) {
     if (error.reason.includes("already verified")) {
       // Show existing record
     }
   }
   ```

## Next Steps

1. Deploy contract to testnet
2. Test with real proofs
3. Integrate with frontend
4. Deploy to mainnet
5. Monitor and optimize

