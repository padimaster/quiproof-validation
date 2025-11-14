# How to Get VERIFIER_ADDRESS

## Option 1: Use Canonical SP1 Verifier Gateway (Recommended) ⭐

**Easiest and recommended approach** - Use the official SP1 Verifier Gateway deployed by Succinct Labs:

### For Arbitrum One (Mainnet)

```bash
export VERIFIER_ADDRESS="0x3B6041173B80E77f038f3F2C0f9744f04837185e"
```

This is the **canonical SP1 Verifier Gateway** address on Arbitrum One. It's:
- ✅ Already deployed and verified
- ✅ Maintained by Succinct Labs
- ✅ Recommended for production use
- ✅ No deployment needed

### For Arbitrum Sepolia (Testnet)

Check the [SP1 documentation](https://docs.succinct.xyz/docs/sp1/verification/contract-addresses) for testnet addresses.

**That's it!** You can use this address directly in your deployment.

---

## Option 2: Deploy Your Own SP1 Verifier

If you prefer to deploy your own verifier contract, follow these steps:

### Fix Foundry Setup

The error you're seeing is because Foundry needs a proper project structure. Here's how to fix it:

#### Step 1: Initialize Foundry Project (if not already)

```bash
# Create a temporary directory for Foundry deployment
mkdir -p /tmp/sp1-verifier-deploy
cd /tmp/sp1-verifier-deploy

# Initialize Foundry project
forge init --no-git .

# Install SP1 SDK
forge install succinctlabs/sp1-sdk
```

#### Step 2: Deploy SP1 Verifier

```bash
# Set your environment variables
export ARBITRUM_ONE_RPC="https://arb1.arbitrum.io/rpc"
export PRIVATE_KEY="0x..."  # Your private key

# Deploy SP1 Verifier
forge create SP1Verifier \
  --rpc-url $ARBITRUM_ONE_RPC \
  --private-key $PRIVATE_KEY \
  --legacy

# Save the address from the output
export VERIFIER_ADDRESS="0x..."  # Copy from deployment output
```

#### Alternative: Use Hardhat (Easier)

If Foundry is giving you trouble, use Hardhat instead:

**Step 1: Create deployment script**

```bash
mkdir -p /tmp/sp1-verifier-deploy
cd /tmp/sp1-verifier-deploy
npm init -y
npm install --save-dev hardhat @nomicfoundation/hardhat-toolbox
npm install @succinctlabs/sp1-sdk
```

**Step 2: Initialize Hardhat**

```bash
npx hardhat init
# Choose: Create a JavaScript project
```

**Step 3: Create deploy script**

Create `scripts/deploy-verifier.js`:

```javascript
const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying SP1 Verifier...");
  
  const SP1Verifier = await ethers.getContractFactory("SP1Verifier");
  const verifier = await SP1Verifier.deploy();
  
  await verifier.waitForDeployment();
  const address = await verifier.getAddress();
  
  console.log("✅ SP1 Verifier deployed to:", address);
  console.log("\nSet this in your .env:");
  console.log(`VERIFIER_ADDRESS=${address}`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
```

**Step 3: Configure Hardhat**

Update `hardhat.config.js`:

```javascript
require("@nomicfoundation/hardhat-toolbox");

module.exports = {
  solidity: "0.8.20",
  networks: {
    arbitrumOne: {
      url: process.env.ARBITRUM_ONE_RPC || "https://arb1.arbitrum.io/rpc",
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [],
    },
  },
};
```

**Step 4: Deploy**

```bash
export ARBITRUM_ONE_RPC="https://arb1.arbitrum.io/rpc"
export PRIVATE_KEY="0x..."  # Your private key

npx hardhat run scripts/deploy-verifier.js --network arbitrumOne
```

---

## Quick Solution: Use Canonical Address

**For immediate deployment, just use:**

```bash
# Add to your .env file
echo 'VERIFIER_ADDRESS=0x3B6041173B80E77f038f3F2C0f9744f04837185e' >> contracts-stylus/.env
```

This is the **recommended approach** - the canonical verifier gateway is maintained by Succinct Labs and is the standard way to verify SP1 proofs on Arbitrum One.

---

## Verify the Address

You can verify the canonical address on Arbiscan:
- Go to: https://arbiscan.io/address/0x3B6041173B80E77f038f3F2C0f9744f04837185e
- Check that it's a verified contract
- This confirms it's the official SP1 Verifier Gateway

---

## Summary

**Recommended:** Use the canonical address `0x3B6041173B80E77f038f3F2C0f9744f04837185e`

**Alternative:** Deploy your own using Hardhat (easier than Foundry for this)

**Next Step:** Once you have `VERIFIER_ADDRESS`, proceed with your Stylus contract deployment!

