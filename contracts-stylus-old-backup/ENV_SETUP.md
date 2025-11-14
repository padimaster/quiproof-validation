# Environment Variables Setup

This guide explains how to set up your `.env` file for Stylus contract deployment.

## Quick Setup

1. **Copy the example file:**
   ```bash
   cp .env.example .env
   ```

2. **Fill in your values:**
   - `VKEY`: Get from proof server
   - `VERIFIER_ADDRESS`: Deploy SP1 verifier first
   - `PRIVATE_KEY`: Your wallet private key
   - `NETWORK`: Choose network (arbitrum-sepolia for testnet)

## Getting Values

### 1. Verification Key (VKEY)

**Option A: From Proof Server**
```bash
curl http://localhost:3000/proof/vkey | jq -r '.vkey_hex'
```

**Option B: From Circuit**
```bash
cd ../circuits/script
cargo run --release --bin vkey
```

**Option C: Manual**
If you know the vkey, paste it directly:
```
VKEY=0x00a43d8424d369ae4eb7740d7a006ce78458458a9185c58932d79ddf4b366052
```

### 2. SP1 Verifier Address (VERIFIER_ADDRESS)

**Step 1: Deploy SP1 Verifier Contract**

Using Foundry:
```bash
forge install succinctlabs/sp1-sdk
forge create SP1Verifier --rpc-url $ARBITRUM_RPC --private-key $PRIVATE_KEY
```

Using Hardhat:
```bash
npx hardhat run scripts/deploy-verifier.js --network arbitrum-sepolia
```

**Step 2: Copy the deployed address**
```
VERIFIER_ADDRESS=0x1234567890123456789012345678901234567890
```

### 3. Private Key (PRIVATE_KEY)

⚠️ **SECURITY WARNING:** Never commit your private key to git!

**Format:**
```
PRIVATE_KEY=0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
```

**Getting from MetaMask:**
1. MetaMask → Account Details → Export Private Key
2. Copy the hex string (with or without 0x prefix)

**Getting from Hardhat:**
```bash
# Use Hardhat's default account
PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

### 4. Network (NETWORK)

**Testnet (Recommended for testing):**
```
NETWORK=arbitrum-sepolia
```

**Mainnet:**
```
NETWORK=arbitrum-one
```

**Nova:**
```
NETWORK=arbitrum-nova
```

## Complete Example

```bash
# .env file

# Verification Key
VKEY=0x00a43d8424d369ae4eb7740d7a006ce78458458a9185c58932d79ddf4b366052

# SP1 Verifier (deploy this first)
VERIFIER_ADDRESS=0x1234567890123456789012345678901234567890

# Private Key (NEVER COMMIT THIS!)
PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# Network
NETWORK=arbitrum-sepolia

# Contract Address (set automatically after deployment)
CONTRACT_ADDRESS=
```

## Security Best Practices

1. **Never commit `.env` to git**
   - `.env` is already in `.gitignore`
   - Use `.env.example` for templates

2. **Use environment-specific files**
   - `.env.local` for local development
   - `.env.production` for production (never commit)

3. **Rotate keys regularly**
   - Change private keys if compromised
   - Update verifier address if redeployed

4. **Use separate accounts**
   - Use a dedicated deployment account
   - Don't use your main wallet's private key

5. **Store secrets securely**
   - Use a password manager
   - Use hardware wallets for production
   - Consider using AWS Secrets Manager or similar

## Verification

After setting up `.env`, verify it's loaded correctly:

```bash
# Check if variables are set
source .env
echo "VKEY: ${VKEY:0:20}..."
echo "VERIFIER: $VERIFIER_ADDRESS"
echo "NETWORK: $NETWORK"
```

## Troubleshooting

### "VKEY not set"
- Ensure `.env` file exists
- Check variable name (case-sensitive)
- Run `source .env` before deployment

### "VERIFIER_ADDRESS not set"
- Deploy SP1 verifier contract first
- Check address format (0x + 40 hex chars)
- Verify contract is deployed on correct network

### "PRIVATE_KEY not set"
- Add your private key to `.env`
- Ensure format is correct (0x + 64 hex chars)
- Check file permissions (should be readable)

### "Invalid network"
- Use: `arbitrum-sepolia`, `arbitrum-one`, or `arbitrum-nova`
- Check network name spelling
- Ensure you have RPC access to network

## Next Steps

Once `.env` is configured:

1. ✅ Verify all variables are set
2. Deploy SP1 verifier (if not already deployed)
3. Run deployment script: `./scripts/deploy.sh`
4. Contract address will be saved to `.env` automatically

