use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const API_BASE_URL: &str = "http://localhost:3000";

#[derive(Serialize, Deserialize, Debug)]
struct ProofRequest {
    dg1_base64: String,
    sod_base64: String,
    age_to_verify: u8,
    current_date: [u16; 3],
    trusted_csca_certs_file: Option<String>,
    proof_system: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProofResponse {
    success: bool,
    proof: Option<ProofData>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProofData {
    proof: String,
    public_values: String,
    vkey: String,
    output: AgeProofOutput,
    proof_system: String,
    cycles: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AgeProofOutput {
    is_valid: bool,
    meets_age_requirement: bool,
    document_number_hash: Vec<u8>,
    debug_dg1_parsed: bool,
    debug_sod_valid: bool,
    debug_not_expired: bool,
    debug_dg1_length: u16,
    debug_sod_length: u16,
    debug_document_number: String,
    debug_birth_date: [u16; 3],
    debug_expiry_date: [u16; 3],
    debug_age: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct VKeyResponse {
    vkey: String,
    vkey_hex: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let test_type = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    println!("==========================================");
    println!("Server API Test Suite");
    println!("==========================================\n");

    // Check if server is running
    if !check_server_health().await {
        eprintln!("❌ Server is not running!");
        eprintln!("   Please start the server first:");
        eprintln!("   cd server && cargo run --release");
        std::process::exit(1);
    }

    match test_type {
        "health" => test_health().await?,
        "vkey" => test_verification_key().await?,
        "valid" => test_valid_certificate().await?,
        "invalid" => test_invalid_certificate().await?,
        "all" => {
            test_health().await?;
            test_verification_key().await?;
            test_valid_certificate().await?;
            test_invalid_certificate().await?;
        }
        _ => {
            eprintln!("Unknown test type: {}", test_type);
            eprintln!("Usage: test-client [health|vkey|valid|invalid|all]");
            std::process::exit(1);
        }
    }

    println!("\n==========================================");
    println!("All tests completed!");
    println!("==========================================");

    Ok(())
}

async fn check_server_health() -> bool {
    let client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();

    client
        .get(&format!("{}/health", API_BASE_URL))
        .send()
        .await
        .is_ok()
}

async fn test_health() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 TEST: Health Check");
    println!("-----------------------------------");

    let client = Client::new();
    let response = client
        .get(&format!("{}/health", API_BASE_URL))
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    let health: HealthResponse = response.json().await?;
    println!("✓ Status: {}", health.status);
    println!("✓ Version: {}", health.version);

    assert_eq!(health.status, "ok");
    println!("✅ Health check passed!\n");

    Ok(())
}

async fn test_verification_key() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 TEST: Verification Key");
    println!("-----------------------------------");

    let client = Client::new();
    let response = client
        .get(&format!("{}/proof/vkey", API_BASE_URL))
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    let vkey: VKeyResponse = response.json().await?;
    println!("✓ VKey: {}", &vkey.vkey[..20]);
    println!("✓ VKey Hex: {}", &vkey.vkey_hex[..30]);

    assert!(!vkey.vkey.is_empty());
    assert!(vkey.vkey_hex.starts_with("0x"));
    println!("✅ Verification key test passed!\n");

    Ok(())
}

async fn test_valid_certificate() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 TEST: Valid Ecuador Certificate");
    println!("-----------------------------------");

    // Load payload.json
    let payload_str = std::fs::read_to_string("payload.json")?;
    let payload: serde_json::Value = serde_json::from_str(&payload_str)?;

    // Update to use valid certificate
    let mut request = ProofRequest {
        dg1_base64: payload["dg1_base64"]
            .as_str()
            .unwrap()
            .to_string(),
        sod_base64: payload["sod_base64"]
            .as_str()
            .unwrap()
            .to_string(),
        age_to_verify: payload["age_to_verify"].as_u64().unwrap() as u8,
        current_date: [
            payload["current_date"][0].as_u64().unwrap() as u16,
            payload["current_date"][1].as_u64().unwrap() as u16,
            payload["current_date"][2].as_u64().unwrap() as u16,
        ],
        trusted_csca_certs_file: Some("trusted_csca_certs_ecuador.json".to_string()),
        proof_system: Some("groth16".to_string()), // Use groth16 for EVM compatibility
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(300)) // 5 minutes for proof generation
        .build()?;

    println!("Sending proof generation request...");
    let response = client
        .post(&format!("{}/proof/generate", API_BASE_URL))
        .json(&request)
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    let proof_response: ProofResponse = response.json().await?;

    if !proof_response.success {
        eprintln!("❌ Proof generation failed: {:?}", proof_response.error);
        std::process::exit(1);
    }

    let proof = proof_response.proof.unwrap();
    println!("✓ Proof generated successfully");
    println!("✓ Proof system: {}", proof.proof_system);
    println!("✓ Proof length: {} bytes", proof.proof.len() / 2 - 2); // Subtract "0x"
    println!("✓ Public values length: {} bytes", proof.public_values.len() / 2 - 2);

    println!("\n=== Validation Results ===");
    println!("Is valid: {}", proof.output.is_valid);
    println!("Meets age requirement: {}", proof.output.meets_age_requirement);

    println!("\n=== Debug Information ===");
    println!("DG1 parsed successfully: {}", proof.output.debug_dg1_parsed);
    println!("SOD verification passed: {}", proof.output.debug_sod_valid);
    println!("Passport not expired: {}", proof.output.debug_not_expired);
    println!("Document number: {}", proof.output.debug_document_number);
    println!("Birth date: {:04}-{:02}-{:02}",
        proof.output.debug_birth_date[0],
        proof.output.debug_birth_date[1],
        proof.output.debug_birth_date[2]
    );
    println!("Expiry date: {:04}-{:02}-{:02}",
        proof.output.debug_expiry_date[0],
        proof.output.debug_expiry_date[1],
        proof.output.debug_expiry_date[2]
    );
    println!("Calculated age: {}", proof.output.debug_age);

    // Assertions
    assert!(proof.output.is_valid, "Expected valid certificate to pass validation");
    assert!(proof.output.meets_age_requirement, "Expected age requirement to be met");
    assert!(proof.output.debug_sod_valid, "Expected SOD verification to pass");
    assert!(proof.output.debug_not_expired, "Expected passport to not be expired");

    println!("\n✅ Valid certificate test passed!\n");

    Ok(())
}

async fn test_invalid_certificate() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 TEST: Invalid Ecuador Certificate");
    println!("-----------------------------------");

    // Load payload.json
    let payload_str = std::fs::read_to_string("payload.json")?;
    let payload: serde_json::Value = serde_json::from_str(&payload_str)?;

    let request = ProofRequest {
        dg1_base64: payload["dg1_base64"]
            .as_str()
            .unwrap()
            .to_string(),
        sod_base64: payload["sod_base64"]
            .as_str()
            .unwrap()
            .to_string(),
        age_to_verify: payload["age_to_verify"].as_u64().unwrap() as u8,
        current_date: [
            payload["current_date"][0].as_u64().unwrap() as u16,
            payload["current_date"][1].as_u64().unwrap() as u16,
            payload["current_date"][2].as_u64().unwrap() as u16,
        ],
        trusted_csca_certs_file: Some("trusted_csca_certs_invalid_ecuador.json".to_string()),
        proof_system: Some("groth16".to_string()), // Use groth16 for EVM compatibility
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(300)) // 5 minutes for proof generation
        .build()?;

    println!("Sending proof generation request with invalid certificate...");
    let response = client
        .post(&format!("{}/proof/generate", API_BASE_URL))
        .json(&request)
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    let proof_response: ProofResponse = response.json().await?;

    if !proof_response.success {
        eprintln!("❌ Proof generation failed: {:?}", proof_response.error);
        std::process::exit(1);
    }

    let proof = proof_response.proof.unwrap();
    println!("✓ Proof generated successfully");
    println!("✓ Proof system: {}", proof.proof_system);

    println!("\n=== Validation Results ===");
    println!("Is valid: {}", proof.output.is_valid);
    println!("Meets age requirement: {}", proof.output.meets_age_requirement);

    println!("\n=== Debug Information ===");
    println!("DG1 parsed successfully: {}", proof.output.debug_dg1_parsed);
    println!("SOD verification passed: {}", proof.output.debug_sod_valid);
    println!("Passport not expired: {}", proof.output.debug_not_expired);

    // Assertions - invalid certificate should fail
    assert!(!proof.output.is_valid, "Expected invalid certificate to fail validation");
    assert!(!proof.output.debug_sod_valid, "Expected SOD verification to fail with invalid certificate");

    println!("\n✅ Invalid certificate test passed! (correctly rejected)\n");

    Ok(())
}
