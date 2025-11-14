//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use clap::Parser;
use serde::{Deserialize, Serialize};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use std::fs;
use std::path::PathBuf;
use base64::{Engine as _, engine::general_purpose};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const AGE_PROOF_ELF: &[u8] = include_elf!("age-proof-program");

#[derive(Serialize, Deserialize, Debug)]
pub struct AgeProofInput {
    pub dg1_data: Vec<u8>,           // Parsed passport DG1 data
    pub sod_data: Vec<u8>,           // Signed Object Document
    pub age_to_verify: u8,           // Minimum age to prove (e.g., 18)
    pub current_date: [u16; 3],      // [year, month, day]
    pub trusted_csca_certs: Option<Vec<TrustedCSCACert>>, // Optional: trusted CSCA certificates
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrustedCSCACert {
    pub country_code: String,        // ISO country code (e.g., "EC", "NZ")
    pub certificate_hash: [u8; 32],  // SHA-256 hash of the certificate
    pub certificate_der: Vec<u8>,    // DER-encoded certificate (optional, for full verification)
    pub serial_number: Option<String>, // Certificate serial number
    pub common_name: Option<String>,   // Certificate common name
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgeProofOutput {
    pub is_valid: bool,              // Is the passport valid?
    pub meets_age_requirement: bool, // Does person meet age requirement?
    pub document_number_hash: [u8; 32], // Hash of document number (for uniqueness)
    // Debug fields
    pub debug_dg1_parsed: bool,      // Was DG1 data successfully parsed?
    pub debug_sod_valid: bool,        // Was SOD verification successful?
    pub debug_not_expired: bool,      // Is passport not expired?
    pub debug_dg1_length: u16,        // Length of DG1 data
    pub debug_sod_length: u16,        // Length of SOD data
    pub debug_document_number: String, // Extracted document number
    pub debug_birth_date: [u16; 3],   // Extracted birth date
    pub debug_expiry_date: [u16; 3],  // Extracted expiry date
    pub debug_age: u32,               // Calculated age
}

#[derive(Deserialize, Debug)]
struct PayloadJson {
    dg1_base64: String,
    sod_base64: String,
    age_to_verify: u8,
    current_date: [u16; 3],
    trusted_csca_certs_file: Option<String>, // Path to JSON file with trusted CSCA certificates
}

/// The arguments for the command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,

    #[arg(long)]
    payload: Option<String>,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    // Load payload from JSON file
    let payload_path = args.payload
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Default to payload.json in the workspace root
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("payload.json")
        });

    let payload_json: PayloadJson = if payload_path.exists() {
        let json_str = fs::read_to_string(&payload_path)
            .expect(&format!("Failed to read payload file: {:?}", payload_path));
        serde_json::from_str(&json_str)
            .expect(&format!("Failed to parse JSON from: {:?}", payload_path))
    } else {
        eprintln!("Warning: Payload file not found at {:?}, using default sample data", payload_path);
        // Fallback to sample data
        PayloadJson {
            dg1_base64: general_purpose::STANDARD.encode(b"P<UTOERIKSSON<<ANNA<MARIA<<<<<<<<<<<<<<<<<<<<L898902C36UTO7408122F1204159ZE184226B<<<<<6"),
            sod_base64: general_purpose::STANDARD.encode(b"sample_sod_data_for_verification"),
            age_to_verify: 18,
            current_date: [2024, 1, 1],
            trusted_csca_certs_file: None,
        }
    };

    // Decode base64 data
    let dg1_data = general_purpose::STANDARD.decode(&payload_json.dg1_base64)
        .expect("Failed to decode dg1_base64");
    let sod_data = general_purpose::STANDARD.decode(&payload_json.sod_base64)
        .expect("Failed to decode sod_base64");

    // Load trusted CSCA certificates if provided
    let trusted_csca = if let Some(certs_file) = &payload_json.trusted_csca_certs_file {
        let certs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(certs_file);
        
        if certs_path.exists() {
            println!("Loading trusted CSCA certificates from: {:?}", certs_path);
            let certs_json = fs::read_to_string(&certs_path)
                .expect("Failed to read CSCA certificates file");
            let certs: Vec<TrustedCSCACert> = serde_json::from_str(&certs_json)
                .expect("Failed to parse CSCA certificates JSON");
            println!("Loaded {} trusted CSCA certificates", certs.len());
            Some(certs)
        } else {
            eprintln!("Warning: CSCA certificates file not found: {:?}", certs_path);
            None
        }
    } else {
        None
    };

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Create input from payload
    let input = AgeProofInput {
        dg1_data,
        sod_data,
        age_to_verify: payload_json.age_to_verify,
        current_date: payload_json.current_date,
        trusted_csca_certs: trusted_csca,
    };

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&input);

    println!("Age to verify: {}", input.age_to_verify);
    println!("Current date: {:?}", input.current_date);
    println!("DG1 data length: {} bytes", input.dg1_data.len());
    println!("SOD data length: {} bytes", input.sod_data.len());

    if args.execute {
        // Execute the program
        let (output, report) = client.execute(AGE_PROOF_ELF, &stdin).run().unwrap();
        println!("Program executed successfully.");

        // Read the output.
        let decoded: AgeProofOutput = bincode::deserialize(output.as_slice()).unwrap();
        println!("\n=== Validation Results ===");
        println!("Is valid: {}", decoded.is_valid);
        println!("Meets age requirement: {}", decoded.meets_age_requirement);
        println!("\n=== Debug Information ===");
        println!("DG1 data length: {} bytes", decoded.debug_dg1_length);
        println!("SOD data length: {} bytes", decoded.debug_sod_length);
        println!("DG1 parsed successfully: {}", decoded.debug_dg1_parsed);
        println!("SOD verification passed: {}", decoded.debug_sod_valid);
        println!("Passport not expired: {}", decoded.debug_not_expired);
        if decoded.debug_dg1_parsed {
            println!("Document number: {}", decoded.debug_document_number);
            println!("Birth date: {:04}-{:02}-{:02}", 
                decoded.debug_birth_date[0], 
                decoded.debug_birth_date[1], 
                decoded.debug_birth_date[2]);
            println!("Expiry date: {:04}-{:02}-{:02}", 
                decoded.debug_expiry_date[0], 
                decoded.debug_expiry_date[1], 
                decoded.debug_expiry_date[2]);
            println!("Calculated age: {}", decoded.debug_age);
        }
        println!("Document number hash: {:?}", decoded.document_number_hash);

        // Record the number of cycles executed.
        println!("\nNumber of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(AGE_PROOF_ELF);

        // Generate the proof
        let proof = client
            .prove(&pk, &stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
