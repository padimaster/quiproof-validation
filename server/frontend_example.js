/**
 * Complete Frontend Example: Proof Generation → Smart Contract Submission
 * 
 * This example shows the complete flow:
 * 1. Generate proof on server (offchain)
 * 2. Validate proof output
 * 3. Submit proof to smart contract (onchain)
 */

const API_BASE_URL = 'http://localhost:3000';
const CONTRACT_ADDRESS = '0x...'; // Your deployed contract address

// Example using ethers.js
import { ethers } from 'ethers';

/**
 * Step 1: Generate proof on server (offchain)
 */
async function generateProof(passportData) {
  const {
    dg1Base64,
    sodBase64,
    ageToVerify = 18,
    currentDate = [2024, 1, 15],
    trustedCscaCertsFile = 'trusted_csca_certs_ecuador.json',
    proofSystem = 'groth16' // Use Groth16 for EVM
  } = passportData;

  console.log('📤 Requesting proof generation from server...');

  const response = await fetch(`${API_BASE_URL}/proof/generate`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      dg1_base64: dg1Base64,
      sod_base64: sodBase64,
      age_to_verify: ageToVerify,
      current_date: currentDate,
      trusted_csca_certs_file: trustedCscaCertsFile,
      proof_system: proofSystem,
    }),
  });

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  const data = await response.json();

  if (!data.success) {
    throw new Error(data.error || 'Proof generation failed');
  }

  console.log('✅ Proof generated successfully');
  console.log(`   Proof system: ${data.proof.proof_system}`);
  console.log(`   Is valid: ${data.proof.output.is_valid}`);
  console.log(`   Meets age requirement: ${data.proof.output.meets_age_requirement}`);

  return data.proof;
}

/**
 * Step 2: Validate proof output (client-side check)
 */
function validateProofOutput(proof) {
  if (!proof.output.is_valid) {
    throw new Error('Proof output indicates invalid passport');
  }

  if (!proof.output.meets_age_requirement) {
    throw new Error('Proof output indicates age requirement not met');
  }

  if (!proof.output.debug_sod_valid) {
    throw new Error('Proof output indicates SOD verification failed');
  }

  console.log('✅ Proof output validated');
  return true;
}

/**
 * Step 3: Submit proof to smart contract (onchain)
 */
async function submitProofToContract(proof, signer) {
  // Get contract instance
  const contract = new ethers.Contract(
    CONTRACT_ADDRESS,
    [
      'function verifyAgeProof(bytes calldata proof, bytes calldata publicValues) external returns (bool)',
      'event AgeVerified(address indexed user, bool isValid, bool meetsAgeRequirement, bytes32 documentNumberHash)'
    ],
    signer
  );

  console.log('📤 Submitting proof to smart contract...');

  // Convert hex strings to bytes
  const proofBytes = ethers.getBytes(proof.proof);
  const publicValuesBytes = ethers.getBytes(proof.public_values);

  // Submit transaction
  const tx = await contract.verifyAgeProof(
    proofBytes,
    publicValuesBytes
  );

  console.log(`   Transaction hash: ${tx.hash}`);
  console.log('   Waiting for confirmation...');

  // Wait for confirmation
  const receipt = await tx.wait();

  console.log('✅ Transaction confirmed');
  console.log(`   Block: ${receipt.blockNumber}`);
  console.log(`   Gas used: ${receipt.gasUsed.toString()}`);

  // Check events
  const events = receipt.logs
    .map(log => {
      try {
        return contract.interface.parseLog(log);
      } catch (e) {
        return null;
      }
    })
    .filter(Boolean);

  if (events.length > 0) {
    const event = events[0];
    console.log('✅ Age verification event emitted:');
    console.log(`   User: ${event.args.user}`);
    console.log(`   Is valid: ${event.args.isValid}`);
    console.log(`   Meets age requirement: ${event.args.meetsAgeRequirement}`);
  }

  return receipt;
}

/**
 * Complete flow: Generate proof → Validate → Submit to contract
 */
async function completeAgeVerificationFlow(passportData, signer) {
  try {
    // Step 1: Generate proof (offchain)
    const proof = await generateProof(passportData);

    // Step 2: Validate proof output (client-side)
    validateProofOutput(proof);

    // Step 3: Submit to smart contract (onchain)
    const receipt = await submitProofToContract(proof, signer);

    return {
      success: true,
      proof,
      receipt,
      message: 'Age verification completed successfully!'
    };
  } catch (error) {
    console.error('❌ Error:', error.message);
    return {
      success: false,
      error: error.message
    };
  }
}

/**
 * React Hook Example
 */
export function useAgeVerification() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const verifyAge = async (passportData, signer) => {
    setLoading(true);
    setError(null);

    try {
      const result = await completeAgeVerificationFlow(passportData, signer);
      return result;
    } catch (err) {
      setError(err.message);
      throw err;
    } finally {
      setLoading(false);
    }
  };

  return { verifyAge, loading, error };
}

/**
 * Usage Example
 */
async function example() {
  // Connect to wallet (MetaMask, WalletConnect, etc.)
  const provider = new ethers.BrowserProvider(window.ethereum);
  const signer = await provider.getSigner();

  // Passport data from user
  const passportData = {
    dg1Base64: 'P<UTOERIKSSON<<ANNA<MARIA...', // From passport scanner
    sodBase64: 'MIIF...', // From passport scanner
    ageToVerify: 18,
    currentDate: [2024, 1, 15],
    trustedCscaCertsFile: 'trusted_csca_certs_ecuador.json',
    proofSystem: 'groth16'
  };

  // Complete flow
  const result = await completeAgeVerificationFlow(passportData, signer);

  if (result.success) {
    console.log('🎉 Age verification successful!');
    console.log('   Proof verified on-chain');
    console.log('   User meets age requirement');
  } else {
    console.error('❌ Age verification failed:', result.error);
  }
}

// Export for use in modules
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    generateProof,
    validateProofOutput,
    submitProofToContract,
    completeAgeVerificationFlow,
    useAgeVerification,
  };
}

