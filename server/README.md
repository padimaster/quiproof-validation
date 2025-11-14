# Age Verification Proof Server

HTTP server for generating zero-knowledge proofs of passport age verification.

## Features

- ✅ Generate SP1 core proofs
- ✅ Generate EVM-compatible proofs (Groth16, PLONK)
- ✅ Return proofs with public inputs for smart contract integration
- ✅ RESTful API with JSON responses
- ✅ CORS enabled for frontend integration

## API Endpoints

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
  "dg1_base64": "string",           // Base64 encoded DG1 data
  "sod_base64": "string",           // Base64 encoded SOD data
  "age_to_verify": 18,              // Minimum age to verify
  "current_date": [2024, 1, 15],    // [year, month, day]
  "trusted_csca_certs_file": "trusted_csca_certs_ecuador.json", // Optional
  "proof_system": "groth16"         // Optional: "groth16", "plonk", or "core" (default)
}
```

**Response:**
```json
{
  "success": true,
  "proof": {
    "proof": "0x...",               // Hex-encoded proof bytes
    "public_values": "0x...",       // Hex-encoded public values
    "vkey": "...",                  // Verification key
    "output": {
      "is_valid": true,
      "meets_age_requirement": true,
      "document_number_hash": [190, 154, ...],
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
    "cycles": null                  // Only for core proofs
  },
  "error": null
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

## Running the Server

### Development

```bash
cd server
cargo run --release
```

The server will start on `http://0.0.0.0:3000`

### Using SP1 Prover Network

For faster proof generation, use the Succinct Prover Network:

```bash
SP1_PROVER=network NETWORK_PRIVATE_KEY=your_key cargo run --release
```

## Example Usage

### Using cURL

```bash
curl -X POST http://localhost:3000/proof/generate \
  -H "Content-Type: application/json" \
  -d '{
    "dg1_base64": "P<UTOERIKSSON<<ANNA<MARIA...",
    "sod_base64": "MIIF...",
    "age_to_verify": 18,
    "current_date": [2024, 1, 15],
    "trusted_csca_certs_file": "trusted_csca_certs_ecuador.json",
    "proof_system": "groth16"
  }'
```

### Using JavaScript/TypeScript

```typescript
const response = await fetch('http://localhost:3000/proof/generate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    dg1_base64: 'P<UTOERIKSSON<<ANNA<MARIA...',
    sod_base64: 'MIIF...',
    age_to_verify: 18,
    current_date: [2024, 1, 15],
    trusted_csca_certs_file: 'trusted_csca_certs_ecuador.json',
    proof_system: 'groth16'
  })
});

const data = await response.json();
if (data.success) {
  const { proof, public_values, vkey, output } = data.proof;
  // Use proof, public_values, and vkey for smart contract verification
  console.log('Proof generated:', proof);
  console.log('Is valid:', output.is_valid);
  console.log('Meets age requirement:', output.meets_age_requirement);
}
```

## Smart Contract Integration

The proof data returned can be used directly in Solidity smart contracts:

```solidity
// Example Solidity verification
function verifyAgeProof(
    bytes calldata proof,
    bytes calldata publicValues,
    bytes32 vkey
) external returns (bool) {
    // Use SP1 verifier contract
    return SP1Verifier.verify(vkey, proof, publicValues);
}
```

### Proof System Selection

- **`groth16`**: Best for EVM, smallest proof size (~2KB), fastest verification
- **`plonk`**: Good for EVM, slightly larger proofs, universal setup
- **`core`**: Default SP1 proof, not EVM-compatible, fastest generation

For smart contract integration, use `groth16` or `plonk`.

## Error Handling

The API returns errors in the response body:

```json
{
  "success": false,
  "proof": null,
  "error": "Error message describing what went wrong"
}
```

Common errors:
- `Invalid dg1_base64`: Base64 decoding failed
- `Invalid sod_base64`: Base64 decoding failed
- `Failed to load trusted certificates`: Certificate file not found or invalid
- `Proof generation error`: Internal error during proof generation

## Environment Variables

- `SP1_PROVER`: Set to `network` to use Succinct Prover Network
- `NETWORK_PRIVATE_KEY`: Private key for prover network (if using network)

## Performance

- **Core Proof**: ~30-60 seconds, ~400K cycles
- **Groth16 Proof**: ~2-5 minutes (requires 16GB+ RAM)
- **PLONK Proof**: ~3-6 minutes (requires 16GB+ RAM)

For production, use the Succinct Prover Network for faster proof generation.

