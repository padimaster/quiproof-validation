const hre = require("hardhat");

async function main() {
  // Get vkey from environment or use default
  // You can get this from: curl http://localhost:3000/proof/vkey
  const vkey = process.env.VKEY || "0x00a43d8424d369ae4eb7740d7a006ce78458458a9185c58932d79ddf4b366052";
  
  console.log("Deploying AgeVerification contract...");
  console.log(`Using vkey: ${vkey}`);
  
  const AgeVerification = await hre.ethers.getContractFactory("AgeVerification");
  const contract = await AgeVerification.deploy(vkey);
  
  await contract.waitForDeployment();
  
  const address = await contract.getAddress();
  console.log(`✅ AgeVerification deployed to: ${address}`);
  console.log(`   Network: ${hre.network.name}`);
  console.log(`   VKey: ${vkey}`);
  
  // Verify contract (if on a network that supports verification)
  if (hre.network.name !== "hardhat" && hre.network.name !== "localhost") {
    console.log("\n⏳ Waiting for block confirmations...");
    await contract.deploymentTransaction().wait(5);
    
    console.log("Verifying contract on Etherscan...");
    try {
      await hre.run("verify:verify", {
        address: address,
        constructorArguments: [vkey],
      });
      console.log("✅ Contract verified!");
    } catch (error) {
      console.log("⚠️  Verification failed:", error.message);
    }
  }
  
  console.log("\n📋 Contract Information:");
  console.log(`   Address: ${address}`);
  console.log(`   VKey: ${vkey}`);
  console.log(`   Verifier: ${await contract.verifier()}`);
  
  return address;
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });

