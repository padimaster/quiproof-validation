# Age Verification Proof Server API

RESTful API for generating zero-knowledge proofs of passport age verification.

## Base URL

```
http://localhost:3000
```

## Endpoints

### `GET /health`

Health check endpoint.

**Response:**
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

### `POST /proof/generate`

Generate a zero-knowledge proof for passport age verification.

**Request Body:**
```json
{
  "dg1_base64": "string",           // Base64 encoded DG1 data (required)
  "sod_base64": "string",           // Base64 encoded SOD data (required)
  "age_to_verify": 18,               // Minimum age to verify (required)
  "current_date": [2024, 1, 15],     // [year, month, day] (required)
  "trusted_csca_certs_file": "trusted_csca_certs_ecuador.json", // Optional
  "proof_system": "groth16"          // Optional: "groth16", "plonk", or "core" (default: "core")
}
```

**Response (Success):**
```json
{
  "success": true,
  "proof": {
    "proof": "0x...",               // Hex-encoded proof bytes (for smart contract)
    "public_values": "0x...",       // Hex-encoded public values (for smart contract)
    "vkey": "...",                  // Verification key (for smart contract)
    "output": {
      "is_valid": true,
      "meets_age_requirement": true,
      "document_number_hash": [190, 154, 236, ...],
      "debug_dg1_parsed": true,
      "debug_sod_valid": true,
      "debug_not_expired": true,
      "debug_dg1_length": 93,
      "debug_sod_length": 3331,
      "debug_document_number": "064132<9E",
      "debug_birth_date": [2001, 7, 4],
      "debug_expiry_date": [2034, 4, 29],
      "debug_age": 24
    },
    "proof_system": "groth16",
    "cycles": null
  },
  "error": null
}
```

**Response (Error):**
```json
{
  "success": false,
  "proof": null,
  "error": "Error message describing what went wrong"
}
```

### `GET /proof/vkey`

Get the verification key for on-chain verification.

**Response:**
```json
{
  "vkey": "...",
  "vkey_hex": "0x..."
}
```

## Smart Contract Integration

### Solidity Example

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {SP1Verifier} from "@succinctlabs/sp1-sdk/contracts/SP1Verifier.sol";

contract AgeVerification {
    SP1Verifier public verifier;
    bytes32 public vkey;
    
    constructor(bytes32 _vkey) {
        verifier = new SP1Verifier();
        vkey = _vkey;
    }
    
    function verifyAgeProof(
        bytes calldata proof,
        bytes calldata publicValues
    ) external view returns (bool) {
        return verifier.verify(vkey, proof, publicValues);
    }
}
```

### Frontend Integration (React/Next.js)

```typescript
import { useState } from 'react';

const API_BASE_URL = 'http://localhost:3000';

export function useAgeVerification() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const generateProof = async (params: {
    dg1Base64: string;
    sodBase64: string;
    ageToVerify: number;
    currentDate: [number, number, number];
    trustedCscaCertsFile?: string;
    proofSystem?: 'groth16' | 'plonk' | 'core';
  }) => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`${API_BASE_URL}/proof/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          dg1_base64: params.dg1Base64,
          sod_base64: params.sodBase64,
          age_to_verify: params.ageToVerify,
          current_date: params.currentDate,
          trusted_csca_certs_file: params.trustedCscaCertsFile,
          proof_system: params.proofSystem || 'groth16',
        }),
      });

      const data = await response.json();

      if (!data.success) {
        throw new Error(data.error || 'Proof generation failed');
      }

      return data.proof;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      throw err;
    } finally {
      setLoading(false);
    }
  };

  return { generateProof, loading, error };
}
```

## Proof Systems

### Groth16 (Recommended for EVM)
- **Proof Size**: ~2KB
- **Verification Gas**: ~200K gas
- **Generation Time**: 2-5 minutes
- **Use Case**: Best for production smart contracts

### PLONK
- **Proof Size**: ~3KB
- **Verification Gas**: ~250K gas
- **Generation Time**: 3-6 minutes
- **Use Case**: Universal setup, good for EVM

### Core (Default)
- **Proof Size**: ~100KB
- **Verification**: Not EVM-compatible
- **Generation Time**: 30-60 seconds
- **Use Case**: Testing, non-blockchain use cases

## Error Codes

The API returns HTTP status codes:

- `200 OK`: Request successful
- `400 Bad Request`: Invalid request parameters
- `500 Internal Server Error`: Server error during proof generation

All errors include a JSON response with `success: false` and an `error` message.

## Rate Limiting

Currently no rate limiting is implemented. For production, consider:
- Adding rate limiting middleware
- Queue system for proof generation
- Caching verification keys

## CORS

CORS is enabled for all origins in development. For production:
- Configure specific allowed origins
- Use environment variables for CORS settings

