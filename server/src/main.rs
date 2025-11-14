use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sp1_sdk::{
    include_elf, HashableKey, ProverClient, SP1Stdin, SP1VerifyingKey,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const AGE_PROOF_ELF: &[u8] = include_elf!("age-proof-program");

// Re-export types from circuit program
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrustedCSCACert {
    pub country_code: String,
    pub certificate_hash: [u8; 32],
    pub certificate_der: Vec<u8>,
    pub serial_number: Option<String>,
    pub common_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgeProofInput {
    pub dg1_data: Vec<u8>,
    pub sod_data: Vec<u8>,
    pub age_to_verify: u8,
    pub current_date: [u16; 3],
    pub trusted_csca_certs: Option<Vec<TrustedCSCACert>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgeProofOutput {
    pub is_valid: bool,
    pub meets_age_requirement: bool,
    pub document_number_hash: [u8; 32],
    pub debug_dg1_parsed: bool,
    pub debug_sod_valid: bool,
    pub debug_not_expired: bool,
    pub debug_dg1_length: u16,
    pub debug_sod_length: u16,
    pub debug_document_number: String,
    pub debug_birth_date: [u16; 3],
    pub debug_expiry_date: [u16; 3],
    pub debug_age: u32,
}

// API Request/Response types
#[derive(Deserialize, Debug)]
pub struct ProofRequest {
    pub dg1_base64: String,
    pub sod_base64: String,
    pub age_to_verify: u8,
    pub current_date: [u16; 3],
    pub trusted_csca_certs_file: Option<String>,
    pub proof_system: Option<String>, // "groth16" or "plonk" for EVM, "core" for default
}

#[derive(Serialize, Debug)]
pub struct ProofResponse {
    pub success: bool,
    pub proof: Option<ProofData>,
    pub error: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ProofData {
    // Proof data (hex encoded)
    pub proof: String,
    pub public_values: String,
    pub vkey: String,
    
    // Decoded output for convenience
    pub output: AgeProofOutput,
    
    // Proof system used
    pub proof_system: String,
    
    // Metadata
    pub cycles: Option<u64>,
}

#[derive(Serialize, Debug)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

// App State
#[derive(Clone)]
pub struct AppState {
    pub vk: Arc<SP1VerifyingKey>,
}

#[tokio::main]
async fn main() {
    // Setup logging
    tracing_subscriber::fmt::init();

    // Initialize SP1 prover client and get verification key
    let client = ProverClient::from_env();
    let (_, vk) = client.setup(AGE_PROOF_ELF);

    let state = AppState {
        vk: Arc::new(vk),
    };

    // Build the router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/proof/generate", post(generate_proof))
        .route("/proof/vkey", get(get_verification_key))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive()) // Allow all origins for development
        )
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("🚀 Server running on http://0.0.0.0:3000");
    println!("📚 API Documentation:");
    println!("   GET  /health - Health check");
    println!("   POST /proof/generate - Generate proof");
    println!("   GET  /proof/vkey - Get verification key");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_verification_key(State(state): State<AppState>) -> Json<serde_json::Value> {
    let vkey_bytes32 = state.vk.bytes32();
    Json(serde_json::json!({
        "vkey": vkey_bytes32.to_string(),
        "vkey_hex": format!("0x{}", hex::encode(vkey_bytes32)),
    }))
}

async fn generate_proof(
    State(state): State<AppState>,
    Json(request): Json<ProofRequest>,
) -> Result<Json<ProofResponse>, StatusCode> {
    // Decode base64 inputs
    use base64::{Engine as _, engine::general_purpose};
    let dg1_data = match general_purpose::STANDARD.decode(&request.dg1_base64) {
        Ok(data) => data,
        Err(e) => {
            return Ok(Json(ProofResponse {
                success: false,
                proof: None,
                error: Some(format!("Invalid dg1_base64: {}", e)),
            }));
        }
    };

    let sod_data = match general_purpose::STANDARD.decode(&request.sod_base64) {
        Ok(data) => data,
        Err(e) => {
            return Ok(Json(ProofResponse {
                success: false,
                proof: None,
                error: Some(format!("Invalid sod_base64: {}", e)),
            }));
        }
    };

    // Load trusted CSCA certificates if provided
    let trusted_csca_certs = if let Some(file_path) = &request.trusted_csca_certs_file {
        match load_trusted_certs(file_path) {
            Ok(certs) => Some(certs),
            Err(e) => {
                return Ok(Json(ProofResponse {
                    success: false,
                    proof: None,
                    error: Some(format!("Failed to load trusted certificates: {}", e)),
                }));
            }
        }
    } else {
        None
    };

    // Create input
    let input = AgeProofInput {
        dg1_data,
        sod_data,
        age_to_verify: request.age_to_verify,
        current_date: request.current_date,
        trusted_csca_certs,
    };

    // Setup stdin
    let mut stdin = SP1Stdin::new();
    stdin.write(&input);

    // Generate proof based on system
    // Default to "groth16" - fastest verification (~200K gas) and smallest proof size (~2KB)
    // This is the optimal choice for on-chain verification (Stylus/Solidity)
    // Note: Requires Docker to be running for proof generation
    // Alternatives: "plonk" (slightly larger) or "core" (off-chain only, no Docker)
    let proof_system = request.proof_system.as_deref().unwrap_or("groth16");
    
    // Create prover client for this request
    let client = ProverClient::from_env();
    
    // Setup the program (get proving key)
    let (pk, _) = client.setup(AGE_PROOF_ELF);
    
    let (proof, proof_bytes, cycles) = match proof_system {
        "groth16" => {
            // Groth16: Fastest verification (~200K gas) and smallest proof size (~2KB)
            // Optimal for on-chain verification (Stylus/Solidity)
            let proof = match client
                .prove(&pk, &stdin)
                .groth16()
                .run()
            {
                Ok(p) => p,
                Err(e) => {
                    let error_msg = format!(
                        "Groth16 proof generation failed: {:?}. \
                        Note: Groth16 requires Docker to be running. \
                        For local testing without Docker, use 'proof_system: \"core\"' (off-chain only).",
                        e
                    );
                    eprintln!("{}", error_msg);
                    return Ok(Json(ProofResponse {
                        success: false,
                        proof: None,
                        error: Some(error_msg),
                    }));
                }
            };
            // Groth16 proofs support .bytes() for onchain verification
            let bytes = proof.bytes();
            (proof, bytes, None)
        }
        "plonk" => {
            // PLONK: Alternative to Groth16, slightly larger proof (~3KB) and higher gas (~250K)
            // Still EVM-compatible, but Groth16 is preferred for optimal performance
            let proof = match client
                .prove(&pk, &stdin)
                .plonk()
                .run()
            {
                Ok(p) => p,
                Err(e) => {
                    let error_msg = format!(
                        "PLONK proof generation failed: {:?}. \
                        Note: PLONK requires Docker to be running. \
                        For local testing without Docker, use 'proof_system: \"core\"' (off-chain only).",
                        e
                    );
                    eprintln!("{}", error_msg);
                    return Ok(Json(ProofResponse {
                        success: false,
                        proof: None,
                        error: Some(error_msg),
                    }));
                }
            };
            // PLONK proofs support .bytes() for onchain verification
            let bytes = proof.bytes();
            (proof, bytes, None)
        }
        "core" => {
            // Core proof - works without Docker, but NOT compatible with on-chain verification
            // Use only for off-chain testing. For production, use "groth16" (recommended) or "plonk"
            let proof = client
                .prove(&pk, &stdin)
                .run()
                .map_err(|e| {
                    eprintln!("Core proof generation error: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            // Core proofs don't have .bytes() method, so we serialize the proof
            // Note: These proofs cannot be verified on-chain (Stylus/Solidity)
            let bytes = bincode::serialize(&proof).map_err(|e| {
                eprintln!("Failed to serialize core proof: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            // Core proofs don't expose cycles directly, use None
            let cycles = None;
            (proof, bytes, cycles)
        }
        _ => {
            return Ok(Json(ProofResponse {
                success: false,
                proof: None,
                error: Some(format!(
                    "Invalid proof_system: {}. Must be 'groth16', 'plonk', or 'core'",
                    proof_system
                )),
            }));
        }
    };

    // Deserialize output from public values
    let public_values_bytes = match proof_system {
        "core" => {
            // For core proofs, public_values is already serialized
            proof.public_values.as_slice().to_vec()
        }
        _ => {
            // For Groth16/PLONK, use the public_values directly
            proof.public_values.as_slice().to_vec()
        }
    };
    
    let output: AgeProofOutput = bincode::deserialize(&public_values_bytes)
        .map_err(|e| {
            eprintln!("Failed to deserialize output: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Create response
    let proof_data = ProofData {
        proof: format!("0x{}", hex::encode(proof_bytes)),
        public_values: format!("0x{}", hex::encode(public_values_bytes)),
        vkey: state.vk.bytes32().to_string(),
        output,
        proof_system: proof_system.to_string(),
        cycles,
    };

    Ok(Json(ProofResponse {
        success: true,
        proof: Some(proof_data),
        error: None,
    }))
}

fn load_trusted_certs(file_path: &str) -> Result<Vec<TrustedCSCACert>, Box<dyn std::error::Error>> {
    use std::fs;
    use std::path::PathBuf;
    
    // Try relative to server directory first, then workspace root
    let paths = vec![
        PathBuf::from(file_path),
        PathBuf::from("..").join(file_path),
        PathBuf::from("../..").join(file_path),
    ];
    
    for path in paths {
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let certs: Vec<TrustedCSCACert> = serde_json::from_str(&content)?;
            return Ok(certs);
        }
    }
    
    Err(format!("Trusted certificates file not found: {}", file_path).into())
}
