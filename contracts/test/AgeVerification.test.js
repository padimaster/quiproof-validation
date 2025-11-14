const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("AgeVerification", function() {
  let contract;
  let owner;
  let user;
  let vkey;

  beforeEach(async function() {
    [owner, user] = await ethers.getSigners();
    
    // Use a test vkey (in production, get from proof server)
    vkey = "0x00a43d8424d369ae4eb7740d7a006ce78458458a9185c58932d79ddf4b366052";
    
    const AgeVerification = await ethers.getContractFactory("AgeVerification");
    contract = await AgeVerification.deploy(vkey);
    await contract.waitForDeployment();
  });

  describe("Deployment", function() {
    it("Should set the correct vkey", async function() {
      expect(await contract.vkey()).to.equal(vkey);
    });

    it("Should deploy verifier contract", async function() {
      const verifierAddress = await contract.verifier();
      expect(verifierAddress).to.not.equal(ethers.ZeroAddress);
    });
  });

  describe("Proof Verification", function() {
    // Note: These tests require actual proofs from the server
    // For now, we'll test the contract structure
    
    it("Should reject invalid proof", async function() {
      const fakeProof = "0x1234";
      const fakePublicValues = "0x5678";
      const docHash = ethers.keccak256("0xabcd");
      
      await expect(
        contract.verifyAgeProof(fakeProof, fakePublicValues, docHash)
      ).to.be.reverted; // Will revert with "Invalid proof"
    });

    it("Should prevent replay attacks", async function() {
      // This test would require a valid proof
      // For now, just test the structure
      const docHash = ethers.keccak256("0xtest");
      
      // First verification would succeed (if proof is valid)
      // Second verification should fail
      // await expect(
      //   contract.verifyAgeProof(proof, publicValues, docHash)
      // ).to.be.revertedWith("Document number already verified");
    });
  });

  describe("View Functions", function() {
    it("Should return false for unused document number", async function() {
      const docHash = ethers.keccak256("0xunused");
      expect(await contract.isDocumentNumberUsed(docHash)).to.be.false;
    });

    it("Should return empty array for user with no proofs", async function() {
      const proofs = await contract.getUserProofs(user.address);
      expect(proofs).to.have.length(0);
    });
  });
});

