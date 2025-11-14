/**
 * Example client for the Age Verification Proof Server
 * This can be used from any frontend application
 */

const API_BASE_URL = 'http://localhost:3000';

/**
 * Generate a proof for passport age verification
 * @param {Object} params - Proof generation parameters
 * @returns {Promise<Object>} Proof data
 */
async function generateProof(params) {
  const {
    dg1Base64,
    sodBase64,
    ageToVerify = 18,
    currentDate = [2024, 1, 15],
    trustedCscaCertsFile = null,
    proofSystem = 'groth16' // 'groth16', 'plonk', or 'core'
  } = params;

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

  return data.proof;
}

/**
 * Get the verification key for on-chain verification
 * @returns {Promise<Object>} Verification key data
 */
async function getVerificationKey() {
  const response = await fetch(`${API_BASE_URL}/proof/vkey`);
  
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  return await response.json();
}

/**
 * Health check
 * @returns {Promise<Object>} Health status
 */
async function healthCheck() {
  const response = await fetch(`${API_BASE_URL}/health`);
  
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  return await response.json();
}

// Example usage
async function example() {
  try {
    // Check server health
    const health = await healthCheck();
    console.log('Server health:', health);

    // Get verification key
    const vkey = await getVerificationKey();
    console.log('Verification key:', vkey);

    // Generate proof (example with placeholder data)
    const proof = await generateProof({
      dg1Base64: 'P<UTOERIKSSON<<ANNA<MARIA...', // Base64 encoded DG1
      sodBase64: 'MIIF...', // Base64 encoded SOD
      ageToVerify: 18,
      currentDate: [2024, 1, 15],
      trustedCscaCertsFile: 'trusted_csca_certs_ecuador.json',
      proofSystem: 'groth16',
    });

    console.log('Proof generated:', {
      proof: proof.proof.substring(0, 50) + '...',
      publicValues: proof.public_values.substring(0, 50) + '...',
      vkey: proof.vkey,
      isValid: proof.output.is_valid,
      meetsAgeRequirement: proof.output.meets_age_requirement,
      documentNumberHash: proof.output.document_number_hash,
    });

    // Use proof data for smart contract
    return {
      proof: proof.proof,
      publicValues: proof.public_values,
      vkey: proof.vkey,
      output: proof.output,
    };
  } catch (error) {
    console.error('Error:', error);
    throw error;
  }
}

// Export for use in modules
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    generateProof,
    getVerificationKey,
    healthCheck,
    example,
  };
}

