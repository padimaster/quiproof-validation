/**
 * Complete Frontend Integration Example
 * Shows how to generate proof, submit to contract, and handle events
 */

import { ethers } from 'ethers';
import { abi } from './AgeVerification.json'; // Contract ABI

const API_BASE_URL = 'http://localhost:3000';
const CONTRACT_ADDRESS = '0x...'; // Your deployed contract address

/**
 * Complete flow: Generate proof → Submit to contract → Handle result
 */
async function completeVerificationFlow(passportData, signer) {
  try {
    // Step 1: Generate proof on server (offchain)
    console.log('📤 Step 1: Generating proof on server...');
    const proof = await generateProofFromServer(passportData);
    
    console.log('✅ Proof generated');
    console.log(`   Is valid: ${proof.output.is_valid}`);
    console.log(`   Meets age requirement: ${proof.output.meets_age_requirement}`);
    
    // Step 2: Validate proof output (optional, saves gas)
    if (!proof.output.is_valid) {
      throw new Error('Proof indicates invalid passport - not submitting to contract');
    }
    
    // Step 3: Prepare contract data
    const contract = new ethers.Contract(CONTRACT_ADDRESS, abi, signer);
    
    // Convert document number hash to bytes32
    const documentNumberHash = ethers.keccak256(
      ethers.getBytes('0x' + proof.output.document_number_hash.map(b => b.toString(16).padStart(2, '0')).join(''))
    );
    
    // Check if already used (optional, saves gas)
    const isUsed = await contract.isDocumentNumberUsed(documentNumberHash);
    if (isUsed) {
      throw new Error('Document number already verified onchain');
    }
    
    // Step 4: Submit to contract
    console.log('📤 Step 2: Submitting proof to smart contract...');
    
    const proofBytes = ethers.getBytes(proof.proof);
    const publicValuesBytes = ethers.getBytes(proof.public_values);
    
    // Estimate gas first
    const gasEstimate = await contract.verifyAgeProof.estimateGas(
      proofBytes,
      publicValuesBytes,
      documentNumberHash
    );
    console.log(`   Estimated gas: ${gasEstimate.toString()}`);
    
    // Submit transaction
    const tx = await contract.verifyAgeProof(
      proofBytes,
      publicValuesBytes,
      documentNumberHash,
      { gasLimit: gasEstimate * 120n / 100n } // Add 20% buffer
    );
    
    console.log(`   Transaction hash: ${tx.hash}`);
    console.log('   Waiting for confirmation...');
    
    // Step 5: Wait for confirmation
    const receipt = await tx.wait();
    console.log(`✅ Transaction confirmed in block ${receipt.blockNumber}`);
    console.log(`   Gas used: ${receipt.gasUsed.toString()}`);
    
    // Step 6: Parse events
    const event = receipt.logs
      .map(log => {
        try {
          return contract.interface.parseLog(log);
        } catch (e) {
          return null;
        }
      })
      .find(e => e && e.name === 'AgeProofVerified');
    
    if (event) {
      console.log('✅ Age verification event:');
      console.log(`   User: ${event.args.user}`);
      console.log(`   Document hash: ${event.args.documentNumberHash}`);
      console.log(`   Is valid: ${event.args.isValid}`);
      console.log(`   Meets age requirement: ${event.args.meetsAgeRequirement}`);
      console.log(`   Timestamp: ${new Date(Number(event.args.timestamp) * 1000).toISOString()}`);
    }
    
    // Step 7: Retrieve stored record
    const record = await contract.getProofRecord(documentNumberHash);
    console.log('\n📋 Stored proof record:');
    console.log(`   User: ${record.user}`);
    console.log(`   Is valid: ${record.isValid}`);
    console.log(`   Meets age requirement: ${record.meetsAgeRequirement}`);
    console.log(`   Timestamp: ${new Date(Number(record.timestamp) * 1000).toISOString()}`);
    
    return {
      success: true,
      txHash: tx.hash,
      receipt,
      record,
      event
    };
    
  } catch (error) {
    console.error('❌ Error:', error.message);
    
    // Check if it's a contract revert
    if (error.reason) {
      console.error(`   Contract revert: ${error.reason}`);
    }
    
    return {
      success: false,
      error: error.message
    };
  }
}

/**
 * Generate proof from server
 */
async function generateProofFromServer(passportData) {
  const response = await fetch(`${API_BASE_URL}/proof/generate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      dg1_base64: passportData.dg1Base64,
      sod_base64: passportData.sodBase64,
      age_to_verify: passportData.ageToVerify || 18,
      current_date: passportData.currentDate || [2024, 1, 15],
      trusted_csca_certs_file: passportData.trustedCscaCertsFile,
      proof_system: 'groth16' // EVM-compatible
    })
  });
  
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  
  const data = await response.json();
  
  if (!data.success) {
    throw new Error(data.error || 'Proof generation failed');
  }
  
  return data.proof;
}

/**
 * React Hook Example
 */
export function useAgeVerification(contractAddress, signer) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [proofs, setProofs] = useState([]);
  
  const verifyAge = async (passportData) => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await completeVerificationFlow(passportData, signer);
      
      if (result.success) {
        // Refresh user's proofs
        const contract = new ethers.Contract(contractAddress, abi, signer);
        const userAddress = await signer.getAddress();
        const userProofHashes = await contract.getUserProofs(userAddress);
        setProofs(userProofHashes);
      }
      
      return result;
    } catch (err) {
      setError(err.message);
      throw err;
    } finally {
      setLoading(false);
    }
  };
  
  const checkDocumentNumber = async (documentNumberHash) => {
    const contract = new ethers.Contract(contractAddress, abi, signer);
    return await contract.isDocumentNumberUsed(documentNumberHash);
  };
  
  const getProofRecord = async (documentNumberHash) => {
    const contract = new ethers.Contract(contractAddress, abi, signer);
    return await contract.getProofRecord(documentNumberHash);
  };
  
  // Listen for events
  useEffect(() => {
    if (!signer) return;
    
    const contract = new ethers.Contract(contractAddress, abi, signer);
    
    const filter = contract.filters.AgeProofVerified();
    contract.on(filter, (user, docHash, isValid, meetsAge, timestamp, event) => {
      console.log('New proof verified:', { user, docHash, isValid, meetsAge });
      // Update UI
    });
    
    return () => {
      contract.removeAllListeners();
    };
  }, [signer, contractAddress]);
  
  return {
    verifyAge,
    checkDocumentNumber,
    getProofRecord,
    loading,
    error,
    proofs
  };
}

/**
 * Usage Example
 */
async function example() {
  // Connect wallet
  const provider = new ethers.BrowserProvider(window.ethereum);
  const signer = await provider.getSigner();
  
  // Passport data
  const passportData = {
    dg1Base64: 'P<UTOERIKSSON<<ANNA<MARIA...',
    sodBase64: 'MIIF...',
    ageToVerify: 18,
    currentDate: [2024, 1, 15],
    trustedCscaCertsFile: 'trusted_csca_certs_ecuador.json'
  };
  
  // Complete flow
  const result = await completeVerificationFlow(passportData, signer);
  
  if (result.success) {
    console.log('🎉 Age verification successful!');
    console.log(`   Transaction: ${result.txHash}`);
    console.log(`   Block: ${result.receipt.blockNumber}`);
  }
}

// Export
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    completeVerificationFlow,
    generateProofFromServer,
    useAgeVerification
  };
}

