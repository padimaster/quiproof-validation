# Quick Start Guide

## 1. Start the Server

```bash
cd server
cargo run --release
```

The server will start on `http://0.0.0.0:3000`

## 2. Test the API

### Health Check
```bash
curl http://localhost:3000/health
```

### Get Verification Key
```bash
curl http://localhost:3000/proof/vkey
```

### Generate Proof
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

## 3. Use from Frontend

See `example_client.js` for a complete JavaScript example.

## 4. Smart Contract Integration

1. Get the verification key: `GET /proof/vkey`
2. Deploy your smart contract with the vkey
3. Generate proofs: `POST /proof/generate`
4. Submit proof to smart contract using `proof`, `public_values`, and `vkey`

See `API.md` for detailed documentation.
