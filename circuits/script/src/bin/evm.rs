//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can have an
//! EVM-Compatible proof generated which can be verified on-chain.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system groth16
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system plonk
//! ```

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use sp1_sdk::{
    include_elf, HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};
use std::path::PathBuf;
    
/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const AGE_PROOF_ELF: &[u8] = include_elf!("age-proof-program");

#[derive(Serialize, Deserialize, Debug)]
pub struct AgeProofInput {
    pub dg1_data: Vec<u8>,           // Parsed passport DG1 data
    pub sod_data: Vec<u8>,           // Signed Object Document
    pub age_to_verify: u8,           // Minimum age to prove (e.g., 18)
    pub current_date: [u16; 3],      // [year, month, day]
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgeProofOutput {
    pub is_valid: bool,              // Is the passport valid?
    pub meets_age_requirement: bool, // Does person meet age requirement?
    pub document_number_hash: [u8; 32], // Hash of document number (for uniqueness)
}

/// The arguments for the EVM command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct EVMArgs {
    #[arg(long, default_value = "18")]
    age_to_verify: u8,
    #[arg(long, default_value = "2024,1,1")]
    current_date: String,
    #[arg(long, value_enum, default_value = "groth16")]
    system: ProofSystem,
}

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1AgeProofFixture {
    is_valid: bool,
    meets_age_requirement: bool,
    document_number_hash: String,
    vkey: String,
    public_values: String,
    proof: String,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = EVMArgs::parse();

    // Parse current date
    let date_parts: Vec<&str> = args.current_date.split(',').collect();
    if date_parts.len() != 3 {
        eprintln!("Error: current_date must be in format YYYY,MM,DD");
        std::process::exit(1);
    }
    let current_date = [
        date_parts[0].parse().expect("Invalid year"),
        date_parts[1].parse().expect("Invalid month"),
        date_parts[2].parse().expect("Invalid day"),
    ];

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Setup the program.
    let (pk, vk) = client.setup(AGE_PROOF_ELF);

    // Create sample input (in production, this would come from actual passport data)
    let input = AgeProofInput {
        dg1_data: b"P<UTOERIKSSON<<ANNA<MARIA<<<<<<<<<<<<<<<<<<<<L898902C36UTO7408122F1204159ZE184226B<<<<<6".to_vec(),
        sod_data: b"sample_sod_data_for_verification".to_vec(),
        age_to_verify: args.age_to_verify,
        current_date,
    };

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&input);

    println!("Age to verify: {}", args.age_to_verify);
    println!("Current date: {:?}", current_date);
    println!("Proof System: {:?}", args.system);

    // Generate the proof based on the selected proof system.
    let proof = match args.system {
        ProofSystem::Plonk => client.prove(&pk, &stdin).plonk().run(),
        ProofSystem::Groth16 => client.prove(&pk, &stdin).groth16().run(),
    }
    .expect("failed to generate proof");

    create_proof_fixture(&proof, &vk, args.system);
}

/// Create a fixture for the given proof.
fn create_proof_fixture(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    system: ProofSystem,
) {
    // Deserialize the public values.
    let bytes = proof.public_values.as_slice();
    let output: AgeProofOutput = bincode::deserialize(bytes).unwrap();

    // Create the testing fixture so we can test things end-to-end.
    let fixture = SP1AgeProofFixture {
        is_valid: output.is_valid,
        meets_age_requirement: output.meets_age_requirement,
        document_number_hash: format!("0x{}", hex::encode(output.document_number_hash)),
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    //
    // Note that the verification key stays the same regardless of the input.
    println!("Verification Key: {}", fixture.vkey);

    // The public values are the values which are publicly committed to by the zkVM.
    //
    // If you need to expose the inputs or outputs of your program, you should commit them in
    // the public values.
    println!("Public Values: {}", fixture.public_values);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("Proof Bytes: {}", fixture.proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(format!("{:?}-fixture.json", system).to_lowercase()),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
